use sqlx::FromRow;

/// Database row representation of the `runs` table.
#[derive(Debug, Clone, FromRow)]
pub struct RunRow {
    pub id: String,
    pub workflow_version_id: String,
    pub created_at: chrono::NaiveDateTime,
}

