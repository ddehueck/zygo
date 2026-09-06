use turso::{Value, params};

use super::paginator::{Cursor, CursorPaginator, Page};
use crate::DbResult;
use crate::db::Db;
use crate::db::db_models::DataReferenceRow;
use crate::db::error::Result;

const SELECT_COLUMNS: &str = "
    id,
    workflow_run_id,
    job_run_id,
    uri,
    is_replay,
    created_at
";

#[derive(Clone)]
pub struct DataReferenceRepository {
    database: Db,
}

impl CursorPaginator for DataReferenceRepository {
    type Item = DataReferenceRow;

    async fn list(&self, cursor: Option<Cursor>, limit: i64) -> DbResult<Page<Self::Item>> {
        let connection = self.database.connection.lock().await;
        let mut rows = match cursor {
            Some(cursor) => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM data_references WHERE id < ?1 ORDER BY id DESC LIMIT ?2"),
                    [turso::Value::from(cursor.id), turso::Value::from(limit + 1)],
                )
                .await?,
            None => connection
                .query(
                    &format!("SELECT {SELECT_COLUMNS} FROM data_references ORDER BY id DESC LIMIT ?1"),
                    [limit + 1],
                )
                .await?,
        };
        let mut data = read_rows(&mut rows).await?;
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
        connection.execute("INSERT INTO data_references (workflow_run_id, job_run_id, job_id, uri, version, is_replay, inserted_at) SELECT workflow_runs.id, job_runs.id, ?3, ?4, ?5, ?6, ?7 FROM workflow_runs JOIN job_runs ON job_runs.workflow_run_id = workflow_runs.id WHERE workflow_runs.public_id = ?1 AND job_runs.public_id = ?2 ON CONFLICT(workflow_run_id, job_run_id, uri, version) DO NOTHING", params![workflow_run_id, job_run_id, job_id, uri, version, i64::from(is_replay), inserted_at]).await?;
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
        let mut rows = connection.query(&format!("SELECT {SELECT_COLUMNS} FROM data_references WHERE workflow_run_id = (SELECT id FROM workflow_runs WHERE public_id = ?1) AND job_run_id = (SELECT id FROM job_runs WHERE public_id = ?2) ORDER BY created_at ASC, id ASC"), [workflow_run_id, job_run_id]).await?;
        read_rows(&mut rows).await
    }

    pub async fn list_by_workflow_run_ids(
        &self,
        workflow_run_ids: &[String],
    ) -> Result<Vec<DataReferenceRow>> {
        if workflow_run_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = (1..=workflow_run_ids.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = workflow_run_ids
            .iter()
            .cloned()
            .map(Value::from)
            .collect::<Vec<_>>();
        let connection = self.database.connection.lock().await;
        let mut rows = connection.query(format!("SELECT {SELECT_COLUMNS} FROM data_references WHERE workflow_run_id IN (SELECT id FROM workflow_runs WHERE public_id IN ({placeholders})) ORDER BY workflow_run_id ASC, created_at ASC, id ASC"), params).await?;
        read_rows(&mut rows).await
    }
}

async fn read_rows(rows: &mut turso::Rows) -> Result<Vec<DataReferenceRow>> {
    let mut result = Vec::new();
    while let Some(row) = rows.next().await? {
        result.push(DataReferenceRow::from_row(&row, rows)?);
    }
    Ok(result)
}
