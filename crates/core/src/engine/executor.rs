//! Executes command IR produced by the arbiter.
use crate::context::ActorContext;

use crate::engine::{Error, Result};
use crate::ipc::v0::RunCommandArgs;
use crate::models::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, Command, Event, JobRunSource,
    JobRunStatus, ReplayJobCommand, ResultCacheItem, RunJobCommand, SetJobRunStatusCommand,
    StreamItem, StreamRecord,
};
use crate::store::StorageProvider;

use super::state::{ResultCache, RunState};

pub struct ExecuteResult {
    pub next_state: RunState,
    pub next_events: Vec<Event>,
}

pub struct Executor<S: StorageProvider> {
    context: ActorContext<S>,
}

impl<S: StorageProvider> Executor<S> {
    pub fn new(context: ActorContext<S>) -> Self {
        Self { context }
    }

    pub async fn execute(
        &self,
        command: Command,
        context: &ResultCache<S>,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        match command {
            Command::RunJob(command) => self.run_job(command, context, state).await,
            Command::ReplayJob(command) => self.replay_job(command, context, state).await,
            Command::CacheJobEventSource(command) => {
                self.cache_job_event_source(command, state).await
            }
            Command::CacheJobRunResult(command) => {
                self.cache_job_run_result(command, context, state).await
            }
            Command::SetJobRunStatus(command) => self.record_job_run_status(command, state).await,
        }
    }

    async fn run_job(
        &self,
        command: RunJobCommand,
        context: &ResultCache<S>,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        let job_args = RunCommandArgs {
            job_id: command.job_id.to_string(),
            data_reference_uri: command.data_reference.uri.clone(),
            data_reference_version: command.data_reference.version.clone(),
            workflow_run_id: self.context.run_id.to_string(),
            job_run_id: command.job_run_id.to_string(),
        };

        let Some(job_entrypoint) = context.schema.get_job_entrypoint(&command.job_id) else {
            return Err(Error::other("job entrypoint not found"));
        };

        let job_run_id = command.job_run_id.clone();
        let source = JobRunSource {
            job_id: command.job_id,
            job_run_id: command.job_run_id,
        };

        self.context.worker_pool.enqueue_job(
            self.context.clone(),
            source,
            job_args,
            job_entrypoint,
        )?;

        Ok(ExecuteResult {
            next_state: state.set_job_status(job_run_id, JobRunStatus::Queued),
            next_events: vec![],
        })
    }

    async fn replay_job(
        &self,
        command: ReplayJobCommand,
        context: &ResultCache<S>,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        // Retrieve cached stream records in the stored key order and turn their events into
        // replay events for the current run.
        let event_keys = &command.cache_item.event_keys;
        let values = self.context.store.get_many(event_keys).await?;
        if values.len() != event_keys.len() {
            return Err(Error::other(format!(
                "cache returned {} values for {} event keys",
                values.len(),
                event_keys.len()
            )));
        }

        let mut replay_events = Vec::with_capacity(values.len());
        for (event_key, value) in event_keys.iter().zip(values) {
            let value = value.ok_or_else(|| {
                Error::other(format!(
                    "cached event record is missing at key {}",
                    event_key.as_str()
                ))
            })?;
            let record = serde_json::from_value::<StreamRecord>(value).map_err(|error| {
                Error::other(format!(
                    "failed to deserialize cached stream record at key {}: {error}",
                    event_key.as_str()
                ))
            })?;
            let StreamItem::Event(event) = record.item else {
                return Err(Error::other(format!(
                    "cached stream record at key {} is not an event",
                    event_key.as_str()
                )));
            };

            replay_events.push(context.make_replay_event(event.kind, command.source.clone()));
        }

        Ok(ExecuteResult {
            next_state: state.clone(),
            next_events: replay_events,
        })
    }

    async fn cache_job_event_source(
        &self,
        command: CacheJobEventSourceCommand,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        // We cache the relation between an event and the job run that produced it.
        // This is how we accumulate event relationships until the job run completes.
        // Once the job run completes, we move the relations to the result cache.
        let next_state = state.add_event_key(command.job_run_id, command.event_key.clone());

        Ok(ExecuteResult {
            next_state,
            next_events: vec![],
        })
    }

    async fn cache_job_run_result(
        &self,
        command: CacheJobRunResultCommand,
        context: &ResultCache<S>,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        // A job run id is constructed from the data input and job content hash.
        // This means that a job run id is deterministic for a given data input and a job fn.
        // This allows us to cache the resulting events of a job run to be used if we need to run the job again.
        let Some(event_keys) = state.event_keys_by_job_run_id.get(&command.job_run_id) else {
            return Err(Error::other("event keys not found"));
        };

        let result_cache_item = ResultCacheItem {
            event_keys: event_keys.clone(),
        };

        context.put(&command.job_run_id, &result_cache_item).await?;

        Ok(ExecuteResult {
            next_state: state.clone(),
            next_events: vec![],
        })
    }

    async fn record_job_run_status(
        &self,
        command: SetJobRunStatusCommand,
        state: &RunState,
    ) -> Result<ExecuteResult> {
        let next_state = state.set_job_status(command.job_run_id.clone(), command.status.clone());

        Ok(ExecuteResult {
            next_state,
            next_events: vec![],
        })
    }
}
