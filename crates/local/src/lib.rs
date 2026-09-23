pub mod api;
mod config;
mod context;
mod dates;
mod db;
mod engine;
mod error;
mod log_watcher;
mod log_writer;
pub mod models;
mod paths;
mod projector;
mod run;
mod runtime;
mod service;
mod sync;

// This is the single entrypoint for the local Zygo service.
pub use config::{DEFAULT_DATABASE_BUSY_TIMEOUT, RunOptions, ZygoLocalConfig};
pub use engine::EngineState;
pub use error::{Error, Result};
pub use run::ActorStateRx;
pub use runtime::LocalRuntime;
pub use service::{ZygoLocalRun, ZygoLocalService, ZygoLocalWorkflow};

use context::RunContext;
use dates::format_database_timestamp;
use run::RunHandle;
use runtime::RunJobArgs;

// Type re-exports only for convenience
pub use db::{
    CdcChangeType, CdcRepository, CdcRow, Cursor, CursorPaginator, DataReferenceModel,
    DataReferenceRepository, DbError, DbResult, JobRunModel, JobRunRepository, LogRow,
    LogsRepository, Page, Repos, TagModel, TagsRepository, WorkflowModel, WorkflowRepository,
    WorkflowRunJobCounts, WorkflowRunModel, WorkflowRunRepository,
};
pub use log_watcher::LogWatcher;
pub use sync::{Delta, DeltaBatch, RowChange, SyncSubscription};
