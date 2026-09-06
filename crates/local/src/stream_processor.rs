use std::collections::HashMap;
use std::time::SystemTime;

use zygo_core::engine::RunCursor;
use zygo_core::models::{EventKind, Source, StreamItem, WorkflowRunId, WorkflowRunStatus};
use zygo_core::stream::{ReadResult, StreamReader};

use crate::db::KvRepository;
use crate::{Repos, format_database_timestamp};

/// Processes workflow stream records and projects local read models.
/// While still exposing the underlying stream for local clients. e.g. ui updates.
pub struct LocalStreamProcessor {
    repos: Repos,
    workflow_run_id: WorkflowRunId,
    stream_reader: StreamReader<KvRepository>,
    job_started_at: HashMap<String, SystemTime>,
}

impl LocalStreamProcessor {
    pub fn new(
        repos: Repos,
        workflow_run_id: WorkflowRunId,
        stream_reader: StreamReader<KvRepository>,
    ) -> Self {
        Self {
            stream_reader,
            repos,
            workflow_run_id,
            job_started_at: HashMap::new(),
        }
    }

    pub async fn process_next(&mut self, cursor: RunCursor) -> anyhow::Result<ReadResult> {
        let result = self.stream_reader.next(cursor).await?;

        let Some(StreamItem::Event(event)) = result.record.as_ref().map(|record| &record.item)
        else {
            return Ok(result);
        };

        let workflow_run_id = self.workflow_run_id.to_string();
        let timestamp = event.timestamp;
        let timestamp_value = format_database_timestamp(timestamp);

        match &event.kind {
            EventKind::JobStarted(data) => {
                let job_run_id = data.job_run_id.to_string();
                self.job_started_at.insert(job_run_id.clone(), timestamp);
                self.repos
                    .job_runs
                    .record_started(&workflow_run_id, &job_run_id, &data.job_id.to_string())
                    .await?;
            }
            EventKind::JobSucceeded(data) => {
                self.record_job_completed(
                    &workflow_run_id,
                    &data.job_run_id.to_string(),
                    &data.job_id.to_string(),
                    "succeeded",
                    timestamp,
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
                )
                .await?;
            }
            EventKind::TagInserted(data) => {
                self.repos
                    .tags
                    .insert(&workflow_run_id, &data.name, &data.value)
                    .await?;
            }
            EventKind::DataReferenceInserted(data) => {
                if let Source::JobRun(source) = &event.source {
                    self.repos
                        .data_references
                        .insert(
                            &workflow_run_id,
                            &source.job_run_id.to_string(),
                            &source.job_id.to_string(),
                            &data.data_reference.uri,
                            &data.data_reference.version,
                            event.is_replay,
                            &timestamp_value,
                        )
                        .await?;
                }
            }
            EventKind::ChannelItemInserted(_) => {}
        }

        self.refresh_workflow_run(&workflow_run_id, &timestamp_value)
            .await?;

        Ok(result)
    }

    async fn record_job_completed(
        &mut self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
        status: &str,
        timestamp: SystemTime,
    ) -> anyhow::Result<()> {
        let duration_ms = self
            .job_started_at
            .get(job_run_id)
            .and_then(|started_at| timestamp.duration_since(*started_at).ok())
            .map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX));

        self.repos
            .job_runs
            .record_completed(workflow_run_id, job_run_id, job_id, status, duration_ms)
            .await?;

        Ok(())
    }

    async fn refresh_workflow_run(
        &self,
        workflow_run_id: &str,
        timestamp: &str,
    ) -> anyhow::Result<()> {
        let counts = self
            .repos
            .job_runs
            .counts_by_workflow_run_id(workflow_run_id)
            .await?;

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
