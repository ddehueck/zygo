use std::{sync::Arc, time::Duration};

use gpui::{Context, Render, Task, Window, div, prelude::*};
use local::ZygoLocalService;
use zygo_core::models::{JobId, JobRunId, WorkflowRunId};
use zygo_core::workers::WorkerLogReader;

use crate::{dependencies, theme::Theme};

const LOG_REFRESH_INTERVAL: Duration = Duration::from_millis(200);

pub struct JobLogsView {
    workflow_run_id: Option<WorkflowRunId>,
    job_id: Option<String>,
    job_run_id: Option<String>,
    contents: String,
    loading: bool,
    error: Option<String>,
    _refresh_task: Option<Task<()>>,
}

impl JobLogsView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            workflow_run_id: None,
            job_id: None,
            job_run_id: None,
            contents: String::new(),
            loading: false,
            error: None,
            _refresh_task: None,
        }
    }

    pub fn set_job(
        &mut self,
        workflow_run_id: WorkflowRunId,
        job_id: String,
        job_run_id: String,
        cx: &mut Context<Self>,
    ) {
        if self.workflow_run_id.as_ref() == Some(&workflow_run_id)
            && self.job_id.as_deref() == Some(job_id.as_str())
            && self.job_run_id.as_deref() == Some(job_run_id.as_str())
        {
            return;
        }

        self.workflow_run_id = Some(workflow_run_id.clone());
        self.job_id = Some(job_id.clone());
        self.job_run_id = Some(job_run_id.clone());
        self.contents.clear();
        self.loading = true;
        self.error = None;
        self._refresh_task = None;

        let parsed_job_run_id = match JobRunId::try_from(job_run_id) {
            Ok(job_run_id) => job_run_id,
            Err(error) => {
                self.loading = false;
                self.error = Some(format!("Invalid job run ID: {error}"));
                cx.notify();
                return;
            }
        };

        let service = dependencies::use_service(cx);
        let task_workflow_run_id = workflow_run_id.clone();
        let task_job_run_id = parsed_job_run_id.clone();
        let task_job_run_id_string = task_job_run_id.to_string();
        let task_job_id = job_id;
        self._refresh_task = Some(cx.spawn(async move |view, cx| {
            let executor = cx.background_executor().clone();
            let cwd = match cx
                .background_spawn(async move {
                    load_job_cwd(service, workflow_run_id, task_job_id).await
                })
                .await
            {
                Ok(cwd) => cwd,
                Err(error) => {
                    let _ = view.update(cx, |view, cx| {
                        if view.workflow_run_id.as_ref() != Some(&task_workflow_run_id) {
                            return;
                        }
                        view.loading = false;
                        view.error = Some(format!("Could not load job working directory: {error}"));
                        cx.notify();
                    });
                    return;
                }
            };

            let mut reader = None;
            loop {
                executor.timer(LOG_REFRESH_INTERVAL).await;

                let reader_to_refresh = reader.take();
                let refresh_job_run_id = task_job_run_id.clone();
                let refresh_cwd = cwd.clone();
                let (next_reader, refresh) = executor
                    .spawn(async move {
                        let mut reader = reader_to_refresh;
                        let refresh = refresh_log(&mut reader, &refresh_job_run_id, &refresh_cwd);
                        (reader, refresh)
                    })
                    .await;
                reader = next_reader;

                let Ok(should_continue) = view.update(cx, |view, cx| {
                    if view.workflow_run_id.as_ref() != Some(&task_workflow_run_id)
                        || view.job_run_id.as_ref() != Some(&task_job_run_id_string)
                    {
                        return false;
                    }

                    view.loading = false;
                    match refresh {
                        LogRefresh::Contents(contents) => {
                            view.contents = contents;
                            view.error = None;
                        }
                        LogRefresh::Waiting => {
                            view.error = None;
                        }
                        LogRefresh::Error(error) => {
                            view.error = Some(error);
                        }
                    }
                    cx.notify();
                    true
                }) else {
                    break;
                };

                if !should_continue {
                    break;
                }
            }
        }));
        cx.notify();
    }
}

impl Render for JobLogsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let job_id = self.job_id.as_deref().unwrap_or("Unknown job");
        let job_run_id = self.job_run_id.as_deref().unwrap_or("Unknown job run");

        let log_content = if self.loading {
            div()
                .text_color(colors.text_secondary)
                .child("Loading logs…")
        } else if self.contents.is_empty() {
            div()
                .text_color(colors.text_secondary)
                .child("Waiting for log output…")
        } else {
            div()
                .text_color(colors.text_primary)
                .child(self.contents.clone())
        };

        let log_content = if let Some(error) = &self.error {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(log_content)
                .child(div().text_color(colors.error).child(error.clone()))
        } else {
            div().child(log_content)
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_8()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Job logs"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .text_color(colors.text_secondary)
                    .child(format!("Job: {job_id}"))
                    .child(format!("Job run: {job_run_id}")),
            )
            .child(
                div()
                    .id("job-logs-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_scroll()
                    .p_4()
                    .border_1()
                    .border_color(colors.border_muted)
                    .bg(colors.surface_sunken)
                    .child(log_content),
            )
    }
}

// This will be caught up in an incoming regactor
async fn load_job_cwd(
    service: Arc<ZygoLocalService>,
    workflow_run_id: WorkflowRunId,
    job_id: String,
) -> anyhow::Result<String> {
    let schema = service
        .base
        .workflow_schema(&workflow_run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("workflow schema not found for run {workflow_run_id}"))?;
    let job_id = JobId::try_from(job_id)?;
    let entrypoint = schema
        .get_job_entrypoint(&job_id)
        .ok_or_else(|| anyhow::anyhow!("job {job_id} not found in workflow schema"))?;

    Ok(entrypoint.cwd().to_owned())
}

enum LogRefresh {
    Contents(String),
    Waiting,
    Error(String),
}

fn refresh_log(
    reader: &mut Option<WorkerLogReader>,
    job_run_id: &JobRunId,
    cwd: &str,
) -> LogRefresh {
    if reader.is_none() {
        match WorkerLogReader::new_sync_in(job_run_id.clone(), cwd) {
            Ok(new_reader) => {
                let contents = String::from_utf8_lossy(new_reader.contents()).into_owned();
                *reader = Some(new_reader);
                return LogRefresh::Contents(contents);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return LogRefresh::Waiting;
            }
            Err(error) => {
                return LogRefresh::Error(format!("Could not open log file: {error}"));
            }
        }
    }

    let Some(reader) = reader.as_mut() else {
        return LogRefresh::Waiting;
    };

    if let Err(error) = reader.refresh_sync() {
        return LogRefresh::Error(format!("Could not read log file: {error}"));
    }

    LogRefresh::Contents(String::from_utf8_lossy(reader.contents()).into_owned())
}
