pub mod channel;
pub mod commands;
pub mod data_reference;
pub mod edge;
pub mod event;
pub mod ids;
pub mod job;
pub mod job_args;
pub mod job_entrypoint;
pub mod job_run;
pub mod orchestrator_mode;
pub mod result_cache;
pub mod run;
pub mod sequence_id;
pub mod stream;
pub mod stream_append_cursor;
pub mod types;
pub mod workflow;
pub mod workflow_version;

/// Domain validation error for model construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainError {
    message: String,
}

impl DomainError {
    pub fn missing(field: &str) -> Self {
        Self {
            message: format!("{field} is required"),
        }
    }

    pub fn empty(field: &str) -> Self {
        Self {
            message: format!("{field} cannot be empty"),
        }
    }

    pub fn invalid(field: &str, reason: &str) -> Self {
        Self {
            message: format!("{field} is invalid: {reason}"),
        }
    }
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DomainError {}

// Re-exports for convenient access
pub use channel::Channel;
pub use commands::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, Command, ReplayJobCommand, RunJobCommand,
    SetJobRunStatusCommand,
};
pub use data_reference::DataReference;
pub use edge::{Edge, EdgeKind};
pub use event::{
    ChannelItemInsertedData, DataReferenceInsertedData, Event, EventKind, InputSource,
    JobFailedData, JobRunSource, JobStartedData, JobSucceededData, Source,
};
pub use ids::*;
pub use job::{Job, job_run_id};
pub use job_args::JobArgs;
pub use job_entrypoint::{JobEntrypoint, LocalEntrypoint, RemoteEntrypoint};
pub use job_run::{JobRun, JobRunStatus};
pub use orchestrator_mode::OrchestratorMode;
pub use result_cache::ResultCacheItem;
pub use run::{Run, RunStatus};
pub use sequence_id::SequenceId;
pub use stream::{StreamItem, StreamRecord};
pub use stream_append_cursor::StreamAppendCursor;
pub use workflow::Workflow;
pub use workflow_version::{WorkflowVersion, WorkflowVersionSchema};
