use local::ZygoLocalService;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::{DataReference, JobRun, Tag, WorkflowRun};

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Deserialize, Type)]
pub struct LoadDataRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
pub struct LoadDataResponse {
    pub workflow_runs: Vec<WorkflowRun>,
    pub job_runs: Vec<JobRun>,
    pub tags: Vec<Tag>,
    pub data_references: Vec<DataReference>,
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

impl From<local::DataReferenceRow> for DataReference {
    fn from(reference: local::DataReferenceRow) -> Self {
        Self {
            id: reference.id.to_string(),
            workflow_run_id: reference.workflow_run_id,
            job_run_id: reference.job_run_id,
            job_id: reference.job_id,
            uri: reference.uri,
            version: reference.version,
            is_replay: reference.is_replay,
            inserted_at: reference.inserted_at,
            created_at: reference.created_at,
        }
    }
}

impl From<local::TagRow> for Tag {
    fn from(tag: local::TagRow) -> Self {
        Self {
            id: tag.id.to_string(),
            workflow_run_id: tag.workflow_run_id,
            key: tag.key,
            value: tag.value,
            created_at: tag.created_at,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn load_data(
    state: State<'_, ZygoLocalService>,
    request: LoadDataRequest,
) -> CommandResult<LoadDataResponse> {
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
    let mut workflow_runs = state
        .repos
        .workflow_runs
        .list_after_id(request.cursor.as_deref(), limit + 1)
        .await
        .map_err(|error| CommandError::internal("load_data_failed", error.to_string()))?;

    let has_more = workflow_runs.len() > limit as usize;
    if has_more {
        workflow_runs.pop();
    }

    let next_cursor = has_more
        .then(|| workflow_runs.last().map(|run| run.id.clone()))
        .flatten();
    let workflow_run_ids = workflow_runs
        .iter()
        .map(|run| run.id.clone())
        .collect::<Vec<_>>();

    let job_runs = state
        .repos
        .job_runs
        .list_by_workflow_run_ids(&workflow_run_ids)
        .await
        .map_err(|error| CommandError::internal("load_data_failed", error.to_string()))?;
    let tags = state
        .repos
        .tags
        .list_by_workflow_run_ids(&workflow_run_ids)
        .await
        .map_err(|error| CommandError::internal("load_data_failed", error.to_string()))?;
    let data_references = state
        .repos
        .data_references
        .list_by_workflow_run_ids(&workflow_run_ids)
        .await
        .map_err(|error| CommandError::internal("load_data_failed", error.to_string()))?;

    Ok(LoadDataResponse {
        workflow_runs: workflow_runs.into_iter().map(WorkflowRun::from).collect(),
        job_runs: job_runs.into_iter().map(JobRun::from).collect(),
        tags: tags.into_iter().map(Tag::from).collect(),
        data_references: data_references
            .into_iter()
            .map(DataReference::from)
            .collect(),
        next_cursor,
    })
}
