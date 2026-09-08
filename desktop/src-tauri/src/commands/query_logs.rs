use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::Log;
use crate::error::{CommandError, CommandResult};

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct QueryLogsRequest {
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    pub limit: u32,
    /// Exclusive lower bound. Present (including 0) pages forward (ASC); absent pages from the newest (DESC).
    #[serde(default)]
    #[specta(type = Option<specta_typescript::Number>)]
    pub after_id: Option<i64>,
    /// Exclusive upper bound for paging older history (DESC).
    #[serde(default)]
    #[specta(type = Option<specta_typescript::Number>)]
    pub before_id: Option<i64>,
}

#[derive(Serialize, Type)]
pub struct QueryLogsResponse {
    pub logs: Vec<Log>,
    pub has_more: bool,
    /// Global watermark log ID from the same read snapshot as the page.
    #[specta(type = specta_typescript::Number)]
    pub global_watermark_id: i64,
}

#[tauri::command]
#[specta::specta]
pub async fn query_logs(
    state: State<'_, ZygoLocalService>,
    request: QueryLogsRequest,
) -> CommandResult<QueryLogsResponse> {
    if request.workflow_run_id <= 0 {
        return Err(CommandError::invalid_input(
            "workflow_run_id",
            "must be greater than zero",
        ));
    }
    if request.limit == 0 || request.limit > MAX_PAGE_SIZE {
        return Err(CommandError::invalid_input(
            "limit",
            format!("must be between 1 and {MAX_PAGE_SIZE}"),
        ));
    }
    if request.after_id.is_some_and(|id| id < 0)
        || request.before_id.is_some_and(|id| id <= 0)
        || matches!((request.after_id, request.before_id), (Some(after), Some(before)) if after >= before)
    {
        return Err(CommandError::invalid_input(
            "bounds",
            "must be nonnegative, ordered exclusive ID bounds",
        ));
    }
    // after_id present (even 0) → ASC tail; otherwise DESC from newest / before_id.
    let ascending = request.after_id.is_some();
    let (mut rows, global_watermark_id) = state
        .repos
        .logs
        .page_by_workflow_run_id(
            request.workflow_run_id,
            request.after_id,
            request.before_id,
            ascending,
            request.limit + 1,
        )
        .await
        .map_err(|error| CommandError::internal("query_logs_failed", error.to_string()))?;
    let has_more = rows.len() > request.limit as usize;
    rows.truncate(request.limit as usize);
    // Every response is in ingestion order, regardless of traversal direction.
    rows.sort_by_key(|row| row.id);
    Ok(QueryLogsResponse {
        logs: rows.into_iter().map(Log::from).collect(),
        has_more,
        global_watermark_id,
    })
}
