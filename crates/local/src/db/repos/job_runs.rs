use turso::transaction::TransactionBehavior;
use turso::{Value, params};

use super::paginator::{Cursor, CursorPaginator, Page};
use crate::DbResult;
use crate::db::Db;
use crate::db::DbResult as Result;
use crate::db::{JobRunModel, WorkflowRunJobCounts};

const SELECT_COLUMNS: &str = "
    id, public_id, workflow_run_id, job_id,
    status, duration_ms, retry_count, created_at
";

#[derive(Clone)]
pub struct JobRunRepository {
    database: Db,
}

impl CursorPaginator for JobRunRepository {
    type Item = JobRunModel;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>> {
        let connection = self.database.connection.lock().await;
        let mut rows = match cursor {
            Some(cursor) => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM job_runs WHERE id < ?1 ORDER BY id DESC LIMIT ?2"),
                    [turso::Value::from(cursor.id), turso::Value::from(limit + 1)],
                )
                .await?,
            None => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM job_runs ORDER BY id DESC LIMIT ?1"),
                    [limit + 1],
                )
                .await?,
        };
        let mut data = Vec::new();
        while let Some(row) = rows.next().await? {
            data.push(JobRunModel::from_row(&row, &rows)?);
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

impl JobRunRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, run: &JobRunModel) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        tx.execute("INSERT INTO job_runs (public_id, workflow_run_id, job_id, status, duration_ms, retry_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6) ON CONFLICT(public_id) DO UPDATE SET job_id = excluded.job_id, status = excluded.status, duration_ms = excluded.duration_ms, retry_count = excluded.retry_count", params![run.public_id.as_str(), run.workflow_run_id, run.job_id.as_str(), run.status.as_str(), run.duration_ms, run.retry_count]).await?;
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
        tx.execute("INSERT INTO job_runs (public_id, workflow_run_id, job_id, status) SELECT ?1, id, ?3, 'running' FROM workflow_runs WHERE public_id = ?2 ON CONFLICT(public_id) DO UPDATE SET job_id = excluded.job_id, status = excluded.status, duration_ms = NULL", [job_run_id, workflow_run_id, job_id]).await?;
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
        tx.execute("INSERT INTO job_runs (public_id, workflow_run_id, job_id, status, duration_ms) SELECT ?1, id, ?3, ?4, ?5 FROM workflow_runs WHERE public_id = ?2 ON CONFLICT(public_id) DO UPDATE SET job_id = excluded.job_id, duration_ms = COALESCE(excluded.duration_ms, job_runs.duration_ms), status = excluded.status", params![job_run_id, workflow_run_id, job_id, status, duration_ms]).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn counts_by_workflow_run_id(
        &self,
        workflow_run_id: &str,
    ) -> Result<WorkflowRunJobCounts> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT COALESCE(SUM(CASE WHEN job_runs.status = 'running' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN job_runs.status = 'succeeded' THEN 1 ELSE 0 END), 0), COALESCE(SUM(CASE WHEN job_runs.status = 'failed' THEN 1 ELSE 0 END), 0) FROM job_runs WHERE workflow_run_id = (SELECT id FROM workflow_runs WHERE public_id = ?1)", [workflow_run_id]).await?;
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

    pub async fn get_by_id(&self, job_run_id: &str) -> Result<Option<JobRunModel>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM job_runs WHERE public_id = ?1"),
                [job_run_id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        Ok(Some(JobRunModel::from_row(&row, &rows)?))
    }

    pub async fn list_by_workflow_run_ids(
        &self,
        workflow_run_ids: &[String],
    ) -> Result<Vec<JobRunModel>> {
        if workflow_run_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = (1..=workflow_run_ids.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = workflow_run_ids
            .iter()
            .cloned()
            .map(Value::from)
            .collect::<Vec<_>>();
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query(format!("SELECT {SELECT_COLUMNS} FROM job_runs WHERE workflow_run_id IN (SELECT id FROM workflow_runs WHERE public_id IN ({placeholders})) ORDER BY workflow_run_id ASC, created_at ASC, id ASC"), params).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(JobRunModel::from_row(&row, &rows)?);
        }
        Ok(result)
    }

    pub async fn list_by_workflow_run_id(&self, workflow_run_id: &str) -> Result<Vec<JobRunModel>> {
        self.list_by_workflow_run_ids(&[workflow_run_id.to_owned()])
            .await
    }
}
