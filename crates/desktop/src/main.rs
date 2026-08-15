use gpui::{
    App, Application, Bounds, Context, Render, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, point, prelude::*, px, size,
};

mod theme;
mod ui;

struct ZygoDesktop {
    runs_started: usize,
}

impl Render for ZygoDesktop {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<theme::Theme>().colors;
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
            .bg(colors.surface_base)
            .text_color(colors.text_primary)
            .child(ui::Titlebar)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_8()
                    .gap_8()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(560.0))
                            .p_8()
                            .gap_5()
                            .rounded_xl()
                            .border_1()
                            .border_color(colors.border)
                            .bg(colors.surface_base)
                            .shadow_xl()
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Build workflows that move."),
                            )
                            .child(
                                div()
                                    .text_color(colors.text_secondary)
                                    .line_height(gpui::relative(1.5))
                                    .child(
                                        "Compose, inspect, and run dependable data workflows from one focused workspace.",
                                    ),
                            )
                            .child(
                                ui::Button::new("start-run", "Start a local run").on_click(
                                    cx.listener(|view, _, _, cx| {
                                        view.runs_started += 1;
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.text_tertiary)
                                    .child(run_label),
                            ),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.set_global(theme::Theme::dark());

        let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(14.0), px(10.0))),
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| ZygoDesktop { runs_started: 0 }),
        )
        .expect("failed to open Zygo desktop window");
    });
}
