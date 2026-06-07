use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::EventId;

use super::data_reference::DataReference;
use super::ids::{ChannelId, JobId, JobRunId, RunId, WorkflowVersionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub is_replay: bool,
    pub timestamp: SystemTime,
    pub kind: EventKind,
    pub source: Source,
    /// Denormalized from the run for convenience in DB/proto serialization.
    /// May be `None` when constructing events in handlers (derived from the run).
    pub workflow_version_id: Option<WorkflowVersionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    DataReferenceInserted(DataReferenceInsertedData),
    ChannelItemInserted(ChannelItemInsertedData),
    JobStarted(JobStartedData),
    JobSucceeded(JobSucceededData),
    JobFailed(JobFailedData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStartedData {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSucceededData {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFailedData {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReferenceInsertedData {
    pub data_reference: DataReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelItemInsertedData {
    pub channel_id: ChannelId,
    pub data_reference: DataReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Input(InputSource),
    JobRun(JobRunSource),
}

impl Source {
    /// Get the workflow run ID from any source variant.
    pub fn workflow_run_id(&self) -> &RunId {
        match self {
            Source::Input(input) => &input.workflow_run_id,
            Source::JobRun(job_run) => &job_run.workflow_run_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRunSource {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
    pub workflow_run_id: RunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSource {
    pub workflow_run_id: RunId,
}
