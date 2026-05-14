use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Client GTID already exists: {0}")]
    DuplicateCgtid(String),

    #[error("Client GTID in progress: {0}")]
    CgtidInProgress(String),

    #[error("Causality violation: required CGTID {0} not committed")]
    CausalityViolation(String),

    #[error("OT transformation failed: {0}")]
    OTTransformFailed(String),

    #[error("Client registry error: {0}")]
    RegistryError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),
}

pub type SyncResult<T> = Result<T, SyncError>;
