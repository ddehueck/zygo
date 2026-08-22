use std::collections::{HashMap, HashSet};

use gpui::{AppContext, Context, EventEmitter, Task};
use local::{TagRow, WorkflowRunRepository};
use zygo_core::models::WorkflowRunId;

#[derive(Debug, Clone)]
pub enum TagStoreEvent {
    Loaded {
        workflow_run_id: WorkflowRunId,
        count: usize,
    },
    LoadFailed {
        workflow_run_id: WorkflowRunId,
        message: String,
    },
}

pub struct TagStore {
    repository: WorkflowRunRepository,
    tags_by_run: HashMap<String, Vec<TagRow>>,
    loading: HashSet<String>,
    errors: HashMap<String, String>,
}

impl EventEmitter<TagStoreEvent> for TagStore {}

impl TagStore {
    pub fn new(repository: WorkflowRunRepository) -> Self {
        Self {
            repository,
            tags_by_run: HashMap::new(),
            loading: HashSet::new(),
            errors: HashMap::new(),
        }
    }

    pub fn tags_for(&self, workflow_run_id: &WorkflowRunId) -> Option<&[TagRow]> {
        self.tags_by_run
            .get(workflow_run_id.as_ref())
            .map(Vec::as_slice)
    }

    pub fn is_loading(&self, workflow_run_id: &WorkflowRunId) -> bool {
        self.loading.contains(workflow_run_id.as_ref())
    }

    pub fn error_for(&self, workflow_run_id: &WorkflowRunId) -> Option<&str> {
        self.errors
            .get(workflow_run_id.as_ref())
            .map(String::as_str)
    }

    pub fn refresh(&mut self, workflow_run_id: WorkflowRunId, cx: &mut Context<Self>) -> Task<()> {
        let run_key = workflow_run_id.to_string();
        self.loading.insert(run_key.clone());
        self.errors.remove(&run_key);
        cx.notify();

        let repository = self.repository.clone();
        let workflow_run_id_for_query = workflow_run_id.clone();
        let task = cx.spawn(async move |store, cx| {
            let result = cx
                .background_spawn(async move {
                    repository
                        .list_tags(&workflow_run_id_for_query.to_string())
                        .await
                })
                .await;

            let _ = store.update(cx, |store, cx| {
                store.loading.remove(&run_key);
                match result {
                    Ok(tags) => {
                        let count = tags.len();
                        store.tags_by_run.insert(run_key.clone(), tags);
                        cx.emit(TagStoreEvent::Loaded {
                            workflow_run_id: workflow_run_id.clone(),
                            count,
                        });
                    }
                    Err(error) => {
                        let message = error.to_string();
                        store.errors.insert(run_key.clone(), message.clone());
                        cx.emit(TagStoreEvent::LoadFailed {
                            workflow_run_id: workflow_run_id.clone(),
                            message,
                        });
                    }
                }
                cx.notify();
            });
        });

        task
    }
}
