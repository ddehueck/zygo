use local::{LogWatcher, ZygoLocalService};
use serde::Serialize;
use specta::Type;
use tauri::ipc::Channel;
use tauri::{Manager, Resource, State, Webview};
use zygo_core::models::JobRunId;

use crate::error::{CommandError, CommandResult};

#[derive(Serialize, Type)]
pub struct LogBatch {
    content: String,
    error: Option<String>,
}

struct LogSubscription {
    task: tauri::async_runtime::JoinHandle<()>,
}

impl Resource for LogSubscription {}

impl Drop for LogSubscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[tauri::command]
#[specta::specta]
pub fn watch_logs(
    state: State<'_, ZygoLocalService>,
    webview: Webview,
    job_run_id: String,
    on_batch: Channel<LogBatch>,
) -> CommandResult<u32> {
    let job_run_id = JobRunId::try_from(job_run_id)
        .map_err(|error| CommandError::invalid_input("job_run_id", error.to_string()))?;
    let mut watcher = LogWatcher::new(state.repos.logs.clone(), job_run_id);
    let task = tauri::async_runtime::spawn(async move {
        loop {
            let batch = match watcher.next_batch().await {
                Ok(rows) => LogBatch {
                    content: rows.into_iter().map(|row| row.content).collect(),
                    error: None,
                },
                Err(error) => LogBatch {
                    content: String::new(),
                    error: Some(format!("Could not read logs: {error}")),
                },
            };
            if on_batch.send(batch).is_err() {
                break;
            }
        }
    });

    // Closing the frontend resource (or its webview) drops and stops the watcher.
    Ok(webview.resources_table().add(LogSubscription { task }))
}
