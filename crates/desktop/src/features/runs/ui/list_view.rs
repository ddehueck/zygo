use gpui::{Context, Entity, MouseButton, Render, Window, div, prelude::*, px};
use local::db::WorkflowRun;
use zygo_core::models::WorkflowRunId;

use crate::{
    Routes, dependencies,
    navigation::{NavigationHandler, WorkflowRunRoutes, WorkflowRunsRoutes},
    theme::Theme,
    ui::{SidebarLayout, SidebarSide},
};

pub struct RunListView {
    layout: Entity<SidebarLayout>,
}

impl RunListView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|_| RunListSidebar);
        let content = cx.new(|_| RunListContent);
        let layout = cx.new(|_| {
            SidebarLayout::new(sidebar, content)
                .sidebar_side(SidebarSide::Right)
                .min_sidebar_width(px(180.0))
                .max_sidebar_width(px(360.0))
                .sidebar_width(px(220.0))
        });

        Self { layout }
    }
}

impl Render for RunListView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.layout.clone())
    }
}

struct RunListSidebar;

impl Render for RunListSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;

        div()
            .flex()
            .flex_col()
            .size_full()
            .gap_5()
            .p_5()
            .bg(colors.surface_sunken)
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Run explorer"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .text_color(colors.text_secondary)
                    .child("Drag the divider to resize this sidebar."),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pt_3()
                    .border_t_1()
                    .border_color(colors.border_muted)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.text_primary)
                            .child("Filters"),
                    )
                    .child(
                        div()
                            .text_color(colors.text_secondary)
                            .child("All workflow runs"),
                    ),
            )
    }
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
                    .child(table_cell("Content hash").font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(table_cell("Created at").font_weight(gpui::FontWeight::SEMIBOLD)),
            )
            .children(
                store
                    .workflow_runs()
                    .iter()
                    .map(|run| run_row(run, colors, navigate.clone())),
            );

        let body = if store.is_loading() && store.workflow_runs().is_empty() {
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
                .child("No workflow runs yet.")
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
    run: &WorkflowRun,
    colors: crate::theme::Colors,
    on_navigate: NavigationHandler,
) -> impl IntoElement {
    let mut row = div()
        .id(format!("workflow-run-{}", run.id))
        .flex()
        .w_full()
        .border_b_1()
        .border_color(colors.border_muted)
        .hover(|style| style.bg(colors.surface_raised))
        .child(table_cell(run.id.clone()))
        .child(table_cell(run.content_hash.clone()))
        .child(table_cell(run.created_at.clone()));

    if let Ok(run_id) = WorkflowRunId::try_from(run.id.clone()) {
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
