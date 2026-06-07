use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{JobEntrypoint, JobName};

use super::ids::{ContentHash, JobId};

/// Namespace for generating deterministic job run IDs.
/// Same job + same data reference always yields the same job_run_id (pure function assumption).
const JOB_RUN_NAMESPACE: Uuid = Uuid::from_u128(0x6ba7_b811_9dad_11d1_80b4_00c04fd430c8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub name: JobName,
    pub content_hash: ContentHash,
    pub entrypoint: JobEntrypoint,
}

/// A core assumption of the system is that a job is a pure function of its input data.
/// Therefore, the job run id is a UUID5 derived from the job id and data reference (uri + etag).
/// This serves as the idempotency boundary for a job run.
pub fn job_run_id(job_id: &JobId, data_reference_uri: &str, data_reference_etag: &str) -> String {
    let name = format!(
        "{}\0{}\0{}",
        job_id.as_ref(),
        data_reference_uri,
        data_reference_etag
    );
    Uuid::new_v5(&JOB_RUN_NAMESPACE, name.as_bytes()).to_string()
}
