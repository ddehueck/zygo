use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
