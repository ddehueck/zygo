use turso::{Value, params};

use super::Db;
use super::db_models::DataReferenceRow;
use super::error::Result;

const SELECT_COLUMNS: &str = "
    id,
    workflow_run_id,
    job_run_id,
    job_id,
    uri,
    version,
    is_replay,
    inserted_at,
    created_at
";

#[derive(Clone)]
pub struct DataReferenceRepository {
    database: Db,
}

impl DataReferenceRepository {
    pub fn new(database: Db) -> Self {
        Self { database }
    }

    pub async fn insert(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
        job_id: &str,
        uri: &str,
        version: &str,
        is_replay: bool,
        inserted_at: &str,
    ) -> Result<()> {
        let connection = self.database.connection.lock().await;
        connection
            .execute(
                "INSERT INTO data_references
                    (workflow_run_id, job_run_id, job_id, uri, version, is_replay, inserted_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(workflow_run_id, job_run_id, uri, version) DO NOTHING",
                params![
                    workflow_run_id,
                    job_run_id,
                    job_id,
                    uri,
                    version,
                    i64::from(is_replay),
                    inserted_at,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<DataReferenceRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!("SELECT {SELECT_COLUMNS} FROM data_references WHERE id = ?1"),
                [id],
            )
            .await?;
        let Some(row) = rows.next().await? else {
            return Ok(None);
        };
        Ok(Some(DataReferenceRow::from_row(&row, &rows)?))
    }

    pub async fn list_by_job_run(
        &self,
        workflow_run_id: &str,
        job_run_id: &str,
    ) -> Result<Vec<DataReferenceRow>> {
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                &format!(
                    "SELECT {SELECT_COLUMNS}
                     FROM data_references
                     WHERE workflow_run_id = ?1 AND job_run_id = ?2
                     ORDER BY inserted_at ASC, id ASC"
                ),
                [workflow_run_id, job_run_id],
            )
            .await?;

        let mut references = Vec::new();
        while let Some(row) = rows.next().await? {
            references.push(DataReferenceRow::from_row(&row, &rows)?);
        }
        Ok(references)
    }

    pub async fn list_by_workflow_run_ids(
        &self,
        workflow_run_ids: &[String],
    ) -> Result<Vec<DataReferenceRow>> {
        if workflow_run_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = (1..=workflow_run_ids.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = workflow_run_ids
            .iter()
            .map(|id| Value::from(id.clone()))
            .collect::<Vec<_>>();
        let connection = self.database.connection.lock().await;
        let mut rows = connection
            .query(
                format!(
                    "SELECT {SELECT_COLUMNS}
                     FROM data_references
                     WHERE workflow_run_id IN ({placeholders})
                     ORDER BY workflow_run_id ASC, inserted_at ASC, id ASC"
                ),
                params,
            )
            .await?;

        let mut references = Vec::new();
        while let Some(row) = rows.next().await? {
            references.push(DataReferenceRow::from_row(&row, &rows)?);
        }
        Ok(references)
    }
}
