//! Serializable models shared with the Python workflow library.
//!
//! These types define the contract between the Python library and the workflow engine.
//! A future update should generate them to keep both implementations consistent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMetadata {
    pub id: String,
    pub accepted_file_extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: String,
    pub content_hash: String,
    pub input_channel_id: String,
    pub output_channel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub input_channel_id: String,
    pub output_channel_id: String,
    pub content_hash: String,
    pub jobs: Vec<JobMetadata>,
    pub channels: Vec<ChannelMetadata>,
}
