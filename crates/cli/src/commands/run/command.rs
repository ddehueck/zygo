use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};

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
use local::api::v0::PythonCli;
use local::models::{DataReferenceUri, FileExtension, JobRunId, WorkflowRunStatus};
use local::{
    DEFAULT_DATABASE_BUSY_TIMEOUT, DbResult, LogRow, LogWatcher, LogsRepository, RunOptions,
    ZygoLocalConfig, ZygoLocalService,
};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::TableState;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::tui::{JobLogView, WorkflowRunView, job_run_at_position};

use super::{JobRunSummary, WorkflowRunSummary};

const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(50);

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
        let job_run_id = JobRunId::try_from(job_run.public_id.clone())?;
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

enum LoopEvent {
    Refresh,
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
    input_path: &str,
    workers: Option<usize>,
    disable_cache: bool,
) -> anyhow::Result<()> {
    // Covert the path into a fsspec URI with an absolute path
    // todo: this needs to mature
    let fsspec_uri = if input_path.starts_with("file://") {
        input_path.to_string()
    } else {
        format!(
            "file://{}",
            std::path::Path::new(input_path).canonicalize()?.display()
        )
    };

    let python_cli = PythonCli::from_env(target).await?;

    // Inspect the workflow metadata
    let metadata = python_cli.run_metadata_command().await?;
    let schema = python_cli.workflow_schema_from_metadata(metadata.clone())?;

    // todo: extend the workflow schema to support easier validation
    let input_extensions = schema
        .channels
        .iter()
        .find(|channel| channel.id == schema.input_channel_id)
        .map_or_else(Vec::new, |channel| channel.accepted_file_extensions.clone());
    let inputs = input_data_references(&fsspec_uri, &input_extensions)?;

    // 4. Create a zygo service and start the workflow
    let config = ZygoLocalConfig {
        database_busy_timeout: DEFAULT_DATABASE_BUSY_TIMEOUT,
    };
    let options = RunOptions {
        num_workers: workers.unwrap_or(1),
        disable_cache,
    };

    let service = ZygoLocalService::new(config).await?;
    let workflow = service.register(schema).await?;
    let run = workflow.run(inputs, options).await?;

    // 5. Watch the engine state in an interactive fullscreen terminal view.
    let mut terminal_input = TerminalInput::new()?;
    terminal_input.enter_alternate_screen()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Fullscreen,
        },
    )?;

    let snapshot_rx = run.subscribe()?;
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
            _ = summary_refresh.tick() => LoopEvent::Refresh,
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
            LoopEvent::Refresh => {
                if snapshot_rx.has_changed().is_err() && !snapshot_rx.borrow().status.is_terminal()
                {
                    let _ = run.cancel().await;
                    return Err(anyhow::anyhow!(
                        "workflow actor stopped before reaching a terminal state"
                    ));
                }
                let refreshed = async {
                    let workflow_run = service
                        .repos
                        .workflow_runs
                        .get_by_workflow_run_id(run.id.as_ref())
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("workflow run {} not found", run.id))?;
                    let job_runs = service
                        .repos
                        .job_runs
                        .list_by_workflow_run_id(run.id.as_ref())
                        .await?;
                    anyhow::Ok(WorkflowRunSummary::from_models(
                        metadata.id.clone(),
                        workflow_run,
                        job_runs,
                    ))
                }
                .await;
                summary = match refreshed {
                    Ok(summary) => summary,
                    Err(error) => {
                        let _ = run.cancel().await;
                        return Err(error);
                    }
                };
                has_snapshot = true;
                if summary.workflow_status == WorkflowRunStatus::Failed.to_string() {
                    let failure_detail = summary
                        .job_runs
                        .iter()
                        .find(|job_run| job_run.status == "failed")
                        .map(|job_run| {
                            format!(
                                "job {} ({}): {}",
                                job_run.job_id,
                                job_run.public_id,
                                job_run
                                    .error_message
                                    .as_deref()
                                    .unwrap_or("no failure details available")
                            )
                        })
                        .unwrap_or_else(|| "no failure details available".to_owned());
                    return Err(anyhow::anyhow!(
                        "workflow `{target}` failed: {failure_detail}"
                    ));
                }

                if let Screen::Logs(log) = &mut screen {
                    log.is_running = summary
                        .job_runs
                        .iter()
                        .find(|job_run| job_run.public_id == log.job_run_id.as_ref())
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
            LoopEvent::Input(Some(_)) => {}
            LoopEvent::Input(None) => should_redraw = false,
        }

        if should_cancel {
            run.cancel().await?;
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
                        WorkflowRunView::new(&summary, target, &fsspec_uri),
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

    drop(terminal);

    Ok(())
}

fn input_data_references(
    input_uri: &str,
    accepted_file_extensions: &[FileExtension],
) -> anyhow::Result<Vec<DataReferenceUri>> {
    let Some(path) = local_path(input_uri) else {
        return Ok(vec![DataReferenceUri::try_from(input_uri.to_owned())?]);
    };

    if !path.is_dir() {
        return Ok(vec![DataReferenceUri::try_from(input_uri.to_owned())?]);
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
        .map(|file| DataReferenceUri::try_from(file.to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?)
}

fn local_path(uri: &str) -> Option<PathBuf> {
    if let Some(path) = uri.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }

    (!uri.contains("://")).then(|| Path::new(uri).to_path_buf())
}
