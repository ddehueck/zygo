use gpui::{AppContext, Context, EventEmitter, Task};
use local::db::{WorkflowRun, WorkflowRunRepository};

#[derive(Debug, Clone)]
pub enum WorkflowRunStoreEvent {
    Loaded { count: usize },
    LoadFailed { message: String },
}

pub struct WorkflowRunStore {
    repository: WorkflowRunRepository,
    workflow_runs: Vec<WorkflowRun>,
    loading: bool,
    error: Option<String>,
}

impl EventEmitter<WorkflowRunStoreEvent> for WorkflowRunStore {}

impl WorkflowRunStore {
    pub fn new(repository: WorkflowRunRepository, cx: &mut Context<Self>) -> Self {
        let mut store = Self {
            repository,
            workflow_runs: Vec::new(),
            loading: false,
            error: None,
        };
        store.refresh(cx).detach();
        store
    }

    pub fn workflow_runs(&self) -> &[WorkflowRun] {
        &self.workflow_runs
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) -> Task<()> {
        self.loading = true;
        self.error = None;
        cx.notify();

        let repository = self.repository.clone();
        let task = cx.spawn(async move |store, cx| {
            let result = cx
                .background_spawn(async move { repository.list_all().await })
                .await;

            let _ = store.update(cx, |store, cx| {
                store.loading = false;
                match result {
                    Ok(workflow_runs) => {
                        let count = workflow_runs.len();
                        store.workflow_runs = workflow_runs;
                        cx.emit(WorkflowRunStoreEvent::Loaded { count });
                    }
                    Err(error) => {
                        let message = error.to_string();
                        store.error = Some(message.clone());
                        cx.emit(WorkflowRunStoreEvent::LoadFailed { message });
                    }
                }
                cx.notify();
            });
        });

        task
    }
}
