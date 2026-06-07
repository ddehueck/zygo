use sqlx::FromRow;

/// Database row representation of the `channels` table.
#[derive(Debug, Clone, FromRow)]
pub struct ChannelRow {
    pub id: String,
    pub workflow_version_id: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

