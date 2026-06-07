use serde::{Deserialize, Serialize};

use crate::models::Source;
use crate::store::keyspace::StoreKey;

use super::ids::{JobId, JobRunId};
use super::result_cache::ResultCacheItem;
use super::{data_reference::DataReference, job_run::JobRunStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    RunJob(RunJobCommand),
    ReplayJob(ReplayJobCommand),
    CacheJobRunResult(CacheJobRunResultCommand),
    CacheJobEventSource(CacheJobEventSourceCommand),
    SetJobRunStatus(SetJobRunStatusCommand),
}

impl Command {
    /// Returns true if the command is safe to issue during an event replay.
    pub fn is_replayable(&self) -> bool {
        match self {
            Command::SetJobRunStatus(_) => true,
            Command::RunJob(_) => true,
            Command::ReplayJob(_) => true,
            Command::CacheJobEventSource(_) => false,
            Command::CacheJobRunResult(_) => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobCommand {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
    pub data_reference: DataReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayJobCommand {
    pub source: Source,
    pub cache_item: ResultCacheItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheJobEventSourceCommand {
    pub job_run_id: JobRunId,
    pub event_key: StoreKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheJobRunResultCommand {
    pub job_run_id: JobRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetJobRunStatusCommand {
    pub job_run_id: JobRunId,
    pub status: JobRunStatus,
}
