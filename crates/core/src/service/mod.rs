mod register;

use crate::{
    actors::ActorPool,
    context::ServiceContext,
    models::{Event, RegisterWorkflowInput, RegisteredWorkflowSummary},
    service::register::register_workflow,
    store::{StorageProvider, Store},
    workers::WorkerPool,
};

struct ZygoService<S: StorageProvider> {
    context: ServiceContext<S>,
    actor_pool: ActorPool<S>,
}

struct ZygoConfig {
    num_workers: usize,
}

impl<S: StorageProvider> ZygoService<S> {
    pub fn new(store: Store<S>, config: ZygoConfig) -> Self {
        let context = ServiceContext::new(store, WorkerPool::new(config.num_workers));
        let actor_pool = ActorPool::new(context.clone());

        Self {
            context,
            actor_pool,
        }
    }

    pub async fn handle_event(&self, event: Event) -> Result<(), anyhow::Error> {
        self.actor_pool.send(event).await
    }

    pub async fn register_workflow(
        &self,
        input: RegisterWorkflowInput,
    ) -> Result<RegisteredWorkflowSummary, anyhow::Error> {
        register_workflow(&self.context.store, input).await
    }
}
