//! Engine runtime state and durable restart snapshots.

use std::{collections::HashMap, time::SystemTime};

use serde::{Deserialize, Serialize};

use crate::{
    context::RunContext,
    models::{
        Event, EventId, EventKind, JobRunId, JobRunStatus, ResultCacheItem, SequenceId, Source,
        WorkflowRunStatus, WorkflowSchema,
    },
    store::{StorageProvider, StoreKey, keyspace::KeySpace},
};

#[derive(Clone)]
pub struct ResultCache<S: StorageProvider> {
    pub context: RunContext<S>,
    pub schema: WorkflowSchema,
}

impl<S: StorageProvider> ResultCache<S> {
    pub fn new(context: RunContext<S>, schema: WorkflowSchema) -> Self {
        Self { context, schema }
    }

    pub async fn get_item(
        &self,
        job_run_id: &JobRunId,
    ) -> Result<Option<ResultCacheItem>, anyhow::Error> {
        let key = KeySpace::cache().result(job_run_id);
        self.context
            .store
            .get(&key)
            .await?
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| {
                anyhow::anyhow!(
                    "failed to deserialize result cache item for job run {job_run_id}: {error}"
                )
            })
    }

    pub async fn put(
        &self,
        job_run_id: &JobRunId,
        result_cache_item: &ResultCacheItem,
    ) -> Result<(), anyhow::Error> {
        let key = KeySpace::cache().result(job_run_id);
        let value = serde_json::to_value(result_cache_item)?;
        self.context.store.put(&[(key, value)]).await
    }

    pub fn make_replay_event(&self, kind: EventKind, source: Source) -> Event {
        Event {
            id: EventId::new(),
            is_replay: true,
            timestamp: SystemTime::now(),
            kind,
            source,
            run_id: self.context.run_id.clone(),
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

impl Default for EngineSnapshot {
    fn default() -> Self {
        Self::new()
    }
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
    pub status: WorkflowRunStatus,
    pub status_by_job_run_id: HashMap<JobRunId, JobRunStatus>,
    pub event_keys_by_job_run_id: HashMap<JobRunId, Vec<StoreKey>>,
}

impl Default for RunState {
    fn default() -> Self {
        Self::new()
    }
}

impl RunState {
    pub fn new() -> Self {
        Self {
            status: WorkflowRunStatus::Running,
            status_by_job_run_id: HashMap::new(),
            event_keys_by_job_run_id: HashMap::new(),
        }
    }

    pub fn from(
        status_by_job_run_id: HashMap<JobRunId, JobRunStatus>,
        event_keys_by_job_run_id: HashMap<JobRunId, Vec<StoreKey>>,
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
            .or_default()
            .push(event_key);

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

fn compute_run_status(status_by_job_run_id: &HashMap<JobRunId, JobRunStatus>) -> WorkflowRunStatus {
    if status_by_job_run_id.is_empty() {
        return WorkflowRunStatus::Running;
    }

    if status_by_job_run_id
        .values()
        .any(|status| *status == JobRunStatus::Failed)
    {
        return WorkflowRunStatus::Failed;
    }

    if status_by_job_run_id
        .values()
        .all(|status| *status == JobRunStatus::Succeeded)
    {
        return WorkflowRunStatus::Succeeded;
    }

    WorkflowRunStatus::Running
}
