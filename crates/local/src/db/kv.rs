use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use turso::{Connection, transaction::TransactionBehavior};

use super::db_models::Kv;

const UPSERT_SQL: &str = "
    INSERT INTO kv (key, value)
    VALUES (?1, ?2)
    ON CONFLICT(key) DO UPDATE SET value = excluded.value
";

#[derive(Clone)]
pub struct KvRepository {
    connection: Arc<Mutex<Connection>>,
}

impl KvRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub async fn upsert(&self, entry: &Kv) -> Result<()> {
        let value = serde_json::to_string(&entry.value)?;
        let connection = self.connection.lock().await;
        connection
            .execute(UPSERT_SQL, [entry.key.as_str(), value.as_str()])
            .await?;

        Ok(())
    }

    pub async fn upsert_many(&self, entries: &[(&str, &serde_json::Value)]) -> Result<()> {
        let mut connection = self.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        for (key, value) in entries {
            let value = serde_json::to_string(value)?;
            tx.execute(UPSERT_SQL, [*key, value.as_str()]).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_by_key(&self, key: &str) -> Result<Option<Kv>> {
        let connection = self.connection.lock().await;
        let mut rows = connection
            .query(
                "SELECT key, value, created_at, updated_at FROM kv WHERE key = ?1",
                [key],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(Kv::from_row(&row, &rows)?))
    }

    pub async fn list(&self, limit: u32, offset: u32) -> Result<Vec<Kv>> {
        let connection = self.connection.lock().await;
        let mut rows = connection
            .query(
                "
                    SELECT key, value, created_at, updated_at
                    FROM kv
                    ORDER BY key
                    LIMIT ?1 OFFSET ?2
                ",
                [i64::from(limit), i64::from(offset)],
            )
            .await?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next().await? {
            entries.push(Kv::from_row(&row, &rows)?);
        }

        Ok(entries)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let connection = self.connection.lock().await;
        connection
            .execute("DELETE FROM kv WHERE key = ?1", [key])
            .await?;

        Ok(())
    }
}
