use zygo_core::models::{
    Channel, ChannelId, ContentHash, Edge, EdgeKind as CoreEdgeKind, Job, JobEntrypoint,
    LocalEntrypoint, WorkflowSchema,
};

use crate::python::types::{EdgeKind, WorkflowMetadata};

/// Converts Python metadata and the command used to launch Python into the core schema.
///
/// The Python metadata does not contain process-launch information, so it is supplied as
/// the second tuple element rather than being invented by this adapter.
pub fn workflow_schema_from_metadata(
    metadata: WorkflowMetadata,
    base_entrypoint: LocalEntrypoint,
) -> Result<WorkflowSchema, zygo_core::models::DomainError> {
    let content_hash = ContentHash::try_from(metadata.content_hash)?;

    let channels = metadata
        .channels
        .into_iter()
        .map(|channel| {
            Ok(Channel {
                id: ChannelId::try_from(channel.id)?,
            })
        })
        .collect::<Result<Vec<_>, zygo_core::models::DomainError>>()?;

    let jobs = metadata
        .jobs
        .into_iter()
        .map(|job| {
            let job_id = job.id.clone();
            let args = [base_entrypoint.args.clone(), vec!["run".into()]].concat();

            Ok(Job {
                id: job_id.try_into()?,
                content_hash: job.content_hash.try_into()?,
                entrypoint: JobEntrypoint::Local(LocalEntrypoint {
                    cwd: base_entrypoint.cwd.clone(),
                    exec: base_entrypoint.exec.clone(),
                    args,
                }),
            })
        })
        .collect::<Result<Vec<_>, zygo_core::models::DomainError>>()?;

    let edges = metadata
        .edges
        .into_iter()
        .map(|edge| {
            Ok(Edge {
                job_id: edge.job_id.try_into()?,
                channel_id: edge.channel_id.try_into()?,
                kind: match edge.kind {
                    EdgeKind::Input => CoreEdgeKind::Input,
                    EdgeKind::Output => CoreEdgeKind::Output,
                },
            })
        })
        .collect::<Result<Vec<_>, zygo_core::models::DomainError>>()?;

    Ok(WorkflowSchema {
        content_hash,
        input_channel_id: ChannelId::try_from(metadata.input_channel)?,
        jobs,
        channels,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::python::types::{ChannelMetadata, EdgeMetadata, JobMetadata};

    #[test]
    fn converts_metadata_into_schema() {
        let metadata = WorkflowMetadata {
            id: "workflow".into(),
            input_channel: "input".into(),
            content_hash: "workflow-hash".into(),
            channels: vec![ChannelMetadata { id: "input".into() }],
            jobs: vec![JobMetadata {
                id: "job".into(),
                content_hash: "job-hash".into(),
            }],
            edges: vec![EdgeMetadata {
                job_id: "job".into(),
                channel_id: "input".into(),
                kind: EdgeKind::Input,
            }],
        };
        let entrypoint = LocalEntrypoint {
            cwd: "/tmp/project".into(),
            exec: "/tmp/project/.venv/bin/python".into(),
            args: vec!["-m".into(), "zygo._internal.ipc.v0".into()],
        };

        let schema = workflow_schema_from_metadata((metadata, entrypoint)).unwrap();

        assert_eq!(schema.content_hash.as_ref(), "workflow-hash");
        assert_eq!(schema.input_channel_id.as_ref(), "input");
        assert!(matches!(schema.entrypoint, JobEntrypoint::Local(_)));
        assert_eq!(schema.jobs[0].id.as_ref(), "job");
        assert_eq!(schema.edges[0].kind, CoreEdgeKind::Input);
    }
}
