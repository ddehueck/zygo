use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::ids::{RunId, WorkflowVersionId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Running,
    Succeeded,
    Failed,
}

impl RunStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, RunStatus::Succeeded | RunStatus::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub workflow_version_id: WorkflowVersionId,
    pub created_at: SystemTime,
}
