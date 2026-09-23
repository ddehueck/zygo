use std::collections::HashMap;
use std::time::SystemTime;

use crate::models::{Event, EventKind, Source, WorkflowRunId, WorkflowRunStatus};
use crate::{JobRunModel, Repos, format_database_timestamp};

/// Maintains the local database read models from events handled by the run actor.
pub struct LocalProjector {
    repos: Repos,
    workflow_run_id: WorkflowRunId,
    job_started_at: HashMap<String, SystemTime>,
}

impl LocalProjector {
    pub fn new(repos: Repos, workflow_run_id: WorkflowRunId) -> Self {
        Self {
            repos,
            workflow_run_id,
            job_started_at: HashMap::new(),
        }
    }

    pub async fn project(&mut self, event: &Event) -> anyhow::Result<()> {
        let workflow_run_id = self.workflow_run_id.to_string();
        let timestamp = event.timestamp;
        let timestamp_value = format_database_timestamp(timestamp);

        match &event.kind {
            EventKind::JobEnqueued(data) => {
                let input_id = match self
                    .repos
                    .data_references
                    .get_id_by_uri(&workflow_run_id, data.input.as_ref())
                    .await?
                {
                    Some(input_id) => input_id,
                    None => {
                        self.repos
                            .data_references
                            .insert(&workflow_run_id, None, data.input.as_ref(), event.is_replay)
                            .await?;
                        self.repos
                            .data_references
                            .get_id_by_uri(&workflow_run_id, data.input.as_ref())
                            .await?
                            .ok_or_else(|| {
                                anyhow::anyhow!("queued job input reference was not inserted")
                            })?
                    }
                };
                let job_run_id = data.job_run_id.to_string();
                if self
                    .repos
                    .job_runs
                    .get_by_public_id(&workflow_run_id, &job_run_id)
                    .await?
                    .is_none()
                {
                    let workflow_run = self
                        .repos
                        .workflow_runs
                        .get_by_workflow_run_id(&workflow_run_id)
                        .await?
                        .ok_or_else(|| {
                            anyhow::anyhow!("workflow run {workflow_run_id} not found")
                        })?;
                    self.repos
                        .job_runs
                        .upsert(&JobRunModel {
                            id: 0,
                            public_id: job_run_id,
                            workflow_run_id: workflow_run.id,
                            input_id,
                            job_id: data.job_id.to_string(),
                            status: "queued".to_owned(),
                            duration_ms: None,
                            error_message: None,
                            retry_count: 0,
                            created_at: timestamp_value.clone(),
                        })
                        .await?;
                }
            }
            EventKind::JobStarted(data) => {
                let job_run_id = data.job_run_id.to_string();
                let input_id = self
                    .repos
                    .data_references
                    .get_id_by_uri(&workflow_run_id, data.input.as_ref())
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "input data reference with URI {:?} not found for workflow run {}",
                            data.input,
                            workflow_run_id
                        )
                    })?;
                self.job_started_at.insert(job_run_id.clone(), timestamp);
                self.repos
                    .job_runs
                    .record_started(
                        &workflow_run_id,
                        &job_run_id,
                        &data.job_id.to_string(),
                        input_id,
                    )
                    .await?;
            }
            EventKind::JobSucceeded(data) => {
                self.record_job_completed(
                    &workflow_run_id,
                    &data.job_run_id.to_string(),
                    &data.job_id.to_string(),
                    "succeeded",
                    timestamp,
                    None,
                )
                .await?;
            }
            EventKind::JobFailed(data) => {
                self.record_job_completed(
                    &workflow_run_id,
                    &data.job_run_id.to_string(),
                    &data.job_id.to_string(),
                    "failed",
                    timestamp,
                    Some(&data.error),
                )
                .await?;
            }
            EventKind::TagInserted(data) => {
                self.repos
                    .tags
                    .insert(&data.value, &workflow_run_id, None, None)
                    .await?;
            }
            EventKind::DataReferenceInserted(data) => {
                if let Source::JobRun(source) = &event.source {
                    self.repos
                        .data_references
                        .insert(
                            &workflow_run_id,
                            Some(&source.job_run_id.to_string()),
                            data.uri.as_ref(),
                            event.is_replay,
                        )
                        .await?;
                }
            }
            EventKind::ChannelItemInserted(data) => {
                self.repos
                    .data_references
                    .insert(&workflow_run_id, None, data.item.as_ref(), event.is_replay)
                    .await?;
            }
        }

        self.refresh_workflow_run(&workflow_run_id, &timestamp_value)
            .await?;

        Ok(())
    }

    async fn record_job_completed(
        &mut self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
        status: &str,
        timestamp: SystemTime,
        error_message: Option<&str>,
    ) -> anyhow::Result<()> {
        let duration_ms = self
            .job_started_at
            .get(job_run_id)
            .and_then(|started_at| timestamp.duration_since(*started_at).ok())
            .map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX));

        self.repos
            .job_runs
            .record_completed(
                workflow_run_id,
                job_run_id,
                job_id,
                status,
                duration_ms,
                error_message,
            )
            .await?;

        Ok(())
    }

    async fn refresh_workflow_run(
        &self,
        workflow_run_id: &str,
        timestamp: &str,
    ) -> anyhow::Result<()> {
        let mut counts = self
            .repos
            .job_runs
            .counts_by_workflow_run_id(workflow_run_id)
            .await?;

        // The repository aggregate counts running jobs only. Read queued jobs from
        // persisted state so starting a job cannot leave a stale queued count.
        let queued_count = self
            .repos
            .job_runs
            .list_by_workflow_run_id(workflow_run_id)
            .await?
            .iter()
            .filter(|run| run.status == "queued")
            .count();
        counts.active_job_count += i64::try_from(queued_count)?;

        let status = if counts.errored_job_count > 0 {
            WorkflowRunStatus::Failed
        } else if counts.active_job_count > 0 {
            WorkflowRunStatus::Running
        } else if counts.succeeded_job_count > 0 {
            WorkflowRunStatus::Succeeded
        } else {
            WorkflowRunStatus::Running
        };

        let existing = self
            .repos
            .workflow_runs
            .get_by_workflow_run_id(workflow_run_id)
            .await?;

        let started_at = existing
            .as_ref()
            .and_then(|run| run.started_at.clone())
            .unwrap_or_else(|| timestamp.to_owned());

        let completed_at = if status.is_terminal() {
            existing
                .as_ref()
                .and_then(|run| run.completed_at.clone())
                .or_else(|| Some(timestamp.to_owned()))
        } else {
            None
        };

        let status = status.to_string();

        self.repos
            .workflow_runs
            .upsert(
                workflow_run_id,
                &status,
                Some(started_at.as_str()),
                completed_at.as_deref(),
                Some(counts.active_job_count),
                Some(counts.succeeded_job_count),
                Some(counts.errored_job_count),
            )
            .await?;

        Ok(())
    }
}
