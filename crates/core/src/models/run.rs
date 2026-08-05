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

impl std::fmt::Display for WorkflowRunStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        };

        formatter.write_str(status)
    }
}
