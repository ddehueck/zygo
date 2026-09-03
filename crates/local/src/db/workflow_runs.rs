use turso::{params, transaction::TransactionBehavior};

use super::{Db, db_models::WorkflowRunRow, error::Result};

const CREATE_SQL: &str = "
    INSERT INTO workflow_runs (id, workflow_id, content_hash, status)
    VALUES (?1, ?2, ?3, 'running')
    ON CONFLICT(id) DO NOTHING
";

const UPDATE_SQL: &str = "
    UPDATE workflow_runs
    SET
        status = ?2,
        started_at = COALESCE(?3, started_at),
        completed_at = ?4,
        active_job_count = ?5,
        succeeded_job_count = ?6,
        errored_job_count = ?7
    WHERE id = ?1
";

const SELECT_COLUMNS: &str = "
    rowid AS row_id,
    id,
    workflow_id,
    content_hash,
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
pub struct WorkflowRunRepository {
    database: Db,
}

impl WorkflowRunRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn insert(
        &self,
        workflow_run_id: &str,
        workflow_id: &str,
        content_hash: &str,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(CREATE_SQL, [workflow_run_id, workflow_id, content_hash])
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert(
        &self,
        workflow_run_id: &str,
        status: &str,
        started_at: Option<&str>,
        completed_at: Option<&str>,
        active_job_count: Option<i64>,
        succeeded_job_count: Option<i64>,
        errored_job_count: Option<i64>,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            UPDATE_SQL,
            params![
                workflow_run_id,
                status,
                started_at,
                completed_at,
                active_job_count.unwrap_or_default(),
                succeeded_job_count.unwrap_or_default(),
                errored_job_count.unwrap_or_default(),
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<Option<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE id = ?1"),
                [workflow_run_id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(WorkflowRunRow::from_row(&row, &rows)?))
    }

    pub async fn get_by_id(&self, workflow_run_id: &str) -> Result<Option<WorkflowRunRow>> {
        self.get_by_workflow_run_id(workflow_run_id).await
    }

    pub async fn list_all(&self) -> Result<Vec<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS} FROM workflow_runs ORDER BY created_at DESC, rowid DESC"
                ),
                (),
            )
            .await?;

        let mut workflow_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            workflow_runs.push(WorkflowRunRow::from_row(&row, &rows)?);
        }

        Ok(workflow_runs)
    }

    /// Lists workflow runs after the supplied workflow-run ID in lexicographic ID order.
    ///
    /// The caller can request one more row than it intends to return to determine
    /// whether another page exists.
    pub async fn list_after_id(
        &self,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<Vec<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let limit = i64::from(limit);
        let mut rows = match cursor {
            Some(cursor) => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE id > ?1 ORDER BY id ASC LIMIT ?2"
                        ),
                        params![cursor, limit],
                    )
                    .await?
            }
            None => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM workflow_runs ORDER BY id ASC LIMIT ?1"
                        ),
                        [limit],
                    )
                    .await?
            }
        };

        let mut workflow_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            workflow_runs.push(WorkflowRunRow::from_row(&row, &rows)?);
        }

        Ok(workflow_runs)
    }

    pub async fn list_by_tag(&self, key: &str, value: &str) -> Result<Vec<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "
                        SELECT {SELECT_COLUMNS}
                        FROM workflow_runs
                        INNER JOIN tag_associations
                            ON tag_associations.workflow_run_id = workflow_runs.id
                        INNER JOIN tags ON tags.id = tag_associations.tag_id
                        WHERE tags.name = ?1 AND tag_associations.value = ?2
                        ORDER BY workflow_runs.created_at ASC, workflow_runs.rowid ASC
                    "
                ),
                [key, value],
            )
            .await?;

        let mut workflow_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            workflow_runs.push(WorkflowRunRow::from_row(&row, &rows)?);
        }

        Ok(workflow_runs)
    }
}
