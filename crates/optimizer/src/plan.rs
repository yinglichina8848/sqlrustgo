//! Optimizer Plan Types Module

use thiserror::Error;

/// Optimizer result type alias
pub type OptimizerResult<T> = Result<T, OptimizerError>;

/// Optimizer error types
#[derive(Error, Debug)]
pub enum OptimizerError {
    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),

    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    #[error("Rule application failed: {0}")]
    RuleFailed(String),
}
