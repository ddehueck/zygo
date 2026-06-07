use sqlx::FromRow;

/// Database row representation of the `jobs` table.
#[derive(Debug, Clone, FromRow)]
pub struct JobRow {
    pub id: String,
    pub workflow_version_id: String,
    pub content_hash: String,
    pub name: String,
    pub entrypoint_json: String,
    pub created_at: chrono::NaiveDateTime,
}

