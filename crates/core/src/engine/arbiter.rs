//! Maps events to commands for a workflow run.
//!
//! The arbiter inspects committed events and current run state, then emits
//! command intent. It does not execute commands or mutate state directly.

use std::sync::Arc;

use crate::models::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, ChannelItemInsertedData, Command,
    DataReference, Event, EventKind, JobFailedData, JobId, JobRunId, JobRunSource, JobRunStatus,
    JobStartedData, JobSucceededData, ReplayJobCommand, RunJobCommand, SetJobRunStatusCommand,
    Source, job_run_id,
};
use crate::store::keyspace::StoreKey;
use crate::store::{StorageProvider, Store};

use super::state::RunContext;

pub struct Arbiter<S: StorageProvider> {
    pub store: Arc<Store<S>>,
}

impl<S: StorageProvider> Arbiter<S> {
    pub fn new(store: Arc<Store<S>>) -> Self {
        Self { store }
    }

    pub async fn arbitrate(
        &self,
        event_key: &StoreKey,
        event: &Event,
        context: &RunContext,
    ) -> Result<Vec<Command>, anyhow::Error> {
        let mut commands = Vec::new();

        commands.extend(self.handle_by_event_source(event, event_key)?);
        commands.extend(self.handle_by_event_kind(event, context).await?);

        // Filter out commands that are not safe to issue during a replay.
        if event.is_replay {
            commands = commands
                .into_iter()
                .filter(|cmd| cmd.is_replayable())
                .collect();
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
            Source::Input(_) => {}
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

    async fn handle_by_event_kind(
        &self,
        event: &Event,
        context: &RunContext,
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

    async fn handle_channel_item_inserted(
        &self,
        data: &ChannelItemInsertedData,
        context: &RunContext,
    ) -> Result<Vec<Command>, anyhow::Error> {
        // Find all jobs that have the given channel as an input.
        // Request each job to be run.
        let channel_id = &data.channel_id;
        let jobs = context.schema.get_jobs_by_input_channel_id(channel_id);

        let mut commands: Vec<Command> = Vec::new();
        for job in jobs {
            let cmd = self
                .resolve_job_request(&job.id, &data.data_reference, context)
                .await?;
            commands.push(cmd);
        }

        Ok(commands)
    }

    async fn resolve_job_request(
        &self,
        job_id: &JobId,
        data_reference: &DataReference,
        context: &RunContext,
    ) -> Result<Command, anyhow::Error> {
        // When a job should be run, we first check if the job is already in the result cache.
        // If it is, we replay the events of the latest succeeded run in sequence order.
        // Otherwise, we actually run the job.
        assert!(
            context.schema.get_job_by_id(job_id).is_some(),
            "job {:?} referenced by JobRequested is not present in run schema",
            job_id
        );

        let job_run_id = JobRunId::try_from(job_run_id(
            job_id,
            &data_reference.uri,
            &data_reference.etag,
        ))?;

        if let Some(cache_item) = context
            .get_result_cache_item(&self.store, &job_run_id)
            .await?
        {
            return Ok(Command::ReplayJob(ReplayJobCommand {
                source: Source::JobRun(JobRunSource {
                    job_id: job_id.clone(),
                    job_run_id: job_run_id.clone(),
                    workflow_run_id: context.scope.run_id.clone(),
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
