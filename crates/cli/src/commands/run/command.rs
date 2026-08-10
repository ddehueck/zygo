use std::{io::stdout, process::Command, time::Duration};

use ratatui::{Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, widgets::Widget};
use zygo_core::{
    MemoryStore, Zygo, ZygoConfig,
    engine::RunCursor,
    models::{DataReference, StreamItem},
    store::Store,
};

use crate::{
    python::{WorkflowMetadata, workflow_schema_from_metadata},
    tui::WorkflowRunView,
};

use super::WorkflowRunSummary;

const ZYGO_PKG_INTERNAL_CLI_MODULE: &str = "zygo._internal.ipc.v0";

pub async fn run_workflow(
    target: &str,
    fsspec_uri: &str,
    workers: Option<usize>,
) -> anyhow::Result<()> {
    // 1. Find the current python executable in the current working directory
    // Start with `uv python` for now
    let python = Command::new("uv").args(["python", "find"]).output()?;
    anyhow::ensure!(
        python.status.success(),
        "Could not find a python executable"
    );
    let python = String::from_utf8_lossy(&python.stdout).trim().to_owned();
    let cwd = std::env::current_dir()?.to_string_lossy().into_owned();
    // println!("{python}");

    // 2. Ensure that the zygo package is in the executable's environment
    let package = Command::new(&python).args(["-c", "import zygo"]).status()?;
    anyhow::ensure!(package.success(), "zygo is not available in {python}");
    // println!("zygo is available in {python}");

    // 3. Use the zygo package to inspect the workflow to build the schema
    let metadata = Command::new(&python)
        .args(["-m", ZYGO_PKG_INTERNAL_CLI_MODULE, "metadata", target])
        .output()?;
    anyhow::ensure!(
        metadata.status.success(),
        "Failed to get metadata for {target}: {}",
        String::from_utf8_lossy(&metadata.stderr).trim()
    );
    let metadata = String::from_utf8_lossy(&metadata.stdout).trim().to_owned();
    // println!("metadata: {metadata}");

    // 3.5 Parse the metadata into a structured format
    let metadata: WorkflowMetadata = serde_json::from_str(&metadata)?;
    // println!("parsed metadata: {metadata:?}");

    let schema =
        workflow_schema_from_metadata(metadata.clone(), cwd.as_ref(), target, python.as_ref())?;
    // println!("built workflow schema: {schema:?}");

    // 4. Create a zygo service and start the workflow
    let num_workers = workers.unwrap_or(1); // TODO: Use CPU core count
    let config = ZygoConfig::new(num_workers);
    let store = Store::new(MemoryStore::new()); // TODO: This is a lil funky wording

    let service = Zygo::new(store, config);

    let data_ref = DataReference {
        uri: fsspec_uri.to_string(),
        etag: "42".into(),
        content_type: None,
        size_bytes: None,
    };

    let run_id = service.run(data_ref, schema).await?;
    // println!("run_id: {run_id:?}");

    // 5. Watch the engine state in an inline terminal view
    let viewport_height = metadata.jobs.len().saturating_add(9).min(u16::MAX as usize) as u16;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(viewport_height),
        },
    )?;

    // Clients read the workflow run state by subscibing to a watch channel
    // that notifies when the engine has made a meaningful update and then
    // reading the stream directly.
    let mut rx = service.subscribe(&run_id).await?;

    let stream = service.stream(&run_id);
    let mut cursor = RunCursor::default();

    let mut summary = WorkflowRunSummary::new(metadata.id.clone());
    let mut refresh = tokio::time::interval(Duration::from_secs(1));
    refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut has_snapshot = false;
    let mut is_terminal = false;

    // // Intentionally block the TUI so job stdout remains visible while the run is active.
    // loop {
    //     let snapshot = rx.borrow_and_update().clone();
    //     if snapshot.state.status.is_terminal() {
    //         break;
    //     }

    //     rx.changed().await.map_err(|_| {
    //         anyhow::anyhow!("workflow actor stopped before reaching a terminal state")
    //     })?;
    // }

    // Let the existing TUI loop consume the terminal snapshot.
    rx.mark_changed();

    loop {
        let snapshot_changed = tokio::select! {
            result = rx.changed() => {
                result.map_err(|_| {
                    anyhow::anyhow!("workflow actor stopped before reaching a terminal state")
                })?;
                true
            }
            _ = refresh.tick() => false,
        };

        if snapshot_changed {
            let snapshot = rx.borrow_and_update().clone();

            loop {
                let read = stream.next(cursor).await?;
                cursor = read.next_cursor;

                let Some(record) = read.record else {
                    break;
                };

                if let StreamItem::Event(event) = record.item {
                    summary.update_by_event(event);
                }
            }

            summary.update_by_snapshot(&snapshot);
            has_snapshot = true;
            is_terminal = snapshot.state.status.is_terminal();
        }

        if !has_snapshot {
            continue;
        }

        if is_terminal {
            let view_height = summary
                .job_runs
                .len()
                .saturating_add(9)
                .min(u16::MAX as usize) as u16;
            terminal.insert_before(view_height, |buffer| {
                WorkflowRunView::new(&summary, target, fsspec_uri).render(buffer.area, buffer);
            })?;
            break;
        }

        terminal.draw(|frame| {
            frame.render_widget(
                WorkflowRunView::new(&summary, target, fsspec_uri),
                frame.area(),
            )
        })?;
    }

    drop(terminal);

    Ok(())
}
