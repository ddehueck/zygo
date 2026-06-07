use crate::db::models::JobRow;
use crate::models::{ContentHash, DomainError, Job, JobId, JobName, WorkflowVersionId};

impl TryFrom<JobRow> for Job {
    type Error = DomainError;

    fn try_from(row: JobRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: JobId::try_from(row.id)?,
            workflow_version_id: WorkflowVersionId::try_from(row.workflow_version_id)?,
            name: JobName::try_from(row.name)?,
            content_hash: ContentHash::try_from(row.content_hash)?,
        })
    }
}

impl From<Job> for JobRow {
    fn from(job: Job) -> Self {
        Self {
            id: job.id.into(),
            workflow_version_id: job.workflow_version_id.into(),
            name: job.name.into(),
            content_hash: job.content_hash.into(),
            entrypoint_json: String::new(), // Entrypoint is not in the domain model
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_row_to_domain() {
        let row = JobRow {
            id: "job_123".to_string(),
            workflow_version_id: "wv_456".to_string(),
            name: "my-job".to_string(),
            content_hash: "hash123".to_string(),
            entrypoint_json: "{}".to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        let job = Job::try_from(row).unwrap();
        assert_eq!(job.id.as_ref(), "job_123");
        assert_eq!(job.workflow_version_id.as_ref(), "wv_456");
        assert_eq!(job.name.as_ref(), "my-job");
        assert_eq!(job.content_hash.as_ref(), "hash123");
    }

    #[test]
    fn job_domain_to_row() {
        let job = Job {
            id: JobId::try_from("job_789".to_string()).unwrap(),
            workflow_version_id: WorkflowVersionId::try_from("wv_012".to_string()).unwrap(),
            name: JobName::try_from("test-job".to_string()).unwrap(),
            content_hash: ContentHash::try_from("hash456".to_string()).unwrap(),
        };

        let row = JobRow::from(job);
        assert_eq!(row.id, "job_789");
        assert_eq!(row.workflow_version_id, "wv_012");
        assert_eq!(row.name, "test-job");
        assert_eq!(row.content_hash, "hash456");
    }
}
