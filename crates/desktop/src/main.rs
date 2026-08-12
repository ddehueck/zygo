use gpui::{
    App, Application, Bounds, Context, MouseButton, MouseDownEvent, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};

struct ZygoDesktop {
    runs_started: usize,
}

impl Render for ZygoDesktop {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let run_label = match self.runs_started {
            0 => "No local runs started".to_owned(),
            1 => "1 local run started".to_owned(),
            count => format!("{count} local runs started"),
        };

        div()
            .id("zygo-desktop")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0b1020))
            .text_color(rgb(0xe7eaf0))
            .p_8()
            .gap_8()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("ZYGO"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x8b95aa))
                            .child("Workflow runtime · Desktop preview"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(560.0))
                            .p_8()
                            .gap_5()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(0x27324a))
                            .bg(rgb(0x121a2d))
                            .shadow_xl()
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Build workflows that move."),
                            )
                            .child(
                                div()
                                    .text_color(rgb(0xaab2c3))
                                    .line_height(gpui::relative(1.5))
                                    .child(
                                        "Compose, inspect, and run dependable data workflows from one focused workspace.",
                                    ),
                            )
                            .child(
                                div()
                                    .id("start-run")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .px_5()
                                    .py_3()
                                    .rounded_lg()
                                    .bg(rgb(0x6d5dfc))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x7d70ff)))
                                    .active(|style| style.bg(rgb(0x5949e8)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|view, _: &MouseDownEvent, _, cx| {
                                            view.runs_started += 1;
                                            cx.notify();
                                        }),
                                    )
                                    .child("Start a local run"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x7f8aa3))
                                    .child(run_label),
                            ),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| ZygoDesktop { runs_started: 0 }),
        )
        .expect("failed to open Zygo desktop window");
    });
}
