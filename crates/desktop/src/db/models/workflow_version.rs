use sqlx::FromRow;

/// Database row representation of the `workflow_versions` table.
#[derive(Debug, Clone, FromRow)]
pub struct WorkflowVersionRow {
    pub id: String,
    pub workflow_id: String,
    pub content_hash: String,
    pub entrypoint_json: String,
    pub created_at: chrono::NaiveDateTime,
}
