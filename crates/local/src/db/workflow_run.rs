use std::sync::Arc;

use tokio::sync::Mutex;
use turso::{Connection, transaction::TransactionBehavior};

use super::db_models::{Tag, WorkflowRun};

const INSERT_WORKFLOW_RUN_SQL: &str = "
    INSERT INTO workflow_runs (id, content_hash)
    VALUES (?1, ?2)
    ON CONFLICT(id) DO NOTHING
";

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

pub struct WorkflowRunRepository {
    connection: Arc<Mutex<Connection>>,
}

impl WorkflowRunRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub async fn insert(
        &self,
        id: &str,
        content_hash: &str,
        tags: &[(&str, &str)],
    ) -> turso::Result<()> {
        let mut connection = self.connection.lock().await;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .await?;

        tx.execute(INSERT_WORKFLOW_RUN_SQL, [id, content_hash])
            .await?;

        for &(key, value) in tags {
            tx.execute(INSERT_TAG_SQL, [key]).await?;
            tx.execute(INSERT_TAG_ASSOCIATION_SQL, [key, value, id])
                .await?;
        }

        tx.commit().await
    }

    pub async fn get_by_id(&self, id: &str) -> turso::Result<Option<WorkflowRun>> {
        let connection = self.connection.lock().await;
        let mut rows = connection
            .query(
                "SELECT id, content_hash, created_at FROM workflow_runs WHERE id = ?1",
                [id],
            )
            .await?;

        let Some(row) = rows.next().await? else {
            return Ok(None);
        };

        Ok(Some(WorkflowRun::from_row(&row, &rows)?))
    }

    pub async fn list_all(&self) -> turso::Result<Vec<WorkflowRun>> {
        let connection = self.connection.lock().await;
        let mut rows = connection
            .query(
                "
                    SELECT id, content_hash, created_at
                    FROM workflow_runs
                    ORDER BY created_at ASC, rowid ASC
                ",
                (),
            )
            .await?;

        Self::collect_runs(&mut rows).await
    }

    pub async fn list_by_tag(&self, key: &str, value: &str) -> turso::Result<Vec<WorkflowRun>> {
        let connection = self.connection.lock().await;
        let mut rows = connection
            .query(
                "
                    SELECT workflow_runs.id, workflow_runs.content_hash, workflow_runs.created_at
                    FROM tags
                    INNER JOIN tag_associations ON tag_associations.tag_id = tags.id
                    INNER JOIN workflow_runs
                        ON workflow_runs.id = tag_associations.workflow_run_id
                    WHERE tags.name = ?1 AND tag_associations.value = ?2
                    ORDER BY workflow_runs.created_at ASC, workflow_runs.rowid ASC
                ",
                [key, value],
            )
            .await?;

        Self::collect_runs(&mut rows).await
    }

    pub async fn list_tags(&self, workflow_run_id: &str) -> turso::Result<Vec<Tag>> {
        let connection = self.connection.lock().await;
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
            tags.push(Tag::from_row(&row, &rows)?);
        }

        Ok(tags)
    }

    async fn collect_runs(rows: &mut turso::Rows) -> turso::Result<Vec<WorkflowRun>> {
        let mut workflow_runs = Vec::new();
        while let Some(row) = rows.next().await? {
            workflow_runs.push(WorkflowRun::from_row(&row, rows)?);
        }

        Ok(workflow_runs)
    }
}
