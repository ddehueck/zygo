use local::SyncSubscription;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter, Wry};

pub const SYNC_POKE_EVENT: &str = "sync-poke";

#[derive(Debug, Clone, Serialize, Type)]
pub struct SyncPoke {}

/// Starts the local CDC poller and forwards each available sync notification
/// to the webview. Polling continues independently of frontend confirmation.
pub fn spawn_sync_poke_emitter(app: AppHandle<Wry>, subscription: SyncSubscription) {
    tauri::async_runtime::spawn(async move {
        subscription.spawn().await;

        loop {
            subscription.wait_for_changes().await;

            if app.emit(SYNC_POKE_EVENT, SyncPoke {}).is_err() {
                break;
            }
        }
    });
}
