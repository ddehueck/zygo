//! Engine runtime state and durable restart snapshots.

use std::{collections::HashMap, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::{
    context::RunContext,
    models::{
        Event, EventId, EventKind, JobRunId, JobRunStatus, ResultCacheItem, RunStatus, SequenceId,
        Source, WorkflowVersionSchema,
    },
    store::{StorageProvider, keyspace::StoreKey},
};

#[derive(Clone)]
pub struct ResultCache<S: StorageProvider> {
    pub context: RunContext<S>,
    pub schema: WorkflowVersionSchema,
}

impl<S: StorageProvider> ResultCache<S> {
    pub fn new(context: RunContext<S>, schema: WorkflowVersionSchema) -> Self {
        Self { context, schema }
    }

    pub async fn get_item(
        &self,
        job_run_id: &JobRunId,
    ) -> Result<Option<ResultCacheItem>, anyhow::Error> {
        self.context
            .store
            .results_cache()
            .get(&self.context.workflow_id, job_run_id)
            .await
    }

    pub async fn put(
        &self,
        job_run_id: &JobRunId,
        result_cache_item: &ResultCacheItem,
    ) -> Result<(), anyhow::Error> {
        self.context
            .store
            .results_cache()
            .put(&self.context.workflow_id, job_run_id, result_cache_item)
            .await
    }

    pub fn make_replay_event(&self, kind: EventKind, source: Source) -> Event {
        Event {
            id: EventId::try_from(uuid::Uuid::now_v7().to_string()).unwrap(),
            is_replay: true,
            timestamp: SystemTime::now(),
            kind,
            source,
            workflow_version_id: self.context.workflow_version_id.clone(),
            workflow_id: self.context.workflow_id.clone(),
            workflow_run_id: self.context.run_id.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RunCursor {
    pub next_id: SequenceId,
}

impl RunCursor {
    pub fn new(next_read_sequence_id: SequenceId) -> Self {
        Self {
            next_id: next_read_sequence_id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub cursor: RunCursor,
    pub state: RunState,
}

impl EngineSnapshot {
    pub fn new() -> Self {
        Self {
            cursor: RunCursor::new(SequenceId::new(0)),
            state: RunState::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RunState {
    pub status: RunStatus,
    pub status_by_job_run_id: HashMap<JobRunId, JobRunStatus>,
    pub event_keys_by_job_run_id: HashMap<JobRunId, Vec<String>>,
}

impl RunState {
    pub fn new() -> Self {
        Self {
            status: RunStatus::Running,
            status_by_job_run_id: HashMap::new(),
            event_keys_by_job_run_id: HashMap::new(),
        }
    }

    pub fn from(
        status_by_job_run_id: HashMap<JobRunId, JobRunStatus>,
        event_keys_by_job_run_id: HashMap<JobRunId, Vec<String>>,
    ) -> Self {
        let status = compute_run_status(&status_by_job_run_id);
        Self {
            status,
            status_by_job_run_id,
            event_keys_by_job_run_id,
        }
    }

    pub fn add_event_key(&self, job_run_id: JobRunId, event_key: StoreKey) -> RunState {
        let mut new_event_keys_by_job_run_id = self.event_keys_by_job_run_id.clone();

        new_event_keys_by_job_run_id
            .entry(job_run_id)
            .or_insert(Vec::new())
            .push(event_key.as_string());

        Self::from(
            self.status_by_job_run_id.clone(),
            new_event_keys_by_job_run_id,
        )
    }

    pub fn set_job_status(&self, job_run_id: JobRunId, status: JobRunStatus) -> RunState {
        let mut new_status_by_job_run_id = self.status_by_job_run_id.clone();

        new_status_by_job_run_id.insert(job_run_id, status);

        Self::from(
            new_status_by_job_run_id,
            self.event_keys_by_job_run_id.clone(),
        )
    }
}

fn compute_run_status(status_by_job_run_id: &HashMap<JobRunId, JobRunStatus>) -> RunStatus {
    if status_by_job_run_id
        .values()
        .any(|status| *status == JobRunStatus::Failed)
    {
        return RunStatus::Failed;
    }

    if status_by_job_run_id
        .values()
        .all(|status| *status == JobRunStatus::Succeeded)
    {
        return RunStatus::Succeeded;
    }

    RunStatus::Running
}
