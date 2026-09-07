mod config;
mod dates;
mod db;
mod log_watcher;
mod log_writer;
mod paths;
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
    CdcChangeType, CdcRepository, CdcRow, Cursor, CursorPaginator, DataReferenceModel,
    DataReferenceRepository, DbError, DbResult, JobRunModel, JobRunRepository, KvModel,
    KvRepository, LogRow, LogsRepository, Page, Repos, TagModel, TagsRepository,
    WorkflowRunJobCounts, WorkflowRunModel, WorkflowRunRepository,
};
pub use log_watcher::LogWatcher;
pub use sync::{Delta, DeltaBatch, RowChange, SyncSubscription};
