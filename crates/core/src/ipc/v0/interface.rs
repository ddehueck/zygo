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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChannelMetadata {
    pub id: String,
    pub accepted_file_extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JobMetadata {
    pub id: String,
    pub content_hash: String,
    pub input_channel_id: String,
    pub output_channel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkflowMetadata {
    pub id: String,
    pub input_channel_id: String,
    pub output_channel_id: String,
    pub content_hash: String,
    pub jobs: Vec<JobMetadata>,
    pub channels: Vec<ChannelMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RunCommandArgs {
    pub job_id: String,
    pub data_reference_uri: String,
    pub data_reference_version: String,
    pub workflow_run_id: String,
    pub job_run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataReference {
    pub uri: String,
    pub version: String,
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
    TagInserted {
        name: String,
        value: String,
        data_reference: Option<DataReference>,
    },
}
