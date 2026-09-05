mod config;
mod dates;
mod db;
mod paths;
mod repos;
mod service;
mod storage;
mod stream_processor;
mod sync;

// This is the single entrypoint for the local Zygo service.
pub use config::{DEFAULT_DATABASE_BUSY_TIMEOUT, ZygoLocalConfig};
pub use service::ZygoLocalService;

use dates::format_database_timestamp;

// Type re-exports only for convenience
pub use db::{
    CdcChangeType, CdcRepository, CdcRow, DbError, DbResult, JobRunRepository, JobRunRow,
    KvRepository, TagRow, TagsRepository, WorkflowRunJobCounts, WorkflowRunRepository,
    WorkflowRunRow,
};
pub use repos::Repos;
pub use sync::{Delta, DeltaBatch, SyncEntity, SyncSubscription};
