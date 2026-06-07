use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct EventRow {
    pub id: String,
    pub workflow_version_id: String,
    pub is_replay: i32,
    pub timestamp: chrono::NaiveDateTime,
    pub workflow_run_id: String,
    pub sequence_number: i64,

    // Source columns
    pub source_type: String,
    pub source_job_id: Option<String>,
    pub source_job_run_id: Option<String>,

    // Event kind
    pub kind: String,

    // For job_requested events
    pub job_id: Option<String>,

    // For channel_item_inserted events
    pub inserted_channel_id: Option<String>,

    // FK to data_references table
    pub data_reference_id: Option<String>,
}
