use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct ListWorkflowRunSummariesRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
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

#[derive(Debug, Serialize, Type)]
pub struct ListWorkflowRunSummariesResponse {
    pub summaries: Vec<WorkflowRunSummary>,
    pub next_cursor: Option<String>,
}

impl From<local::WorkflowRunSummaryRow> for WorkflowRunSummary {
    fn from(summary: local::WorkflowRunSummaryRow) -> Self {
        Self {
            workflow_run_id: summary.workflow_run_id,
            status: summary.status,
            started_at: summary.started_at,
            completed_at: summary.completed_at,
            active_job_count: summary.active_job_count,
            succeeded_job_count: summary.succeeded_job_count,
            errored_job_count: summary.errored_job_count,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn list_workflow_run_summaries(
    state: State<'_, ZygoLocalService>,
    request: ListWorkflowRunSummariesRequest,
) -> Result<ListWorkflowRunSummariesResponse, String> {
    if request.limit == 0 {
        return Err("limit must be greater than zero".to_owned());
    }
    if request.limit > MAX_PAGE_SIZE {
        return Err(format!("limit must not be greater than {MAX_PAGE_SIZE}"));
    }

    let limit = request.limit;
    let mut summaries = state
        .list_workflow_run_summaries(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| error.to_string())?;

    let has_more = summaries.len() > limit as usize;
    if has_more {
        summaries.pop();
    }

    let next_cursor = has_more
        .then(|| {
            summaries
                .last()
                .map(|summary| summary.workflow_run_id.clone())
        })
        .flatten();

    Ok(ListWorkflowRunSummariesResponse {
        summaries: summaries
            .into_iter()
            .map(WorkflowRunSummary::from)
            .collect(),
        next_cursor,
    })
}
