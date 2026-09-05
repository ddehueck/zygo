use local::{Delta, SyncEntity, SyncSubscription, ZygoLocalService};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::SyncUpsert;

#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SyncEntityKind {
    WorkflowRun,
    JobRun,
    Tag,
}

#[derive(Debug, Serialize, Type)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum SyncDelta {
    Resync,
    Delete { entity: SyncEntityKind, id: String },
    Upsert { payload: SyncUpsert },
}

fn entity_kind(entity: SyncEntity) -> SyncEntityKind {
    match entity {
        SyncEntity::WorkflowRun => SyncEntityKind::WorkflowRun,
        SyncEntity::JobRun => SyncEntityKind::JobRun,
        SyncEntity::Tag => SyncEntityKind::Tag,
    }
}

impl TryFrom<Delta> for SyncDelta {
    type Error = serde_json::Error;

    fn try_from(delta: Delta) -> Result<Self, Self::Error> {
        match delta {
            Delta::Resync => Ok(Self::Resync),
            Delta::Delete { entity, id } => Ok(Self::Delete {
                entity: entity_kind(entity),
                id,
            }),
            Delta::Upsert { entity, id, data } => {
                let payload = match entity {
                    SyncEntity::WorkflowRun => SyncUpsert::WorkflowRun {
                        id,
                        data: serde_json::from_value(data)?,
                    },
                    SyncEntity::JobRun => {
                        let data = match data {
                            serde_json::Value::Object(mut data) => {
                                data.insert("id".to_owned(), serde_json::Value::String(id.clone()));
                                serde_json::Value::Object(data)
                            }
                            data => data,
                        };
                        SyncUpsert::JobRun {
                            id,
                            data: serde_json::from_value(data)?,
                        }
                    }
                    SyncEntity::Tag => SyncUpsert::Tag {
                        id,
                        data: serde_json::from_value(data)?,
                    },
                };

                Ok(Self::Upsert { payload })
            }
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
