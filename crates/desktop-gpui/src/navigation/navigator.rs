use std::rc::Rc;

use gpui::{App, Context, Window};

use super::{Breadcrumb, Routes};

pub type NavigationHandler = Rc<dyn Fn(&Routes, &mut Window, &mut App) + 'static>;

pub struct Navigator {
    stack: Vec<Routes>,
}

impl Navigator {
    pub fn new(initial: Routes) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    pub fn current(&self) -> &Routes {
        self.stack.last().expect("navigator cannot be empty")
    }

    pub fn breadcrumbs(&self) -> Vec<Breadcrumb> {
        self.current().breadcrumbs()
    }

    pub fn can_go_back(&self) -> bool {
        self.stack.len() > 1
    }

    pub fn push<T: 'static>(&mut self, route: Routes, cx: &mut Context<T>) {
        self.stack.push(route);
        cx.notify();
    }

    pub fn replace<T: 'static>(&mut self, route: Routes, cx: &mut Context<T>) {
        *self.stack.last_mut().expect("navigator cannot be empty") = route;
        cx.notify();
    }

    pub fn back<T: 'static>(&mut self, cx: &mut Context<T>) {
        if self.can_go_back() {
            self.stack.pop();
            cx.notify();
        }
    }

    pub fn reset<T: 'static>(&mut self, route: Routes, cx: &mut Context<T>) {
        self.stack.clear();
        self.stack.push(route);
        cx.notify();
    }
}
