use crate::{
    actors::ActorPool,
    context::ServiceContext,
    engine::EngineSnapshot,
    models::{DataReference, WorkflowRunId, WorkflowSchema},
    store::{StorageProvider, Store, keyspace::KeySpace},
    stream::StreamReader,
    workers::WorkerPool,
};

pub struct Zygo<S: StorageProvider> {
    actor_pool: ActorPool<S>,
    store: Store<S>,
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
        let context = ServiceContext::new(store.clone(), WorkerPool::new(config.num_workers));
        let actor_pool = ActorPool::new(context.clone());

        Self { actor_pool, store }
    }

    pub async fn run(
        &self,
        id: WorkflowRunId,
        input: DataReference,
        schema: WorkflowSchema,
    ) -> Result<(), anyhow::Error> {
        self.actor_pool.run_with_actor(id, input, schema).await
    }

    pub async fn subscribe(
        &self,
        run_id: &WorkflowRunId,
    ) -> Result<tokio::sync::watch::Receiver<EngineSnapshot>, anyhow::Error> {
        self.actor_pool.subscribe(run_id).await
    }

    pub fn stream(&self, run_id: &WorkflowRunId) -> StreamReader<S> {
        StreamReader::new(self.store.clone(), run_id)
    }

    pub async fn workflow_schema(
        &self,
        run_id: &WorkflowRunId,
    ) -> Result<Option<WorkflowSchema>, anyhow::Error> {
        let schema_value = self.store.get(&KeySpace::run(run_id).schema()).await?;

        schema_value
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }
}
