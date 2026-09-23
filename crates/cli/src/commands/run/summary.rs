use local::{JobRunModel, WorkflowRunModel};

pub struct WorkflowRunSummary {
    pub workflow_id: String,
    pub workflow_status: String,
    pub job_runs: Vec<JobRunSummary>,
}

pub struct JobRunSummary {
    pub job_id: String,
    pub public_id: String,
    pub status: String,
    pub created_at: String,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
}

impl WorkflowRunSummary {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            workflow_status: "running".to_owned(),
            job_runs: Vec::new(),
        }
    }

    pub fn from_models(
        workflow_id: String,
        workflow_run: WorkflowRunModel,
        job_runs: Vec<JobRunModel>,
    ) -> Self {
        Self {
            workflow_id,
            workflow_status: workflow_run.status,
            job_runs: job_runs
                .into_iter()
                .map(|job_run| JobRunSummary {
                    job_id: job_run.job_id,
                    public_id: job_run.public_id,
                    status: job_run.status,
                    created_at: job_run.created_at,
                    duration_ms: job_run.duration_ms,
                    error_message: job_run.error_message,
                })
                .collect(),
        }
    }
}
