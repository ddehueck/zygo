use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::WorkflowRun;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct ListWorkflowRunsRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
pub struct ListWorkflowRunsResponse {
    pub runs: Vec<WorkflowRun>,
    pub next_cursor: Option<String>,
}

impl From<local::WorkflowRunRow> for WorkflowRun {
    fn from(run: local::WorkflowRunRow) -> Self {
        Self {
            id: run.id,
            workflow_id: run.workflow_id,
            status: run.status,
            started_at: run.started_at,
            completed_at: run.completed_at,
            active_job_count: run.active_job_count,
            succeeded_job_count: run.succeeded_job_count,
            errored_job_count: run.errored_job_count,
            created_at: run.created_at,
            updated_at: run.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn list_workflow_runs(
    state: State<'_, ZygoLocalService>,
    request: ListWorkflowRunsRequest,
) -> CommandResult<ListWorkflowRunsResponse> {
    if request.limit == 0 {
        return Err(CommandError::invalid_input(
            "limit",
            "must be greater than zero",
        ));
    }
    if request.limit > MAX_PAGE_SIZE {
        return Err(CommandError::invalid_input(
            "limit",
            format!("must not be greater than {MAX_PAGE_SIZE}"),
        ));
    }

    let limit = request.limit;
    let mut runs = state
        .repos
        .workflow_runs
        .list_after_id(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| CommandError::internal("list_workflow_runs_failed", error.to_string()))?;

    let has_more = runs.len() > limit as usize;
    if has_more {
        runs.pop();
    }

    let next_cursor = has_more
        .then(|| runs.last().map(|run| run.id.clone()))
        .flatten();

    Ok(ListWorkflowRunsResponse {
        runs: runs.into_iter().map(WorkflowRun::from).collect(),
        next_cursor,
    })
}
