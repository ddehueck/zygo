use turso::transaction::TransactionBehavior;

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
                "
                    SELECT
                        tag_associations.workflow_run_id AS workflow_run_id,
                        tags.name AS key,
                        tag_associations.value AS value,
                        tag_associations.created_at AS created_at
                    FROM tag_associations
                    INNER JOIN tags ON tags.id = tag_associations.tag_id
                    WHERE tag_associations.workflow_run_id = ?1
                    ORDER BY tags.name ASC, tag_associations.value ASC
                ",
                [workflow_run_id],
            )
            .await?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().await? {
            tags.push(TagRow::from_row(&row, &rows)?);
        }

        Ok(tags)
    }
}
