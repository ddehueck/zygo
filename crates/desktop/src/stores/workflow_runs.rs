use std::collections::{HashMap, HashSet};

use gpui::{AppContext, Context, EventEmitter, Task};
use local::{Tag, WorkflowRun, WorkflowRunRepository};

use crate::features::runs::filters::FilterSet;

#[derive(Debug, Clone)]
pub enum WorkflowRunStoreEvent {
    Loaded { count: usize },
    LoadFailed { message: String },
}

pub struct WorkflowRunStore {
    repository: WorkflowRunRepository,
    all_workflow_runs: Vec<WorkflowRun>,
    workflow_runs: Vec<WorkflowRun>,
    tags_by_run: HashMap<String, Vec<Tag>>,
    available_tags: Vec<Tag>,
    active_filter: FilterSet,
    loading: bool,
    error: Option<String>,
}

impl EventEmitter<WorkflowRunStoreEvent> for WorkflowRunStore {}

impl WorkflowRunStore {
    pub fn new(repository: WorkflowRunRepository, cx: &mut Context<Self>) -> Self {
        let mut store = Self {
            repository,
            all_workflow_runs: Vec::new(),
            workflow_runs: Vec::new(),
            tags_by_run: HashMap::new(),
            available_tags: Vec::new(),
            active_filter: FilterSet::default(),
            loading: false,
            error: None,
        };
        store.refresh(cx).detach();
        store
    }

    pub fn workflow_runs(&self) -> &[WorkflowRun] {
        &self.workflow_runs
    }

    pub fn all_workflow_runs(&self) -> &[WorkflowRun] {
        &self.all_workflow_runs
    }

    pub fn active_filter(&self) -> &FilterSet {
        &self.active_filter
    }

    pub fn available_tags(&self) -> &[Tag] {
        &self.available_tags
    }

    /// Applies an exact, AND-based filter to the runs already loaded in memory.
    pub fn filter(&mut self, filter_set: FilterSet) {
        self.active_filter = filter_set;
        self.apply_filter();
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
                .background_spawn(async move {
                    let workflow_runs = repository.list_all().await?;
                    let mut tags_by_run = HashMap::with_capacity(workflow_runs.len());
                    let mut available_tags = Vec::new();
                    let mut seen_tags = HashSet::new();

                    for workflow_run in &workflow_runs {
                        let tags = repository.list_tags(&workflow_run.id).await?;
                        for tag in &tags {
                            if seen_tags.insert((tag.key.clone(), tag.value.clone())) {
                                available_tags.push(tag.clone());
                            }
                        }
                        tags_by_run.insert(workflow_run.id.clone(), tags);
                    }

                    Ok::<_, anyhow::Error>((workflow_runs, tags_by_run, available_tags))
                })
                .await;

            let _ = store.update(cx, |store, cx| {
                store.loading = false;
                match result {
                    Ok((workflow_runs, tags_by_run, available_tags)) => {
                        let count = workflow_runs.len();
                        store.all_workflow_runs = workflow_runs;
                        store.tags_by_run = tags_by_run;
                        store.available_tags = available_tags;
                        store.apply_filter();
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

    fn apply_filter(&mut self) {
        self.workflow_runs = self
            .all_workflow_runs
            .iter()
            .filter(|workflow_run| {
                self.tags_by_run
                    .get(&workflow_run.id)
                    .is_some_and(|tags| self.active_filter.matches(tags))
            })
            .cloned()
            .collect();
    }
}
