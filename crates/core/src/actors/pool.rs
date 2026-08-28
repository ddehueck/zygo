use crate::{
    CancellationGroup,
    actors::actor::{ActorHandle, ActorMessage},
    context::{RunContext, ServiceContext},
    engine::EngineSnapshot,
    models::{
        ChannelItemInsertedData, DataReference, Event, EventId, EventKind, Source, WorkflowRunId,
        WorkflowSchema,
    },
    store::{StorageProvider, keyspace::KeySpace},
};
use std::{collections::HashMap, time::SystemTime};
use tokio::sync::Mutex;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

pub struct ActorPool<S: StorageProvider> {
    context: ServiceContext<S>,
    registry: Mutex<HashMap<WorkflowRunId, ActorHandle>>,
}

impl<S: StorageProvider> ActorPool<S> {
    pub fn new(context: ServiceContext<S>) -> Self {
        Self {
            context,
            registry: Mutex::new(HashMap::new()),
        }
    }

    ///! Idempotently initializes a workflow run with an actor.
    pub async fn run_with_actor(
        &self,
        id: &WorkflowRunId,
        input: DataReference,
        schema: WorkflowSchema,
    ) -> Result<(), anyhow::Error> {
        self.run_with_actor_many(id, vec![input], schema).await
    }

    pub async fn run_with_actor_many(
        &self,
        id: &WorkflowRunId,
        inputs: Vec<DataReference>,
        schema: WorkflowSchema,
    ) -> Result<(), anyhow::Error> {
        anyhow::ensure!(
            !inputs.is_empty(),
            "a workflow run requires at least one input"
        );

        let mut registry = self.registry.lock().await;

        if registry.contains_key(&id) {
            return Ok(());
        }

        let actor_handle = self.create_actor(&id, &schema).await?;
        registry.insert(id.clone(), actor_handle.clone());
        drop(registry); // Release lock after actor handle is inserted

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

        self.send_events(&actor_handle, input_events).await?;

        Ok(())
    }

    pub async fn cancel(&self, workflow_run_id: &WorkflowRunId) {
        let actor_handle = {
            let registry = self.registry.lock().await;
            registry.get(workflow_run_id).cloned()
        };

        if let Some(actor_handle) = actor_handle {
            actor_handle.cancel().await;
        }
    }

    pub async fn subscribe(
        &self,
        workflow_run_id: &WorkflowRunId,
    ) -> Result<tokio::sync::watch::Receiver<EngineSnapshot>, anyhow::Error> {
        let state_rx = {
            let registry = self.registry.lock().await;
            let handle = registry.get(workflow_run_id).ok_or_else(|| {
                anyhow::anyhow!("No actor found for workflow run id: {}", workflow_run_id)
            })?;
            handle.state_rx.clone()
        };

        Ok(state_rx)
    }

    async fn create_actor(
        &self,
        workflow_run_id: &WorkflowRunId,
        schema: &WorkflowSchema,
    ) -> Result<ActorHandle, anyhow::Error> {
        let schema_key = KeySpace::run(workflow_run_id).schema();
        let schema_value = serde_json::to_value(schema)?;
        self.context
            .store
            .put(&[(schema_key, schema_value)])
            .await?;

        let cancellation = CancellationGroup::new();
        let run_context = RunContext::new(&self.context, workflow_run_id, cancellation);
        ActorHandle::spawn(&run_context).await
    }

    async fn send_events(
        &self,
        actor_handle: &ActorHandle,
        events: Vec<Event>,
    ) -> Result<(), anyhow::Error> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        actor_handle
            .tx
            .send_timeout(ActorMessage { events, reply_tx }, DEFAULT_TIMEOUT)
            .await
            .map_err(|error| anyhow::anyhow!("failed to send events to workflow actor: {error}"))?;

        reply_rx
            .await
            .map_err(|_| anyhow::anyhow!("workflow actor dropped the events response"))??;

        Ok(())
    }
}
