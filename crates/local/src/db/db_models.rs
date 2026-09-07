use serde::{Deserialize, Serialize};
use serde_json::Value;
use turso::{Row, Rows, Value as SqlValue};

use super::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum CdcChangeType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdcRow {
    pub change_id: i64,
    pub change_time: i64,
    pub change_txn_id: i64,
    pub change_type: CdcChangeType,
    pub table_name: String,
    pub id: SqlValue,
    /// The row state after the change, decoded from Turso's CDC binary record.
    /// This is `None` for deletes because CDC only captures the post-change
    /// state when the row still exists.
    pub after: Option<Value>,
}

impl CdcRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        let change_type = row.get(rows.column_index("change_type")?)?;
        let change_type = match change_type {
            1 => CdcChangeType::Insert,
            0 => CdcChangeType::Update,
            -1 => CdcChangeType::Delete,
            _ => return Err(Error::InvalidChangeType(change_type)),
        };

        let after: Option<String> = row.get(rows.column_index("after")?)?;
        let after = after
            .map(|after| serde_json::from_str(&after))
            .transpose()?;

        Ok(Self {
            change_id: row.get(rows.column_index("change_id")?)?,
            change_time: row.get(rows.column_index("change_time")?)?,
            change_txn_id: row.get(rows.column_index("change_txn_id")?)?,
            change_type,
            table_name: row.get(rows.column_index("table_name")?)?,
            id: row.get_value(rows.column_index("id")?)?,
            after,
        })
    }
}

// SQL rows retain literal storage encodings. Both query reads and CDC
// deserialization go through their conversions into persisted models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunModel {
    pub id: i64,
    pub public_id: String,
    pub workflow_id: String,
    pub content_hash: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub active_job_count: i64,
    pub succeeded_job_count: i64,
    pub errored_job_count: i64,
    pub created_at: String,
}

impl WorkflowRunModel {
    pub fn from_sql_value(value: Value) -> Result<Self> {
        let sql_row: WorkflowRunSqlRow = serde_json::from_value(value)?;
        Ok(sql_row.into())
    }

    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(WorkflowRunSqlRow::from_row(row, rows)?.into())
    }
}

#[derive(Deserialize)]
struct WorkflowRunSqlRow {
    id: i64,
    public_id: String,
    workflow_id: String,
    content_hash: String,
    status: String,
    started_at: Option<String>,
    completed_at: Option<String>,
    active_job_count: i64,
    succeeded_job_count: i64,
    errored_job_count: i64,
    created_at: String,
}

impl WorkflowRunSqlRow {
    fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            public_id: row.get(rows.column_index("public_id")?)?,
            workflow_id: row.get(rows.column_index("workflow_id")?)?,
            content_hash: row.get(rows.column_index("content_hash")?)?,
            status: row.get(rows.column_index("status")?)?,
            started_at: row.get(rows.column_index("started_at")?)?,
            completed_at: row.get(rows.column_index("completed_at")?)?,
            active_job_count: row.get(rows.column_index("active_job_count")?)?,
            succeeded_job_count: row.get(rows.column_index("succeeded_job_count")?)?,
            errored_job_count: row.get(rows.column_index("errored_job_count")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

impl From<WorkflowRunSqlRow> for WorkflowRunModel {
    fn from(row: WorkflowRunSqlRow) -> Self {
        Self {
            id: row.id,
            public_id: row.public_id,
            workflow_id: row.workflow_id,
            content_hash: row.content_hash,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            active_job_count: row.active_job_count,
            succeeded_job_count: row.succeeded_job_count,
            errored_job_count: row.errored_job_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRunModel {
    pub id: i64,
    pub public_id: String,
    pub workflow_run_id: i64,
    pub job_id: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub created_at: String,
}

impl JobRunModel {
    pub fn from_sql_value(value: Value) -> Result<Self> {
        let sql_row: JobRunSqlRow = serde_json::from_value(value)?;
        Ok(sql_row.into())
    }

    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(JobRunSqlRow::from_row(row, rows)?.into())
    }
}

#[derive(Deserialize)]
struct JobRunSqlRow {
    id: i64,
    public_id: String,
    workflow_run_id: i64,
    job_id: String,
    status: String,
    duration_ms: Option<i64>,
    retry_count: i64,
    created_at: String,
}

impl JobRunSqlRow {
    fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            public_id: row.get(rows.column_index("public_id")?)?,
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            job_id: row.get(rows.column_index("job_id")?)?,
            status: row.get(rows.column_index("status")?)?,
            duration_ms: row.get(rows.column_index("duration_ms")?)?,
            retry_count: row.get(rows.column_index("retry_count")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

impl From<JobRunSqlRow> for JobRunModel {
    fn from(row: JobRunSqlRow) -> Self {
        Self {
            id: row.id,
            public_id: row.public_id,
            workflow_run_id: row.workflow_run_id,
            job_id: row.job_id,
            status: row.status,
            duration_ms: row.duration_ms,
            retry_count: row.retry_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagModel {
    pub id: i64,
    pub workflow_run_id: i64,
    pub job_run_id: Option<i64>,
    pub data_reference_id: Option<i64>,
    pub value: String,
    pub created_at: String,
}

impl TagModel {
    pub fn from_sql_value(value: Value) -> Result<Self> {
        let sql_row: TagSqlRow = serde_json::from_value(value)?;
        Ok(sql_row.into())
    }

    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(TagSqlRow::from_row(row, rows)?.into())
    }
}

#[derive(Deserialize)]
struct TagSqlRow {
    id: i64,
    workflow_run_id: i64,
    job_run_id: Option<i64>,
    data_reference_id: Option<i64>,
    value: String,
    created_at: String,
}

impl TagSqlRow {
    fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            job_run_id: row.get(rows.column_index("job_run_id")?)?,
            data_reference_id: row.get(rows.column_index("data_reference_id")?)?,
            value: row.get(rows.column_index("value")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

impl From<TagSqlRow> for TagModel {
    fn from(row: TagSqlRow) -> Self {
        Self {
            id: row.id,
            workflow_run_id: row.workflow_run_id,
            job_run_id: row.job_run_id,
            data_reference_id: row.data_reference_id,
            value: row.value,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataReferenceModel {
    pub id: i64,
    pub workflow_run_id: i64,
    pub job_run_id: i64,
    pub uri: String,
    pub is_replay: bool,
    pub created_at: String,
}

impl DataReferenceModel {
    pub fn from_sql_value(value: Value) -> Result<Self> {
        let sql_row: DataReferenceSqlRow = serde_json::from_value(value)?;
        Ok(sql_row.into())
    }

    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(DataReferenceSqlRow::from_row(row, rows)?.into())
    }
}

#[derive(Deserialize)]
struct DataReferenceSqlRow {
    id: i64,
    workflow_run_id: i64,
    job_run_id: i64,
    uri: String,
    is_replay: i64,
    created_at: String,
}

impl DataReferenceSqlRow {
    fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            job_run_id: row.get(rows.column_index("job_run_id")?)?,
            uri: row.get(rows.column_index("uri")?)?,
            is_replay: row.get(rows.column_index("is_replay")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

impl From<DataReferenceSqlRow> for DataReferenceModel {
    fn from(row: DataReferenceSqlRow) -> Self {
        Self {
            id: row.id,
            workflow_run_id: row.workflow_run_id,
            job_run_id: row.job_run_id,
            uri: row.uri,
            is_replay: row.is_replay != 0,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KvModel {
    pub key: String,
    pub value: Value,
    pub created_at: String,
}

impl KvModel {
    pub fn from_sql_value(value: Value) -> Result<Self> {
        let sql_row: KvSqlRow = serde_json::from_value(value)?;
        sql_row.try_into()
    }

    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        KvSqlRow::from_row(row, rows)?.try_into()
    }
}

#[derive(Deserialize)]
struct KvSqlRow {
    key: String,
    value: String,
    created_at: String,
}

impl KvSqlRow {
    fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            key: row.get(rows.column_index("key")?)?,
            value: row.get(rows.column_index("value")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

impl TryFrom<KvSqlRow> for KvModel {
    type Error = Error;

    fn try_from(row: KvSqlRow) -> Result<Self> {
        Ok(Self {
            key: row.key,
            value: serde_json::from_str(&row.value)?,
            created_at: row.created_at,
        })
    }
}

// TODO: remove this
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkflowRunJobCounts {
    pub active_job_count: i64,
    pub succeeded_job_count: i64,
    pub errored_job_count: i64,
}
