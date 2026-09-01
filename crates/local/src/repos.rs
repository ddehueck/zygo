use crate::db::{
    CdcRepository, JobRunSummaryRepository, KvRepository, WorkflowRunRepository,
    WorkflowRunSummaryRepository,
};

/// Repositories owned by a [`ZygoLocalService`](crate::ZygoLocalService).
///
/// All repositories share the service's database connection.
#[derive(Clone)]
pub struct Repos {
    pub cdc: CdcRepository,
    pub kv: KvRepository,
    pub workflow_runs: WorkflowRunRepository,
    pub workflow_run_summaries: WorkflowRunSummaryRepository,
    pub job_run_summaries: JobRunSummaryRepository,
}
