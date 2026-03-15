// Executor Tests - Volcano Model
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_types::Value;

#[test]
fn test_executor_result_new() {
    let result = ExecutorResult::new(vec![], 0);
    assert!(result.rows.is_empty());
}

#[test]
fn test_executor_result_empty() {
    let result = ExecutorResult::empty();
    assert!(result.rows.is_empty());
    assert_eq!(result.affected_rows, 0);
}

#[test]
fn test_executor_result_with_data() {
    let rows = vec![vec![Value::Integer(1)], vec![Value::Integer(2)]];
    let result = ExecutorResult::new(rows, 2);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.affected_rows, 2);
}

#[test]
fn test_executor_result_affected_rows() {
    let result = ExecutorResult::new(vec![], 100);
    assert_eq!(result.affected_rows, 100);
}
