pub mod channel;
pub mod commands;
pub mod data_reference;
pub mod event;
pub mod ids;
pub mod job;
pub mod job_entrypoint;
pub mod job_run;
pub mod mode;
pub mod result_cache;
pub mod run;
pub mod schema;
pub mod sequence_id;
pub mod stream;

/// Domain validation error for model construction.
/// TODO: Not this or at least not this here.
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

pub use channel::Channel;
pub use commands::{
    CacheJobEventSourceCommand, CacheJobRunResultCommand, Command, ReplayJobCommand, RunJobCommand,
    SetJobRunStatusCommand,
};
pub use data_reference::DataReference;
pub use event::{
    ChannelItemInsertedData, DataReferenceInsertedData, Event, EventKind, JobFailedData,
    JobRunSource, JobStartedData, JobSucceededData, Source,
};
pub use ids::*;
pub use job::{Job, job_run_id};
pub use job_entrypoint::JobEntrypoint;
pub use job_run::{JobRun, JobRunStatus};
pub use mode::OrchestratorMode;
pub use result_cache::ResultCacheItem;
pub use run::WorkflowRunStatus;
pub use schema::WorkflowSchema;
pub use sequence_id::SequenceId;
pub use stream::{StreamItem, StreamRecord};
