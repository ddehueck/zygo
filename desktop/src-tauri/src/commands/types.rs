use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct Workflow {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub name: String,
    pub path: String,
    pub schema: String,
    pub created_at: String,
}

impl From<local::WorkflowModel> for Workflow {
    fn from(workflow: local::WorkflowModel) -> Self {
        Self {
            id: workflow.id,
            name: workflow.name,
            path: workflow.path,
            schema: workflow.schema,
            created_at: workflow.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Type)]
pub struct WorkflowRun {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub public_id: String,
    #[specta(type = specta_typescript::Number)]
    pub workflow_id: i64,
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

impl From<local::WorkflowRunModel> for WorkflowRun {
    fn from(run: local::WorkflowRunModel) -> Self {
        Self {
            id: run.id,
            public_id: run.public_id,
            workflow_id: run.workflow_id,
            status: run.status,
            started_at: run.started_at,
            completed_at: run.completed_at,
            active_job_count: run.active_job_count,
            succeeded_job_count: run.succeeded_job_count,
            errored_job_count: run.errored_job_count,
            created_at: run.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct JobRun {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    pub public_id: String,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    #[specta(type = specta_typescript::Number)]
    pub input_id: i64,
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
pub struct Log {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    pub job_run_id: String,
    pub content: String,
    pub created_at: String,
}

impl From<local::LogRow> for Log {
    fn from(log: local::LogRow) -> Self {
        Self {
            id: log.id,
            workflow_run_id: log.workflow_run_id,
            job_run_id: log.job_run_id,
            content: log.content,
            created_at: log.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct TauriDataReference {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    #[specta(type = Option<specta_typescript::Number>)]
    pub source_job_run_id: Option<i64>,
    pub uri: String,
    pub is_replay: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityKind {
    Workflow,
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
    Workflow {
        #[specta(type = specta_typescript::Number)]
        change_id: i64,
        change: RowChange<Workflow>,
    },
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
