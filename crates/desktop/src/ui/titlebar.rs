use gpui::*;

use crate::theme::Theme;

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
                    .justify_center()
                    // Allows dragging and native titlebar interactions.
                    .window_control_area(WindowControlArea::Drag)
                    .child("zygo")
                    .text_color(colors.text_tertiary),
            )
    }
}
