use gpui::*;

use crate::{dependencies, theme::Theme};

#[derive(IntoElement)]
pub struct Titlebar;

impl RenderOnce for Titlebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;

        div()
            .h(px(34.0))
            .w_full()
            .flex()
            .items_center()
            .flex_shrink_0()
            .bg(colors.surface_base)
            .border_b_1()
            .border_color(colors.border_muted)
            // Keep content clear of the native traffic lights.
            .child(div().w(px(82.0)).flex_shrink_0())
            .child(
                div()
                    .h_full()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_start()
                    // Allows dragging and native titlebar interactions.
                    .window_control_area(WindowControlArea::Drag)
                    .child(Breadcrumbs),
            )
    }
}

#[derive(IntoElement)]
struct Breadcrumbs;

impl RenderOnce for Breadcrumbs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let mut breadcrumb_bar = div().flex().items_center();

        // The titlebar renders during startup before navigation dependencies exist.
        if !cx.has_global::<dependencies::AppDeps>() {
            return breadcrumb_bar
                .text_color(colors.text_tertiary)
                .child("zygo");
        }

        let breadcrumbs = dependencies::use_navigator(cx).read(cx).breadcrumbs();
        let navigate = dependencies::use_navigation(cx);

        for (index, breadcrumb) in breadcrumbs.into_iter().enumerate() {
            if index > 0 {
                breadcrumb_bar =
                    breadcrumb_bar.child(div().mx_1().text_color(colors.text_tertiary).child("/"));
            }

            let route = breadcrumb.route;
            let navigate = navigate.clone();
            breadcrumb_bar = breadcrumb_bar.child(
                div()
                    .id(format!("titlebar-breadcrumb-{index}"))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .text_color(colors.text_secondary)
                    .hover(move |style| style.bg(colors.surface_raised))
                    .active(move |style| style.bg(colors.surface_sunken))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        navigate(&route, window, cx);
                    })
                    .child(breadcrumb.label),
            );
        }

        breadcrumb_bar
    }
}
