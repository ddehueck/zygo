use gpui::{Context, Render, Window, div, prelude::*, px};
use gpuikit::elements::input::input;
use gpuikit::input::InputState;
use local::{JobRunSummaryRow, TagRow};
use zygo_core::models::WorkflowRunId;

use crate::{
    Routes, dependencies,
    navigation::{NavigationHandler, WorkflowRunRoutes, WorkflowRunsRoutes},
    theme::Theme,
    ui::Button,
};

pub struct RunDetailView {
    run_id: Option<WorkflowRunId>,
    workflow_run_input: gpui::Entity<InputState>,
}

impl RunDetailView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            run_id: None,
            workflow_run_input: cx.new(|cx| InputState::new_singleline(cx)),
        }
    }

    pub fn set_run_id(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) {
        if self.run_id.as_ref() != Some(&run_id) {
            self.run_id = Some(run_id.clone());

            let details = dependencies::use_run_details(cx);
            details.update(cx, |store, cx| {
                store.refresh(run_id.clone(), cx).detach();
            });

            let tags = dependencies::use_tags(cx);
            tags.update(cx, |store, cx| {
                store.refresh(run_id.clone(), cx).detach();
            });

            cx.notify();
        }
    }
}

impl Render for RunDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let navigate = dependencies::use_navigation(cx);
        let list_navigation = navigate.clone();
        let new_run_navigation = navigate.clone();
        let run_id = self
            .run_id
            .as_ref()
            .expect("detail view must have a run ID before rendering")
            .clone();
        let workflow_run_input = self.workflow_run_input.clone();
        let details = dependencies::use_run_details(cx);
        let detail_store = details.read(cx);
        let job_runs = detail_store.summaries_for(&run_id).unwrap_or(&[]);
        let jobs_loading = detail_store.is_loading(&run_id);
        let jobs_error = detail_store.error_for(&run_id);

        let tags = dependencies::use_tags(cx);
        let tag_store = tags.read(cx);
        let run_tags = tag_store.tags_for(&run_id).unwrap_or(&[]);
        let tags_loading = tag_store.is_loading(&run_id);
        let tags_error = tag_store.error_for(&run_id);

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_8()
            .gap_5()
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Workflow run detail"),
            )
            .child(
                div()
                    .text_color(colors.text_secondary)
                    .child(format!("Run ID: {run_id}")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Workflow input"),
                    )
                    .child(
                        div()
                            .id("workflow-run-input")
                            .flex()
                            .items_center()
                            .w_full()
                            .h(px(44.0))
                            .px_3()
                            .rounded_md()
                            .border_1()
                            .border_color(colors.border_muted)
                            .bg(colors.surface_input)
                            .child(
                                input(&workflow_run_input, cx)
                                    .size_full()
                                    .text_base()
                                    .text_color(colors.text_primary)
                                    .placeholder("Enter a value for this workflow run…"),
                            ),
                    ),
            )
            .child(tags_section(run_tags, tags_loading, tags_error, colors))
            .child(job_runs_section(
                job_runs,
                jobs_loading,
                jobs_error,
                colors,
                run_id.clone(),
                navigate,
            ))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(Button::new("detail-runs", "Workflow runs").on_click(
                        move |_, window, cx| {
                            list_navigation(
                                &Routes::WorkflowRuns(WorkflowRunsRoutes::Index),
                                window,
                                cx,
                            );
                        },
                    ))
                    .child(Button::new("detail-new", "New workflow run").on_click(
                        move |_, window, cx| {
                            new_run_navigation(
                                &Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                                    id: run_id.clone(),
                                    routes: WorkflowRunRoutes::New,
                                }),
                                window,
                                cx,
                            );
                        },
                    )),
            )
    }
}

fn tags_section(
    tags: &[TagRow],
    loading: bool,
    error: Option<&str>,
    colors: crate::theme::Colors,
) -> gpui::Div {
    let content = if loading {
        div()
            .id("workflow-run-tags-loading")
            .text_color(colors.text_secondary)
            .child("Loading tags…")
    } else if let Some(error) = error {
        div()
            .id("workflow-run-tags-error")
            .text_color(colors.error)
            .child(format!("Unable to load tags: {error}"))
    } else if tags.is_empty() {
        div()
            .id("workflow-run-tags-empty")
            .text_color(colors.text_secondary)
            .child("No tags recorded for this workflow run.")
    } else {
        div()
            .id("workflow-run-tags-table")
            .flex_1()
            .min_h_0()
            .overflow_scroll()
            .w_full()
            .border_1()
            .border_color(colors.border_muted)
            .child(
                div()
                    .flex()
                    .w_full()
                    .bg(colors.surface_raised)
                    .border_b_1()
                    .border_color(colors.border_muted)
                    .child(tag_table_cell("Key").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(tag_table_cell("Value").font_weight(gpui::FontWeight::SEMIBOLD)),
            )
            .children(
                tags.iter()
                    .enumerate()
                    .map(|(index, tag)| tag_row(tag, index, colors)),
            )
    };

    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .min_h_0()
        .child(
            div().flex().items_center().justify_between().child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(format!("Tags ({})", tags.len())),
            ),
        )
        .child(content)
}

fn tag_table_cell(value: impl Into<gpui::SharedString>) -> gpui::Div {
    div()
        .flex_1()
        .min_w_0()
        .px_4()
        .py_3()
        .truncate()
        .child(value.into())
}

fn tag_row(tag: &TagRow, index: usize, colors: crate::theme::Colors) -> impl IntoElement {
    div()
        .id(format!("workflow-run-tag-{index}"))
        .flex()
        .w_full()
        .border_b_1()
        .border_color(colors.border_muted)
        .hover(|style| style.bg(colors.surface_raised))
        .child(tag_table_cell(tag.key.clone()))
        .child(tag_table_cell(tag.value.clone()))
}

fn job_runs_section(
    job_runs: &[JobRunSummaryRow],
    loading: bool,
    error: Option<&str>,
    colors: crate::theme::Colors,
    run_id: WorkflowRunId,
    on_navigate: NavigationHandler,
) -> gpui::Div {
    let content = if loading && job_runs.is_empty() {
        div()
            .id("workflow-run-jobs-loading")
            .text_color(colors.text_secondary)
            .child("Loading jobs…")
    } else if let Some(error) = error {
        div()
            .id("workflow-run-jobs-error")
            .text_color(colors.error)
            .child(format!("Unable to load jobs: {error}"))
    } else if job_runs.is_empty() {
        div()
            .id("workflow-run-jobs-empty")
            .text_color(colors.text_secondary)
            .child("No jobs recorded for this workflow run.")
    } else {
        div()
            .id("workflow-run-jobs-table")
            .flex_1()
            .min_h_0()
            .overflow_scroll()
            .w_full()
            .border_1()
            .border_color(colors.border_muted)
            .child(
                div()
                    .flex()
                    .w_full()
                    .bg(colors.surface_raised)
                    .border_b_1()
                    .border_color(colors.border_muted)
                    .child(job_table_cell("Status").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(job_table_cell("Job ID").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(job_table_cell("Job run ID").font_weight(gpui::FontWeight::SEMIBOLD)),
            )
            .children(
                job_runs.iter().map(|job_run| {
                    job_run_row(job_run, colors, run_id.clone(), on_navigate.clone())
                }),
            )
    };

    div()
        .flex()
        .flex_col()
        .gap_2()
        .flex_1()
        .min_h_0()
        .child(
            div().flex().items_center().justify_between().child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(format!("Jobs ({})", job_runs.len())),
            ),
        )
        .child(content)
}

fn job_table_cell(value: impl Into<gpui::SharedString>) -> gpui::Div {
    div()
        .flex_1()
        .min_w_0()
        .px_4()
        .py_3()
        .truncate()
        .child(value.into())
}

fn job_run_row(
    job_run: &JobRunSummaryRow,
    colors: crate::theme::Colors,
    run_id: WorkflowRunId,
    on_navigate: NavigationHandler,
) -> impl IntoElement {
    let job_id = job_run.job_id.clone();
    let job_run_id = job_run.job_run_id.clone();
    let route = Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
        id: run_id,
        routes: WorkflowRunRoutes::JobLog { job_run_id, job_id },
    });

    div()
        .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
            on_navigate(&route, window, cx);
        })
        .id(format!("workflow-run-job-{}", job_run.job_run_id))
        .flex()
        .w_full()
        .border_b_1()
        .border_color(colors.border_muted)
        .hover(|style| style.bg(colors.surface_raised))
        .child(
            job_table_cell(job_run.status.clone())
                .text_color(job_status_color(&job_run.status, colors)),
        )
        .child(job_table_cell(job_run.job_id.clone()))
        .child(job_table_cell(job_run.job_run_id.clone()))
}

fn job_status_color(status: &str, colors: crate::theme::Colors) -> gpui::Hsla {
    match status {
        "running" => colors.warning,
        "succeeded" => colors.success,
        "failed" => colors.error,
        _ => colors.text_primary,
    }
}
