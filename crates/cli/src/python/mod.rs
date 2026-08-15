//! These commands interact with the zygo python library.
//! They allow the CLI to retrieve metadata and run workflow jobs.

mod adapter;
mod types;

pub use adapter::workflow_schema_from_metadata;
pub use types::WorkflowMetadata;
