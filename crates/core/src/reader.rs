use crate::{
    Result,
    engine::EngineSnapshot,
    models::{WorkflowRunId, WorkflowRunStatus, WorkflowSchema},
    store::{StorageProvider, Store, keyspace::KeySpace},
    stream::StreamReader,
};

/// Reads the persisted state associated with a workflow run.
///
/// This reader does not coordinate with the running actor. Each method returns the state committed to the store at the time of that read.
#[derive(Clone)]
pub struct WorkflowRunReader<S: StorageProvider> {
    store: Store<S>,
    run_id: WorkflowRunId,
}

impl<S: StorageProvider> WorkflowRunReader<S> {
    pub fn new(store: Store<S>, run_id: WorkflowRunId) -> Self {
        Self { store, run_id }
    }

    pub fn run_id(&self) -> &WorkflowRunId {
        &self.run_id
    }

    /// Reads the schema stored for this run.
    ///
    /// Returns `None` when the run does not exist in the store.
    pub async fn schema(&self) -> Result<Option<WorkflowSchema>> {
        let key = KeySpace::run(&self.run_id).schema();
        self.store
            .get(&key)
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Reads the latest engine snapshot committed for this run.
    ///
    /// A run can have a stored schema before its first snapshot is committed, so
    /// `None` does not necessarily mean that the run does not exist.
    pub async fn snapshot(&self) -> Result<Option<EngineSnapshot>> {
        let key = KeySpace::run(&self.run_id).snapshot();
        self.store
            .get(&key)
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    /// Reads the status from the latest committed snapshot.
    pub async fn status(&self) -> Result<Option<WorkflowRunStatus>> {
        Ok(self.snapshot().await?.map(|snapshot| snapshot.state.status))
    }

    /// Creates a reader for the run's persisted event and command stream.
    pub fn stream(&self) -> StreamReader<S> {
        StreamReader::new(self.store.clone(), &self.run_id)
    }
}
