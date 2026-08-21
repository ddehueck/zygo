#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)]
    Turso(#[from] turso::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

pub type DbResult<T> = Result<T, DbError>;
