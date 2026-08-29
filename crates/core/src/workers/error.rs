use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("closed")]
    Closed,

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
