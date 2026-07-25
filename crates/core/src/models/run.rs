use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRunStatus {
    Running,
    Succeeded,
    Failed,
}

impl WorkflowRunStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowRunStatus::Succeeded | WorkflowRunStatus::Failed
        )
    }
}
