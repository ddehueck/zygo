use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::JobRun;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct ListJobRunsRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
pub struct ListJobRunsResponse {
    pub runs: Vec<JobRun>,
    pub next_cursor: Option<String>,
}

impl From<local::JobRunRow> for JobRun {
    fn from(run: local::JobRunRow) -> Self {
        Self {
            id: run.id,
            workflow_run_id: run.workflow_run_id,
            job_id: run.job_id,
            status: run.status,
            duration_ms: run.duration_ms,
            retry_count: run.retry_count,
            created_at: run.created_at,
            updated_at: run.updated_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn list_job_runs(
    state: State<'_, ZygoLocalService>,
    request: ListJobRunsRequest,
) -> CommandResult<ListJobRunsResponse> {
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
        .job_runs
        .list_after_id(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| CommandError::internal("list_job_runs_failed", error.to_string()))?;

    let has_more = runs.len() > limit as usize;
    if has_more {
        runs.pop();
    }

    let next_cursor = has_more
        .then(|| runs.last().map(|run| run.id.clone()))
        .flatten();

    Ok(ListJobRunsResponse {
        runs: runs.into_iter().map(JobRun::from).collect(),
        next_cursor,
    })
}
