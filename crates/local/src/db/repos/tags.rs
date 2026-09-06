use turso::{Value, transaction::TransactionBehavior};

use super::paginator::{Cursor, CursorPaginator, Page};
use crate::DbResult;
use crate::db::Db;
use crate::db::db_models::TagRow;
use crate::db::error::Result;

const INSERT_TAG_SQL: &str = "
    INSERT INTO tags (value, workflow_run_id, job_run_id, data_reference_id)
    SELECT ?1, workflow_runs.id,
           (SELECT id FROM job_runs WHERE public_id = ?3 AND workflow_run_id = workflow_runs.id), ?4
    FROM workflow_runs
    WHERE workflow_runs.public_id = ?2
    ON CONFLICT(workflow_run_id, job_run_id, data_reference_id, value) DO NOTHING
";

const SELECT_COLUMNS: &str = "
    id, value, workflow_run_id, job_run_id, data_reference_id, created_at
";

#[derive(Clone)]
pub struct TagsRepository {
    database: Db,
}

impl CursorPaginator for TagsRepository {
    type Item = TagRow;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>> {
        let connection = self.database.connection.lock().await;
        let mut rows = match cursor {
            Some(cursor) => connection
                .query(
                    &format!(
                        "SELECT {SELECT_COLUMNS} FROM tags WHERE id < ?1 ORDER BY id DESC LIMIT ?2"
                    ),
                    [turso::Value::from(cursor.id), turso::Value::from(limit + 1)],
                )
                .await?,
            None => {
                connection
                    .query(
                        &format!("SELECT {SELECT_COLUMNS} FROM tags ORDER BY id DESC LIMIT ?1"),
                        [limit + 1],
                    )
                    .await?
            }
        };
        let mut data = Vec::new();
        while let Some(row) = rows.next().await? {
            data.push(TagRow::from_row(&row, &rows)?);
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

impl TagsRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn insert(
        &self,
        value: &str,
        workflow_run_id: &str,
        job_run_id: Option<&str>,
        data_reference_id: Option<i64>,
    ) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            INSERT_TAG_SQL,
            [
                Value::from(value),
                Value::from(workflow_run_id),
                Value::from(job_run_id.map(str::to_owned)),
                Value::from(data_reference_id),
            ],
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self, workflow_run_id: &str) -> Result<Vec<TagRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "
                        SELECT {SELECT_COLUMNS}
                        FROM tags
                        WHERE workflow_run_id = (SELECT id FROM workflow_runs WHERE public_id = ?1)
                        ORDER BY value ASC, id ASC
                    "
                ),
                [workflow_run_id],
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            tags.push(TagRow::from_row(&row, &rows)?);
        }

        Ok(tags)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<TagRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS}
                     FROM tags
                     WHERE id = ?1"
                ),
                [id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(TagRow::from_row(&row, &rows)?))
    }

    pub async fn list_by_workflow_run_ids(
        &self,
        workflow_run_ids: &[String],
    ) -> Result<Vec<TagRow>> {
        if workflow_run_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = (1..=workflow_run_ids.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = workflow_run_ids
            .iter()
            .cloned()
            .map(Value::from)
            .collect::<Vec<_>>();
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                format!(
                    "SELECT {SELECT_COLUMNS}
                     FROM tags
                     WHERE workflow_run_id IN (SELECT id FROM workflow_runs WHERE public_id IN ({placeholders}))
                     ORDER BY workflow_run_id ASC, value ASC, id ASC"
                ),
                params,
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            tags.push(TagRow::from_row(&row, &rows)?);
        }

        Ok(tags)
    }
}
