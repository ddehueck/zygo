//! Executes command IR produced by the arbiter.

use std::sync::Arc;

use crate::models::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, Command, Event, JobArgs,
    ReplayJobCommand, ResultCacheItem, RunJobCommand, SetJobRunStatusCommand,
};
use crate::store::{StorageProvider, Store};

use super::runner::Runner;
use super::state::{RunContext, RunState};

pub struct ExecuteResult {
    pub next_state: RunState,
    pub next_events: Vec<Event>,
}

pub struct Executor<S: StorageProvider> {
    _store: Arc<Store<S>>,
    runner: Runner<S>,
}

impl<S: StorageProvider> Executor<S> {
    pub fn new(store: Arc<Store<S>>) -> Self {
        Self {
            _store: Arc::clone(&store),
            runner: Runner::new(store),
        }
    }

    pub async fn execute(
        &self,
        command: Command,
        context: &RunContext,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
        match command {
            Command::RunJob(command) => self.run_job(command, context, state).await,
            Command::ReplayJob(command) => self.replay_job(command, context, state).await,
            Command::CacheJobEventSource(command) => {
                self.cache_job_event_source(command, &state).await
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
        context: &RunContext,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
        let job_args = JobArgs {
            run_id: context.scope.run_id.to_string(),
            workflow_id: context.scope.workflow_id.to_string(),
            workflow_version_id: context.scope.workflow_version_id.to_string(),
            job_id: command.job_id.to_string(),
            data_reference_uri: command.data_reference.uri.clone(),
            data_reference_etag: command.data_reference.etag.clone(),
            channel_ids_by_name: context.schema.get_channel_ids_by_name(),
            job_run_id: command.job_run_id.to_string(),
        };

        let Some(job_entrypoint) = context.schema.get_job_entrypoint(&command.job_id) else {
            return Err(anyhow::anyhow!("job entrypoint not found"));
        };

        self.runner.execute(job_args, job_entrypoint).await?;

        Ok(ExecuteResult {
            next_state: state.clone(),
            next_events: vec![],
        })
    }

    async fn replay_job(
        &self,
        command: ReplayJobCommand,
        context: &RunContext,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
        // We replay a job by retrieving the events referenced in the result cache.
        // We then return the events to be appended to the stream as replay events.
        let original_events = self
            ._store
            .results_cache()
            .get_events(&command.cache_item)
            .await?;

        let replay_events = original_events
            .into_iter()
            .map(|event| context.make_replay_event(event.kind, command.source.clone()))
            .collect();

        Ok(ExecuteResult {
            next_state: state.clone(),
            next_events: replay_events,
        })
    }

    async fn cache_job_event_source(
        &self,
        command: CacheJobEventSourceCommand,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
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
        context: &RunContext,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
        // A job run id is constructed from the data input and job content hash.
        // This means that a job run id is unique for a given data input and a job fn.
        // This allows us to cache the resulting events of a job run to be used if we need to run the job again.
        let Some(event_keys) = state.event_keys_by_job_run_id.get(&command.job_run_id) else {
            return Err(anyhow::anyhow!("event keys not found"));
        };

        let result_cache_item = ResultCacheItem {
            event_keys: event_keys.clone(),
        };

        self._store
            .results_cache()
            .put(
                &context.scope.workflow_id,
                &command.job_run_id,
                &result_cache_item,
            )
            .await?;

        Ok(ExecuteResult {
            next_state: state.clone(),
            next_events: vec![],
        })
    }

    async fn record_job_run_status(
        &self,
        command: SetJobRunStatusCommand,
        state: &RunState,
    ) -> Result<ExecuteResult, anyhow::Error> {
        let next_state = state.set_job_status(command.job_run_id.clone(), command.status.clone());

        Ok(ExecuteResult {
            next_state,
            next_events: vec![],
        })
    }
}
