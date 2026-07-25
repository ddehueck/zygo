use crate::{
    actors::actor::{ActorHandle, ActorMessage},
    context::{RunContext, ServiceContext},
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
        input: DataReference,
        schema: WorkflowSchema,
    ) -> Result<WorkflowRunId, anyhow::Error> {
        let mut registry = self.registry.lock().await;
        let workflow_run_id = WorkflowRunId::new(&schema.content_hash, &input)?;

        if registry.contains_key(&workflow_run_id) {
            return Ok(workflow_run_id);
        }

        let actor_handle = self.create_actor(&workflow_run_id, &schema).await?;
        registry.insert(workflow_run_id.clone(), actor_handle.clone());
        drop(registry); // Release lock after actor handle is inserted

        let input_channel_inserted_event = Event {
            id: EventId::new(),
            is_replay: false,
            timestamp: SystemTime::now(),
            kind: EventKind::ChannelItemInserted(ChannelItemInsertedData {
                channel_id: schema.input_channel_id,
                data_reference: input,
            }),
            source: Source::Input,
            run_id: workflow_run_id.clone(),
        };

        self.send_event(&actor_handle, input_channel_inserted_event)
            .await?;

        Ok(workflow_run_id)
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

        let run_context = RunContext::new(&self.context, workflow_run_id);
        ActorHandle::spawn(&run_context).await
    }

    async fn send_event(
        &self,
        actor_handle: &ActorHandle,
        event: Event,
    ) -> Result<(), anyhow::Error> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        actor_handle
            .tx
            .send_timeout(ActorMessage { event, reply_tx }, DEFAULT_TIMEOUT)
            .await
            .map_err(|error| anyhow::anyhow!("failed to send event to workflow actor: {error}"))?;

        reply_rx
            .await
            .map_err(|_| anyhow::anyhow!("workflow actor dropped the event response"))??;

        Ok(())
    }
}
