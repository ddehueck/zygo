use crate::CdcChangeType;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::DbError),
    #[error("CDC row {change_id} for table `{table_name}` is missing its after state")]
    MissingAfter {
        change_id: i64,
        change_type: CdcChangeType,
        table_name: String,
    },
    #[error("CDC table `{0}` is not supported by local sync")]
    UnsupportedTable(String),
    #[error("CDC row {change_id} for table `{table_name}` has a non-integer row ID")]
    InvalidRowId { change_id: i64, table_name: String },
    #[error("CDC row ID `{id}` for a tag association is not an integer")]
    InvalidTagId { id: String },
    #[error("CDC row ID `{id}` for a data reference is not an integer")]
    InvalidDataReferenceId { id: String },
    #[error("failed to serialize a synchronized row: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
