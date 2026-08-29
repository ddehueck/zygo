use turso::{params, transaction::TransactionBehavior};

use super::{Db, db_models::WorkflowRunSummaryRow, error::Result};

const UPSERT_SQL: &str = "
    INSERT INTO workflow_run_summary (
        workflow_run_id,
        status,
        started_at,
        completed_at,
        active_job_count,
        succeeded_job_count,
        errored_job_count
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    ON CONFLICT(workflow_run_id) DO UPDATE SET
        status = excluded.status,
        started_at = excluded.started_at,
        completed_at = excluded.completed_at,
        active_job_count = excluded.active_job_count,
        succeeded_job_count = excluded.succeeded_job_count,
        errored_job_count = excluded.errored_job_count
";

const SELECT_COLUMNS: &str = "
    workflow_run_id,
    status,
    started_at,
    completed_at,
    active_job_count,
    succeeded_job_count,
    errored_job_count,
    created_at,
    updated_at
";

#[derive(Clone)]
pub struct WorkflowRunSummaryRepository {
    database: Db,
}

impl WorkflowRunSummaryRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, summary: &WorkflowRunSummaryRow) -> Result<()> {
        self.upsert_values(
            &summary.workflow_run_id,
            &summary.status,
            summary.started_at,
            summary.completed_at,
            summary.active_job_count,
            summary.succeeded_job_count,
            summary.errored_job_count,
        )
        .await
    }

    pub async fn upsert_projection(
        &self,
        workflow_run_id: &str,
        status: &str,
        started_at: Option<i64>,
        completed_at: Option<i64>,
        active_job_count: i64,
        succeeded_job_count: i64,
        errored_job_count: i64,
    ) -> Result<()> {
        self.upsert_values(
            workflow_run_id,
            status,
            started_at,
            completed_at,
            active_job_count,
            succeeded_job_count,
            errored_job_count,
        )
        .await
    }

    async fn upsert_values(
        &self,
        workflow_run_id: &str,
        status: &str,
        started_at: Option<i64>,
        completed_at: Option<i64>,
        active_job_count: i64,
        succeeded_job_count: i64,
        errored_job_count: i64,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            UPSERT_SQL,
            params![
                workflow_run_id,
                status,
                started_at,
                completed_at,
                active_job_count,
                succeeded_job_count,
                errored_job_count,
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<Option<WorkflowRunSummaryRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM workflow_run_summary WHERE workflow_run_id = ?1"
                ),
                [workflow_run_id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(WorkflowRunSummaryRow::from_row(&row, &rows)?))
    }

    pub async fn list_all(&self) -> Result<Vec<WorkflowRunSummaryRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM workflow_run_summary ORDER BY created_at ASC, rowid ASC"
                ),
                (),
            )
            .await?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            summaries.push(WorkflowRunSummaryRow::from_row(&row, &rows)?);
        }

        Ok(summaries)
    }

    /// Lists summaries after the supplied workflow-run ID in lexicographic ID order.
    ///
    /// The caller can request one more row than it intends to return to determine
    /// whether another page exists.
    pub async fn list_after_id(
        &self,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<Vec<WorkflowRunSummaryRow>> {
        let connection = self.database.connection.lock().await;
        let limit = i64::from(limit);
        let mut rows = match cursor {
            Some(cursor) => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM workflow_run_summary WHERE workflow_run_id > ?1 ORDER BY workflow_run_id ASC LIMIT ?2"
                        ),
                        turso::params![cursor, limit],
                    )
                    .await?
            }
            None => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM workflow_run_summary ORDER BY workflow_run_id ASC LIMIT ?1"
                        ),
                        [limit],
                    )
                    .await?
            }
        };

        let mut summaries = Vec::new();
        while let Some(row) = rows.next().await? {
            summaries.push(WorkflowRunSummaryRow::from_row(&row, &rows)?);
        }

        Ok(summaries)
    }
}
