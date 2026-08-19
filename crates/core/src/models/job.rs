use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::JobEntrypoint;

use super::ids::{ChannelId, ContentHash, JobId};

/// Namespace for generating deterministic job run IDs.
/// The same job definition and data reference always yield the same job run ID.
const JOB_RUN_NAMESPACE: Uuid = Uuid::from_u128(0x6ba7_b811_9dad_11d1_80b4_00c0_4fd4_30c8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub input_channel_id: ChannelId,
    pub output_channel_id: ChannelId,
    pub content_hash: ContentHash,
    pub entrypoint: JobEntrypoint,
}

/// A core assumption of the system is that a job is a pure function of its input data.
/// Therefore, the job run ID is a UUID5 derived from the job ID, job content hash,
/// and data reference (URI + etag). This is the global idempotency boundary for a job run.
pub fn job_run_id(job: &Job, data_reference_uri: &str, data_reference_etag: &str) -> String {
    let name = format!(
        "{}\0{}\0{}\0{}",
        job.id.as_ref(),
        job.content_hash.as_ref(),
        data_reference_uri,
        data_reference_etag
    );
    Uuid::new_v5(&JOB_RUN_NAMESPACE, name.as_bytes()).to_string()
}
