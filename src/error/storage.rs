//! Storage Error Module

use thiserror::Error;

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Page not found: {0}")]
    PageNotFound(u64),

    #[error("Corruption: {0}")]
    Corruption(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Index error: {0}")]
    IndexError(String),
}

impl StorageError {
    pub fn new(message: &str) -> Self {
        StorageError::IoError(message.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IoError(e.to_string())
    }
}
