//! Serializable models shared with the Python workflow library.
//!
//! These types define the contract between the Python library and the workflow engine.
//! A future update should generate them to keep both implementations consistent.
//!
//! NB: THere should be no model imports in this file. This should be entirely self-contained.

use serde::{Deserialize, Serialize};

// TODO: How to best serialize a python module CLI interface?
pub const STDOUT_IPC_PREFIX: &str = "ZYGO_IPC=";
pub const ZYGO_PKG_INTERNAL_CLI_MODULE: &str = "zygo._internal.ipc.v0"; // TODO: Better name? Maybe CLI target?
pub const RUN_CMD: &str = "run";
pub const METADATA_CMD: &str = "metadata";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelMetadata {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JobMetadata {
    pub id: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EdgeMetadata {
    pub job_id: String,
    pub channel_id: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowMetadata {
    pub id: String,
    pub input_channel: String,
    pub content_hash: String,
    pub channels: Vec<ChannelMetadata>,
    pub jobs: Vec<JobMetadata>,
    pub edges: Vec<EdgeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RunCommandArgs {
    pub job_id: String,
    pub data_reference_uri: String,
    pub data_reference_etag: String,
    pub workflow_run_id: String,
    pub job_run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataReference {
    pub uri: String,
    pub etag: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdoutIPCMessage {
    DataReferenceCreated {
        data_reference: DataReference,
    },
    ChannelItemInserted {
        channel_id: String,
        data_reference: DataReference,
    },
}
