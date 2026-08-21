mod paths;
mod repos;
mod stream_processor;

mod db;
mod service;

// This is the single entrypoint for the local Zygo service.
pub use service::ZygoLocalService;

// Type re-exports only for convenience
pub use db::{KvRepository, Tag, WorkflowRun, WorkflowRunRepository};
pub use repos::Repos;
