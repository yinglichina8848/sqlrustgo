// SQLRustGo executor module

pub mod executor;
pub mod filter;
pub mod local_executor;
pub mod projection;

pub use executor::{Executor, ExecutorResult, VolcanoExecutor, VolIterator, execute_collect};
pub use filter::FilterVolcanoExecutor;
pub use local_executor::LocalExecutor;
pub use projection::{ProjectionBuilder, ProjectionConfig, ProjectionExecutor, ProjectionExpr};

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_module_exports() {
        let _executor: Option<Box<dyn Executor>> = None;
        let _result = ExecutorResult::empty();
    }

    #[test]
    fn test_executor_result_creation() {
        let result = ExecutorResult::new(vec![], 0);
        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_executor_result_with_data() {
        let rows = vec![vec![Value::Integer(1)]];
        let result = ExecutorResult::new(rows, 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.affected_rows, 1);
    }
}
