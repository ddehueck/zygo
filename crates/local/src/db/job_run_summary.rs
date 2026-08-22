use turso::{params, transaction::TransactionBehavior};

use super::{
    Db,
    db_models::{JobRunSummaryCounts, JobRunSummaryRow},
    error::Result,
};

const UPSERT_SQL: &str = "
    INSERT INTO job_run_summary (
        workflow_run_id,
        job_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    ON CONFLICT(workflow_run_id, job_run_id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = excluded.duration_ms,
        retry_count = excluded.retry_count
";

const SELECT_COLUMNS: &str = "
    workflow_run_id,
    job_run_id,
    job_id,
    status,
    duration_ms,
    retry_count,
    created_at,
    updated_at
";

const RECORD_STARTED_SQL: &str = "
    INSERT INTO job_run_summary (
        workflow_run_id,
        job_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, 'running', NULL, 0)
    ON CONFLICT(workflow_run_id, job_run_id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = NULL
";

const RECORD_COMPLETED_SQL: &str = "
    INSERT INTO job_run_summary (
        workflow_run_id,
        job_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, ?4, ?5, 0)
    ON CONFLICT(workflow_run_id, job_run_id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = COALESCE(excluded.duration_ms, job_run_summary.duration_ms)
";

#[derive(Clone)]
pub struct JobRunSummaryRepository {
    database: Db,
}

impl JobRunSummaryRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, summary: &JobRunSummaryRow) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            UPSERT_SQL,
            params![
                summary.workflow_run_id.as_str(),
                summary.job_run_id.as_str(),
                summary.job_id.as_str(),
                summary.status.as_str(),
                summary.duration_ms,
                summary.retry_count,
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn record_started(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(RECORD_STARTED_SQL, [workflow_run_id, job_run_id, job_id])
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn record_completed(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
        status: &str,
        duration_ms: Option<i64>,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            RECORD_COMPLETED_SQL,
            turso::params![workflow_run_id, job_run_id, job_id, status, duration_ms],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn counts_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<JobRunSummaryCounts> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                "
                    SELECT
                        COALESCE(SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0)
                    FROM job_run_summary
                    WHERE workflow_run_id = ?1
                ",
                [workflow_run_id],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(turso::Error::QueryReturnedNoRows)?;
        Ok(JobRunSummaryCounts {
            active_job_count: row.get(0)?,
            succeeded_job_count: row.get(1)?,
            errored_job_count: row.get(2)?,
        })
    }

    pub async fn get_by_id(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
    ) -> Result<Option<JobRunSummaryRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM job_run_summary WHERE workflow_run_id = ?1 AND job_run_id = ?2"
                ),
                [workflow_run_id, job_run_id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(JobRunSummaryRow::from_row(&row, &rows)?))
    }

    pub async fn list_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<Vec<JobRunSummaryRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM job_run_summary WHERE workflow_run_id = ?1 ORDER BY created_at ASC, rowid ASC"
                ),
                [workflow_run_id],
            )
            .await?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            summaries.push(JobRunSummaryRow::from_row(&row, &rows)?);
        }

        Ok(summaries)
    }
}
