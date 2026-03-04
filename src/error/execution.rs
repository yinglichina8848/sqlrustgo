//! Execution Error Module

use thiserror::Error;

/// Execution error types
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Execution failed: {0}")]
    Failed(String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Null value error: {0}")]
    NullValueError(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),
}

impl ExecutionError {
    pub fn new(message: &str) -> Self {
        ExecutionError::Failed(message.to_string())
    }
}
