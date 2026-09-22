mod config;
mod dates;
mod db;
mod event_stream;
mod log_watcher;
mod log_writer;
mod paths;
mod runtime;
mod service;
mod stream_processor;
mod sync;

// This is the single entrypoint for the local Zygo service.
pub use config::{DEFAULT_DATABASE_BUSY_TIMEOUT, RunOptions, ZygoLocalConfig};
pub use event_stream::LocalEventStream;
pub use runtime::LocalRuntime;
pub use service::{ZygoLocalRun, ZygoLocalService, ZygoLocalWorkflow};
pub use stream_processor::{LocalStreamProcessor, ReadResult};

use dates::format_database_timestamp;

// Type re-exports only for convenience
pub use db::{
    CdcChangeType, CdcRepository, CdcRow, Cursor, CursorPaginator, DataReferenceModel,
    DataReferenceRepository, DbError, DbResult, EventStreamRepository, JobRunModel,
    JobRunRepository, LogRow, LogsRepository, Page, Repos, TagModel, TagsRepository, WorkflowModel,
    WorkflowRepository, WorkflowRunJobCounts, WorkflowRunModel, WorkflowRunRepository,
};
pub use log_watcher::LogWatcher;
pub use sync::{Delta, DeltaBatch, RowChange, SyncSubscription};
