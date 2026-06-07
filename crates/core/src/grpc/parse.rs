use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tonic::Status;

use crate::models::data_reference::DataReference;
use crate::models::event::{InputSource, JobRunSource, Source};
use crate::models::{
    ChannelItemInsertedData, ChannelName, ContentHash, DataReferenceInsertedData, DomainError,
    Event, EventId, EventKind, JobEntrypoint, JobFailedData, JobId, JobName, JobRunId,
    JobStartedData, JobSucceededData, RunId, WorkflowId, WorkflowName, WorkflowVersionId,
    ChannelId,
};
use crate::orchestrator_proto;

#[derive(Debug, Clone)]
pub struct RegisterWorkflowInput {
    pub name: WorkflowName,
    pub content_hash: ContentHash,
    pub channels: Vec<ChannelSchemaInput>,
    pub jobs: Vec<JobSchemaInput>,
}

#[derive(Debug, Clone)]
pub struct ChannelSchemaInput {
    pub name: ChannelName,
}

#[derive(Debug, Clone)]
pub struct JobSchemaInput {
    pub name: JobName,
    pub content_hash: ContentHash,
    pub input_channel_name: ChannelName,
    pub output_channel_names: Vec<ChannelName>,
    pub entrypoint: JobEntrypoint,
}

/// Parse a proto DataReference into a domain DataReference.
fn parse_data_reference(
    proto: orchestrator_proto::DataReference,
) -> Result<DataReference, Status> {
    if proto.uri.trim().is_empty() {
        return Err(Status::invalid_argument("data_reference.uri is required"));
    }
    if proto.etag.trim().is_empty() {
        return Err(Status::invalid_argument("data_reference.etag is required"));
    }
    Ok(DataReference {
        uri: proto.uri,
        etag: proto.etag,
        content_type: proto.content_type,
        size_bytes: proto.size_bytes,
    })
}

/// Parse a proto Source oneof into a domain Source.
fn parse_source(
    source: Option<orchestrator_proto::job_run_event::Source>,
    workflow_run_id: RunId,
) -> Result<Source, Status> {
    match source {
        Some(orchestrator_proto::job_run_event::Source::InputSource(_)) | None => {
            Ok(Source::Input(InputSource { workflow_run_id }))
        }
        Some(orchestrator_proto::job_run_event::Source::JobRunSource(jr)) => {
            let job_id = JobId::try_from(jr.job_id)
                .map_err(|e| Status::invalid_argument(format!("invalid source job_id: {e}")))?;
            let job_run_id = JobRunId::try_from(jr.job_run_id).map_err(|e| {
                Status::invalid_argument(format!("invalid source job_run_id: {e}"))
            })?;
            Ok(Source::JobRun(JobRunSource {
                job_id,
                job_run_id,
                workflow_run_id,
            }))
        }
    }
}

fn job_run_ids_from_source(source: &Source) -> Result<(JobId, JobRunId), Status> {
    match source {
        Source::JobRun(job_run) => Ok((job_run.job_id.clone(), job_run.job_run_id.clone())),
        Source::Input(_) => Err(Status::invalid_argument(
            "job lifecycle events require a JobRunSource",
        )),
    }
}

fn proto_timestamp(timestamp: Option<prost_types::Timestamp>) -> SystemTime {
    timestamp
        .map(|ts| {
            UNIX_EPOCH
                + Duration::from_secs(ts.seconds as u64)
                + Duration::from_nanos(ts.nanos as u64)
        })
        .unwrap_or_else(SystemTime::now)
}

/// Result of parsing a JobRunEvent.
pub struct ParsedJobRunEvent {
    pub event: Event,
    pub workflow_id: WorkflowId,
    pub workflow_version_id: WorkflowVersionId,
}

/// Parse a proto JobRunEvent into a domain Event.
pub fn parse_job_run_event(
    proto: orchestrator_proto::JobRunEvent,
) -> Result<ParsedJobRunEvent, Status> {
    let run_id = proto
        .run_id
        .ok_or_else(|| Status::invalid_argument("run_id is required"))?;

    let workflow_id = WorkflowId::try_from(run_id.workflow_id)
        .map_err(|e| Status::invalid_argument(format!("invalid workflow_id: {e}")))?;

    let workflow_version_id = WorkflowVersionId::try_from(run_id.workflow_version_id)
        .map_err(|e| Status::invalid_argument(format!("invalid workflow_version_id: {e}")))?;

    let workflow_run_id = RunId::try_from(run_id.workflow_run_id)
        .map_err(|e| Status::invalid_argument(format!("invalid workflow_run_id: {e}")))?;

    let source = parse_source(proto.source, workflow_run_id)?;

    let proto_event = proto
        .event
        .ok_or_else(|| Status::invalid_argument("event kind is required"))?;

    let (kind, timestamp) = match proto_event {
        orchestrator_proto::job_run_event::Event::DataReferenceInserted(e) => {
            let dr_proto = e
                .data_reference
                .ok_or_else(|| Status::invalid_argument("data_reference is required"))?;
            let data_reference = parse_data_reference(dr_proto)?;
            (
                EventKind::DataReferenceInserted(DataReferenceInsertedData { data_reference }),
                SystemTime::now(),
            )
        }
        orchestrator_proto::job_run_event::Event::ChannelItemInserted(e) => {
            let channel_id = ChannelId::try_from(e.channel_id)
                .map_err(|err| Status::invalid_argument(format!("invalid channel_id: {err}")))?;
            let dr_proto = e
                .data_reference
                .ok_or_else(|| Status::invalid_argument("data_reference is required"))?;
            let data_reference = parse_data_reference(dr_proto)?;
            (
                EventKind::ChannelItemInserted(ChannelItemInsertedData {
                    channel_id,
                    data_reference,
                }),
                SystemTime::now(),
            )
        }
        orchestrator_proto::job_run_event::Event::JobRequested(_) => {
            return Err(Status::invalid_argument(
                "JobRequested events are not supported by the engine",
            ));
        }
        orchestrator_proto::job_run_event::Event::JobStarted(e) => {
            let (job_id, job_run_id) = job_run_ids_from_source(&source)?;
            (
                EventKind::JobStarted(JobStartedData { job_id, job_run_id }),
                proto_timestamp(e.started_at),
            )
        }
        orchestrator_proto::job_run_event::Event::JobSucceeded(e) => {
            let (job_id, job_run_id) = job_run_ids_from_source(&source)?;
            (
                EventKind::JobSucceeded(JobSucceededData { job_id, job_run_id }),
                proto_timestamp(e.succeeded_at),
            )
        }
        orchestrator_proto::job_run_event::Event::JobFailed(e) => {
            let (job_id, job_run_id) = job_run_ids_from_source(&source)?;
            (
                EventKind::JobFailed(JobFailedData { job_id, job_run_id }),
                proto_timestamp(e.failed_at),
            )
        }
    };

    let event_id = EventId::try_from(proto.id)
        .map_err(|e| Status::invalid_argument(format!("invalid event id: {e}")))?;

    let event = Event {
        id: event_id,
        is_replay: false,
        timestamp,
        kind,
        source,
        workflow_version_id: Some(workflow_version_id.clone()),
    };

    Ok(ParsedJobRunEvent {
        event,
        workflow_id,
        workflow_version_id,
    })
}

/// Parse a proto RegisterWorkflowRequest into a RegisterWorkflowInput.
pub fn parse_register_workflow_request(
    request: orchestrator_proto::RegisterWorkflowRequest,
) -> Result<RegisterWorkflowInput, Status> {
    let name = WorkflowName::try_from(request.name)
        .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

    let content_hash = ContentHash::try_from(request.content_hash)
        .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

    let channels: Vec<ChannelSchemaInput> = request
        .channels
        .into_iter()
        .map(|ch| {
            let channel_name = ChannelName::try_from(ch.name)
                .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;
            Ok(ChannelSchemaInput { name: channel_name })
        })
        .collect::<Result<Vec<_>, Status>>()?;

    let jobs: Vec<JobSchemaInput> = request
        .jobs
        .into_iter()
        .map(|job| {
            let job_name = JobName::try_from(job.name)
                .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

            let job_content_hash = ContentHash::try_from(job.content_hash)
                .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

            let input_channel_name = ChannelName::try_from(job.input_channel_name)
                .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

            let output_channel_names: Vec<ChannelName> = job
                .output_channel_names
                .into_iter()
                .map(|n| {
                    ChannelName::try_from(n)
                        .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))
                })
                .collect::<Result<Vec<_>, Status>>()?;

            let entrypoint = job
                .entrypoint
                .ok_or_else(|| Status::invalid_argument("job entrypoint is required"))?
                .try_into()
                .map_err(|e: DomainError| Status::invalid_argument(e.to_string()))?;

            Ok(JobSchemaInput {
                name: job_name,
                content_hash: job_content_hash,
                input_channel_name,
                output_channel_names,
                entrypoint,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;

    Ok(RegisterWorkflowInput {
        name,
        content_hash,
        channels,
        jobs,
    })
}
