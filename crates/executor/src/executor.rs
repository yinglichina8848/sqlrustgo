//! Executor trait - abstraction for query execution
//! Decouples the execution layer for remote execution support

use sqlrustgo_planner::PhysicalPlan;
use sqlrustgo_types::{SqlResult, Value};

/// Volcano-style iterator executor trait (open/next/close).
/// Used by window functions and pipeline-style execution.
pub trait VolcanoExecutor: Send + Sync {
    /// Initialize the executor and prepare to produce rows.
    fn open(&mut self) -> SqlResult<()>;

    /// Return the next row, or None when exhausted.
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;

    /// Release resources.
    fn close(&mut self) -> SqlResult<()>;
}

/// Adapter that wraps a pre-computed `ExecutorResult` (Vec<Vec<Value>>)
/// into the Volcano open/next/close model.
/// Used to bridge `LocalExecutor` (batch model) into `WindowVolcanoExecutor`.
pub struct LocalExecutorAdapter {
    rows: Vec<Vec<Value>>,
    position: usize,
    opened: bool,
}

impl LocalExecutorAdapter {
    /// Create from pre-computed rows.
    pub fn new(rows: Vec<Vec<Value>>) -> Self {
        Self {
            rows,
            position: 0,
            opened: false,
        }
    }

    /// Create from a `LocalExecutor` execution result.
    pub fn from_result(result: ExecutorResult) -> Self {
        Self::new(result.rows)
    }
}

impl VolcanoExecutor for LocalExecutorAdapter {
    fn open(&mut self) -> SqlResult<()> {
        self.position = 0;
        self.opened = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.opened {
            return Err("executor not opened".into());
        }
        if self.position >= self.rows.len() {
            return Ok(None);
        }
        let row = self.rows[self.position].clone();
        self.position += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.opened = false;
        self.position = 0;
        Ok(())
    }
}

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

    #[test]
    fn test_executor_result_empty() {
        let result = ExecutorResult::empty();
        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_executor_result_with_rows() {
        let rows = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let result = ExecutorResult::new(rows.clone(), 0);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_executor_result_affected_rows() {
        let result = ExecutorResult::new(vec![], 100);
        assert_eq!(result.affected_rows, 100);
    }

    #[test]
    fn test_executor_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MockExecutor>();
        _check::<ExecutorResult>();
    }

    #[test]
    fn test_executor_result_clone() {
        let result = ExecutorResult::new(vec![vec![Value::Integer(1)]], 1);
        let cloned = result.clone();
        assert_eq!(cloned.rows.len(), 1);
    }

    #[test]
    fn test_mock_executor_name() {
        let executor = MockExecutor::new();
        assert_eq!(executor.name(), "mock");
    }

    #[test]
    fn test_mock_executor_is_ready() {
        let executor = MockExecutor::new();
        assert!(executor.is_ready());
    }

    #[test]
    fn test_executor_result_with_null() {
        let rows = vec![
            vec![Value::Null, Value::Integer(1)],
            vec![Value::Text("test".to_string()), Value::Null],
        ];
        let result = ExecutorResult::new(rows, 0);
        assert_eq!(result.rows.len(), 2);
        assert!(matches!(result.rows[0][0], Value::Null));
    }

    #[test]
    fn test_executor_result_with_float() {
        let rows = vec![
            vec![Value::Float(3.14), Value::Integer(1)],
            vec![Value::Float(-2.5), Value::Integer(2)],
        ];
        let result = ExecutorResult::new(rows, 0);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_executor_result_with_blob() {
        let rows = vec![vec![Value::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF])]];
        let result = ExecutorResult::new(rows, 0);
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_executor_result_large_affected_rows() {
        let result = ExecutorResult::new(vec![], 1_000_000);
        assert_eq!(result.affected_rows, 1_000_000);
    }

    #[test]
    fn test_executor_result_zero_rows() {
        let result = ExecutorResult::new(vec![], 0);
        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, 0);
    }
}
