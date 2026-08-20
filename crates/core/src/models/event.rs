use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::models::{EventId, WorkflowRunId};

use super::data_reference::DataReference;
use super::ids::{ChannelId, JobId, JobRunId};

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
    JobStarted(JobStartedData),
    JobSucceeded(JobSucceededData),
    JobFailed(JobFailedData),
    TagInserted(TagInsertedData),
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
pub struct TagInsertedData {
    pub name: String,
    pub value: String,
    pub data_reference: Option<DataReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelItemInsertedData {
    pub channel_id: ChannelId,
    pub data_reference: DataReference,
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
