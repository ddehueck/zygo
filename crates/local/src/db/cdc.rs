use super::{Db, db_models::CdcRow, error::Result};

const SELECT_COLUMNS: &str = "
    change_id,
    change_time,
    change_txn_id,
    change_type,
    table_name,
    id,
    bin_record_json_object(
        table_columns_json_array(table_name),
        after
    ) AS after
";

/// Reads rows from Turso's built-in change data capture table.
#[derive(Clone)]
pub struct CdcRepository {
    database: Db,
}

impl CdcRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    /// Lists changes after `after_change_id` and through `through_change_id`.
    ///
    /// The lower bound is exclusive because it represents the caller's last
    /// confirmed high-water mark. The upper bound is inclusive so callers can
    /// use a change ID observed before fetching the batch as a stable boundary.
    pub async fn list_between(
        &self,
        after_change_id: i64,
        through_change_id: i64,
    ) -> Result<Vec<CdcRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS}
                     FROM turso_cdc
                     WHERE change_id > ?1 AND change_id <= ?2
                       AND change_type != 2
                     ORDER BY change_id ASC"
                ),
                [after_change_id, through_change_id],
            )
            .await?;

        let mut changes = Vec::new();
        while let Some(row) = rows.next().await? {
            changes.push(CdcRow::from_row(&row, &rows)?);
        }

        Ok(changes)
    }

    pub async fn has_changes_after(&self, change_id: i64, table_names: &[String]) -> Result<bool> {
        if table_names.is_empty() {
            return Ok(false);
        }

        let table_placeholders = (2..=table_names.len() + 1)
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT 1
             FROM turso_cdc
             WHERE change_id > ?1
               AND change_type != 2
               AND table_name IN ({table_placeholders})
             LIMIT 1"
        );

        let mut params = Vec::with_capacity(table_names.len() + 1);
        params.push(turso::Value::from(change_id));
        params.extend(table_names.iter().cloned().map(turso::Value::from));

        let connection = self.database.connection.lock().await;
        let mut rows = connection.query(sql, params).await?;
        Ok(rows.next().await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use serde_json::json;

    use super::{CdcRepository, Db};

    #[tokio::test]
    async fn list_between_decodes_after_as_a_row_object() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "zygo-cdc-test-{}-{}.db",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let path = path.to_string_lossy().into_owned();
        let database = Db::open(&path, Duration::from_secs(5), true).await?;
        let repository = CdcRepository::new(database.clone());

        let connection = database.connection.lock().await;
        let mut rows = connection
            .query("SELECT COALESCE(MAX(change_id), 0) FROM turso_cdc", ())
            .await?;
        let row = rows.next().await?.expect("max change ID row");
        let before_change_id: i64 = row.get(0)?;

        connection
            .execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)",
                ["cdc-test", "value"],
            )
            .await?;
        drop(connection);

        let changes = repository.list_between(before_change_id, i64::MAX).await?;
        let change = changes
            .iter()
            .find(|change| change.table_name == "kv")
            .expect("the kv insert should be captured");

        assert_eq!(
            change.after.as_ref().and_then(|row| row.get("key")),
            Some(&json!("cdc-test"))
        );
        assert_eq!(
            change.after.as_ref().and_then(|row| row.get("value")),
            Some(&json!("value"))
        );

        drop(repository);
        drop(database);
        fs::remove_file(path)?;

        Ok(())
    }
}
