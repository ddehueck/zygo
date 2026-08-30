use local::{Delta, SyncEntity, SyncSubscription};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

#[derive(Debug, Deserialize, Type)]
pub struct ConfirmSyncRequest {
    #[specta(type = specta_typescript::Number)]
    pub change_id: i64,
}

#[derive(Debug, Deserialize, Type)]
pub struct GetSyncDeltasRequest {
    #[specta(type = specta_typescript::Number)]
    pub since: i64,
    pub max_deltas: u32,
}

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityKind {
    WorkflowRunSummary,
    WorkflowRun,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum SyncDelta {
    Resync,
    Delete {
        entity: SyncEntityKind,
    },
    Upsert {
        entity: SyncEntityKind,
        #[specta(type = specta_typescript::Unknown)]
        data: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Type)]
pub struct GetSyncDeltasResponse {
    pub deltas: Vec<SyncDelta>,
    #[specta(type = Option<specta_typescript::Number>)]
    pub next_change_id: Option<i64>,
}

fn entity_kind(entity: SyncEntity) -> SyncEntityKind {
    match entity {
        SyncEntity::WorkflowRunSummary => SyncEntityKind::WorkflowRunSummary,
        SyncEntity::WorkflowRun => SyncEntityKind::WorkflowRun,
    }
}

impl From<Delta> for SyncDelta {
    fn from(delta: Delta) -> Self {
        match delta {
            Delta::Resync => Self::Resync,
            Delta::Delete { entity } => Self::Delete {
                entity: entity_kind(entity),
            },
            Delta::Upsert { entity, data } => Self::Upsert {
                entity: entity_kind(entity),
                data,
            },
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_sync_deltas(
    state: State<'_, SyncSubscription>,
    request: GetSyncDeltasRequest,
) -> Result<GetSyncDeltasResponse, String> {
    const MAX_DELTAS: u32 = 1000;

    if request.since < 0 {
        return Err("sync change ID cannot be negative".to_owned());
    }
    if request.max_deltas == 0 || request.max_deltas > MAX_DELTAS {
        return Err(format!("max_deltas must be between 1 and {MAX_DELTAS}"));
    }

    let deltas = state
        .get_deltas(request.since, request.max_deltas as usize)
        .await
        .map_err(|error| error.to_string())?;
    let needs_resync = deltas.iter().any(|delta| matches!(delta, Delta::Resync));

    Ok(GetSyncDeltasResponse {
        deltas: deltas.into_iter().map(SyncDelta::from).collect(),
        next_change_id: (!needs_resync).then_some(request.since + i64::from(request.max_deltas)),
    })
}

#[tauri::command]
#[specta::specta]
pub fn confirm_sync(
    state: State<'_, SyncSubscription>,
    request: ConfirmSyncRequest,
) -> Result<(), String> {
    if request.change_id < 0 {
        return Err("sync change ID cannot be negative".to_owned());
    }

    state.set_last_confirmed_change_id(request.change_id);
    Ok(())
}
