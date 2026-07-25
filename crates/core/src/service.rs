use crate::{
    actors::ActorPool,
    context::ServiceContext,
    models::{DataReference, WorkflowRunId, WorkflowSchema},
    store::{StorageProvider, Store},
    workers::WorkerPool,
};

pub struct Zygo<S: StorageProvider> {
    actor_pool: ActorPool<S>,
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

impl<S: StorageProvider> Zygo<S> {
    pub fn new(store: Store<S>, config: ZygoConfig) -> Self {
        let context = ServiceContext::new(store, WorkerPool::new(config.num_workers));
        let actor_pool = ActorPool::new(context.clone());

        Self { actor_pool }
    }

    pub async fn run(
        &self,
        input: DataReference,
        schema: WorkflowSchema,
    ) -> Result<WorkflowRunId, anyhow::Error> {
        self.actor_pool.run_with_actor(input, schema).await
    }
}
