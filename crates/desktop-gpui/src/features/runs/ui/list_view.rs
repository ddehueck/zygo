use gpui::{
    AppContext, Context, Entity, MouseButton, Render, Subscription, Window, div, prelude::*, px,
};
use gpuikit::elements::input::input;
use gpuikit::input::InputState;
use local::{TagRow, WorkflowRunSummaryRow};
use zygo_core::models::WorkflowRunId;

use crate::{
    Routes, dependencies,
    features::runs::filters::FilterSet,
    navigation::{NavigationHandler, WorkflowRunRoutes, WorkflowRunsRoutes},
    theme::Theme,
    ui::{SidebarLayout, SidebarSide},
};

pub struct RunListView {
    layout: Entity<SidebarLayout>,
}

impl RunListView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(RunListSidebar::new);
        let content = cx.new(|_| RunListContent);
        let layout = cx.new(|_| {
            SidebarLayout::new(sidebar, content)
                .sidebar_side(SidebarSide::Right)
                .min_sidebar_width(px(200.0))
                .max_sidebar_width(px(500.0))
                .sidebar_width(px(250.0))
        });

        Self { layout }
    }
}

impl Render for RunListView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.layout.clone())
    }
}

const TAG_PREVIEW_LIMIT: usize = 8;

struct RunListSidebar {
    tag_key_input: Entity<InputState>,
    tag_value_input: Entity<InputState>,
    _filter_subscriptions: Vec<Subscription>,
}

impl RunListSidebar {
    fn new(cx: &mut Context<Self>) -> Self {
        let tag_key_input = cx.new(|cx| InputState::new_singleline(cx));
        let tag_value_input = cx.new(|cx| InputState::new_singleline(cx));
        let key_input_for_filter = tag_key_input.clone();
        let value_input_for_filter = tag_value_input.clone();
        let key_subscription = cx.observe(&tag_key_input, move |_, _, cx| {
            apply_tag_filter(&key_input_for_filter, &value_input_for_filter, cx);
        });
        let key_input_for_filter = tag_key_input.clone();
        let value_input_for_filter = tag_value_input.clone();
        let value_subscription = cx.observe(&tag_value_input, move |_, _, cx| {
            apply_tag_filter(&key_input_for_filter, &value_input_for_filter, cx);
        });

        Self {
            tag_key_input,
            tag_value_input,
            _filter_subscriptions: vec![key_subscription, value_subscription],
        }
    }
}

fn apply_tag_filter(
    tag_key_input: &Entity<InputState>,
    tag_value_input: &Entity<InputState>,
    cx: &mut Context<RunListSidebar>,
) {
    let key = tag_key_input.read(cx).content().to_owned();
    let value = tag_value_input.read(cx).content().to_owned();
    let filter_set = FilterSet::from_inputs(&key, &value);
    let runs = dependencies::use_runs(cx);

    runs.update(cx, |store, cx| {
        store.filter(filter_set);
        cx.notify();
    });
}

impl Render for RunListSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let runs = dependencies::use_runs(cx);
        let available_tags = runs.read(cx).available_tags().to_vec();
        let tag_key_input = self.tag_key_input.clone();
        let tag_value_input = self.tag_value_input.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .gap_5()
            .p_5()
            .bg(colors.surface_base)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.text_primary)
                            .child("Filter By Tag"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(filter_input(
                                "workflow-run-tag-key-filter",
                                &tag_key_input,
                                "Tag",
                                "environment",
                                colors,
                                cx,
                            ))
                            .child(filter_input(
                                "workflow-run-tag-value-filter",
                                &tag_value_input,
                                "Value",
                                "production",
                                colors,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.text_tertiary)
                            .child("Leave value empty to match any value."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child("Available tags"),
                    )
                    .child(div().flex().flex_wrap().gap_1().children(
                        available_tags.iter().take(TAG_PREVIEW_LIMIT).map(|tag| {
                            tag_badge(tag, colors, tag_key_input.clone(), tag_value_input.clone())
                        }),
                    )),
            )
    }
}

fn filter_input(
    id: &'static str,
    input_state: &Entity<InputState>,
    label: &'static str,
    placeholder: &'static str,
    colors: crate::theme::Colors,
    cx: &mut Context<RunListSidebar>,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .flex_col()
        .gap_1()
        .flex_1()
        .child(
            div()
                .text_xs()
                .text_color(colors.text_tertiary)
                .child(label),
        )
        .child(
            div()
                .flex()
                .items_center()
                .w_full()
                .h(px(32.0))
                .px_2()
                .rounded_md()
                .border_1()
                .border_color(colors.border_muted)
                .bg(colors.surface_input)
                .child(
                    input(input_state, cx)
                        .size_full()
                        .text_sm()
                        .text_color(colors.text_primary)
                        .placeholder(placeholder),
                ),
        )
}

fn tag_badge(
    tag: &TagRow,
    colors: crate::theme::Colors,
    tag_key_input: Entity<InputState>,
    tag_value_input: Entity<InputState>,
) -> impl IntoElement {
    let key = tag.key.clone();
    let value = tag.value.clone();
    let label = format!("{}={}", key, value);
    let badge_id = format!("workflow-run-tag-preview-{}-{}", key, value);

    div()
        .id(badge_id)
        .self_start()
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(colors.accent)
        .text_xs()
        .text_color(colors.accent)
        .hover(|style| style.bg(colors.surface_raised))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            tag_key_input.update(cx, |input, cx| input.set_content(key.clone(), cx));
            tag_value_input.update(cx, |input, cx| input.set_content(value.clone(), cx));
        })
        .child(label)
}

struct RunListContent;

impl Render for RunListContent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let navigate = dependencies::use_navigation(cx);
        let runs = dependencies::use_runs(cx);
        let store = runs.read(cx);

        let table = div()
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
                    .child(table_cell("ID").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Status").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Active jobs").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Succeeded").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Errored").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Started at").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Completed at").font_weight(gpui::FontWeight::SEMIBOLD)),
            )
            .children(
                store
                    .workflow_runs()
                    .iter()
                    .map(|run| run_row(run, colors, navigate.clone())),
            );

        let body = if store.is_loading() && store.all_workflow_runs().is_empty() {
            div()
                .id("workflow-runs-loading")
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(colors.text_secondary)
                .child("Loading workflow runs…")
        } else if let Some(error) = store.error() {
            div()
                .id("workflow-runs-error")
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(colors.error)
                .child(format!("Unable to load workflow runs: {error}"))
        } else if store.workflow_runs().is_empty() {
            div()
                .id("workflow-runs-empty")
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(colors.text_secondary)
                .child(if store.active_filter().is_empty() {
                    "No workflow runs yet."
                } else {
                    "No workflow runs match these filters."
                })
        } else {
            div()
                .id("workflow-runs-table")
                .flex_1()
                .min_h_0()
                .overflow_scroll()
                .child(table)
        };

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
                    .child("Workflow runs"),
            )
            .child(body)
    }
}

fn table_cell(value: impl Into<gpui::SharedString>) -> gpui::Div {
    div()
        .flex_1()
        .min_w_0()
        .px_4()
        .py_3()
        .truncate()
        .child(value.into())
}

fn run_row(
    run: &WorkflowRunSummaryRow,
    colors: crate::theme::Colors,
    on_navigate: NavigationHandler,
) -> impl IntoElement {
    let mut row = div()
        .id(format!("workflow-run-{}", run.workflow_run_id))
        .flex()
        .w_full()
        .border_b_1()
        .border_color(colors.border_muted)
        .hover(|style| style.bg(colors.surface_raised))
        .child(table_cell(run.workflow_run_id.clone()))
        .child(table_cell(run.status.clone()))
        .child(table_cell(run.active_job_count.to_string()))
        .child(table_cell(run.succeeded_job_count.to_string()))
        .child(table_cell(run.errored_job_count.to_string()))
        .child(table_cell(
            run.started_at
                .map(|timestamp| timestamp.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ))
        .child(table_cell(
            run.completed_at
                .map(|timestamp| timestamp.to_string())
                .unwrap_or_else(|| "—".to_owned()),
        ));

    if let Ok(run_id) = WorkflowRunId::try_from(run.workflow_run_id.clone()) {
        row = row.on_mouse_down(MouseButton::Left, move |_, window, cx| {
            on_navigate(
                &Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                    id: run_id.clone(),
                    routes: WorkflowRunRoutes::Index,
                }),
                window,
                cx,
            );
        });
    }

    row
}
