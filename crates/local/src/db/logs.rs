use turso::{params, transaction::TransactionBehavior};

use super::{Db, DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub job_run_id: String,
    pub order: i64,
    pub content: String,
    pub created_at: String,
}

/// Persists and reads append-only job output in commit order.
#[derive(Clone)]
pub struct LogsRepository {
    database: Db,
}

impl LogsRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    /// Appends a batch atomically, allocating order under the database's write lock.
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
            // todo: avoid this race structurally.
            // Do not overwrite status if the stream processor has already projected this job.
            tx.execute(
                "INSERT INTO job_runs (id, workflow_run_id, job_id, status)
                 VALUES (?1, ?2, ?3, 'running') ON CONFLICT(id) DO NOTHING",
                params![job_run_id, workflow_run_id, job_id],
            )
            .await?;

            let mut rows = tx
                .query(
                    "SELECT COALESCE(MAX(\"order\"), 0) FROM logs WHERE job_run_id = ?1",
                    [job_run_id],
                )
                .await?;
            let row = rows
                .next()
                .await?
                .ok_or(turso::Error::QueryReturnedNoRows)?;
            let mut order: i64 = row.get(0)?;
            drop(rows);

            for content in lines {
                order = order.checked_add(1).ok_or(DbError::LogOrderOverflow)?;
                tx.execute(
                    "INSERT INTO logs (job_run_id, \"order\", content) VALUES (?1, ?2, ?3)",
                    params![job_run_id, order, *content],
                )
                .await?;
            }
            Ok(())
        }
        .await;

        if let Err(error) = result {
            tx.rollback().await?;
            return Err(error);
        }
        tx.commit().await?;
        Ok(())
    }

    /// Use order zero to start reading a job's history.
    pub async fn list_after(
        &self,
        job_run_id: &str,
        after_order: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                "SELECT job_run_id, \"order\", content, created_at
                 FROM logs
                 WHERE job_run_id = ?1 AND \"order\" > ?2
                 ORDER BY \"order\" ASC LIMIT ?3",
                params![job_run_id, after_order, i64::from(limit)],
            )
            .await?;
        read_rows(&mut rows).await
    }
}

async fn read_rows(rows: &mut turso::Rows) -> DbResult<Vec<LogRow>> {
    let mut logs = Vec::new();
    while let Some(row) = rows.next().await? {
        logs.push(LogRow {
            job_run_id: row.get(0)?,
            order: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
        });
    }
    Ok(logs)
}
