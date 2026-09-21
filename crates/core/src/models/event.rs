use super::ids::{ChannelId, JobId, JobRunId};
use crate::models::{DataReferenceUri, EventId, WorkflowRunId};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub is_replay: bool,
    pub timestamp: SystemTime,
    pub kind: EventKind,
    pub source: Source,
    pub run_id: WorkflowRunId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    DataReferenceInserted(DataReferenceInsertedData),
    ChannelItemInserted(ChannelItemInsertedData),
    JobEnqueued(JobEnqueuedData),
    JobStarted(JobStartedData),
    JobSucceeded(JobSucceededData),
    JobFailed(JobFailedData),
    TagInserted(TagInsertedData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEnqueuedData {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
    pub input: DataReferenceUri,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStartedData {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
    pub input: DataReferenceUri,
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
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReferenceInsertedData {
    pub uri: DataReferenceUri,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInsertedData {
    pub value: String,
    pub data_reference: Option<DataReferenceUri>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelItemInsertedData {
    pub channel_id: ChannelId,
    pub item: DataReferenceUri,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Input,
    JobRun(JobRunSource),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRunSource {
    pub job_id: JobId,
    pub job_run_id: JobRunId,
}
