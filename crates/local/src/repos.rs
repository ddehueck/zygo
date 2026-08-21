use crate::db::{KvRepository, WorkflowRunRepository};

/// Repositories owned by a [`ZygoLocalService`](crate::ZygoLocalService).
///
/// All repositories share the service's database connection.
#[derive(Clone)]
pub struct Repos {
    pub kv: KvRepository,
    pub workflow_runs: WorkflowRunRepository,
}
