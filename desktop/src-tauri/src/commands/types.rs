use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct WorkflowRun {
    pub id: i64,
    pub public_id: String,
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
    pub id: i64,
    pub public_id: String,
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
pub struct Tag {
    pub id: i64,
    pub workflow_run_id: String,
    pub job_run_id: Option<String>,
    pub data_reference_id: Option<i64>,
    pub value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct DataReference {
    pub id: i64,
    pub workflow_run_id: String,
    pub job_run_id: String,
    pub job_id: String,
    pub uri: String,
    pub version: String,
    pub is_replay: bool,
    pub inserted_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RowChange<T> {
    Insert { row: T },
    Update { row: T },
    Delete { id: i64 },
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum SyncDelta {
    WorkflowRun {
        change_id: i64,
        change: RowChange<WorkflowRun>,
    },
    JobRun {
        change_id: i64,
        change: RowChange<JobRun>,
    },
    Tag {
        change_id: i64,
        change: RowChange<Tag>,
    },
    DataReference {
        change_id: i64,
        change: RowChange<DataReference>,
    },
}
