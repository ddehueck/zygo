use sqlx::FromRow;

/// Database row representation of the `data_references` table.
#[derive(Debug, Clone, FromRow)]
pub struct DataReferenceRow {
    pub id: String,
    pub uri: String,
    pub etag: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: chrono::NaiveDateTime,
}
