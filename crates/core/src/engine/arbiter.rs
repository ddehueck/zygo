//! Maps events to commands for a workflow run.
//!
//! The arbiter inspects committed events and current run state, then emits
//! command intent. It does not execute commands or mutate state directly.

use crate::models::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, ChannelItemInsertedData, Command,
    DataReference, EdgeKind, Event, EventKind, JobFailedData, JobId, JobRunId, JobRunSource,
    JobRunStatus, JobStartedData, JobSucceededData, ReplayJobCommand, RunJobCommand,
    SetJobRunStatusCommand, Source, job_run_id,
};
use crate::store::{StorageProvider, StoreKey};

use super::state::ResultCache;

pub struct Arbiter;

impl Arbiter {
    pub async fn arbitrate<S: StorageProvider>(
        &self,
        event_key: &StoreKey,
        event: &Event,
        cache: &ResultCache<S>,
    ) -> Result<Vec<Command>, anyhow::Error> {
        let mut commands = Vec::new();

        commands.extend(self.handle_by_event_source(event, event_key)?);
        commands.extend(self.handle_by_event_kind(event, cache).await?);

        // Filter out commands that are not safe to issue during a replay.
        if event.is_replay {
            commands.retain(Command::is_replayable);
        }

        Ok(commands)
    }

    fn handle_by_event_source(
        &self,
        event: &Event,
        event_key: &StoreKey,
    ) -> Result<Vec<Command>, anyhow::Error> {
        let mut commands = Vec::new();

        match &event.source {
            Source::Input => {}
            Source::JobRun(job_run) => {
                let cmd = Command::CacheJobEventSource(CacheJobEventSourceCommand {
                    job_run_id: job_run.job_run_id.clone(),
                    event_key: event_key.clone(),
                });
                commands.push(cmd);
            }
        }

        Ok(commands)
    }

    async fn handle_by_event_kind<S: StorageProvider>(
        &self,
        event: &Event,
        context: &ResultCache<S>,
    ) -> Result<Vec<Command>, anyhow::Error> {
        match &event.kind {
            EventKind::JobStarted(data) => self.handle_job_started(data),
            EventKind::JobSucceeded(data) => self.handle_job_succeeded(data),
            EventKind::JobFailed(data) => self.handle_job_failed(data),
            EventKind::DataReferenceInserted(_) => self.noop(),
            EventKind::ChannelItemInserted(data) => {
                self.handle_channel_item_inserted(data, context).await
            }
        }
    }

    fn noop(&self) -> Result<Vec<Command>, anyhow::Error> {
        Ok(Vec::new())
    }

    async fn handle_channel_item_inserted<S: StorageProvider>(
        &self,
        data: &ChannelItemInsertedData,
        context: &ResultCache<S>,
    ) -> Result<Vec<Command>, anyhow::Error> {
        // Find all jobs that have the given channel as an input.
        // Request each job to be run.
        let job_ids = context
            .schema
            .edges
            .iter()
            .filter(|edge| edge.channel_id == data.channel_id && edge.kind == EdgeKind::Input)
            .map(|edge| edge.job_id.clone())
            .collect::<Vec<_>>();

        let mut commands = Vec::new();
        for job_id in job_ids {
            let command = self
                .resolve_job_request(&job_id, &data.data_reference, context)
                .await?;
            commands.push(command);
        }

        Ok(commands)
    }

    async fn resolve_job_request<S: StorageProvider>(
        &self,
        job_id: &JobId,
        data_reference: &DataReference,
        context: &ResultCache<S>,
    ) -> Result<Command, anyhow::Error> {
        // When a job should be run, we first check if it is already in the result cache.
        // If it is, we replay the events of the latest succeeded run in sequence order.
        // Otherwise, we actually run the job.
        let job = context.schema.get_job_by_id(job_id).ok_or_else(|| {
            anyhow::anyhow!("job {job_id} referenced by an edge is not present in the run schema")
        })?;

        let job_run_id =
            JobRunId::try_from(job_run_id(job, &data_reference.uri, &data_reference.etag))?;

        if let Some(cache_item) = context.get_item(&job_run_id).await? {
            return Ok(Command::ReplayJob(ReplayJobCommand {
                source: Source::JobRun(JobRunSource {
                    job_id: job_id.clone(),
                    job_run_id: job_run_id.clone(),
                }),
                cache_item,
            }));
        }

        let cmd = Command::RunJob(RunJobCommand {
            job_id: job_id.clone(),
            job_run_id: job_run_id.clone(),
            data_reference: data_reference.clone(),
        });

        Ok(cmd)
    }

    fn handle_job_started(&self, data: &JobStartedData) -> Result<Vec<Command>, anyhow::Error> {
        let cmd = Command::SetJobRunStatus(SetJobRunStatusCommand {
            job_run_id: data.job_run_id.clone(),
            status: JobRunStatus::Running,
        });
        Ok(vec![cmd])
    }

    fn handle_job_succeeded(&self, data: &JobSucceededData) -> Result<Vec<Command>, anyhow::Error> {
        let record_status = Command::SetJobRunStatus(SetJobRunStatusCommand {
            job_run_id: data.job_run_id.clone(),
            status: JobRunStatus::Succeeded,
        });

        let cache_result = Command::CacheJobRunResult(CacheJobRunResultCommand {
            job_run_id: data.job_run_id.clone(),
        });

        Ok(vec![record_status, cache_result])
    }

    fn handle_job_failed(&self, data: &JobFailedData) -> Result<Vec<Command>, anyhow::Error> {
        let cmd = Command::SetJobRunStatus(SetJobRunStatusCommand {
            job_run_id: data.job_run_id.clone(),
            status: JobRunStatus::Failed,
        });
        Ok(vec![cmd])
    }
}
