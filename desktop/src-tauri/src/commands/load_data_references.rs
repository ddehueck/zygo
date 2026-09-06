use local::ZygoLocalService;
use serde::Deserialize;
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::DataReference;

#[derive(Debug, Deserialize, Type)]
pub struct LoadDataReferencesRequest {
    pub workflow_run_id: String,
    pub job_run_id: String,
}

#[tauri::command]
#[specta::specta]
pub async fn load_data_references(
    state: State<'_, ZygoLocalService>,
    request: LoadDataReferencesRequest,
) -> CommandResult<Vec<DataReference>> {
    if request.workflow_run_id.is_empty() || request.job_run_id.is_empty() {
        return Err(CommandError::invalid_input(
            "ids",
            "workflow_run_id and job_run_id must not be empty",
        ));
    }

    let references = state
        .repos
        .data_references
        .list_by_job_run(&request.workflow_run_id, &request.job_run_id)
        .await
        .map_err(|error| {
            CommandError::internal("load_data_references_failed", error.to_string())
        })?;

    Ok(references.into_iter().map(DataReference::from).collect())
}
