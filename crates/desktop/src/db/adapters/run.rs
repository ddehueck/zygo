use crate::db::models::RunRow;
use crate::models::run::Run;
use crate::models::{DomainError, RunId, WorkflowVersionId};

use super::datetime::{naive_datetime_to_system_time, system_time_to_naive_datetime};

impl TryFrom<RunRow> for Run {
    type Error = DomainError;

    fn try_from(row: RunRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: RunId::try_from(row.id)?,
            workflow_version_id: WorkflowVersionId::try_from(row.workflow_version_id)?,
            created_at: naive_datetime_to_system_time(row.created_at),
        })
    }
}

impl From<Run> for RunRow {
    fn from(run: Run) -> Self {
        Self {
            id: run.id.into(),
            workflow_version_id: run.workflow_version_id.into(),
            created_at: system_time_to_naive_datetime(run.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn run_row_to_domain() {
        let now = chrono::Utc::now().naive_utc();
        let row = RunRow {
            id: "run_123".to_string(),
            workflow_version_id: "wv_456".to_string(),
            created_at: now,
        };

        let run = Run::try_from(row).unwrap();
        assert_eq!(run.id.as_ref(), "run_123");
        assert_eq!(run.workflow_version_id.as_ref(), "wv_456");
    }

    #[test]
    fn run_domain_to_row() {
        let run = Run {
            id: RunId::try_from("run_789".to_string()).unwrap(),
            workflow_version_id: WorkflowVersionId::try_from("wv_012".to_string()).unwrap(),
            created_at: SystemTime::now(),
        };

        let row = RunRow::from(run);
        assert_eq!(row.id, "run_789");
        assert_eq!(row.workflow_version_id, "wv_012");
    }
}

