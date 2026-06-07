use crate::models::job_entrypoint::{JobEntrypoint, LocalEntrypoint, RemoteEntrypoint};
use crate::models::DomainError;
use crate::orchestrator_proto;

impl From<JobEntrypoint> for orchestrator_proto::job_schema::Entrypoint {
    fn from(entrypoint: JobEntrypoint) -> Self {
        match entrypoint {
            JobEntrypoint::Local(local) => orchestrator_proto::job_schema::Entrypoint::LocalEntrypoint(
                orchestrator_proto::LocalEntrypoint {
                    cwd: local.cwd,
                    exec: local.exec,
                },
            ),
            JobEntrypoint::Remote(remote) => {
                orchestrator_proto::job_schema::Entrypoint::RemoteEntrypoint(
                    orchestrator_proto::RemoteEntrypoint {
                        url: remote.url,
                        headers: remote.headers,
                    },
                )
            }
        }
    }
}

impl TryFrom<orchestrator_proto::job_schema::Entrypoint> for JobEntrypoint {
    type Error = DomainError;

    fn try_from(
        entrypoint: orchestrator_proto::job_schema::Entrypoint,
    ) -> Result<Self, Self::Error> {
        match entrypoint {
            orchestrator_proto::job_schema::Entrypoint::LocalEntrypoint(local) => {
                if local.cwd.trim().is_empty() {
                    return Err(DomainError::missing("local_entrypoint.cwd"));
                }
                if local.exec.trim().is_empty() {
                    return Err(DomainError::missing("local_entrypoint.exec"));
                }
                Ok(JobEntrypoint::Local(LocalEntrypoint {
                    cwd: local.cwd,
                    exec: local.exec,
                }))
            }
            orchestrator_proto::job_schema::Entrypoint::RemoteEntrypoint(remote) => {
                if remote.url.trim().is_empty() {
                    return Err(DomainError::missing("remote_entrypoint.url"));
                }
                Ok(JobEntrypoint::Remote(RemoteEntrypoint {
                    url: remote.url,
                    headers: remote.headers,
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::JobEntrypoint;

    #[test]
    fn converts_local_entrypoint() {
        let entrypoint = orchestrator_proto::job_schema::Entrypoint::LocalEntrypoint(
            orchestrator_proto::LocalEntrypoint {
                cwd: "/tmp".to_string(),
                exec: "run.sh".to_string(),
            },
        );

        let parsed: JobEntrypoint = entrypoint.try_into().expect("valid local entrypoint");
        assert!(matches!(parsed, JobEntrypoint::Local(_)));
    }

    #[test]
    fn rejects_empty_local_cwd() {
        let entrypoint = orchestrator_proto::job_schema::Entrypoint::LocalEntrypoint(
            orchestrator_proto::LocalEntrypoint {
                cwd: "  ".to_string(),
                exec: "run.sh".to_string(),
            },
        );

        let parsed: Result<JobEntrypoint, _> = entrypoint.try_into();
        assert!(parsed.is_err());
    }
}
