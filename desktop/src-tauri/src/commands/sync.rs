use local::{Delta, SyncEntity, SyncSubscription, ZygoLocalService};
use serde::de::DeserializeOwned;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::{RowChange, SyncDelta};

fn deserialize_change<T: DeserializeOwned>(
    change: RowChange<serde_json::Value>,
) -> Result<RowChange<T>, serde_json::Error> {
    match change {
        RowChange::Insert { row } => Ok(RowChange::Insert {
            row: serde_json::from_value(row)?,
        }),
        RowChange::Update { row } => Ok(RowChange::Update {
            row: serde_json::from_value(row)?,
        }),
        RowChange::Delete { id } => Ok(RowChange::Delete { id }),
    }
}

impl TryFrom<Delta> for SyncDelta {
    type Error = serde_json::Error;

    fn try_from(delta: Delta) -> Result<Self, Self::Error> {
        let (entity, change_id, change) = match delta {
            Delta::Insert {
                change_id,
                entity,
                data,
            } => (entity, change_id, RowChange::Insert { row: data }),
            Delta::Update {
                change_id,
                entity,
                data,
            } => (entity, change_id, RowChange::Update { row: data }),
            Delta::Delete {
                change_id,
                entity,
                id,
            } => (entity, change_id, RowChange::Delete { id }),
        };

        let delta = match entity {
            SyncEntity::WorkflowRun => Self::WorkflowRun {
                change_id,
                change: deserialize_change(change)?,
            },
            SyncEntity::JobRun => Self::JobRun {
                change_id,
                change: deserialize_change(change)?,
            },
            SyncEntity::Tag => Self::Tag {
                change_id,
                change: deserialize_change(change)?,
            },
            SyncEntity::DataReference => Self::DataReference {
                change_id,
                change: deserialize_change(change)?,
            },
        };

        Ok(delta)
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
            let delta = SyncDelta::try_from(delta).map_err(|error| {
                CommandError::internal(
                    "invalid_sync_delta",
                    format!("failed to parse sync payload: {error}"),
                )
            })?;

            on_event.send(delta).map_err(|error| {
                CommandError::internal("sync_channel_failed", error.to_string())
            })?;
        }

        subscription.set_last_confirmed_change_id(max_change_id);
    }
}
