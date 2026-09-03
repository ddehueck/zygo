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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunRow {
    pub row_id: i64,
    pub id: String,
    pub workflow_id: String,
    pub content_hash: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub active_job_count: i64,
    pub succeeded_job_count: i64,
    pub errored_job_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl WorkflowRunRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            row_id: row.get(rows.column_index("row_id")?)?,
            id: row.get(rows.column_index("id")?)?,
            workflow_id: row.get(rows.column_index("workflow_id")?)?,
            content_hash: row.get(rows.column_index("content_hash")?)?,
            status: row.get(rows.column_index("status")?)?,
            started_at: row.get(rows.column_index("started_at")?)?,
            completed_at: row.get(rows.column_index("completed_at")?)?,
            active_job_count: row.get(rows.column_index("active_job_count")?)?,
            succeeded_job_count: row.get(rows.column_index("succeeded_job_count")?)?,
            errored_job_count: row.get(rows.column_index("errored_job_count")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
            updated_at: row.get(rows.column_index("updated_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobRunRow {
    pub row_id: i64,
    pub id: String,
    pub workflow_run_id: String,
    pub job_id: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl JobRunRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            row_id: row.get(rows.column_index("row_id")?)?,
            id: row.get(rows.column_index("id")?)?,
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            job_id: row.get(rows.column_index("job_id")?)?,
            status: row.get(rows.column_index("status")?)?,
            duration_ms: row.get(rows.column_index("duration_ms")?)?,
            retry_count: row.get(rows.column_index("retry_count")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
            updated_at: row.get(rows.column_index("updated_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRow {
    pub id: i64,
    pub workflow_run_id: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
}

impl TagRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            key: row.get(rows.column_index("key")?)?,
            value: row.get(rows.column_index("value")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KvRow {
    pub key: String,
    pub value: Value,
    pub created_at: String,
    pub updated_at: String,
}

impl KvRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        let value: String = row.get(rows.column_index("value")?)?;

        Ok(Self {
            key: row.get(rows.column_index("key")?)?,
            value: serde_json::from_str(&value).map_err(Error::from)?,
            created_at: row.get(rows.column_index("created_at")?)?,
            updated_at: row.get(rows.column_index("updated_at")?)?,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkflowRunJobCounts {
    pub active_job_count: i64,
    pub succeeded_job_count: i64,
    pub errored_job_count: i64,
}
