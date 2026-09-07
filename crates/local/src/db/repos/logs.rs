use turso::params;
use turso::transaction::TransactionBehavior;

use crate::db::{Db, DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub id: i64,
    pub workflow_run_id: i64,
    pub job_run_id: String,
    pub order: i64,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct LogsRepository {
    database: Db,
}

impl LogsRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn append(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
        lines: &[&str],
    ) -> DbResult<()> {
        if lines.is_empty() {
            return Ok(());
        }
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        let result: DbResult<()> = async {
            tx.execute("INSERT INTO job_runs (public_id, workflow_run_id, job_id, status) SELECT ?1, workflow_runs.id, ?3, 'running' FROM workflow_runs WHERE workflow_runs.public_id = ?2 ON CONFLICT(public_id) DO NOTHING", params![job_run_id, workflow_run_id, job_id]).await?;
            let mut rows = tx.query("SELECT COALESCE(MAX(\"order\"), 0) FROM logs WHERE job_run_id = (SELECT id FROM job_runs WHERE public_id = ?1)", [job_run_id]).await?;
            let row = rows.next().await?.ok_or(turso::Error::QueryReturnedNoRows)?;
            let mut order: i64 = row.get(0)?;
            drop(rows);
            for content in lines {
                order = order.checked_add(1).ok_or(DbError::LogOrderOverflow)?;
                tx.execute("INSERT INTO logs (job_run_id, \"order\", content) SELECT id, ?2, ?3 FROM job_runs WHERE public_id = ?1", params![job_run_id, order, *content]).await?;
            }
            Ok(())
        }.await;
        if let Err(error) = result {
            tx.rollback().await?;
            return Err(error);
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_by_workflow_run_id(
        &self,
        workflow_run_id: i64,
        offset: u32,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, workflow_run_id, job_runs.public_id, logs.\"order\", logs.content, logs.created_at FROM logs JOIN job_runs ON job_runs.id = logs.job_run_id WHERE workflow_run_id = ?1 ORDER BY logs.id DESC LIMIT ?2 OFFSET ?3", params![workflow_run_id, i64::from(limit), i64::from(offset)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            let order: i64 = row.get(3)?;
            result.push(LogRow {
                id: row.get(0)?,
                workflow_run_id: row.get(1)?,
                job_run_id: row.get(2)?,
                order,
                content: row.get(4)?,
                created_at: row.get(5)?,
            });
        }
        Ok(result)
    }

    pub async fn list_after_by_workflow_run_id(
        &self,
        workflow_run_id: i64,
        after_id: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, job_runs.workflow_run_id, job_runs.public_id, logs.\"order\", logs.content, logs.created_at FROM logs JOIN job_runs ON job_runs.id = logs.job_run_id WHERE job_runs.workflow_run_id = ?1 AND logs.id > ?2 ORDER BY logs.id ASC LIMIT ?3", params![workflow_run_id, after_id, i64::from(limit)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(LogRow {
                id: row.get(0)?,
                workflow_run_id: row.get(1)?,
                job_run_id: row.get(2)?,
                order: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            });
        }
        Ok(result)
    }

    pub async fn list_after_by_id(
        &self,
        job_run_id: i64,
        after_order: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, job_runs.workflow_run_id, job_runs.public_id, logs.\"order\", logs.content, logs.created_at FROM logs JOIN job_runs ON job_runs.id = logs.job_run_id WHERE logs.job_run_id = ?1 AND logs.\"order\" > ?2 ORDER BY logs.\"order\" ASC LIMIT ?3", params![job_run_id, after_order, i64::from(limit)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            let order: i64 = row.get(3)?;
            result.push(LogRow {
                id: row.get(0)?,
                workflow_run_id: row.get(1)?,
                job_run_id: row.get(2)?,
                order,
                content: row.get(4)?,
                created_at: row.get(5)?,
            });
        }
        Ok(result)
    }

    pub async fn list_after(
        &self,
        job_run_id: &str,
        after_order: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, job_runs.workflow_run_id, job_runs.public_id, logs.\"order\", logs.content, logs.created_at FROM logs JOIN job_runs ON job_runs.id = logs.job_run_id WHERE job_runs.public_id = ?1 AND logs.\"order\" > ?2 ORDER BY logs.\"order\" ASC LIMIT ?3", params![job_run_id, after_order, i64::from(limit)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(LogRow {
                id: row.get(0)?,
                workflow_run_id: row.get(1)?,
                job_run_id: row.get(2)?,
                order: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
            });
        }
        Ok(result)
    }
}
