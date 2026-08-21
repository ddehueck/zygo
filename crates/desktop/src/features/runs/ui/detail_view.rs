use gpui::{Context, Render, Window, div, prelude::*, px};
use gpuikit::elements::input::input;
use gpuikit::input::InputState;
use zygo_core::models::WorkflowRunId;

use crate::{
    Routes, dependencies,
    navigation::{WorkflowRunRoutes, WorkflowRunsRoutes},
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
            self.run_id = Some(run_id);
            cx.notify();
        }
    }
}

impl Render for RunDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let navigate = dependencies::use_navigation(cx);
        let list_navigation = navigate.clone();
        let new_run_navigation = navigate;
        let run_id = self
            .run_id
            .as_ref()
            .expect("detail view must have a run ID before rendering")
            .clone();
        let workflow_run_input = self.workflow_run_input.clone();

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
                    .child(format!("Stub detail view for run {run_id}.")),
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
