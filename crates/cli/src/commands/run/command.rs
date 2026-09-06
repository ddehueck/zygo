use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as TerminalEvent, KeyCode, KeyEventKind,
    KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use local::{
    DEFAULT_DATABASE_BUSY_TIMEOUT, DbResult, LogRow, LogWatcher, LogsRepository, ZygoLocalConfig,
    ZygoLocalService,
};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::TableState;
use ratatui::{Terminal, TerminalOptions, Viewport};
use zygo_core::ZygoConfig;
use zygo_core::engine::{EngineSnapshot, RunCursor};
use zygo_core::ipc::v0::PythonCli;
use zygo_core::models::{DataReference, Event, JobRunId, StreamItem};

use crate::tui::{JobLogView, WorkflowRunView, job_run_at_position};

use super::{JobRunSummary, WorkflowRunSummary};

const ZYGO_PKG_INTERNAL_CLI_MODULE: &str = "zygo._internal.ipc.v0";
const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(50);
const STREAM_RECORD_BATCH_SIZE: usize = 64;

struct TerminalInput {
    alternate_screen: bool,
}

impl TerminalInput {
    fn new() -> std::io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(stdout(), EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error);
        }

        Ok(Self {
            alternate_screen: false,
        })
    }

    fn enter_alternate_screen(&mut self) -> std::io::Result<()> {
        execute!(stdout(), EnterAlternateScreen)?;
        self.alternate_screen = true;
        Ok(())
    }

    fn leave_alternate_screen(&mut self) -> std::io::Result<()> {
        if self.alternate_screen {
            execute!(stdout(), LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }
        Ok(())
    }
}

impl Drop for TerminalInput {
    fn drop(&mut self) {
        let _ = self.leave_alternate_screen();
        let _ = execute!(stdout(), DisableMouseCapture, Show);
        let _ = disable_raw_mode();
    }
}

enum Screen {
    Summary,
    Logs(LogViewState),
}

struct LogViewState {
    job_id: String,
    job_run_id: JobRunId,
    watcher: LogWatcher,
    contents: String,
    error: Option<String>,
    is_running: bool,
}

impl LogViewState {
    fn new(job_run: &JobRunSummary, repository: LogsRepository) -> anyhow::Result<Self> {
        let job_run_id = JobRunId::try_from(job_run.job_run_id.clone())?;
        let watcher = LogWatcher::new(repository, job_run_id.clone());
        Ok(Self {
            job_id: job_run.job_id.clone(),
            job_run_id,
            watcher,
            contents: String::new(),
            error: None,
            is_running: job_run.status == "running",
        })
    }

    fn apply_batch(&mut self, batch: DbResult<Vec<LogRow>>) {
        match batch {
            Ok(rows) => {
                for row in rows {
                    self.contents.push_str(&row.content);
                }
                self.error = None;
            }
            Err(error) => {
                self.error = Some(format!("Could not read logs: {error}"));
            }
        }
    }

    fn display_contents(&self) -> String {
        let mut contents = self.contents.clone();

        if contents.is_empty() {
            if let Some(error) = &self.error {
                return error.clone();
            }
            return if self.is_running {
                "Waiting for log output…".to_owned()
            } else {
                "No log output.".to_owned()
            };
        }

        if let Some(error) = &self.error {
            contents.push_str(&format!("\n\n{error}"));
        }

        contents
    }
}

struct StreamUpdate {
    snapshot: EngineSnapshot,
    events: Vec<Event>,
}

enum StreamMessage {
    Update(StreamUpdate),
    Error(anyhow::Error),
}

enum LoopEvent {
    StreamUpdate(StreamUpdate),
    StreamError(anyhow::Error),
    Redraw,
    Input(Option<TerminalEvent>),
    LogBatch(DbResult<Vec<LogRow>>),
}

fn select_previous(state: &mut TableState, item_count: usize) {
    if item_count == 0 {
        state.select(None);
        return;
    }

    let selected = state.selected().unwrap_or_default().saturating_sub(1);
    state.select(Some(selected));
}

fn select_next(state: &mut TableState, item_count: usize) {
    if item_count == 0 {
        state.select(None);
        return;
    }

    let selected = state
        .selected()
        .map_or(0, |selected| selected.saturating_add(1))
        .min(item_count - 1);
    state.select(Some(selected));
}

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
    let python_cli = PythonCli::new(python.clone(), cwd.clone(), target.to_owned());
    let metadata = PythonCli::parse_metadata_response(&metadata)?;
    // println!("parsed metadata: {metadata:?}");

    let input_extensions = metadata
        .channels
        .iter()
        .find(|channel| channel.id == metadata.input_channel_id)
        .map_or_else(Vec::new, |channel| channel.accepted_file_extensions.clone());
    let inputs = input_data_references(fsspec_uri, &input_extensions)?;

    let schema = python_cli.workflow_schema_from_metadata(metadata.clone())?;
    // println!("built workflow schema: {schema:?}");

    // 4. Create a zygo service and start the workflow
    let num_workers = workers.unwrap_or(1); // TODO: Use CPU core count
    let config = ZygoConfig::new(num_workers);
    let service = ZygoLocalService::new(ZygoLocalConfig {
        base: config,
        database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
    })
    .await?;

    let run_id = service.run(inputs, schema).await?;
    // println!("run_id: {run_id:?}");

    // 5. Watch the engine state in an interactive fullscreen terminal view.
    let mut terminal_input = TerminalInput::new()?;
    terminal_input.enter_alternate_screen()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Fullscreen,
        },
    )?;

    // Keep stream projection and database work off the UI task. Updates are
    // delivered in bounded batches so a large backlog cannot starve input or
    // redraws.
    let mut snapshot_rx = service.base.subscribe(&run_id).await?;
    let mut stream_processor = service.stream_processor(&run_id);
    let (stream_updates_tx, mut stream_updates_rx) = tokio::sync::mpsc::channel(1);
    let stream_task = tokio::spawn(async move {
        let mut cursor = RunCursor::default();
        let mut pending_records = false;
        snapshot_rx.mark_changed();

        loop {
            let snapshot = if pending_records {
                snapshot_rx.borrow().clone()
            } else {
                if snapshot_rx.changed().await.is_err() {
                    let _ = stream_updates_tx
                        .send(StreamMessage::Error(anyhow::anyhow!(
                            "workflow actor stopped before reaching a terminal state"
                        )))
                        .await;
                    return;
                }
                snapshot_rx.borrow_and_update().clone()
            };

            let mut events = Vec::with_capacity(STREAM_RECORD_BATCH_SIZE);
            let mut reached_end = false;
            for _ in 0..STREAM_RECORD_BATCH_SIZE {
                let read = match stream_processor.process_next(cursor.clone()).await {
                    Ok(read) => read,
                    Err(error) => {
                        let _ = stream_updates_tx.send(StreamMessage::Error(error)).await;
                        return;
                    }
                };
                cursor = read.next_cursor;

                let Some(record) = read.record else {
                    reached_end = true;
                    break;
                };

                if let StreamItem::Event(event) = record.item {
                    events.push(event);
                }
            }
            pending_records = !reached_end;
            let is_complete = reached_end && snapshot.state.status.is_terminal();

            if stream_updates_tx
                .send(StreamMessage::Update(StreamUpdate { snapshot, events }))
                .await
                .is_err()
            {
                return;
            }

            if is_complete {
                // Keep the sender alive while the terminal summary remains open. Otherwise the UI
                // would interpret normal stream closure as an actor failure before q/Ctrl-C.
                stream_updates_tx.closed().await;
                return;
            }

            tokio::task::yield_now().await;
        }
    });

    let mut summary = WorkflowRunSummary::new(metadata.id.clone());
    let mut summary_refresh = tokio::time::interval(Duration::from_secs(1));
    let mut input_poll = tokio::time::interval(INPUT_POLL_INTERVAL);
    summary_refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    input_poll.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut has_snapshot = false;
    let mut screen = Screen::Summary;
    let mut table_state = TableState::default();
    let mut last_area = ratatui::layout::Rect::default();

    loop {
        let loop_event = tokio::select! {
            message = stream_updates_rx.recv() => {
                match message {
                    Some(StreamMessage::Update(update)) => LoopEvent::StreamUpdate(update),
                    Some(StreamMessage::Error(error)) => LoopEvent::StreamError(error),
                    None => LoopEvent::StreamError(anyhow::anyhow!(
                        "workflow stream processor stopped unexpectedly"
                    )),
                }
            }
            _ = summary_refresh.tick() => LoopEvent::Redraw,
            _ = input_poll.tick() => {
                let input = if event::poll(Duration::ZERO)? {
                    Some(event::read()?)
                } else {
                    None
                };
                LoopEvent::Input(input)
            }
            batch = async {
                match &mut screen {
                    // The watcher owns polling and its cursor is cancellation-safe when input wins.
                    Screen::Logs(log) => log.watcher.next_batch().await,
                    Screen::Summary => std::future::pending().await,
                }
            } => LoopEvent::LogBatch(batch),
        };

        let mut open_job_index = None;
        let mut should_cancel = false;
        let mut should_redraw = true;

        match loop_event {
            LoopEvent::StreamUpdate(update) => {
                for event in update.events {
                    summary.update_by_event(event);
                }

                summary.update_by_snapshot(&update.snapshot);
                has_snapshot = true;

                if let Screen::Logs(log) = &mut screen {
                    log.is_running = summary
                        .job_runs
                        .iter()
                        .find(|job_run| job_run.job_run_id == log.job_run_id.as_ref())
                        .is_some_and(|job_run| job_run.status == "running");
                }
            }
            LoopEvent::Input(Some(TerminalEvent::Key(key)))
                if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                let cancel_key = key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
                if cancel_key {
                    should_cancel = true;
                } else {
                    match &screen {
                        Screen::Summary => match key.code {
                            KeyCode::Up => {
                                select_previous(&mut table_state, summary.job_runs.len())
                            }
                            KeyCode::Down => select_next(&mut table_state, summary.job_runs.len()),
                            KeyCode::Enter => open_job_index = table_state.selected(),
                            _ => {}
                        },
                        Screen::Logs(_) if key.code == KeyCode::Esc => screen = Screen::Summary,
                        Screen::Logs(_) => {}
                    }
                }
            }
            LoopEvent::Input(Some(TerminalEvent::Mouse(mouse)))
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                    && matches!(&screen, Screen::Summary) =>
            {
                if let Some(index) = job_run_at_position(
                    last_area,
                    table_state.offset(),
                    mouse.column,
                    mouse.row,
                    summary.job_runs.len(),
                ) {
                    table_state.select(Some(index));
                    open_job_index = Some(index);
                }
            }
            LoopEvent::LogBatch(batch) => {
                if let Screen::Logs(log) = &mut screen {
                    log.apply_batch(batch);
                }
            }
            LoopEvent::StreamError(error) => {
                stream_task.abort();
                service.cancel(&run_id).await?;
                return Err(error);
            }
            LoopEvent::Redraw | LoopEvent::Input(Some(_)) => {}
            LoopEvent::Input(None) => should_redraw = false,
        }

        if should_cancel {
            stream_task.abort();
            service.cancel(&run_id).await?;
            break;
        }

        if summary.job_runs.is_empty() {
            table_state.select(None);
        } else if table_state
            .selected()
            .is_none_or(|selected| selected >= summary.job_runs.len())
        {
            table_state.select(Some(0));
        }

        if let Some(index) = open_job_index
            && let Some(job_run) = summary.job_runs.get(index)
        {
            screen = Screen::Logs(LogViewState::new(job_run, service.repos.logs.clone())?);
        }

        if !has_snapshot || !should_redraw {
            continue;
        }

        match &screen {
            Screen::Summary => {
                terminal.draw(|frame| {
                    last_area = frame.area();
                    frame.render_stateful_widget(
                        WorkflowRunView::new(&summary, target, fsspec_uri),
                        frame.area(),
                        &mut table_state,
                    );
                })?;
            }
            Screen::Logs(log) => {
                let contents = log.display_contents();
                terminal.draw(|frame| {
                    frame.render_widget(
                        JobLogView::new(
                            &log.job_id,
                            log.job_run_id.as_ref(),
                            &contents,
                            log.is_running,
                        ),
                        frame.area(),
                    );
                })?;
            }
        }
    }

    stream_task.abort();
    drop(terminal);

    Ok(())
}

fn input_data_references(
    input_uri: &str,
    accepted_file_extensions: &[String],
) -> anyhow::Result<Vec<DataReference>> {
    let Some(path) = local_path(input_uri) else {
        return Ok(vec![DataReference {
            uri: input_uri.to_owned(),
            version: String::from("1"),
        }]);
    };

    if !path.is_dir() {
        return Ok(vec![DataReference {
            uri: input_uri.to_owned(),
            version: String::from("1"),
        }]);
    }

    let accepted_extensions = accepted_file_extensions
        .iter()
        .map(|extension| extension.trim_start_matches('.').to_ascii_lowercase())
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
            uri: file.to_string_lossy().into_owned(),
            version: String::from("1"),
        })
        .collect())
}

fn local_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = uri.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }

    (!uri.contains("://")).then(|| Path::new(uri).to_path_buf())
}
