use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct WorkflowRun {
    #[specta(type = specta_typescript::Number)]
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
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct JobRun {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub public_id: String,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    pub job_id: String,
    pub status: String,
    #[specta(type = Option<specta_typescript::Number>)]
    pub duration_ms: Option<i64>,
    #[specta(type = specta_typescript::Number)]
    pub retry_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct Tag {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub job_run_id: Option<i64>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub data_reference_id: Option<i64>,
    pub value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct TauriDataReference {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub job_run_id: i64,
    pub uri: String,
    pub is_replay: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityKind {
    WorkflowRun,
    JobRun,
    Tag,
    DataReference,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RowChange<T> {
    Insert {
        row: T,
    },
    Update {
        row: T,
    },
    Delete {
        #[specta(type = specta_typescript::Number)]
        id: i64,
    },
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum SyncDelta {
    WorkflowRun {
        #[specta(type = specta_typescript::Number)]
        change_id: i64,
        change: RowChange<WorkflowRun>,
    },
    JobRun {
        #[specta(type = specta_typescript::Number)]
        change_id: i64,
        change: RowChange<JobRun>,
    },
    Tag {
        #[specta(type = specta_typescript::Number)]
        change_id: i64,
        change: RowChange<Tag>,
    },
    DataReference {
        #[specta(type = specta_typescript::Number)]
        change_id: i64,
        change: RowChange<TauriDataReference>,
    },
}
