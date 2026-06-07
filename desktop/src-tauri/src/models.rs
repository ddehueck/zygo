use serde::{Deserialize, Serialize};
use specta::Type;

use crate::grpc_client;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, Default)]
pub enum SortOrder {
    #[default]
    Unspecified,
    Asc,
    Desc,
}

impl From<SortOrder> for i32 {
    fn from(s: SortOrder) -> i32 {
        match s {
            SortOrder::Unspecified => 0,
            SortOrder::Asc => 1,
            SortOrder::Desc => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Workflow {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListWorkflowsParams {
    pub workflow_id: Option<String>,
    pub sort: Option<SortOrder>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Run {
    pub id: String,
    pub workflow_version_id: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListRunsParams {
    pub workflow_id: String,
    pub run_id: Option<String>,
    pub sort: Option<SortOrder>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Event {
    pub id: String,
    pub workflow_version_id: String,
    pub workflow_run_id: String,
    pub is_replay: bool,
    pub sequence_number: Option<i64>,
    pub timestamp: Option<String>,
    pub event_kind: String,
    pub job_id: Option<String>,
    pub job_run_id: Option<String>,
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListRunEventsParams {
    pub run_id: String,
    pub sequence_number: Option<i64>,
    pub sort: Option<SortOrder>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Job {
    pub id: String,
    pub workflow_version_id: String,
    pub name: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Channel {
    pub id: String,
    pub workflow_version_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Edge {
    pub workflow_version_id: String,
    pub job_id: String,
    pub channel_id: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WorkflowVersionSchema {
    pub jobs: Vec<Job>,
    pub channels: Vec<Channel>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetWorkflowVersionSchemaParams {
    pub workflow_version_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamWorkflowCursor {
    pub workflow_id: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamRunCursor {
    pub run_id: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamEventCursor {
    pub event_id: Option<String>,
    pub sequence_number: Option<i64>,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamCursor {
    pub workflows: Option<LivestreamWorkflowCursor>,
    pub runs: Option<LivestreamRunCursor>,
    pub events: Option<LivestreamEventCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamParams {
    pub cursor: Option<LivestreamCursor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LivestreamResponse {
    pub workflows: Vec<Workflow>,
    pub runs: Vec<Run>,
    pub events: Vec<Event>,
    pub next_cursor: Option<LivestreamCursor>,
}

impl From<grpc_client::ProtoWorkflow> for Workflow {
    fn from(p: grpc_client::ProtoWorkflow) -> Self {
        Self {
            id: p.id,
            name: p.name,
        }
    }
}

impl From<grpc_client::ProtoRun> for Run {
    fn from(p: grpc_client::ProtoRun) -> Self {
        Self {
            id: p.id,
            workflow_version_id: p.workflow_version_id,
            created_at: p.created_at.map(|ts| {
                chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            }),
        }
    }
}

impl From<grpc_client::ProtoEvent> for Event {
    fn from(p: grpc_client::ProtoEvent) -> Self {
        use grpc_client::orchestrator_proto::proto_event::{Kind, Source};

        // Extract job_id and job_run_id from the source
        let (source_job_id, source_job_run_id) = match &p.source {
            Some(Source::JobRunSource(jr)) => {
                (Some(jr.job_id.clone()), Some(jr.job_run_id.clone()))
            }
            _ => (None, None),
        };

        let (event_kind, job_id, channel_id) = match &p.kind {
            Some(Kind::DataReferenceInserted(_)) => {
                ("data_reference_inserted", None, None)
            }
            Some(Kind::ChannelItemInserted(e)) => (
                "channel_item_inserted",
                None,
                Some(e.channel_id.clone()),
            ),
            Some(Kind::JobRequested(e)) => (
                "job_requested",
                Some(e.job_id.clone()),
                None,
            ),
            Some(Kind::JobStarted(_)) => (
                "job_started",
                source_job_id.clone(),
                None,
            ),
            Some(Kind::JobSucceeded(_)) => (
                "job_succeeded",
                source_job_id.clone(),
                None,
            ),
            Some(Kind::JobFailed(_)) => (
                "job_failed",
                source_job_id.clone(),
                None,
            ),
            None => ("unknown", None, None),
        };

        Self {
            id: p.id,
            workflow_version_id: p.workflow_version_id,
            workflow_run_id: p.workflow_run_id,
            is_replay: p.is_replay,
            sequence_number: p.sequence_number,
            timestamp: p.timestamp.map(|ts| {
                chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            }),
            event_kind: event_kind.to_string(),
            job_id,
            job_run_id: source_job_run_id,
            channel_id,
        }
    }
}

impl From<grpc_client::ProtoJob> for Job {
    fn from(p: grpc_client::ProtoJob) -> Self {
        Self {
            id: p.id,
            workflow_version_id: p.workflow_version_id,
            name: p.name,
            content_hash: p.content_hash,
        }
    }
}

impl From<grpc_client::ProtoChannel> for Channel {
    fn from(p: grpc_client::ProtoChannel) -> Self {
        Self {
            id: p.id,
            workflow_version_id: p.workflow_version_id,
            name: p.name,
        }
    }
}

impl From<grpc_client::ProtoEdge> for Edge {
    fn from(p: grpc_client::ProtoEdge) -> Self {
        let kind = match p.kind {
            0 => "unspecified",
            1 => "input",
            2 => "output",
            _ => "unknown",
        };
        Self {
            workflow_version_id: p.workflow_version_id,
            job_id: p.job_id,
            channel_id: p.channel_id,
            kind: kind.to_string(),
        }
    }
}

// Convert from model LivestreamCursor to gRPC LivestreamCursor (for requests)
impl From<LivestreamCursor> for grpc_client::LivestreamCursor {
    fn from(c: LivestreamCursor) -> Self {
        Self {
            workflows: c.workflows.map(|w| grpc_client::LivestreamWorkflowCursor {
                workflow_id: w.workflow_id,
                limit: w.limit,
            }),
            runs: c.runs.map(|r| grpc_client::LivestreamRunCursor {
                run_id: r.run_id,
                limit: r.limit,
            }),
            events: c.events.map(|e| grpc_client::LivestreamEventCursor {
                event_id: e.event_id,
                sequence_number: e.sequence_number,
                limit: e.limit,
            }),
        }
    }
}

// Convert from gRPC LivestreamCursor to model LivestreamCursor (for responses)
impl From<grpc_client::LivestreamCursor> for LivestreamCursor {
    fn from(c: grpc_client::LivestreamCursor) -> Self {
        Self {
            workflows: c.workflows.map(|w| LivestreamWorkflowCursor {
                workflow_id: w.workflow_id,
                limit: w.limit,
            }),
            runs: c.runs.map(|r| LivestreamRunCursor {
                run_id: r.run_id,
                limit: r.limit,
            }),
            events: c.events.map(|e| LivestreamEventCursor {
                event_id: e.event_id,
                sequence_number: e.sequence_number,
                limit: e.limit,
            }),
        }
    }
}
