use crate::{
    CancellationGroup,
    actor::ActorHandle,
    context::{RunContext, ServiceContext},
    dependencies::{AppDeps, StorageProvider},
    engine::EngineSnapshot,
    models::{
        ChannelItemInsertedData, DataReference, Event, EventId, EventKind, Source, WorkflowRunId,
        WorkflowSchema,
    },
    store::KeySpace,
    stream::StreamReader,
    workers::WorkerPool,
};
use std::time::SystemTime;
use tokio::sync::Mutex;

pub struct Zygo<D: AppDeps> {
    context: ServiceContext<D>,
    actor: Mutex<Option<(WorkflowRunId, ActorHandle)>>,
}

pub struct ZygoConfig {
    /// Number of worker processes to use for running workflow jobs.
    num_workers: usize,
}

impl ZygoConfig {
    pub fn new(num_workers: usize) -> Self {
        Self { num_workers }
    }
}

impl<D: AppDeps> Zygo<D> {
    pub fn new(deps: D, config: ZygoConfig) -> Self {
        Self {
            context: ServiceContext::new(deps, WorkerPool::new(config.num_workers)),
            actor: Mutex::new(None),
        }
    }

    pub async fn run(
        &self,
        id: &WorkflowRunId,
        inputs: Vec<DataReference>,
        schema: WorkflowSchema,
    ) -> Result<(), anyhow::Error> {
        anyhow::ensure!(
            !inputs.is_empty(),
            "a workflow run requires at least one input"
        );

        let input_events = inputs
            .into_iter()
            .map(|input| Event {
                id: EventId::new(),
                is_replay: false,
                timestamp: SystemTime::now(),
                kind: EventKind::ChannelItemInserted(ChannelItemInsertedData {
                    channel_id: schema.input_channel_id.clone(),
                    data_reference: input,
                }),
                source: Source::Input,
                run_id: id.clone(),
            })
            .collect();

        let schema_key = KeySpace::run(id).schema();
        let schema_value = serde_json::to_value(&schema)?;
        self.context
            .deps
            .store()
            .put(&[(schema_key, schema_value)])
            .await?;

        let cancellation = CancellationGroup::new();
        let run_context = RunContext::new(&self.context, id, cancellation);
        let actor = ActorHandle::spawn(&run_context, input_events).await?;
        *self.actor.lock().await = Some((id.clone(), actor));

        Ok(())
    }

    pub async fn cancel(&self, run_id: &WorkflowRunId) -> Result<(), anyhow::Error> {
        let actor = {
            let active_actor = self.actor.lock().await;
            active_actor
                .as_ref()
                .filter(|(active_run_id, _)| active_run_id == run_id)
                .map(|(_, actor)| actor.clone())
        };

        if let Some(actor) = actor {
            actor.signal_cancel();
            if let Err(error) = self.context.worker_pool.cancel_run(run_id) {
                eprintln!("failed to remove queued jobs for run {run_id}: {error}");
            }
            actor.cancel().await;
        }

        Ok(())
    }

    pub async fn subscribe(
        &self,
        run_id: &WorkflowRunId,
    ) -> Result<tokio::sync::watch::Receiver<EngineSnapshot>, anyhow::Error> {
        let active_actor = self.actor.lock().await;
        let (_, actor) = active_actor
            .as_ref()
            .filter(|(active_run_id, _)| active_run_id == run_id)
            .ok_or_else(|| anyhow::anyhow!("No actor found for workflow run id: {run_id}"))?;

        Ok(actor.state_rx.clone())
    }

    pub fn stream(&self, run_id: &WorkflowRunId) -> StreamReader<D::Store> {
        StreamReader::new(self.context.deps.store().clone(), run_id)
    }

    pub async fn workflow_schema(
        &self,
        run_id: &WorkflowRunId,
    ) -> Result<Option<WorkflowSchema>, anyhow::Error> {
        let schema_value = self
            .context
            .deps
            .store()
            .get(&KeySpace::run(run_id).schema())
            .await?;

        schema_value
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }
}
