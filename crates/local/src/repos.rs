use crate::db::{
    CdcRepository, DataReferenceRepository, JobRunRepository, KvRepository, LogsRepository,
    TagsRepository, WorkflowRunRepository,
};

/// Repositories owned by a [`ZygoLocalService`](crate::ZygoLocalService).
///
/// All repositories share the service's database connection.
#[derive(Clone)]
pub struct Repos {
    pub cdc: CdcRepository,
    pub kv: KvRepository,
    pub tags: TagsRepository,
    pub data_references: DataReferenceRepository,
    pub workflow_runs: WorkflowRunRepository,
    pub job_runs: JobRunRepository,
    pub logs: LogsRepository,
}
