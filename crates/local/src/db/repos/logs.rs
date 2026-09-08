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

    pub async fn page_by_workflow_run_id(
        &self,
        workflow_run_id: i64,
        after_id: Option<i64>,
        before_id: Option<i64>,
        ascending: bool,
        limit: u32,
    ) -> DbResult<(Vec<LogRow>, i64)> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Deferred)
            .await?;
        let mut watermark_rows = tx
            .query("SELECT COALESCE(MAX(id), 0) FROM logs", ())
            .await?;
        let watermark_row = watermark_rows
            .next()
            .await?
            .ok_or(turso::Error::QueryReturnedNoRows)?;
        let observed_through_id: i64 = watermark_row.get(0)?;
        drop(watermark_rows);
        let direction = if ascending { "ASC" } else { "DESC" };
        let lower_bound = after_id.unwrap_or(0);
        let upper_bound = before_id.map(|id| id.saturating_sub(1)).unwrap_or(i64::MAX);
        let sql = format!(
            "SELECT logs.id, job_runs.workflow_run_id, job_runs.public_id, logs.\"order\", logs.content, logs.created_at FROM logs JOIN job_runs ON job_runs.id = logs.job_run_id WHERE job_runs.workflow_run_id = ?1 AND logs.id > ?2 AND logs.id <= ?3 ORDER BY logs.id {direction} LIMIT ?4"
        );
        let mut rows = tx
            .query(
                &sql,
                params![workflow_run_id, lower_bound, upper_bound, i64::from(limit)],
            )
            .await?;
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
        drop(rows);
        tx.commit().await?;
        Ok((result, observed_through_id))
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;
    use turso::Builder;

    use super::LogsRepository;
    use crate::db::Db;

    #[tokio::test]
    async fn log_pages_use_exclusive_bounds_and_snapshot_watermark() {
        let database = Builder::new_local(":memory:").build().await.unwrap();
        let connection = database.connect().unwrap();
        connection.execute("CREATE TABLE job_runs (id INTEGER PRIMARY KEY, workflow_run_id INTEGER, public_id TEXT)", ()).await.unwrap();
        connection.execute("CREATE TABLE logs (id INTEGER PRIMARY KEY, job_run_id INTEGER, \"order\" INTEGER, content TEXT, created_at TEXT)", ()).await.unwrap();
        connection
            .execute(
                "INSERT INTO job_runs VALUES (1, 1, 'one'), (2, 2, 'two')",
                (),
            )
            .await
            .unwrap();
        connection.execute("INSERT INTO logs VALUES (2, 1, 1, 'a', 'today'), (4, 1, 2, 'b', 'today'), (6, 1, 3, 'c', 'today'), (8, 1, 4, 'd', 'today'), (10, 2, 1, 'other', 'today')", ()).await.unwrap();
        let repo = LogsRepository::new(Db {
            connection: Arc::new(Mutex::new(connection)),
            is_cdc_enabled: false,
        });
        let (newest, watermark) = repo
            .page_by_workflow_run_id(1, None, None, false, 2)
            .await
            .unwrap();
        assert_eq!(
            newest.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![8, 6]
        );
        assert_eq!(watermark, 10);
        let (bounded, _) = repo
            .page_by_workflow_run_id(1, Some(2), Some(8), true, 3)
            .await
            .unwrap();
        assert_eq!(
            bounded.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![4, 6]
        );
        let (older, _) = repo
            .page_by_workflow_run_id(1, None, Some(6), false, 3)
            .await
            .unwrap();
        assert_eq!(
            older.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![4, 2]
        );
        let (empty, watermark) = repo
            .page_by_workflow_run_id(1, Some(8), None, true, 3)
            .await
            .unwrap();
        assert!(empty.is_empty());
        assert_eq!(watermark, 10);
    }
}
