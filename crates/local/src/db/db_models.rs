use serde_json::Value;
use turso::{Row, Rows};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowRun {
    pub id: String,
    pub content_hash: String,
    pub created_at: String,
}

impl WorkflowRun {
    pub fn from_row(row: &Row, rows: &Rows) -> turso::Result<Self> {
        Ok(Self {
            id: row.get(rows.column_index("id")?)?,
            content_hash: row.get(rows.column_index("content_hash")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub workflow_run_id: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
}

impl Tag {
    pub fn from_row(row: &Row, rows: &Rows) -> turso::Result<Self> {
        Ok(Self {
            workflow_run_id: row.get(rows.column_index("workflow_run_id")?)?,
            key: row.get(rows.column_index("key")?)?,
            value: row.get(rows.column_index("value")?)?,
            created_at: row.get(rows.column_index("created_at")?)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Kv {
    pub key: String,
    pub value: Value,
    pub created_at: String,
    pub updated_at: String,
}

impl Kv {
    pub fn from_row(row: &Row, rows: &Rows) -> anyhow::Result<Self> {
        let value: String = row.get(rows.column_index("value")?)?;

        Ok(Self {
            key: row.get(rows.column_index("key")?)?,
            value: serde_json::from_str(&value)?,
            created_at: row.get(rows.column_index("created_at")?)?,
            updated_at: row.get(rows.column_index("updated_at")?)?,
        })
    }
}
