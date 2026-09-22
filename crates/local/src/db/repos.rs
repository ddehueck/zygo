mod cdc;
mod data_references;
mod event_stream;
mod job_runs;

mod logs;
mod paginator;

pub use paginator::{Cursor, CursorPaginator, Page};
mod tags;
mod workflow_runs;
mod workflows;

pub use self::{
    cdc::CdcRepository,
    data_references::DataReferenceRepository,
    event_stream::EventStreamRepository,
    job_runs::JobRunRepository,
    logs::{LogRow, LogsRepository},
    tags::TagsRepository,
    workflow_runs::WorkflowRunRepository,
    workflows::WorkflowRepository,
};

/// Repositories owned by a [`ZygoLocalService`](crate::ZygoLocalService).
///
/// All repositories share the service's database connection.
#[derive(Clone)]
pub struct Repos {
    pub cdc: CdcRepository,
    pub events: EventStreamRepository,

    pub tags: TagsRepository,
    pub data_references: DataReferenceRepository,
    pub workflow_runs: WorkflowRunRepository,
    pub workflows: WorkflowRepository,
    pub job_runs: JobRunRepository,
    pub logs: LogsRepository,
}
