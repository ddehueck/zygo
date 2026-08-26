use std::collections::{HashMap, HashSet};

use gpui::{AppContext, Context, Task};
use local::{JobRunSummaryRepository, JobRunSummaryRow};
use zygo_core::models::WorkflowRunId;

pub struct WorkflowRunDetailStore {
    repository: JobRunSummaryRepository,
    summaries_by_run: HashMap<String, Vec<JobRunSummaryRow>>,
    loading: HashSet<String>,
    errors: HashMap<String, String>,
    refresh_generations: HashMap<String, u64>,
}

impl WorkflowRunDetailStore {
    pub fn new(repository: JobRunSummaryRepository) -> Self {
        Self {
            repository,
            summaries_by_run: HashMap::new(),
            loading: HashSet::new(),
            errors: HashMap::new(),
            refresh_generations: HashMap::new(),
        }
    }

    pub fn summaries_for(&self, run_id: &WorkflowRunId) -> Option<&[JobRunSummaryRow]> {
        self.summaries_by_run
            .get(run_id.as_ref())
            .map(Vec::as_slice)
    }

    pub fn is_loading(&self, run_id: &WorkflowRunId) -> bool {
        self.loading.contains(run_id.as_ref())
    }

    pub fn error_for(&self, run_id: &WorkflowRunId) -> Option<&str> {
        self.errors.get(run_id.as_ref()).map(String::as_str)
    }

    pub fn refresh(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) -> Task<()> {
        let run_key = run_id.to_string();
        let generation = self.refresh_generations.entry(run_key.clone()).or_default();
        *generation += 1;
        let generation = *generation;

        self.loading.insert(run_key.clone());
        self.errors.remove(&run_key);
        cx.notify();

        let repository = self.repository.clone();
        let query_run_key = run_key.clone();
        cx.spawn(async move |store, cx| {
            let result = cx
                .background_spawn(async move {
                    repository.list_by_workflow_run_id(&query_run_key).await
                })
                .await;

            let _ = store.update(cx, |store, cx| {
                if store.refresh_generations.get(&run_key).copied() != Some(generation) {
                    return;
                }

                store.loading.remove(&run_key);
                match result {
                    Ok(summaries) => {
                        store.summaries_by_run.insert(run_key.clone(), summaries);
                    }
                    Err(error) => {
                        store.errors.insert(run_key.clone(), error.to_string());
                    }
                }
                cx.notify();
            });
        })
    }
}
