//! Optimizer Plan Types Module

use std::any::Any;
use thiserror::Error;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
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
        let err = OptimizerError::OptimizationFailed("timeout".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Optimization failed"));
        assert!(display.contains("timeout"));
    }

    #[test]
    fn test_optimizer_error_invalid_plan() {
        let err = OptimizerError::InvalidPlan("missing table".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid plan"));
        assert!(display.contains("missing table"));
    }

    #[test]
    fn test_optimizer_error_rule_failed() {
        let err = OptimizerError::RuleFailed("rule not applicable".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Rule application failed"));
        assert!(display.contains("rule not applicable"));
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
            Err(OptimizerError::OptimizationFailed("fail".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("fail"));
    }

    #[test]
    fn test_optimizer_error_debug() {
        let err = OptimizerError::OptimizationFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("OptimizationFailed"));
    }

    #[test]
    fn test_optimizer_error_all_variants() {
        let errors = vec![
            OptimizerError::OptimizationFailed("opt".to_string()),
            OptimizerError::InvalidPlan("plan".to_string()),
            OptimizerError::RuleFailed("rule".to_string()),
        ];
        for err in errors {
            let msg = err.to_string();
            assert!(!msg.is_empty());
        }
    }
}
