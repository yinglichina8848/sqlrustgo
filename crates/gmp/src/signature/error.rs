use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Invalid key format: {0}")]
    InvalidFormat(String),
    #[error("Key read failed: {0}")]
    IoError(#[from] std::io::Error),
}
