use local::ZygoLocalService;
use serde::Deserialize;
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::types::Log;

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct QueryLogsRequest {
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    pub limit: u32,
    pub offset: u32,
    #[specta(type = Option<specta_typescript::Number>)]
    pub after_id: Option<i64>,
}

#[tauri::command]
#[specta::specta]
pub async fn query_logs(
    state: State<'_, ZygoLocalService>,
    request: QueryLogsRequest,
) -> CommandResult<Vec<Log>> {
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

    let rows = match request.after_id {
        Some(after_id) => {
            if after_id < 0 {
                return Err(CommandError::invalid_input(
                    "after_id",
                    "must be greater than or equal to zero",
                ));
            }
            if request.offset != 0 {
                return Err(CommandError::invalid_input(
                    "offset",
                    "must be zero when after_id is provided",
                ));
            }
            state
                .repos
                .logs
                .list_after_by_workflow_run_id(request.workflow_run_id, after_id, request.limit)
                .await
        }
        None => {
            state
                .repos
                .logs
                .list_by_workflow_run_id(request.workflow_run_id, request.offset, request.limit)
                .await
        }
    };

    rows.map(|rows| rows.into_iter().map(Log::from).collect())
        .map_err(|error| CommandError::internal("query_logs_failed", error.to_string()))
}
