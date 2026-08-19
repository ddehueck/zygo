use zygo_core::ipc;
use zygo_core::models::{ChannelId, ContentHash, Job, JobEntrypoint, WorkflowSchema};

use crate::python::types::WorkflowMetadata;

/// Converts Python metadata and the command used to launch Python into the core schema.
///
/// The Python metadata does not contain process-launch information, so it is supplied as
/// the second tuple element rather than being invented by this adapter.
pub fn workflow_schema_from_metadata(
    metadata: WorkflowMetadata,
    cwd: &str,
    target: &str,
    python: &str,
) -> Result<WorkflowSchema, zygo_core::models::DomainError> {
    let content_hash = ContentHash::try_from(metadata.content_hash)?;

    let jobs = metadata
        .jobs
        .into_iter()
        .map(|job| {
            let python_cli = ipc::v0::PythonCli::new(python.into(), cwd.into(), target.into());

            Ok(Job {
                id: job.id.try_into()?,
                content_hash: job.content_hash.try_into()?,
                entrypoint: JobEntrypoint::Python(python_cli),
                input_channel_id: ChannelId::try_from(job.input_channel_id)?,
                output_channel_id: ChannelId::try_from(job.output_channel_id)?,
            })
        })
        .collect::<Result<Vec<_>, zygo_core::models::DomainError>>()?;

    Ok(WorkflowSchema {
        content_hash,
        input_channel_id: ChannelId::try_from(metadata.input_channel_id)?,
        output_channel_id: ChannelId::try_from(metadata.output_channel_id)?,
        jobs,
    })
}
