use std::{rc::Rc, sync::Arc};

use gpui::{App, AppContext, Entity, Global};
use local::ZygoLocalService;
use tokio::runtime::{Handle, Runtime};

use crate::{
    features::runs::RunSync,
    navigation::{NavigationHandler, Navigator, Routes, WorkflowRunsRoutes},
    stores::{TagStore, WorkflowRunDetailStore, WorkflowRunStore},
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
    run_details: Entity<WorkflowRunDetailStore>,
    run_sync: Entity<RunSync>,
    tokio_handle: Handle,
    _tokio_runtime: Arc<Runtime>,
}

impl Global for AppDeps {}

impl AppDeps {
    pub fn new(service: ZygoLocalService, cx: &mut App) -> Self {
        let tokio_runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to create the desktop Tokio runtime"),
        );
        let tokio_handle = tokio_runtime.handle().clone();
        let service = Arc::new(service);
        let runs_repository = service.repos.workflow_runs.clone();
        let run_summaries_repository = service.repos.workflow_run_summaries.clone();
        let job_run_summaries_repository = service.repos.job_run_summaries.clone();
        let tags_repository = service.repos.workflow_runs.clone();

        let runs =
            cx.new(|cx| WorkflowRunStore::new(runs_repository, run_summaries_repository, cx));
        let run_details = cx.new(|_| WorkflowRunDetailStore::new(job_run_summaries_repository));
        let run_sync =
            cx.new(|cx| RunSync::new(service.clone(), runs.clone(), run_details.clone(), cx));

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
            runs,
            tags: cx.new(|_| TagStore::new(tags_repository)),
            run_details,
            run_sync,
            tokio_handle,
            _tokio_runtime: tokio_runtime,
        }
    }

    pub fn tokio_handle(&self) -> Handle {
        self.tokio_handle.clone()
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

pub fn use_run_details(cx: &App) -> Entity<WorkflowRunDetailStore> {
    use_app_dependencies(cx).run_details.clone()
}

pub fn use_run_sync(cx: &App) -> Entity<RunSync> {
    use_app_dependencies(cx).run_sync.clone()
}

pub fn use_service(cx: &App) -> Arc<ZygoLocalService> {
    use_app_dependencies(cx).service()
}

pub fn use_tokio_handle(cx: &App) -> Handle {
    use_app_dependencies(cx).tokio_handle()
}
