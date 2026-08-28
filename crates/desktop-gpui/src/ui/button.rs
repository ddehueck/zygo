use gpui::{
    App, ElementId, MouseButton, MouseDownEvent, RenderOnce, SharedString, Window, div, prelude::*,
};

use crate::theme::Theme;

type ClickHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.global::<Theme>().colors;
        let mut button = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .px_5()
            .py_3()
            .rounded_lg()
            .bg(colors.accent)
            .text_color(colors.on_accent)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .hover(move |style| style.bg(colors.accent_hover))
            .active(move |style| style.bg(colors.accent_active));

        if let Some(on_click) = self.on_click {
            button = button.on_mouse_down(MouseButton::Left, on_click);
        }

        button.child(self.label)
    }
}
