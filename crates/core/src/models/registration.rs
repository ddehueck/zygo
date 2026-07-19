use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{
    ChannelId, ChannelName, ContentHash, JobEntrypoint, JobId, JobName, WorkflowId, WorkflowName,
    WorkflowVersionId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkflowInput {
    pub name: WorkflowName,
    pub content_hash: ContentHash,
    pub channels: Vec<ChannelSchema>,
    pub jobs: Vec<JobSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredWorkflowSummary {
    pub workflow_id: WorkflowId,
    pub workflow_version_id: WorkflowVersionId,
    pub channel_ids_by_name: HashMap<ChannelName, ChannelId>,
    pub job_ids_by_name: HashMap<JobName, JobId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSchema {
    pub name: ChannelName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSchema {
    pub name: JobName,
    pub content_hash: ContentHash,
    pub input_channel_name: ChannelName,
    pub output_channel_names: Vec<ChannelName>,
    pub entrypoint: JobEntrypoint,
}
