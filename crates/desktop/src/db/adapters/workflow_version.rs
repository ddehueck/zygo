use crate::db::models::WorkflowVersionRow;
use crate::models::{ContentHash, DomainError, WorkflowId, WorkflowVersion, WorkflowVersionId};

impl TryFrom<WorkflowVersionRow> for WorkflowVersion {
    type Error = DomainError;

    fn try_from(row: WorkflowVersionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: WorkflowVersionId::try_from(row.id)?,
            workflow_id: WorkflowId::try_from(row.workflow_id)?,
            content_hash: ContentHash::try_from(row.content_hash)?,
        })
    }
}

impl From<WorkflowVersion> for WorkflowVersionRow {
    fn from(version: WorkflowVersion) -> Self {
        Self {
            id: version.id.into(),
            workflow_id: version.workflow_id.into(),
            content_hash: version.content_hash.into(),
            entrypoint_json: String::new(), // Entrypoint is not in the domain model
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_version_row_to_domain() {
        let row = WorkflowVersionRow {
            id: "wv_123".to_string(),
            workflow_id: "wf_456".to_string(),
            content_hash: "abc123".to_string(),
            entrypoint_json: "{}".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let version = WorkflowVersion::try_from(row).unwrap();
        assert_eq!(version.id.as_ref(), "wv_123");
        assert_eq!(version.workflow_id.as_ref(), "wf_456");
        assert_eq!(version.content_hash.as_ref(), "abc123");
    }

    #[test]
    fn workflow_version_domain_to_row() {
        let version = WorkflowVersion {
            id: WorkflowVersionId::try_from("wv_789".to_string()).unwrap(),
            workflow_id: WorkflowId::try_from("wf_012".to_string()).unwrap(),
            content_hash: ContentHash::try_from("def456".to_string()).unwrap(),
        };

        let row = WorkflowVersionRow::from(version);
        assert_eq!(row.id, "wv_789");
        assert_eq!(row.workflow_id, "wf_012");
        assert_eq!(row.content_hash, "def456");
    }
}

