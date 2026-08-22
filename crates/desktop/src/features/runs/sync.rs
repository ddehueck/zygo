use std::{collections::HashSet, sync::Arc, time::Duration};

use gpui::{AppContext, Context, Entity, Task};
use local::{WorkflowRunSummaryRow, ZygoLocalService};
use zygo_core::models::WorkflowRunId;

use crate::stores::{WorkflowRunDetailStore, WorkflowRunStore};

const RUN_POLL_INTERVAL: Duration = Duration::from_secs(1);
const SUMMARY_POLL_INTERVAL: Duration = Duration::from_secs(1);
const REFRESH_DEBOUNCE: Duration = Duration::from_millis(250);

/// Coordinates workflow-run discovery and background updates to the run store.
///
/// A service actor can be subscribed to when it belongs to this process. Runs
/// created by another process are synchronized by watching their summary row.
pub struct RunSync {
    service: Arc<ZygoLocalService>,
    runs: Entity<WorkflowRunStore>,
    details: Entity<WorkflowRunDetailStore>,
    observed_runs: HashSet<String>,
    pending_runs: HashSet<WorkflowRunId>,
    tasks: Vec<Task<()>>,
    refresh_task: Option<Task<()>>,
    refresh_requested: bool,
    _poll_task: Task<()>,
}

impl RunSync {
    pub fn new(
        service: Arc<ZygoLocalService>,
        runs: Entity<WorkflowRunStore>,
        details: Entity<WorkflowRunDetailStore>,
        cx: &mut Context<Self>,
    ) -> Self {
        let repository = service.repos.workflow_runs.clone();
        let poll_task = cx.spawn(async move |sync, cx| {
            let executor = cx.background_executor().clone();
            let mut high_water_mark = 0;

            loop {
                let repository = repository.clone();
                let result = cx
                    .background_spawn(async move { repository.list_after(high_water_mark).await })
                    .await;

                match result {
                    Ok(workflow_runs) => {
                        if let Some(last_run) = workflow_runs.last() {
                            high_water_mark = last_run.row_id;
                        }

                        for workflow_run in workflow_runs {
                            let Ok(run_id) = WorkflowRunId::try_from(workflow_run.id) else {
                                continue;
                            };

                            if sync
                                .update(cx, |sync, cx| sync.observe_run(run_id, cx))
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        // Keep the cursor unchanged so a transient database error
                        // does not cause a run to be skipped.
                    }
                }

                executor.timer(RUN_POLL_INTERVAL).await;
            }
        });

        Self {
            service,
            runs,
            details,
            observed_runs: HashSet::new(),
            pending_runs: HashSet::new(),
            tasks: Vec::new(),
            refresh_task: None,
            refresh_requested: false,
            _poll_task: poll_task,
        }
    }

    fn observe_run(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) {
        let run_key = run_id.to_string();
        if !self.observed_runs.insert(run_key) {
            return;
        }

        // A run may already have summary data before discovery, so refresh once
        // immediately rather than waiting for the first notification.
        self.request_refresh(run_id.clone(), cx);

        let service = self.service.clone();
        let subscription_run_id = run_id.clone();
        let fallback_run_id = run_id;
        let task = cx.spawn(async move |sync, cx| {
            match service.base.subscribe(&subscription_run_id).await {
                Ok(mut receiver) => {
                    if sync
                        .update(cx, |sync, cx| {
                            sync.request_refresh(subscription_run_id.clone(), cx)
                        })
                        .is_err()
                    {
                        return;
                    }

                    loop {
                        if receiver.changed().await.is_err() {
                            return;
                        }

                        if sync
                            .update(cx, |sync, cx| {
                                sync.request_refresh(subscription_run_id.clone(), cx)
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                Err(_) => {
                    let _ = sync.update(cx, |sync, cx| {
                        sync.start_summary_poller(fallback_run_id, cx);
                    });
                }
            }
        });
        self.tasks.push(task);
    }

    fn start_summary_poller(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) {
        let repository = self.service.repos.workflow_run_summaries.clone();
        let task_run_id = run_id.clone();
        let task = cx.spawn(async move |sync, cx| {
            let executor = cx.background_executor().clone();
            let mut previous: Option<WorkflowRunSummaryRow> = None;

            loop {
                let repository = repository.clone();
                let query_run_id = task_run_id.clone();
                let result = cx
                    .background_spawn(async move {
                        repository
                            .get_by_workflow_run_id(query_run_id.as_ref())
                            .await
                    })
                    .await;

                match result {
                    Ok(Some(summary)) => {
                        let changed = previous.as_ref() != Some(&summary);
                        let terminal = matches!(summary.status.as_str(), "succeeded" | "failed");
                        previous = Some(summary);

                        if changed
                            && sync
                                .update(cx, |sync, cx| {
                                    sync.request_refresh(task_run_id.clone(), cx)
                                })
                                .is_err()
                        {
                            return;
                        }

                        if terminal {
                            return;
                        }
                    }
                    Ok(None) | Err(_) => {}
                }

                executor.timer(SUMMARY_POLL_INTERVAL).await;
            }
        });
        self.tasks.push(task);
    }

    fn request_refresh(&mut self, run_id: WorkflowRunId, cx: &mut Context<Self>) {
        self.pending_runs.insert(run_id);
        self.refresh_requested = true;
        if self.refresh_task.is_some() {
            return;
        }

        let runs = self.runs.clone();
        let details = self.details.clone();
        let task = cx.spawn(async move |sync, cx| {
            let executor = cx.background_executor().clone();
            executor.timer(REFRESH_DEBOUNCE).await;

            let run_ids = match sync.update(cx, |sync, _| {
                sync.refresh_requested = false;
                sync.refresh_task.take();
                sync.pending_runs.drain().collect::<Vec<_>>()
            }) {
                Ok(run_ids) => run_ids,
                Err(_) => return,
            };

            if run_ids.is_empty() {
                return;
            }

            let _ = runs.update(cx, |store, cx| {
                store.refresh(cx).detach();
            });
            for run_id in run_ids {
                let _ = details.update(cx, |store, cx| {
                    store.refresh(run_id, cx).detach();
                });
            }
        });
        self.refresh_task = Some(task);
    }
}
