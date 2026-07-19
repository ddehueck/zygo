use crate::{
    actors::actor::{ActorHandle, ActorMessage},
    context::{RunContext, ServiceContext},
    models::{Event, RunId, WorkflowId, WorkflowVersionId},
    store::StorageProvider,
};
use std::collections::HashMap;
use tokio::sync::Mutex;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

type RunKey = (WorkflowId, WorkflowVersionId, RunId);

pub struct ActorPool<S: StorageProvider> {
    context: ServiceContext<S>,
    registry: Mutex<HashMap<RunKey, ActorHandle>>,
}

impl<S: StorageProvider> ActorPool<S> {
    pub fn new(context: ServiceContext<S>) -> Self {
        Self {
            context,
            registry: Mutex::new(HashMap::new()),
        }
    }

    /// Sends an event to the actor responsible for its run context.
    ///
    /// Creates and caches an actor for the event's run if one does not already exist,
    /// then waits for the actor to acknowledge that the event was ingested.
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be confirmed to be persisted.
    pub async fn send(&self, event: Event) -> Result<(), anyhow::Error> {
        let run_context = RunContext::new(
            &self.context,
            event.workflow_id.clone(),
            event.workflow_version_id.clone(),
            event.workflow_run_id.clone(),
        );
        let handle = self.get_or_create_actor(run_context).await?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let message = ActorMessage {
            event,
            reply_tx: tx,
        };

        handle
            .tx
            .send_timeout(message, DEFAULT_TIMEOUT)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // TODO: is this the right error handling?
        rx.await.map_err(|e| anyhow::anyhow!(e))??;
        Ok(())
    }

    async fn get_or_create_actor(
        &self,
        run_context: RunContext<S>,
    ) -> Result<ActorHandle, anyhow::Error> {
        let mut registry = self.registry.lock().await;
        let run_key = (
            run_context.workflow_id.clone(),
            run_context.workflow_version_id.clone(),
            run_context.run_id.clone(),
        );

        if let Some(actor_handle) = registry.get(&run_key) {
            return Ok(actor_handle.clone());
        }

        let actor_handle = ActorHandle::spawn(run_context).await?;
        registry.insert(run_key, actor_handle.clone());
        Ok(actor_handle)
    }
}
