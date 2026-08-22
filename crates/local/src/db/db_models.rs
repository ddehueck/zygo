use serde_json::Value;
use turso::{Row, Rows};

use super::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunRow {
    pub id: String,
    pub workflow_id: String,
    pub content_hash: String,
    pub created_at: String,
}

impl WorkflowRunRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            workflow_id: row.get(rows.column_index("workflow_id")?)?,
            content_hash: row.get(rows.column_index("content_hash")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRunSummaryRow {
    pub workflow_run_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub active_job_count: i64,
    pub succeeded_job_count: i64,
    pub errored_job_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl WorkflowRunSummaryRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRunSummaryRow {
    pub workflow_run_id: String,
    pub job_run_id: String,
    pub job_id: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub retry_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl JobRunSummaryRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            job_run_id: row.get(rows.column_index("job_run_id")?)?,
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
    pub workflow_run_id: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
}

impl TagRow {
    pub fn from_row(row: &Row, rows: &Rows) -> Result<Self> {
        Ok(Self {
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
