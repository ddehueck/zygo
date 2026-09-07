use local::{CursorPaginator, ZygoLocalService};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::{JobRun, SyncEntityKind, Tag, TauriDataReference, WorkflowRun};

const MAX_PAGE_SIZE: u32 = 1000;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Type)]
pub struct SyncCursor {
    #[specta(type = specta_typescript::Number)]
    pub id: i64,
}

#[derive(Debug, Deserialize, Type)]
pub struct LoadSyncableDataRequest {
    pub entity: SyncEntityKind,
    pub cursor: Option<SyncCursor>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Type)]
pub struct SyncPage<T> {
    pub next: Option<SyncCursor>,
    pub data: Vec<T>,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "entity", rename_all = "snake_case")]
pub enum LoadSyncableDataResponse {
    WorkflowRun { page: SyncPage<WorkflowRun> },
    JobRun { page: SyncPage<JobRun> },
    Tag { page: SyncPage<Tag> },
    DataReference { page: SyncPage<TauriDataReference> },
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

impl From<local::JobRunModel> for JobRun {
    fn from(run: local::JobRunModel) -> Self {
        Self {
            id: run.id,
            public_id: run.public_id,
            workflow_run_id: run.workflow_run_id,
            job_id: run.job_id,
            status: run.status,
            duration_ms: run.duration_ms,
            retry_count: run.retry_count,
            created_at: run.created_at,
        }
    }
}

impl From<local::DataReferenceModel> for TauriDataReference {
    fn from(reference: local::DataReferenceModel) -> Self {
        Self {
            id: reference.id,
            workflow_run_id: reference.workflow_run_id,
            job_run_id: reference.job_run_id,
            uri: reference.uri,
            is_replay: reference.is_replay,
            created_at: reference.created_at,
        }
    }
}

impl From<local::TagModel> for Tag {
    fn from(tag: local::TagModel) -> Self {
        Self {
            id: tag.id,
            workflow_run_id: tag.workflow_run_id,
            job_run_id: tag.job_run_id,
            data_reference_id: tag.data_reference_id,
            value: tag.value,
            created_at: tag.created_at,
        }
    }
}

async fn load_page<R, T>(
    repo: &R,
    cursor: Option<local::Cursor>,
    limit: i64,
) -> CommandResult<SyncPage<T>>
where
    R: CursorPaginator,
    T: From<R::Item>,
{
    let page = repo
        .list(cursor, limit)
        .await
        .map_err(|error| CommandError::internal("load_syncable_data_failed", error.to_string()))?;

    Ok(SyncPage {
        next: page.next.map(|cursor| SyncCursor { id: cursor.id }),
        data: page.data.into_iter().map(T::from).collect(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn load_syncable_data(
    state: State<'_, ZygoLocalService>,
    request: LoadSyncableDataRequest,
) -> CommandResult<LoadSyncableDataResponse> {
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

    let cursor = request.cursor.map(|cursor| local::Cursor { id: cursor.id });
    let limit = i64::from(request.limit);
    let repos = &state.repos;

    let response = match request.entity {
        SyncEntityKind::WorkflowRun => LoadSyncableDataResponse::WorkflowRun {
            page: load_page(&repos.workflow_runs, cursor, limit).await?,
        },
        SyncEntityKind::JobRun => LoadSyncableDataResponse::JobRun {
            page: load_page(&repos.job_runs, cursor, limit).await?,
        },
        SyncEntityKind::Tag => LoadSyncableDataResponse::Tag {
            page: load_page(&repos.tags, cursor, limit).await?,
        },
        SyncEntityKind::DataReference => LoadSyncableDataResponse::DataReference {
            page: load_page(&repos.data_references, cursor, limit).await?,
        },
    };

    Ok(response)
}
