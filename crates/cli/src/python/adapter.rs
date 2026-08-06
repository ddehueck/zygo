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
    target: String,
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
            let args = [
                base_entrypoint.args.clone(),
                vec!["run".into(), target.clone().into()],
            ]
            .concat();

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
