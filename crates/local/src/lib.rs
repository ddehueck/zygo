mod config;
mod paths;
mod repos;
mod stream_processor;

mod db;
mod service;

// This is the single entrypoint for the local Zygo service.
pub use config::{DEFAULT_DATABASE_BUSY_TIMEOUT, ZygoLocalConfig};
pub use service::ZygoLocalService;

// Type re-exports only for convenience
pub use db::{
    DbError, DbResult, JobRunSummaryCounts, JobRunSummaryRepository, JobRunSummaryRow,
    KvRepository, TagRow, WorkflowRunRepository, WorkflowRunRow, WorkflowRunSummaryRepository,
    WorkflowRunSummaryRow,
};
pub use repos::Repos;
