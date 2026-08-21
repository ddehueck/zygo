use gpui::{App, RenderOnce, Window, div, prelude::*};
use zygo_core::models::WorkflowRunId;

use crate::{
    Routes, dependencies,
    navigation::{WorkflowRunRoutes, WorkflowRunsRoutes},
    theme::Theme,
    ui::Button,
};

#[derive(IntoElement)]
pub struct NewRunView {
    example_run_id: WorkflowRunId,
}

impl NewRunView {
    pub fn new(example_run_id: WorkflowRunId) -> Self {
        Self { example_run_id }
    }
}

impl RenderOnce for NewRunView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let navigate = dependencies::use_navigation(cx);
        let list_navigation = navigate.clone();
        let detail_navigation = navigate;
        let example_run_id = self.example_run_id;

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
                    .child("New workflow run"),
            )
            .child(
                div()
                    .text_color(colors.text_secondary)
                    .child("This is a stub for the future run configuration flow."),
            )
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
                    ))
                    .child(Button::new("new-detail", "Open example detail").on_click(
                        move |_, window, cx| {
                            detail_navigation(
                                &Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                                    id: example_run_id.clone(),
                                    routes: WorkflowRunRoutes::Index,
                                }),
                                window,
                                cx,
                            );
                        },
                    )),
            )
    }
}
