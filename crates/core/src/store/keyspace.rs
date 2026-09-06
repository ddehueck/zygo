//! Single source of truth for every key and prefix in the store.
//!
//! ```text
//! run/{run_id}/schema                              ← workflow schema
//! run/{run_id}/stream/{sequence_id}                ← stream item
//! run/{run_id}/tail                                ← sequence tail
//! run/{run_id}/snapshot                            ← state snapshot at the tail
//! cache/result/{job_run_id}                        ← result cache item
//! ```
//!

use serde::{Deserialize, Serialize};

use crate::models::{
    SequenceId,
    ids::{JobRunId, WorkflowRunId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StoreKey(String);

impl From<&str> for StoreKey {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for StoreKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl StoreKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct RunKeySpace {
    prefix: String,
}

pub struct CacheKeySpace {
    prefix: String,
}

pub struct KeySpace;

impl KeySpace {
    pub fn cache() -> CacheKeySpace {
        CacheKeySpace::new()
    }

    pub fn run(run_id: &WorkflowRunId) -> RunKeySpace {
        RunKeySpace::new(run_id)
    }
}

impl RunKeySpace {
    pub fn new(run_id: &WorkflowRunId) -> Self {
        Self {
            prefix: format!("run/{}", run_id),
        }
    }

    pub fn schema(&self) -> StoreKey {
        StoreKey(format!("{}/schema", self.prefix))
    }

    pub fn snapshot(&self) -> StoreKey {
        StoreKey(format!("{}/snapshot", self.prefix))
    }

    /// e.g. "run/{run_id}/tail"
    pub fn tail(&self) -> StoreKey {
        StoreKey(format!("{}/tail", self.prefix))
    }

    // e.g. "run/{run_id}/stream/{sequence_id}"
    pub fn stream_item(&self, sequence_id: &SequenceId) -> StoreKey {
        StoreKey(format!("{}/stream/{}", self.prefix, sequence_id))
    }
}

impl CacheKeySpace {
    pub fn new() -> Self {
        Self {
            prefix: String::from("cache"),
        }
    }

    pub fn result(&self, job_run_id: &JobRunId) -> StoreKey {
        StoreKey(format!("{}/result/{}", self.prefix, job_run_id))
    }
}
