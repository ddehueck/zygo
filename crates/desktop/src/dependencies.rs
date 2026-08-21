use std::{rc::Rc, sync::Arc};

use gpui::{App, AppContext, Entity, Global};
use local::ZygoLocalService;

use crate::{
    navigation::{NavigationHandler, Navigator, Routes, WorkflowRunsRoutes},
    stores::{TagStore, WorkflowRunStore},
};

/// Application-wide handles for shared state.
///
/// The service is retained here so the stores and future command-oriented
/// features share the same local service lifetime and database connection.
#[derive(Clone, Default)]
pub struct AppStartup {
    error: Option<String>,
}

impl Global for AppStartup {}

impl AppStartup {
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }
}

#[derive(Clone)]
pub struct AppDeps {
    service: Arc<ZygoLocalService>,
    navigator: Entity<Navigator>,
    navigation: NavigationHandler,
    runs: Entity<WorkflowRunStore>,
    tags: Entity<TagStore>,
}

impl Global for AppDeps {}

impl AppDeps {
    pub fn new(service: ZygoLocalService, cx: &mut App) -> Self {
        let service = Arc::new(service);
        let runs_repository = service.repos.workflow_runs.clone();
        let tags_repository = service.repos.workflow_runs.clone();

        let navigator = cx.new(|_| Navigator::new(Routes::WorkflowRuns(WorkflowRunsRoutes::Index)));
        let navigation_navigator = navigator.clone();
        let navigation: NavigationHandler = Rc::new(move |route, _, cx| {
            let _ = navigation_navigator.update(cx, |navigator, cx| {
                navigator.push(route.clone(), cx);
            });
        });

        Self {
            service,
            navigator,
            navigation,
            runs: cx.new(|cx| WorkflowRunStore::new(runs_repository, cx)),
            tags: cx.new(|_| TagStore::new(tags_repository)),
        }
    }

    pub fn service(&self) -> Arc<ZygoLocalService> {
        self.service.clone()
    }

    pub fn navigator(&self) -> Entity<Navigator> {
        self.navigator.clone()
    }

    pub fn navigation(&self) -> NavigationHandler {
        self.navigation.clone()
    }

    pub fn runs(&self) -> Entity<WorkflowRunStore> {
        self.runs.clone()
    }

    pub fn tags(&self) -> Entity<TagStore> {
        self.tags.clone()
    }
}

pub fn use_app_dependencies(cx: &App) -> AppDeps {
    cx.global::<AppDeps>().clone()
}

pub fn use_navigator(cx: &App) -> Entity<Navigator> {
    use_app_dependencies(cx).navigator()
}

pub fn use_navigation(cx: &App) -> NavigationHandler {
    use_app_dependencies(cx).navigation()
}

pub fn use_runs(cx: &App) -> Entity<WorkflowRunStore> {
    use_app_dependencies(cx).runs()
}

pub fn use_tags(cx: &App) -> Entity<TagStore> {
    use_app_dependencies(cx).tags()
}

pub fn use_service(cx: &App) -> Arc<ZygoLocalService> {
    use_app_dependencies(cx).service()
}
