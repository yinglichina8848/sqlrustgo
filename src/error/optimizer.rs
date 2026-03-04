//! Optimizer Error Module

use thiserror::Error;

/// Optimizer error types
#[derive(Error, Debug)]
pub enum OptimizerError {
    #[error("Planning failed: {0}")]
    PlanningFailed(String),

    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    #[error("Statistics error: {0}")]
    StatisticsError(String),

    #[error("Optimization failed: {0}")]
    OptimizationFailed(String),
}

impl OptimizerError {
    pub fn new(message: &str) -> Self {
        OptimizerError::PlanningFailed(message.to_string())
    }
}
