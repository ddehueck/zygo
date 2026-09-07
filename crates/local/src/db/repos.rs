mod cdc;
mod data_references;
mod job_runs;
mod kv;
mod logs;
mod paginator;

pub use paginator::{Cursor, CursorPaginator, Page};
mod tags;
mod workflow_runs;

pub use self::{
    cdc::CdcRepository,
    data_references::DataReferenceRepository,
    job_runs::JobRunRepository,
    kv::KvRepository,
    logs::{LogRow, LogsRepository},
    tags::TagsRepository,
    workflow_runs::WorkflowRunRepository,
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
