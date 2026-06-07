use sqlx::FromRow;

/// Database row representation of the `workflows` table.
#[derive(Debug, Clone, FromRow)]
pub struct WorkflowRow {
    pub id: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}
