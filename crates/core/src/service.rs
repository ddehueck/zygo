use crate::CancellationGroup;
use crate::actor::{ActorHandle, ActorStateRx};
use crate::context::{RunContext, ServiceContext};
use crate::dependencies::{AppDeps, StorageProvider};
use crate::models::{
    ChannelItemInsertedData, DataReference, Event, EventId, EventKind, Source, WorkflowRunId,
    WorkflowSchema,
};
use crate::store::KeySpace;
use crate::stream::StreamReader;
use crate::workers::WorkerPool;
use std::time::SystemTime;

pub struct ZygoConfig {
    /// Number of worker processes to use for running workflow jobs.
    pub num_workers: usize,
}

#[derive(Clone)]
pub struct Zygo<D: AppDeps> {
    context: ServiceContext<D>,
}

impl<D: AppDeps> Zygo<D> {
    pub fn new(deps: D, config: ZygoConfig) -> Self {
        Self {
            context: ServiceContext::new(deps, WorkerPool::new(config.num_workers)),
        }
    }

    pub async fn run(
        &self,
        id: &WorkflowRunId,
        inputs: Vec<DataReference>,
        schema: WorkflowSchema,
    ) -> Result<ZygoRun<D>, anyhow::Error> {
        ZygoRun::start(id, inputs, schema, self.context.clone()).await
    }
}

pub struct ZygoRun<D: AppDeps> {
    pub id: WorkflowRunId,
    actor: ActorHandle,
    context: ServiceContext<D>,
}

impl<D: AppDeps> ZygoRun<D> {
    pub async fn start(
        id: &WorkflowRunId,
        inputs: Vec<DataReference>,
        schema: WorkflowSchema,
        context: ServiceContext<D>,
    ) -> Result<Self, anyhow::Error> {
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
        context
            .deps
            .store()
            .put(&[(schema_key, schema_value)])
            .await?;

        let cancellation = CancellationGroup::new();
        let run_context = RunContext::new(&context, id, cancellation);
        let actor = ActorHandle::spawn(&run_context, input_events).await?;

        Ok(Self {
            id: id.clone(),
            actor,
            context,
        })
    }

    pub async fn cancel(&self, run_id: &WorkflowRunId) -> Result<(), anyhow::Error> {
        self.actor.signal_cancel();
        if let Err(error) = self.context.worker_pool.cancel_run(run_id) {
            eprintln!("failed to remove queued jobs for run {run_id}: {error}");
        }
        self.actor.cancel().await;

        Ok(())
    }

    pub fn subscribe(&self) -> Result<ActorStateRx, anyhow::Error> {
        Ok(self.actor.state_rx.clone())
    }

    pub fn stream(&self, run_id: &WorkflowRunId) -> StreamReader<D::Store> {
        StreamReader::new(self.context.deps.store().clone(), run_id)
    }
}
