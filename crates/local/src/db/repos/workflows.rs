use turso::transaction::TransactionBehavior;

use super::paginator::{Cursor, CursorPaginator, Page};
use crate::DbResult;
use crate::db::{Db, DbResult as Result, WorkflowModel};

const SELECT_COLUMNS: &str = "id, name, path, schema, created_at";

const UPSERT_SQL: &str = "
    INSERT INTO workflows (name, path, schema)
    VALUES (?1, ?2, ?3)
    ON CONFLICT(path) DO UPDATE SET name = excluded.name, schema = excluded.schema
";

#[derive(Clone)]
pub struct WorkflowRepository {
    database: Db,
}

impl CursorPaginator for WorkflowRepository {
    type Item = WorkflowModel;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>> {
        let connection = self.database.connection.lock().await;
        let mut rows = match cursor {
            Some(cursor) => {
                connection
                    .query(
                        &format!(
                            "SELECT {SELECT_COLUMNS} FROM workflows WHERE id < ?1 ORDER BY id DESC LIMIT ?2"
                        ),
                        [turso::Value::from(cursor.id), turso::Value::from(limit + 1)],
                    )
                    .await?
            }
            None => {
                connection
                    .query(
                        &format!("SELECT {SELECT_COLUMNS} FROM workflows ORDER BY id DESC LIMIT ?1"),
                        [limit + 1],
                    )
                    .await?
            }
        };
        let mut data = Vec::new();
        while let Some(row) = rows.next().await? {
            data.push(WorkflowModel::from_row(&row, &rows)?);
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

impl WorkflowRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, name: &str, path: &str, schema: &str) -> Result<WorkflowModel> {
        {
            let mut connection = self.database.connection.lock().await;
            let tx = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .await?;
            tx.execute(UPSERT_SQL, [name, path, schema]).await?;
            tx.commit().await?;
        }

        self.get_by_path(path)
            .await?
            .ok_or(turso::Error::QueryReturnedNoRows.into())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<WorkflowModel>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM workflows WHERE id = ?1"),
                [id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(WorkflowModel::from_row(&row, &rows)?))
    }

    async fn get_by_path(&self, path: &str) -> Result<Option<WorkflowModel>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM workflows WHERE path = ?1"),
                [path],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(WorkflowModel::from_row(&row, &rows)?))
    }
}
