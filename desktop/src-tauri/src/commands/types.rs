use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
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
pub struct JobRun {
    pub id: String,
    pub workflow_run_id: String,
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
    WorkflowRun { id: String, data: WorkflowRun },
    JobRun { id: String, data: JobRun },
}
