//! QMD Bridge error types

use thiserror::Error;

/// QMD Bridge specific errors
#[derive(Error, Debug)]
pub enum QmdBridgeError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("QMD not available: {0}")]
    NotAvailable(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// Result type for QMD Bridge operations
pub type QmdResult<T> = Result<T, QmdBridgeError>;
