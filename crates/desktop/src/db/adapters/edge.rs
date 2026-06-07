use crate::db::models::JobChannelEdgeRow;
use crate::models::{ChannelId, DomainError, Edge, EdgeKind, JobId, WorkflowVersionId};

impl TryFrom<JobChannelEdgeRow> for Edge {
    type Error = DomainError;

    fn try_from(row: JobChannelEdgeRow) -> Result<Self, Self::Error> {
        let kind = match row.kind.as_str() {
            "input" => EdgeKind::Input,
            "output" => EdgeKind::Output,
            other => {
                return Err(DomainError::invalid(
                    "kind",
                    &format!("unknown edge kind: {}", other),
                ));
            }
        };

        Ok(Self::new(
            WorkflowVersionId::try_from(row.workflow_version_id)?,
            JobId::try_from(row.job_id)?,
            ChannelId::try_from(row.channel_id)?,
            kind,
        ))
    }
}

impl From<Edge> for JobChannelEdgeRow {
    fn from(edge: Edge) -> Self {
        let kind = match edge.kind {
            EdgeKind::Input => "input".to_string(),
            EdgeKind::Output => "output".to_string(),
        };

        Self {
            workflow_version_id: edge.workflow_version_id.into(),
            job_id: edge.job_id.into(),
            channel_id: edge.channel_id.into(),
            kind,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_row_to_domain_input() {
        let row = JobChannelEdgeRow {
            workflow_version_id: "wv_123".to_string(),
            job_id: "job_456".to_string(),
            channel_id: "ch_789".to_string(),
            kind: "input".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let edge = Edge::try_from(row).unwrap();
        assert_eq!(edge.workflow_version_id.as_ref(), "wv_123");
        assert_eq!(edge.job_id.as_ref(), "job_456");
        assert_eq!(edge.channel_id.as_ref(), "ch_789");
        assert_eq!(edge.kind, EdgeKind::Input);
    }

    #[test]
    fn edge_row_to_domain_output() {
        let row = JobChannelEdgeRow {
            workflow_version_id: "wv_123".to_string(),
            job_id: "job_456".to_string(),
            channel_id: "ch_789".to_string(),
            kind: "output".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let edge = Edge::try_from(row).unwrap();
        assert_eq!(edge.kind, EdgeKind::Output);
    }

    #[test]
    fn edge_row_to_domain_invalid_kind() {
        let row = JobChannelEdgeRow {
            workflow_version_id: "wv_123".to_string(),
            job_id: "job_456".to_string(),
            channel_id: "ch_789".to_string(),
            kind: "invalid".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let result = Edge::try_from(row);
        assert!(result.is_err());
    }

    #[test]
    fn edge_domain_to_row() {
        let edge = Edge::new(
            WorkflowVersionId::try_from("wv_123".to_string()).unwrap(),
            JobId::try_from("job_456".to_string()).unwrap(),
            ChannelId::try_from("ch_789".to_string()).unwrap(),
            EdgeKind::Input,
        );

        let row = JobChannelEdgeRow::from(edge);
        assert_eq!(row.workflow_version_id, "wv_123");
        assert_eq!(row.job_id, "job_456");
        assert_eq!(row.channel_id, "ch_789");
        assert_eq!(row.kind, "input");
    }
}
