use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use gpui::{
    AnyElement, AppContext, Context, PathPromptOptions, Render, Task, Window, div, prelude::*,
};
use local::ZygoLocalService;
use zygo_core::{
    engine::RunCursor,
    ipc::v0::PythonCli,
    models::{DataReference, Entrypoint, EventId, FileExtension, WorkflowRunId, WorkflowSchema},
};

use crate::{
    Routes, dependencies,
    navigation::{WorkflowRunRoutes, WorkflowRunsRoutes},
    theme::Theme,
    ui::Button,
};

pub struct NewRunView {
    run_id: Option<WorkflowRunId>,
    schema: Option<WorkflowSchema>,
    input_extensions: Vec<FileExtension>,
    selected_file: Option<PathBuf>,
    loading_metadata: bool,
    running: bool,
    error: Option<String>,
    _metadata_task: Option<Task<()>>,
    _run_task: Option<Task<()>>,
}

impl NewRunView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            run_id: None,
            schema: None,
            input_extensions: Vec::new(),
            selected_file: None,
            loading_metadata: false,
            running: false,
            error: None,
            _metadata_task: None,
            _run_task: None,
        }
    }

    pub fn set_run_id(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) {
        if self.run_id.as_ref() == Some(&run_id) {
            return;
        }

        self.run_id = Some(run_id.clone());
        self.schema = None;
        self.input_extensions.clear();
        self.selected_file = None;
        self.loading_metadata = true;
        self.running = false;
        self.error = None;
        cx.notify();

        let service = dependencies::use_service(cx);
        self._metadata_task = Some(cx.spawn(async move |view, cx| {
            let result = cx
                .background_spawn(
                    async move { load_workflow_metadata(service, run_id.clone()).await },
                )
                .await;

            let _ = view.update(cx, |view, cx| {
                view.loading_metadata = false;
                match result {
                    Ok((schema, input_extensions)) => {
                        view.schema = Some(schema);
                        view.input_extensions = input_extensions;
                        view.error = None;
                    }
                    Err(error) => view.error = Some(error.to_string()),
                }
                cx.notify();
            });
        }));
    }
}

impl Render for NewRunView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let navigate = dependencies::use_navigation(cx);
        let list_navigation = navigate.clone();
        let view = cx.entity();
        let schema = self.schema.clone();
        let input_extensions = self.input_extensions.clone();
        let selected_file = self.selected_file.clone();
        let service = dependencies::use_service(cx);
        let tokio_handle = dependencies::use_tokio_handle(cx);
        let sync = dependencies::use_run_sync(cx);
        let running = self.running;
        let loading_metadata = self.loading_metadata;
        let error = self.error.clone();

        let content: AnyElement = if loading_metadata {
            div()
                .id("new-run-metadata-loading")
                .flex()
                .flex_col()
                .gap_4()
                .text_color(colors.text_secondary)
                .child("Loading workflow metadata…")
                .into_any_element()
        } else if schema.is_none() {
            div()
                .id("new-run-metadata-error")
                .flex()
                .flex_col()
                .gap_4()
                .text_color(colors.error)
                .child(format!(
                    "Unable to prepare workflow: {}",
                    error.as_deref().unwrap_or("metadata is unavailable")
                ))
                .into_any_element()
        } else if schema.is_some() {
            let extensions = if input_extensions.is_empty() {
                "any file".to_owned()
            } else {
                input_extensions
                    .iter()
                    .map(|extension| format!(".{}", extension.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let selected_label = selected_file
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "No file selected".to_owned());
            let select_view = view.clone();
            let select_extensions = input_extensions.clone();
            let select_button = Button::new("new-run-select-file", "Choose local file").on_click(
                move |_, _, cx| {
                    let receiver = cx.prompt_for_paths(PathPromptOptions {
                        files: true,
                        directories: false,
                        multiple: false,
                        prompt: Some("Choose workflow input".into()),
                    });
                    let select_view = select_view.clone();
                    let select_extensions = select_extensions.clone();
                    let task = cx.spawn(async move |cx| {
                        let result = receiver.await;
                        let _ = select_view.update(cx, |view, cx| {
                            match result {
                                Ok(Ok(Some(paths))) => {
                                    if let Some(path) = paths.into_iter().next() {
                                        if matches_extension(&path, &select_extensions) {
                                            view.selected_file = Some(path);
                                            view.error = None;
                                        } else {
                                            view.selected_file = None;
                                            view.error = Some(format!(
                                                "Choose a file ending in {}.",
                                                format_extensions(&select_extensions),
                                            ));
                                        }
                                    }
                                }
                                Ok(Ok(None)) => {}
                                Ok(Err(error)) => view.error = Some(error.to_string()),
                                Err(error) => view.error = Some(error.to_string()),
                            }
                            cx.notify();
                        });
                    });
                    task.detach();
                },
            );

            let mut run_controls = div().flex().items_center().gap_3().child(select_button);
            if let (Some(schema), Some(path)) = (schema, selected_file) {
                let run_view = view.clone();
                let run_navigation = navigate.clone();
                let window_handle = _window.window_handle();
                let run_button_label = if running {
                    "Starting…"
                } else {
                    "Run workflow"
                };
                let run_button =
                    Button::new("new-run-submit", run_button_label).on_click(move |_, _, cx| {
                        if running {
                            return;
                        }

                        let schema = schema.clone();
                        let path = path.clone();
                        let data_reference =
                            DataReference::new(file_uri(&path), EventId::new().to_string());
                        let service = service.clone();
                        let tokio_handle = tokio_handle.clone();
                        let sync = sync.clone();
                        let task_view = run_view.clone();
                        let run_navigation = run_navigation.clone();
                        let task = cx.spawn(async move |cx| {
                            let result = cx
                                .background_spawn(async move {
                                    tokio_handle
                                        .spawn(async move {
                                            let new_run_id =
                                                service.run(data_reference, schema).await?;
                                            let processor_service = service.clone();
                                            let processor_run_id = new_run_id.clone();
                                            tokio::spawn(async move {
                                                process_run_stream(
                                                    processor_service,
                                                    processor_run_id,
                                                )
                                                .await;
                                            });
                                            Ok::<_, anyhow::Error>(new_run_id)
                                        })
                                        .await
                                        .map_err(|error| {
                                            anyhow::anyhow!(
                                                "local service task terminated unexpectedly: {error}"
                                            )
                                        })?
                                })
                                .await;

                            match result {
                                Ok(new_run_id) => {
                                    let _ = sync.update(cx, |sync, cx| {
                                        sync.observe_new_run(new_run_id.clone(), cx);
                                    });
                                    let _ = task_view.update(cx, |view, cx| {
                                        view.running = false;
                                        view.error = None;
                                        cx.notify();
                                    });
                                    let _ = cx.update_window(window_handle, |_, window, cx| {
                                        run_navigation(
                                            &Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                                                id: new_run_id,
                                                routes: WorkflowRunRoutes::Index,
                                            }),
                                            window,
                                            cx,
                                        );
                                    });
                                }
                                Err(error) => {
                                    let _ = task_view.update(cx, |view, cx| {
                                        view.running = false;
                                        view.error =
                                            Some(format!("Unable to start workflow: {error}"));
                                        cx.notify();
                                    });
                                }
                            }
                        });
                        run_view.update(cx, |view, cx| {
                            view.running = true;
                            view._run_task = Some(task);
                            cx.notify();
                        });
                    });
                run_controls = run_controls.child(run_button);
            }

            let mut form = div()
                .id("new-run-form")
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_color(colors.text_secondary)
                        .child(format!("Input files accepted: {extensions}")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(div().flex_1().truncate().child(selected_label))
                        .child(run_controls),
                );
            if let Some(error) = error {
                form = form.child(div().text_color(colors.error).child(error));
            }
            form.into_any_element()
        } else {
            div()
                .text_color(colors.text_secondary)
                .child("Loading workflow metadata…")
                .into_any_element()
        };

        div()
            .id("new-run-view")
            .flex()
            .flex_col()
            .size_full()
            .p_8()
            .gap_5()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("New workflow run"),
            )
            .child(
                div()
                    .text_color(colors.text_secondary)
                    .child("Select a local input file to start this workflow."),
            )
            .child(content)
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(Button::new("new-runs", "Workflow runs").on_click(
                        move |_, window, cx| {
                            list_navigation(
                                &Routes::WorkflowRuns(WorkflowRunsRoutes::Index),
                                window,
                                cx,
                            );
                        },
                    )),
            )
    }
}

async fn process_run_stream(service: Arc<ZygoLocalService>, run_id: WorkflowRunId) {
    let mut receiver = match service.base.subscribe(&run_id).await {
        Ok(receiver) => receiver,
        Err(error) => {
            eprintln!("failed to subscribe to workflow run {run_id}: {error}");
            return;
        }
    };
    let mut processor = service.stream_processor(&run_id);
    let mut cursor = RunCursor::default();

    // Consume the stream once immediately, then process each batch after the
    // engine publishes a new snapshot.
    receiver.mark_changed();
    loop {
        if receiver.changed().await.is_err() {
            return;
        }

        loop {
            match processor.process_next(cursor.clone()).await {
                Ok(result) => {
                    cursor = result.next_cursor;
                    if result.record.is_none() {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("failed to process stream for workflow run {run_id}: {error}");
                    return;
                }
            }
        }

        if receiver.borrow_and_update().state.status.is_terminal() {
            return;
        }
    }
}

async fn load_workflow_metadata(
    service: Arc<ZygoLocalService>,
    run_id: WorkflowRunId,
) -> anyhow::Result<(WorkflowSchema, Vec<FileExtension>)> {
    let stored_schema = service
        .base
        .workflow_schema(&run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("workflow schema not found for run {run_id}"))?;

    let entrypoint = match stored_schema.entrypoint {
        Entrypoint::Python(entrypoint) => entrypoint,
    };
    let output = entrypoint.metadata_entrypoint().output()?;
    if !output.status.success() {
        anyhow::bail!(
            "metadata command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let metadata = PythonCli::parse_metadata_response(&String::from_utf8_lossy(&output.stdout))?;
    let schema = entrypoint.workflow_schema_from_metadata(metadata)?;
    let input_extensions = schema
        .channels
        .iter()
        .find(|channel| channel.id == schema.input_channel_id)
        .map(|channel| channel.accepted_file_extensions.clone())
        .unwrap_or_default();

    Ok((schema, input_extensions))
}

fn matches_extension(path: &Path, accepted: &[FileExtension]) -> bool {
    accepted.is_empty()
        || accepted.iter().any(|extension| {
            extension.as_str() == "*"
                || path.extension().is_some_and(|actual| {
                    actual
                        .to_string_lossy()
                        .eq_ignore_ascii_case(extension.as_str())
                })
        })
}

fn format_extensions(extensions: &[FileExtension]) -> String {
    extensions
        .iter()
        .map(|extension| format!(".{}", extension.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn file_uri(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", path.to_string_lossy())
}
