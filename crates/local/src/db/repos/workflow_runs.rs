use turso::{params, transaction::TransactionBehavior};

use super::paginator::{Cursor, CursorPaginator, Page};
use crate::DbResult;
use crate::db::{Db, db_models::WorkflowRunRow, error::Result};

const CREATE_SQL: &str = "
    INSERT INTO workflow_runs (public_id, workflow_id, content_hash)
    VALUES (?1, ?2, ?3)
    ON CONFLICT(public_id) DO NOTHING
";

const UPDATE_SQL: &str = "
    UPDATE workflow_runs
    SET status = ?2, started_at = COALESCE(?3, started_at), completed_at = ?4,
        active_job_count = ?5, succeeded_job_count = ?6, errored_job_count = ?7
    WHERE public_id = ?1
";

const SELECT_COLUMNS: &str = "
    id, public_id, workflow_id, content_hash, status,
    started_at, completed_at, active_job_count, succeeded_job_count,
    errored_job_count, created_at, updated_at
";

#[derive(Clone)]
pub struct WorkflowRunRepository {
    database: Db,
}

impl CursorPaginator for WorkflowRunRepository {
    type Item = WorkflowRunRow;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>> {
        let connection = self.database.connection.lock().await;
        let mut rows = match cursor {
            Some(cursor) => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE id < ?1 ORDER BY id DESC LIMIT ?2"),
                    [turso::Value::from(cursor.id), turso::Value::from(limit + 1)],
                )
                .await?,
            None => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM workflow_runs ORDER BY id DESC LIMIT ?1"),
                    [limit + 1],
                )
                .await?,
        };
        let mut data = Vec::new();
        while let Some(row) = rows.next().await? {
            data.push(WorkflowRunRow::from_row(&row, &rows)?);
        }
        let next = (limit > 0 && data.len() > limit as usize).then(|| {
            let next_id = data[..limit as usize]
                .iter()
                .map(|row| row.id)
                .min()
                .expect("page is non-empty when limit is positive");
            data.truncate(limit as usize);
            Cursor { id: next_id }
        });
        Ok(Page { next, data })
    }
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
                errored_job_count.unwrap_or_default()
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
                &format!("SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE public_id = ?1"),
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
                    "SELECT {SELECT_COLUMNS} FROM workflow_runs ORDER BY created_at DESC, id DESC"
                ),
                (),
            )
            .await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(WorkflowRunRow::from_row(&row, &rows)?);
        }
        Ok(result)
    }

    pub async fn list_after_id(
        &self,
        cursor: Option<&str>,
        limit: u32,
    ) -> Result<Vec<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let limit = i64::from(limit);
        let mut rows = match cursor {
            Some(cursor) => connection.query(&format!("SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE public_id > ?1 ORDER BY public_id ASC LIMIT ?2"), params![cursor, limit]).await?,
            None => connection.query(&format!("SELECT {SELECT_COLUMNS} FROM workflow_runs ORDER BY public_id ASC LIMIT ?1"), [limit]).await?,
        };
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(WorkflowRunRow::from_row(&row, &rows)?);
        }
        Ok(result)
    }

    pub async fn list_by_tag(&self, _key: &str, value: &str) -> Result<Vec<WorkflowRunRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query(&format!("SELECT {SELECT_COLUMNS} FROM workflow_runs WHERE id IN (SELECT workflow_run_id FROM tags WHERE value = ?1) ORDER BY created_at ASC, id ASC"), [value]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(WorkflowRunRow::from_row(&row, &rows)?);
        }
        Ok(result)
    }
}
