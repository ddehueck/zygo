use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::WorkflowRunSummary;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct ListWorkflowRunSummariesRequest {
    pub cursor: Option<String>,
    pub limit: u32,
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
) -> CommandResult<ListWorkflowRunSummariesResponse> {
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
    let mut summaries = state
        .repos
        .workflow_run_summaries
        .list_after_id(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| {
            CommandError::internal("list_workflow_run_summaries_failed", error.to_string())
        })?;

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
