use turso::params;
use turso::transaction::TransactionBehavior;

use crate::db::{Db, DbResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub id: i64,
    pub workflow_run_id: i64,
    pub job_run_id: String,
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
            for content in lines {
                tx.execute(
                    "INSERT INTO logs (workflow_run_id, job_run_id, content) VALUES (?1, ?2, ?3)",
                    params![workflow_run_id, job_run_id, *content],
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

    pub async fn page_by_workflow_run_id(
        &self,
        workflow_run_id: i64,
        after_id: Option<i64>,
        before_id: Option<i64>,
        ascending: bool,
        limit: u32,
        search: Option<&str>,
        job_run_id: Option<i64>,
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
        let global_watermark_id: i64 = watermark_row.get(0)?;
        drop(watermark_rows);
        let direction = if ascending { "ASC" } else { "DESC" };
        let lower_bound = after_id.unwrap_or(0);
        let upper_bound = before_id.map(|id| id.saturating_sub(1)).unwrap_or(i64::MAX);
        let search = search.map(str::trim).filter(|query| !query.is_empty());
        let select = "SELECT logs.id, ?1, logs.job_run_id, logs.content, logs.created_at FROM logs WHERE logs.workflow_run_id = (SELECT public_id FROM workflow_runs WHERE id = ?1) AND logs.id > ?2 AND logs.id <= ?3";
        let mut result = Vec::new();
        match (search, job_run_id) {
            (Some(query), Some(job_run_id)) => {
                let sql = format!(
                    "{select} AND fts_match(logs.content, ?5) AND logs.job_run_id = (SELECT public_id FROM job_runs WHERE id = ?6) ORDER BY logs.id {direction} LIMIT ?4"
                );
                let mut rows = tx
                    .query(
                        &sql,
                        params![
                            workflow_run_id,
                            lower_bound,
                            upper_bound,
                            i64::from(limit),
                            query,
                            job_run_id
                        ],
                    )
                    .await?;
                while let Some(row) = rows.next().await? {
                    result.push(log_row_from_query_row(&row)?);
                }
                drop(rows);
            }
            (Some(query), None) => {
                let sql = format!(
                    "{select} AND fts_match(logs.content, ?5) ORDER BY logs.id {direction} LIMIT ?4"
                );
                let mut rows = tx
                    .query(
                        &sql,
                        params![
                            workflow_run_id,
                            lower_bound,
                            upper_bound,
                            i64::from(limit),
                            query
                        ],
                    )
                    .await?;
                while let Some(row) = rows.next().await? {
                    result.push(log_row_from_query_row(&row)?);
                }
                drop(rows);
            }
            (None, Some(job_run_id)) => {
                let sql = format!(
                    "{select} AND logs.job_run_id = (SELECT public_id FROM job_runs WHERE id = ?5) ORDER BY logs.id {direction} LIMIT ?4"
                );
                let mut rows = tx
                    .query(
                        &sql,
                        params![
                            workflow_run_id,
                            lower_bound,
                            upper_bound,
                            i64::from(limit),
                            job_run_id
                        ],
                    )
                    .await?;
                while let Some(row) = rows.next().await? {
                    result.push(log_row_from_query_row(&row)?);
                }
                drop(rows);
            }
            (None, None) => {
                let sql = format!("{select} ORDER BY logs.id {direction} LIMIT ?4");
                let mut rows = tx
                    .query(
                        &sql,
                        params![workflow_run_id, lower_bound, upper_bound, i64::from(limit)],
                    )
                    .await?;
                while let Some(row) = rows.next().await? {
                    result.push(log_row_from_query_row(&row)?);
                }
                drop(rows);
            }
        }
        tx.commit().await?;
        Ok((result, global_watermark_id))
    }

    pub async fn list_after_by_id(
        &self,
        job_run_id: i64,
        after_id: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, COALESCE((SELECT id FROM workflow_runs WHERE public_id = logs.workflow_run_id), 0), logs.job_run_id, logs.content, logs.created_at FROM logs WHERE logs.job_run_id = (SELECT public_id FROM job_runs WHERE id = ?1) AND logs.id > ?2 ORDER BY logs.id ASC LIMIT ?3", params![job_run_id, after_id, i64::from(limit)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(log_row_from_query_row(&row)?);
        }
        Ok(result)
    }

    pub async fn list_after(
        &self,
        job_run_id: &str,
        after_id: i64,
        limit: u32,
    ) -> DbResult<Vec<LogRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query("SELECT logs.id, COALESCE((SELECT id FROM workflow_runs WHERE public_id = logs.workflow_run_id), 0), logs.job_run_id, logs.content, logs.created_at FROM logs WHERE logs.job_run_id = ?1 AND logs.id > ?2 ORDER BY logs.id ASC LIMIT ?3", params![job_run_id, after_id, i64::from(limit)]).await?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().await? {
            result.push(log_row_from_query_row(&row)?);
        }
        Ok(result)
    }
}

fn log_row_from_query_row(row: &turso::Row) -> DbResult<LogRow> {
    Ok(LogRow {
        id: row.get(0)?,
        workflow_run_id: row.get(1)?,
        job_run_id: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
    })
}
