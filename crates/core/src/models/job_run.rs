use serde::{Deserialize, Serialize};

use super::ids::{JobId, JobRunId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRun {
    pub id: JobRunId,
    pub job_id: JobId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobRunStatus {
    Running,
    Succeeded,
    Failed,
}

impl JobRunStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobRunStatus::Succeeded | JobRunStatus::Failed)
    }
}
