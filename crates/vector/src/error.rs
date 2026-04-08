//! Vector index error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectorError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Index not built: call build_index() first")]
    IndexNotBuilt,

    #[error("Empty index: no vectors inserted")]
    EmptyIndex,

    #[error("ID not found: {0}")]
    IdNotFound(u64),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("K-means error: {0}")]
    KMeansError(String),
}

pub type VectorResult<T> = Result<T, VectorError>;
