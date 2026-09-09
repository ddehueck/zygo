#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Turso(#[from] turso::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("invalid change type: {0}")]
    InvalidChangeType(i64),
}

pub type Result<T> = std::result::Result<T, Error>;
