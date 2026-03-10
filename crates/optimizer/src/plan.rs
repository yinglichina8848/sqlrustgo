//! Optimizer Plan Types Module

use std::any::Any;
use thiserror::Error;

/// Trait for types that can be converted to Any for dynamic dispatch
pub trait AsAny: Any {
    /// Convert reference to Any
    fn as_any(&self) -> &dyn Any;
    /// Convert mutable reference to Any
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_error_optimization_failed() {
        let err = OptimizerError::OptimizationFailed("cost too high".to_string());
        assert_eq!(err.to_string(), "Optimization failed: cost too high");
    }

    #[test]
    fn test_optimizer_error_invalid_plan() {
        let err = OptimizerError::InvalidPlan("missing table".to_string());
        assert_eq!(err.to_string(), "Invalid plan: missing table");
    }

    #[test]
    fn test_optimizer_error_rule_failed() {
        let err = OptimizerError::RuleFailed("predicate pushdown error".to_string());
        assert_eq!(
            err.to_string(),
            "Rule application failed: predicate pushdown error"
        );
    }

    #[test]
    fn test_optimizer_error_debug() {
        let err = OptimizerError::OptimizationFailed("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("OptimizationFailed"));
    }

    #[test]
    fn test_optimizer_result_ok() {
        let result: OptimizerResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_optimizer_result_err() {
        let result: OptimizerResult<i32> =
            Err(OptimizerError::OptimizationFailed("failed".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed"));
    }
}
