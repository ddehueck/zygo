use serde::Serialize;

/// Arguments passed to the job entrypoint via `--job-args`.
#[derive(Debug, Clone, Serialize)]
pub struct JobArgs {
    pub job_id: String,
    pub data_reference_uri: String,
    pub data_reference_etag: String,
}
