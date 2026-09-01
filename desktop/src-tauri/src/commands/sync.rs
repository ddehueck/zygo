use local::{Delta, SyncEntity, SyncSubscription, ZygoLocalService};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityKind {
    WorkflowRunSummary,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum SyncDelta {
    Resync,
    Delete {
        entity: SyncEntityKind,
        id: String,
    },
    Upsert {
        entity: SyncEntityKind,
        id: String,
        #[specta(type = specta_typescript::Unknown)]
        data: serde_json::Value,
    },
}

fn entity_kind(entity: SyncEntity) -> SyncEntityKind {
    match entity {
        SyncEntity::WorkflowRunSummary => SyncEntityKind::WorkflowRunSummary,
    }
}

impl From<Delta> for SyncDelta {
    fn from(delta: Delta) -> Self {
        match delta {
            Delta::Resync => Self::Resync,
            Delta::Delete { entity, id } => Self::Delete {
                entity: entity_kind(entity),
                id,
            },
            Delta::Upsert { entity, id, data } => Self::Upsert {
                entity: entity_kind(entity),
                id,
                data,
            },
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn sync(
    state: State<'_, ZygoLocalService>,
    on_event: tauri::ipc::Channel<SyncDelta>,
) -> CommandResult<()> {
    const MAX_DELTAS: usize = 1000;

    let subscription = SyncSubscription::new(state.repos.clone());
    subscription.load_last_change_id().await;
    // todo: we don't really need need to spawn. the polling can just run here.
    subscription.spawn().await;

    loop {
        subscription.wait_for_changes().await;

        // Get the delta payload and send each delta to the tauri channel.
        // The high-water mark is confirmed only after the whole batch has
        // been delivered, so a failed channel send can be retried safely.
        let batch = subscription
            .next_delta_batch(MAX_DELTAS)
            .await
            .map_err(|error| CommandError::internal("sync_deltas_failed", error.to_string()))?;
        let max_change_id = batch.max_change_id;

        for delta in batch.deltas {
            on_event.send(delta.into()).map_err(|error| {
                CommandError::internal("sync_channel_failed", error.to_string())
            })?;
        }

        subscription.set_last_confirmed_change_id(max_change_id);
    }
}
