use turso::{Value, transaction::TransactionBehavior};

use super::Db;
use super::db_models::TagRow;
use super::error::Result;

const INSERT_TAG_SQL: &str = "
    INSERT INTO tags (name)
    VALUES (?1)
    ON CONFLICT(name) DO NOTHING
";

const INSERT_TAG_ASSOCIATION_SQL: &str = "
    INSERT INTO tag_associations (tag_id, value, workflow_run_id)
    SELECT tags.id, ?2, ?3
    FROM tags
    WHERE tags.name = ?1
        AND NOT EXISTS (
            SELECT 1
            FROM tag_associations
            WHERE tag_associations.tag_id = tags.id
                AND tag_associations.value = ?2
                AND tag_associations.workflow_run_id = ?3
        )
";

const SELECT_COLUMNS: &str = "
    tag_associations.id AS id,
    tag_associations.workflow_run_id AS workflow_run_id,
    tags.name AS key,
    tag_associations.value AS value,
    tag_associations.created_at AS created_at
";

#[derive(Clone)]
pub struct TagsRepository {
    database: Db,
}

impl TagsRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn insert(&self, workflow_run_id: &str, key: &str, value: &str) -> Result<()> {
        let mut connection = self.database.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(INSERT_TAG_SQL, [key]).await?;
        tx.execute(INSERT_TAG_ASSOCIATION_SQL, [key, value, workflow_run_id])
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
                        FROM tag_associations
                        INNER JOIN tags ON tags.id = tag_associations.tag_id
                        WHERE tag_associations.workflow_run_id = ?1
                        ORDER BY tags.name ASC, tag_associations.value ASC
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
                     FROM tag_associations
                     INNER JOIN tags ON tags.id = tag_associations.tag_id
                     WHERE tag_associations.id = ?1"
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
                     FROM tag_associations
                     INNER JOIN tags ON tags.id = tag_associations.tag_id
                     WHERE tag_associations.workflow_run_id IN ({placeholders})
                     ORDER BY tag_associations.workflow_run_id ASC, tags.name ASC, tag_associations.value ASC"
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::{Db, TagsRepository};

    #[tokio::test]
    async fn lists_tags_for_loaded_workflow_runs() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "zygo-tags-test-{}-{}.db",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let path = path.to_string_lossy().into_owned();
        let database = Db::open(&path, Duration::from_secs(5), true).await?;
        let repository = TagsRepository::new(database.clone());

        {
            let connection = database.connection.lock().await;
            connection
                .execute(
                    "INSERT INTO workflow_runs (id, workflow_id, content_hash, status)
                     VALUES (?1, ?2, ?3, ?4)",
                    ["run-1", "workflow-1", "hash-1", "running"],
                )
                .await?;
        }

        repository.insert("run-1", "environment", "test").await?;
        let tags = repository
            .list_by_workflow_run_ids(&["run-1".to_owned()])
            .await?;

        assert_eq!(tags.len(), 1);
        assert!(tags[0].id > 0);
        assert_eq!(tags[0].workflow_run_id, "run-1");
        assert_eq!(tags[0].key, "environment");
        assert_eq!(tags[0].value, "test");

        assert!(
            repository
                .list_by_workflow_run_ids(&["run-2".to_owned()])
                .await?
                .is_empty()
        );

        drop(repository);
        drop(database);
        fs::remove_file(path)?;

        Ok(())
    }
}
