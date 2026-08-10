use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        let message: String = message.into();
        Self::Other(anyhow::anyhow!(message))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
