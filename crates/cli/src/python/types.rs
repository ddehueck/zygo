//! Serializable models shared with the Python workflow library.
//!
//! These types define the contract between the Python library and the workflow engine.
//! A future update should generate them to keep both implementations consistent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub job_id: String,
    pub channel_id: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub input_channel: String,
    pub content_hash: String,
    pub channels: Vec<ChannelMetadata>,
    pub jobs: Vec<JobMetadata>,
    pub edges: Vec<EdgeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRunArgs {
    pub job_id: String,
    pub data_reference_uri: String,
    pub data_reference_etag: String,
    pub workflow_run_id: String,
    pub job_run_id: String,
}
