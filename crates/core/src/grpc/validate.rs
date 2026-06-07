use tonic::Status;

use crate::models::{JobEntrypoint, OrchestratorMode};
use super::parse::JobSchemaInput;

pub(super) fn validate_jobs_by_mode(
    jobs: &[JobSchemaInput],
    mode: &OrchestratorMode,
) -> Result<(), Status> {
    for job in jobs {
        match (mode, &job.entrypoint) {
            (OrchestratorMode::Local, JobEntrypoint::Remote(_)) => {
                return Err(Status::invalid_argument(format!(
                    "Orchestrator is in local mode but job '{}' has a remote entrypoint",
                    job.name.as_ref()
                )));
            }
            (OrchestratorMode::Remote, JobEntrypoint::Local(_)) => {
                return Err(Status::invalid_argument(format!(
                    "Orchestrator is in remote mode but job '{}' has a local entrypoint",
                    job.name.as_ref()
                )));
            }
            _ => {}
        }
    }
    Ok(())
}
