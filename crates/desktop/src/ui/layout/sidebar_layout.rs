use gpui::{
    AnyView, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Render, Window, div, prelude::*, px,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SidebarSide {
    Left,
    Right,
}

pub struct SidebarLayout {
    sidebar: AnyView,
    content: AnyView,
    sidebar_side: SidebarSide,
    sidebar_width: Pixels,
    min_sidebar_width: Pixels,
    max_sidebar_width: Pixels,
    is_resizing: bool,
    resize_start_x: Pixels,
    resize_start_width: Pixels,
}

impl SidebarLayout {
    pub fn new(sidebar: impl Into<AnyView>, content: impl Into<AnyView>) -> Self {
        Self {
            sidebar: sidebar.into(),
            content: content.into(),
            sidebar_side: SidebarSide::Left,
            sidebar_width: px(240.0),
            min_sidebar_width: px(160.0),
            max_sidebar_width: px(420.0),
            is_resizing: false,
            resize_start_x: px(0.0),
            resize_start_width: px(240.0),
        }
    }

    pub fn sidebar_side(mut self, side: SidebarSide) -> Self {
        self.sidebar_side = side;
        self
    }

    pub fn sidebar_width(mut self, width: Pixels) -> Self {
        self.sidebar_width = width.clamp(self.min_sidebar_width, self.max_sidebar_width);
        self
    }

    pub fn min_sidebar_width(mut self, width: Pixels) -> Self {
        self.min_sidebar_width = width;
        self.sidebar_width = self.sidebar_width.max(width);
        self
    }

    pub fn max_sidebar_width(mut self, width: Pixels) -> Self {
        self.max_sidebar_width = width;
        self.sidebar_width = self.sidebar_width.min(width);
        self
    }

    fn start_resize(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }

        self.is_resizing = true;
        self.resize_start_x = event.position.x;
        self.resize_start_width = self.sidebar_width;
        cx.notify();
    }

    fn resize(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_resizing || !event.dragging() {
            return;
        }

        let movement = event.position.x - self.resize_start_x;
        let movement = match self.sidebar_side {
            SidebarSide::Left => movement,
            SidebarSide::Right => -movement,
        };
        let width = (self.resize_start_width + movement)
            .clamp(self.min_sidebar_width, self.max_sidebar_width);

        if width != self.sidebar_width {
            self.sidebar_width = width;
            cx.notify();
        }
    }

    fn stop_resize(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_resizing {
            self.is_resizing = false;
            cx.notify();
        }
    }
}

impl Render for SidebarLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.global::<crate::theme::Theme>().colors;

        let sidebar = self.sidebar.clone();
        let content = self.content.clone();

        let sidebar_side = self.sidebar_side;
        let sidebar_width = self.sidebar_width;

        let resize_handle = div()
            .id("sidebar-resize-handle")
            .absolute()
            .top_0()
            .bottom_0()
            .w(px(8.0))
            .flex()
            .justify_center()
            .cursor(CursorStyle::ResizeColumn)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::start_resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::stop_resize))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::stop_resize))
            .child(div().w(px(1.0)).h_full().bg(if self.is_resizing {
                colors.accent
            } else {
                colors.border_muted
            }));

        let resize_handle = match sidebar_side {
            SidebarSide::Left => resize_handle.left(sidebar_width - px(4.0)),
            SidebarSide::Right => resize_handle.right(sidebar_width - px(4.0)),
        };

        let sidebar = div()
            .id("sidebar-layout-sidebar")
            .h_full()
            .w(sidebar_width)
            .flex_shrink_0()
            .overflow_hidden()
            .child(sidebar);

        let content = div()
            .id("sidebar-layout-content")
            .h_full()
            .min_w_0()
            .min_h_0()
            .flex_1()
            .overflow_hidden()
            .child(content);

        let layout = match sidebar_side {
            SidebarSide::Left => div().flex().size_full().child(sidebar).child(content),
            SidebarSide::Right => div().flex().size_full().child(content).child(sidebar),
        };

        // TODO: id will need to be set by the caller
        layout
            .id("sidebar-layout")
            .relative()
            .child(resize_handle)
            .on_mouse_move(cx.listener(Self::resize))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::stop_resize))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::stop_resize))
    }
}
