//! Executor trait - abstraction for query execution
//! Decouples the execution layer for remote execution support

use sqlrustgo_planner::PhysicalPlan;
use sqlrustgo_types::{SqlResult, Value};

/// Execution result containing rows and metadata
#[derive(Debug, Clone)]
pub struct ExecutorResult {
    /// Result rows
    pub rows: Vec<Vec<Value>>,
    /// Number of affected rows (for INSERT/UPDATE/DELETE)
    pub affected_rows: usize,
}

impl ExecutorResult {
    /// Create a new executor result
    pub fn new(rows: Vec<Vec<Value>>, affected_rows: usize) -> Self {
        Self {
            rows,
            affected_rows,
        }
    }

    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            rows: vec![],
            affected_rows: 0,
        }
    }
}

/// Executor trait - abstraction for executing physical plans
/// Enables decoupling execution layer and supports remote execution
pub trait Executor: Send + Sync {
    /// Execute a physical plan and return results
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;

    /// Get executor name
    fn name(&self) -> &str;

    /// Check if executor is ready
    fn is_ready(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::PhysicalPlan;

    /// Mock executor for testing
    pub struct MockExecutor;

    impl MockExecutor {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for MockExecutor {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Executor for MockExecutor {
        fn execute(&self, _plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
            Ok(ExecutorResult::empty())
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn is_ready(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_executor_trait() {
        let _executor: Box<dyn Executor> = Box::new(MockExecutor::new());
        assert!(MockExecutor::new().is_ready());
        assert_eq!(MockExecutor::new().name(), "mock");
    }

    #[test]
    fn test_executor_result() {
        let result = ExecutorResult::new(vec![], 0);
        assert!(result.rows.is_empty());

        let result = ExecutorResult::new(vec![vec![Value::Integer(1)]], 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.affected_rows, 1);
    }
}
