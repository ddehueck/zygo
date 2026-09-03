use turso::{params, transaction::TransactionBehavior};

use super::Db;
use super::db_models::{JobRunRow, WorkflowRunJobCounts};
use super::error::Result;

const UPSERT_SQL: &str = "
    INSERT INTO job_runs (
        id,
        workflow_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    ON CONFLICT(id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = excluded.duration_ms,
        retry_count = excluded.retry_count
";

const SELECT_COLUMNS: &str = "
    rowid AS row_id,
    id,
    workflow_run_id,
    job_id,
    status,
    duration_ms,
    retry_count,
    created_at,
    updated_at
";

const RECORD_STARTED_SQL: &str = "
    INSERT INTO job_runs (
        id,
        workflow_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, 'running', NULL, 0)
    ON CONFLICT(id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = NULL
";

const RECORD_COMPLETED_SQL: &str = "
    INSERT INTO job_runs (
        id,
        workflow_run_id,
        job_id,
        status,
        duration_ms,
        retry_count
    )
    VALUES (?1, ?2, ?3, ?4, ?5, 0)
    ON CONFLICT(id) DO UPDATE SET
        job_id = excluded.job_id,
        status = excluded.status,
        duration_ms = COALESCE(excluded.duration_ms, job_runs.duration_ms)
";

#[derive(Clone)]
pub struct JobRunRepository {
    database: Db,
}

impl JobRunRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, run: &JobRunRow) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            UPSERT_SQL,
            params![
                run.id.as_str(),
                run.workflow_run_id.as_str(),
                run.job_id.as_str(),
                run.status.as_str(),
                run.duration_ms,
                run.retry_count,
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

        tx.execute(RECORD_STARTED_SQL, [job_run_id, workflow_run_id, job_id])
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
            params![job_run_id, workflow_run_id, job_id, status, duration_ms],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn counts_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<WorkflowRunJobCounts> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                "
                    SELECT
                        COALESCE(SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0)
                    FROM job_runs
                    WHERE workflow_run_id = ?1
                ",
                [workflow_run_id],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .ok_or(turso::Error::QueryReturnedNoRows)?;
        Ok(WorkflowRunJobCounts {
            active_job_count: row.get(0)?,
            succeeded_job_count: row.get(1)?,
            errored_job_count: row.get(2)?,
        })
    }

    pub async fn get_by_id(&self, job_run_id: &str) -> Result<Option<JobRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM job_runs WHERE id = ?1"),
                [job_run_id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(JobRunRow::from_row(&row, &rows)?))
    }

    /// Lists job runs after the supplied job-run ID in lexicographic ID order.
    ///
    /// The caller can request one more row than it intends to return to determine
    /// whether another page exists.
    pub async fn list_after_id(&self, cursor: Option<&str>, limit: u32) -> Result<Vec<JobRunRow>> {
        let connection = self.database.connection.lock().await;
        let limit = i64::from(limit);
        let mut rows = match cursor {
            Some(cursor) => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM job_runs WHERE id > ?1 ORDER BY id ASC LIMIT ?2"
                        ),
                        params![cursor, limit],
                    )
                    .await?
            }
            None => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM job_runs ORDER BY id ASC LIMIT ?1"
                        ),
                        [limit],
                    )
                    .await?
            }
        };

        let mut job_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            job_runs.push(JobRunRow::from_row(&row, &rows)?);
        }

        Ok(job_runs)
    }

    pub async fn list_by_workflow_run_id(&self, workflow_run_id: &str) -> Result<Vec<JobRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM job_runs WHERE workflow_run_id = ?1 ORDER BY created_at ASC, rowid ASC"
                ),
                [workflow_run_id],
            )
            .await?;

        let mut job_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            job_runs.push(JobRunRow::from_row(&row, &rows)?);
        }

        Ok(job_runs)
    }
}
