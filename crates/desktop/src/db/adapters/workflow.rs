use crate::db::models::WorkflowRow;
use crate::models::{DomainError, Workflow, WorkflowId, WorkflowName};

impl TryFrom<WorkflowRow> for Workflow {
    type Error = DomainError;

    fn try_from(row: WorkflowRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: WorkflowId::try_from(row.id)?,
            name: WorkflowName::try_from(row.name)?,
        })
    }
}

impl From<Workflow> for WorkflowRow {
    fn from(workflow: Workflow) -> Self {
        Self {
            id: workflow.id.into(),
            name: workflow.name.into(),
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_row_to_domain() {
        let row = WorkflowRow {
            id: "wf_123".to_string(),
            name: "my-workflow".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let workflow = Workflow::try_from(row).unwrap();
        assert_eq!(workflow.id.as_ref(), "wf_123");
        assert_eq!(workflow.name.as_ref(), "my-workflow");
    }

    #[test]
    fn workflow_domain_to_row() {
        let workflow = Workflow {
            id: WorkflowId::try_from("wf_456".to_string()).unwrap(),
            name: WorkflowName::try_from("test-workflow".to_string()).unwrap(),
        };

        let row = WorkflowRow::from(workflow);
        assert_eq!(row.id, "wf_456");
        assert_eq!(row.name, "test-workflow");
    }
}

