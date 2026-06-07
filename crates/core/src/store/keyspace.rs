//! Single source of truth for every key and prefix in the store.
//!
//! ```text
//! wf/{wf_id}/meta                                  ← workflow blob
//! v/{wf_id}/{ver_id}/meta                          ← version blob
//! r/{wf_id}/{ver_id}/{run_id}/meta                 ← run blob
//! s/{wf_id}/{ver_id}/{run_id}/{sequence_id}        ← stream item
//! ```
//!
//! Each stream has its own top-level prefix, so `list_range` on any
//! prefix stays scoped to that stream — no delimiter support required
//! from the backend.
//!
//! Stream sequence ids are fixed-width integers, so lexicographic order
//! matches engine processing order.
//!

use serde::{Deserialize, Serialize};

use crate::models::ids::{RunId, WorkflowId, WorkflowVersionId};

const WORKFLOW_PREFIX: &str = "w";
const VERSION_PREFIX: &str = "v";
const RUN_PREFIX: &str = "r";
const STREAM_PREFIX: &str = "s";
const STREAM_APPEND_CURSOR_PREFIX: &str = "sac";
const ENGINE_SNAPSHOT_PREFIX: &str = "es";
const RESULT_CACHE_PREFIX: &str = "rc";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreKey(String);

impl StoreKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

// TODO: Add this an add builder methods to build keys from the prefix.
// pub struct StorexPrefix(String);

pub(crate) struct KeySpace;

impl KeySpace {
    // ----- global stream prefixes -----

    pub fn workflows_prefix() -> String {
        format!("{WORKFLOW_PREFIX}/")
    }

    pub fn versions_prefix() -> String {
        format!("{VERSION_PREFIX}/")
    }

    pub fn runs_prefix() -> String {
        format!("{RUN_PREFIX}/")
    }

    pub fn stream_items_prefix() -> String {
        format!("{STREAM_PREFIX}/")
    }

    // ----- key-space builders -----

    pub fn workflow(workflow_id: WorkflowId) -> WorkflowKeySpace {
        WorkflowKeySpace { workflow_id }
    }
}

#[derive(Clone)]
pub(crate) struct WorkflowKeySpace {
    workflow_id: WorkflowId,
}

impl WorkflowKeySpace {
    /// e.g. "wf/wf_xyz/meta"
    pub fn meta(&self) -> StoreKey {
        StoreKey(format!("{}/{}/meta", WORKFLOW_PREFIX, self.workflow_id))
    }

    /// e.g. "rc/wf_xyz/job_hash_input_data_hash"
    pub fn cache_result(self, job_run_id: &str) -> StoreKey {
        StoreKey(format!(
            "{}/{}/{}",
            RESULT_CACHE_PREFIX, self.workflow_id, job_run_id
        ))
    }

    pub fn version(self, ver: WorkflowVersionId) -> VersionKeySpace {
        VersionKeySpace {
            workflow_id: self.workflow_id.clone(),
            version_id: ver.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct VersionKeySpace {
    workflow_id: WorkflowId,
    version_id: WorkflowVersionId,
}

impl VersionKeySpace {
    /// e.g. "v/wf_xyz/v_xyz/meta"
    pub fn meta(&self) -> StoreKey {
        StoreKey(format!(
            "{}/{}/{}/meta",
            VERSION_PREFIX, self.workflow_id, self.version_id
        ))
    }

    /// Prefix for every run under this (workflow, version), e.g. "r/wf_xyz/v_xyz/".
    /// Because each run's meta key is "r/wf/ver/run/meta", a lexicographic scan
    /// of this prefix walks runs in deterministic (run_id) order.
    pub fn runs_prefix(&self) -> String {
        format!("{}/{}/{}/", RUN_PREFIX, self.workflow_id, self.version_id)
    }

    pub fn run(self, run: RunId) -> RunKeySpace {
        RunKeySpace {
            workflow_id: self.workflow_id.clone(),
            version_id: self.version_id.clone(),
            run_id: run.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RunKeySpace {
    workflow_id: WorkflowId,
    version_id: WorkflowVersionId,
    run_id: RunId,
}

impl RunKeySpace {
    fn path(&self) -> String {
        format!("{}/{}/{}", self.workflow_id, self.version_id, self.run_id)
    }

    /// e.g. "r/wf_xyz/v_abc/r_xyz/meta"
    pub fn meta(&self) -> StoreKey {
        StoreKey(format!("{}/{}/meta", RUN_PREFIX, self.path()))
    }

    /// Prefix for every engine stream item under this run.
    pub fn stream_items_prefix(&self) -> String {
        format!("{}/{}/", STREAM_PREFIX, self.path())
    }

    /// e.g. "s/wf_xyz/v_abc/r_xyz/00000000000000000001"
    pub fn stream_item(&self, sequence_id: &str) -> StoreKey {
        StoreKey(format!("{}/{}/{}", STREAM_PREFIX, self.path(), sequence_id))
    }

    /// e.g. "sac/wf_xyz/v_abc/r_xyz/append_cursor"
    pub fn stream_append_cursor(&self) -> StoreKey {
        StoreKey(format!(
            "{}/{}/append_cursor",
            STREAM_APPEND_CURSOR_PREFIX,
            self.path()
        ))
    }

    /// e.g. "es/wf_xyz/v_abc/r_xyz/snapshot"
    pub fn engine_snapshot(&self) -> StoreKey {
        StoreKey(format!(
            "{}/{}/snapshot",
            ENGINE_SNAPSHOT_PREFIX,
            self.path()
        ))
    }
}
