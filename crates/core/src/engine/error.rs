use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Worker(#[from] crate::workers::Error),

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
