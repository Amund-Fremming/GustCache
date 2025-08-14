use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Write error: {0}")]
    Write(String),

    #[error("Read error: {0}")]
    Read(String),
}
