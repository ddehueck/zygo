use crate::models::{DataReferenceUri, Event, JobId, JobRunId, WorkflowRunId, WorkflowSchema};

pub struct RunJobArgs {
    pub input: DataReferenceUri,
    pub job_id: JobId,
    pub workflow_run_id: WorkflowRunId,
    pub job_run_id: JobRunId,
}

// Language-agnostic workflow API interface
pub trait WorkflowApiInterface {
    fn get_schema(&self) -> Result<WorkflowSchema, Box<dyn std::error::Error>>;
    fn run_job(&self, args: RunJobArgs) -> Result<RunJobResult, Box<dyn std::error::Error>>;
}

pub enum RunJobResult {
    Enqueued,
    Cached { events: Vec<Event> },
}
