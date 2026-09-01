use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct WorkflowRunSummary {
    pub workflow_run_id: String,
    pub status: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub started_at: Option<i64>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub completed_at: Option<i64>,
    #[specta(type = specta_typescript::Number)]
    pub active_job_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub succeeded_job_count: i64,
    #[specta(type = specta_typescript::Number)]
    pub errored_job_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct JobRunSummary {
    pub id: String,
    pub workflow_run_id: String,
    pub job_run_id: String,
    pub job_id: String,
    pub status: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub duration_ms: Option<i64>,
    #[specta(type = specta_typescript::Number)]
    pub retry_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum SyncUpsert {
    WorkflowRunSummary {
        id: String,
        data: WorkflowRunSummary,
    },
    JobRunSummary {
        id: String,
        data: JobRunSummary,
    },
}
