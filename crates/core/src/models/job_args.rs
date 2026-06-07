use std::collections::HashMap;

use crate::models::{ChannelId, ChannelName};

/// Arguments passed to the job entrypoint via `--job-args`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct JobArgs {
    pub run_id: String,
    pub workflow_id: String,
    pub workflow_version_id: String,
    pub job_id: String,
    pub data_reference_uri: String,
    pub data_reference_etag: String,
    pub channel_ids_by_name: HashMap<ChannelName, ChannelId>,
    pub job_run_id: String,
}
