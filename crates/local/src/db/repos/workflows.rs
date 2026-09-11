use turso::transaction::TransactionBehavior;

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
