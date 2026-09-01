use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::JobRunSummary;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct ListJobRunSummariesRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
pub struct ListJobRunSummariesResponse {
    pub summaries: Vec<JobRunSummary>,
    pub next_cursor: Option<String>,
}

impl From<local::JobRunSummaryRow> for JobRunSummary {
    fn from(summary: local::JobRunSummaryRow) -> Self {
        Self {
            id: summary.row_id.to_string(),
            workflow_run_id: summary.workflow_run_id,
            job_run_id: summary.job_run_id,
            job_id: summary.job_id,
            status: summary.status,
            duration_ms: summary.duration_ms,
            retry_count: summary.retry_count,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn list_job_run_summaries(
    state: State<'_, ZygoLocalService>,
    request: ListJobRunSummariesRequest,
) -> CommandResult<ListJobRunSummariesResponse> {
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
        .job_run_summaries
        .list_after_id(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| {
            CommandError::internal("list_job_run_summaries_failed", error.to_string())
        })?;

    let has_more = summaries.len() > limit as usize;
    if has_more {
        summaries.pop();
    }

    let next_cursor = has_more
        .then(|| summaries.last().map(|summary| summary.row_id.to_string()))
        .flatten();

    Ok(ListJobRunSummariesResponse {
        summaries: summaries.into_iter().map(JobRunSummary::from).collect(),
        next_cursor,
    })
}
