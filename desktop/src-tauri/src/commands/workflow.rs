use std::fs;
use std::path::{Path, PathBuf};

use local::{ZygoLocalRun, ZygoLocalService};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use zygo_core::engine::RunCursor;
use zygo_core::models::{ChannelItemInsertedData, DataReference, FileExtension};

use crate::error::{CommandError, CommandResult};

const STREAM_RECORD_BATCH_SIZE: usize = 64;

#[derive(Debug, Deserialize, Type)]
pub struct StartWorkflowRunRequest {
    #[specta(type = specta_typescript::Number)]
    pub workflow_id: i64,
    pub input_paths: Vec<String>,
}

#[derive(Debug, Serialize, Type)]
pub struct StartWorkflowRunResponse {
    #[specta(type = specta_typescript::Number)]
    pub workflow_run_id: i64,
    pub public_id: String,
}

#[tauri::command]
#[specta::specta]
pub async fn start_workflow_run(
    state: State<'_, ZygoLocalService>,
    request: StartWorkflowRunRequest,
) -> CommandResult<StartWorkflowRunResponse> {
    if request.input_paths.is_empty() {
        return Err(CommandError::invalid_input(
            "input_paths",
            "at least one input path is required",
        ));
    }

    let workflow = state.load(request.workflow_id).await.map_err(|error| {
        if error.to_string().contains("was not found") {
            CommandError::invalid_input("workflow_id", error.to_string())
        } else {
            CommandError::internal("load_workflow_failed", error.to_string())
        }
    })?;

    let accepted_file_extensions = workflow
        .schema
        .channels
        .iter()
        .find(|channel| channel.id == workflow.schema.input_channel_id)
        .map(|channel| channel.accepted_file_extensions.clone())
        .unwrap_or_default();

    let mut inputs = Vec::new();
    for path in &request.input_paths {
        inputs.extend(
            input_data_references(path, &accepted_file_extensions)
                .map_err(|error| CommandError::invalid_input("input_paths", error.to_string()))?
                .into_iter()
                .map(|data_reference| ChannelItemInsertedData {
                    channel_id: workflow.schema.input_channel_id.clone(),
                    data_reference,
                }),
        );
    }

    let run = workflow
        .run(inputs, None)
        .await
        .map_err(|error| CommandError::internal("start_workflow_run_failed", error.to_string()))?;

    let response = StartWorkflowRunResponse {
        workflow_run_id: run.db_id,
        public_id: run.id.to_string(),
    };

    // todo: add a service level workflow run pool, so that we can have a list
    // of active workflow runs and be able to issue commands to them
    tauri::async_runtime::spawn_blocking(async move {
        if let Err(error) = process_run_until_complete(run).await {
            eprintln!("workflow run stream processor failed: {error}");
        }
    });

    Ok(response)
}

async fn process_run_until_complete(run: ZygoLocalRun) -> anyhow::Result<()> {
    let mut snapshot_rx = run.subscribe()?;
    let mut stream_processor = run.stream_processor();
    let mut cursor = RunCursor::default();
    let mut pending_records = false;
    snapshot_rx.mark_changed();

    loop {
        let snapshot = if pending_records {
            snapshot_rx.borrow().clone()
        } else {
            if snapshot_rx.changed().await.is_err() {
                anyhow::bail!("workflow actor stopped before reaching a terminal state");
            }
            snapshot_rx.borrow_and_update().clone()
        };

        let mut reached_end = false;
        for _ in 0..STREAM_RECORD_BATCH_SIZE {
            let read = stream_processor.process_next(cursor.clone()).await?;
            cursor = read.next_cursor;

            if read.record.is_none() {
                reached_end = true;
                break;
            }
        }
        pending_records = !reached_end;

        if reached_end && snapshot.state.status.is_terminal() {
            return Ok(());
        }

        tokio::task::yield_now().await;
    }
}

// todo: how do we really want to do validation?
// maybe really in python as source of truth with UI doing some pre-validation for UX
fn input_data_references(
    input_path: &str,
    accepted_file_extensions: &[FileExtension],
) -> anyhow::Result<Vec<DataReference>> {
    let fsspec_uri = if input_path.starts_with("file://") {
        input_path.to_owned()
    } else {
        format!("file://{}", Path::new(input_path).canonicalize()?.display())
    };

    let Some(path) = local_path(&fsspec_uri) else {
        return Ok(vec![DataReference {
            uri: fsspec_uri,
            version: String::from("1"),
        }]);
    };

    if !path.is_dir() {
        ensure_extension_accepted(&path, accepted_file_extensions)?;
        return Ok(vec![DataReference {
            uri: fsspec_uri,
            version: String::from("1"),
        }]);
    }

    let accepted_extensions = accepted_file_extensions
        .iter()
        .map(|extension| extension.as_str().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let mut files = fs::read_dir(&path)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|file| {
        file.is_file()
            && (accepted_extensions.is_empty()
                || file.extension().is_some_and(|extension| {
                    let extension = extension.to_string_lossy().to_ascii_lowercase();
                    accepted_extensions
                        .iter()
                        .any(|accepted| accepted == &extension)
                }))
    });
    files.sort();

    anyhow::ensure!(
        !files.is_empty(),
        "input directory '{}' contains no files accepted by the input channel",
        path.display()
    );

    Ok(files
        .into_iter()
        .map(|file| DataReference {
            uri: format!("file://{}", file.display()),
            version: String::from("1"),
        })
        .collect())
}

fn ensure_extension_accepted(
    path: &Path,
    accepted_file_extensions: &[FileExtension],
) -> anyhow::Result<()> {
    if accepted_file_extensions.is_empty() {
        return Ok(());
    }

    let accepted_extensions = accepted_file_extensions
        .iter()
        .map(|extension| extension.as_str().to_ascii_lowercase())
        .collect::<Vec<_>>();

    let matches = path.extension().is_some_and(|extension| {
        let extension = extension.to_string_lossy().to_ascii_lowercase();
        accepted_extensions
            .iter()
            .any(|accepted| accepted == &extension)
    });

    anyhow::ensure!(
        matches,
        "file '{}' does not match accepted extensions ({})",
        path.display(),
        accepted_extensions.join(", ")
    );

    Ok(())
}

fn local_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = uri.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }

    (!uri.contains("://")).then(|| Path::new(uri).to_path_buf())
}
