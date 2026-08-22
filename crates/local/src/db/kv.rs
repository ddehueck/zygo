use serde_json::Value;
use turso::transaction::TransactionBehavior;

use super::{Db, db_models::KvRow, error::Result};

const UPSERT_SQL: &str = "
    INSERT INTO kv (key, value)
    VALUES (?1, ?2)
    ON CONFLICT(key) DO UPDATE SET value = excluded.value
";

#[derive(Clone)]
pub struct KvRepository {
    database: Db,
}

impl KvRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn upsert(&self, entry: &KvRow) -> Result<()> {
        let value = serde_json::to_string(&entry.value)?;
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        tx.execute(UPSERT_SQL, [entry.key.as_str(), value.as_str()])
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_many(&self, entries: &[(&str, &Value)]) -> Result<()> {
        let entries = entries
            .iter()
            .map(|(key, value)| Ok((*key, serde_json::to_string(value)?)))
            .collect::<Result<Vec<_>>>()?;
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        for (key, value) in &entries {
            tx.execute(UPSERT_SQL, [*key, value.as_str()]).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_by_key(&self, key: &str) -> Result<Option<KvRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                "SELECT key, value, created_at, updated_at FROM kv WHERE key = ?1",
                [key],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(KvRow::from_row(&row, &rows)?))
    }

    pub async fn list(&self, limit: u32, offset: u32) -> Result<Vec<KvRow>> {
        let connection = self.database.connection.lock().await;
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
            entries.push(KvRow::from_row(&row, &rows)?);
        }

        Ok(entries)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;
        tx.execute("DELETE FROM kv WHERE key = ?1", [key]).await?;
        tx.commit().await?;
        Ok(())
    }
}
