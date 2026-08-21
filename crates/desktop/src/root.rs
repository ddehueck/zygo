use gpui::{AnyElement, App, Context, Entity, Render, RenderOnce, Window, div, prelude::*};

use crate::{
    Routes, dependencies, features,
    features::runs::ui::RunDetailView,
    navigation::{WorkflowRunRoutes, WorkflowRunsRoutes},
    theme, ui,
};

pub struct ZygoDesktop {
    detail_view: Entity<RunDetailView>,
}

impl ZygoDesktop {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // TODO: should the router be incharge of loading view entities lazily as needed?
        // Or just have optional entities on the root entity that we lazy load via helpers
        Self {
            detail_view: cx.new(RunDetailView::new),
        }
    }
}

impl Render for ZygoDesktop {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<theme::Theme>().colors;

        // Catch error on startup when initializing dependencies
        if !cx.has_global::<dependencies::AppDeps>() {
            let startup_message = cx
                .global::<dependencies::AppStartup>()
                .error()
                .map(str::to_owned)
                .unwrap_or_else(|| "Starting Zygo...".to_owned());

            return RootLoading::new(startup_message).into_any_element();
        }

        let navigator = dependencies::use_navigator(cx);
        let current_route = navigator.read(cx).current().clone();

        let content: AnyElement = match current_route {
            Routes::WorkflowRuns(WorkflowRunsRoutes::Index) => {
                features::runs::ui::RunListView::new().into_any_element()
            }
            Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                id,
                routes: WorkflowRunRoutes::Index,
            }) => {
                self.detail_view.update(cx, |view, cx| {
                    view.set_run_id(id, cx);
                });
                self.detail_view.clone().into_any_element()
            }
            Routes::WorkflowRuns(WorkflowRunsRoutes::Run {
                id,
                routes: WorkflowRunRoutes::New,
            }) => features::runs::ui::NewRunView::new(id).into_any_element(),
        };

        div()
            .id("root")
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.surface_base)
            .text_color(colors.text_primary)
            .child(ui::Titlebar)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(div().flex_1().min_h_0().overflow_hidden().child(content)),
            )
            .into_any_element()
    }
}

#[derive(IntoElement)]
struct RootLoading {
    startup_message: String,
}

impl RootLoading {
    fn new(startup_message: String) -> Self {
        Self { startup_message }
    }
}

impl RenderOnce for RootLoading {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.global::<theme::Theme>().colors;

        div()
            .id("root-loading")
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.surface_base)
            .text_color(colors.text_primary)
            .child(ui::Titlebar)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(colors.text_secondary)
                    .child(self.startup_message),
            )
    }
}
