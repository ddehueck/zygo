use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use zygo_core::{
    engine::RunCursor,
    models::{EventKind, StreamItem, WorkflowRunId, WorkflowRunStatus},
    stream::{ReadResult, StreamReader},
};

use crate::{Repos, db::KvRepository};

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
        let timestamp_value = timestamp_ms(timestamp);

        match &event.kind {
            EventKind::JobStarted(data) => {
                let job_run_id = data.job_run_id.to_string();
                self.job_started_at.insert(job_run_id.clone(), timestamp);
                self.repos
                    .job_run_summaries
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
                    .workflow_runs
                    .insert_tag(&workflow_run_id, &data.name, &data.value)
                    .await?;
            }
            EventKind::DataReferenceInserted(_) | EventKind::ChannelItemInserted(_) => {}
        }

        self.refresh_workflow_summary(&workflow_run_id, timestamp_value)
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
            .job_run_summaries
            .record_completed(workflow_run_id, job_run_id, job_id, status, duration_ms)
            .await?;

        Ok(())
    }

    async fn refresh_workflow_summary(
        &self,
        workflow_run_id: &str,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        let counts = self
            .repos
            .job_run_summaries
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
            .workflow_run_summaries
            .get_by_workflow_run_id(workflow_run_id)
            .await?;
        let started_at = existing
            .as_ref()
            .and_then(|summary| summary.started_at)
            .unwrap_or(timestamp);
        let completed_at = if status.is_terminal() {
            existing
                .as_ref()
                .and_then(|summary| summary.completed_at)
                .or(Some(timestamp))
        } else {
            None
        };

        let status = status.to_string();

        self.repos
            .workflow_run_summaries
            .upsert_projection(
                workflow_run_id,
                &status,
                Some(started_at),
                completed_at,
                counts.active_job_count,
                counts.succeeded_job_count,
                counts.errored_job_count,
            )
            .await?;

        Ok(())
    }
}

fn timestamp_ms(timestamp: SystemTime) -> i64 {
    timestamp
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| duration.as_millis().try_into().ok())
        .unwrap_or(i64::MAX)
}
