use local::{Delta, SyncSubscription, ZygoLocalService};
use tauri::State;

use crate::error::{CommandError, CommandResult};

use super::{RowChange, SyncDelta};

#[tauri::command]
#[specta::specta]
pub async fn open_sync_channel(
    state: State<'_, ZygoLocalService>,
    on_event: tauri::ipc::Channel<SyncDelta>,
    on_ready: tauri::ipc::Channel<()>,
) -> CommandResult<()> {
    const MAX_DELTAS: usize = 1000;

    let subscription = SyncSubscription::new(state.repos.clone());
    subscription.load_last_change_id().await;
    // todo: we don't really need need to spawn. the polling can just run here.
    subscription.spawn().await;

    // The client must not start pagination until the stream cursor is fixed.
    // Otherwise, the client may miss deltas.
    on_ready
        .send(())
        .map_err(|error| CommandError::internal("sync_channel_failed", error.to_string()))?;

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
            let delta = SyncDelta::from(delta);

            on_event.send(delta).map_err(|error| {
                CommandError::internal("sync_channel_failed", error.to_string())
            })?;
        }

        subscription.set_last_confirmed_change_id(max_change_id);
    }
}

impl<M, T: From<M>> From<local::RowChange<M>> for RowChange<T> {
    fn from(change: local::RowChange<M>) -> Self {
        match change {
            local::RowChange::Insert { row } => Self::Insert { row: row.into() },
            local::RowChange::Update { row } => Self::Update { row: row.into() },
            local::RowChange::Delete { id } => Self::Delete { id },
        }
    }
}

impl From<Delta> for SyncDelta {
    fn from(delta: Delta) -> Self {
        match delta {
            Delta::WorkflowRun { change_id, change } => Self::WorkflowRun {
                change_id,
                change: change.into(),
            },
            Delta::JobRun { change_id, change } => Self::JobRun {
                change_id,
                change: change.into(),
            },
            Delta::Tag { change_id, change } => Self::Tag {
                change_id,
                change: change.into(),
            },
            Delta::DataReference { change_id, change } => Self::DataReference {
                change_id,
                change: change.into(),
            },
        }
    }
}
