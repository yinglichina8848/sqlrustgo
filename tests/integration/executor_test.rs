// Executor Tests - Volcano Model
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

#[test]
fn test_batch_insert() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();

    let result = engine
        .execute(
            parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')").unwrap(),
        )
        .unwrap();

    assert_eq!(result.affected_rows, 3);

    let result = engine
        .execute(parse("SELECT * FROM users").unwrap())
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

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

#[test]
fn test_operator_metrics() {
    use sqlrustgo_planner::OperatorMetrics;
    use std::time::Duration;

    let mut metrics = OperatorMetrics::new("SeqScan".to_string())
        .with_table("users".to_string())
        .with_timing(Duration::from_millis(10), 100);

    let child =
        OperatorMetrics::new("Filter".to_string()).with_timing(Duration::from_millis(5), 50);
    metrics.add_child(child);

    let output = metrics.to_string(0);
    assert!(output.contains("SeqScan"));
    assert!(output.contains("time="));
    assert!(output.contains("rows=100"));
}
