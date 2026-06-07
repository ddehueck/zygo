use sqlx::FromRow;

/// Database row representation of the `job_channel_edges` table.
#[derive(Debug, Clone, FromRow)]
pub struct JobChannelEdgeRow {
    pub workflow_version_id: String,
    pub job_id: String,
    pub channel_id: String,
    pub kind: String,
    pub created_at: chrono::NaiveDateTime,
}

