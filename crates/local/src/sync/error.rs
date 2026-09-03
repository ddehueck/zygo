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
    #[error("CDC row ID `{id}` for a tag association is not an integer")]
    InvalidTagId { id: String },
}

pub type Result<T> = std::result::Result<T, Error>;
