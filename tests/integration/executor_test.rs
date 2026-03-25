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
fn test_materialized_view() {
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine
        .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
        .unwrap();

    let result = engine
        .execute(parse("CREATE VIEW user_view AS SELECT * FROM users").unwrap())
        .unwrap();

    assert_eq!(result.affected_rows, 0);

    let storage = engine.storage.read().unwrap();
    assert!(storage.has_view("user_view"));
}

#[test]
fn test_auto_increment_column() {
    let result = parse("CREATE TABLE orders (id INTEGER AUTO_INCREMENT, name TEXT)");
    assert!(
        result.is_ok(),
        "Failed to parse AUTO_INCREMENT: {:?}",
        result
    );
    let stmt = result.unwrap();
    if let sqlrustgo_parser::Statement::CreateTable(create) = stmt {
        assert_eq!(create.columns.len(), 2);
        assert!(create.columns[0].auto_increment);
    } else {
        panic!("Expected CreateTable statement");
    }
}

#[test]
fn test_primary_key_column() {
    let result = parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)");
    assert!(result.is_ok(), "Failed to parse PRIMARY KEY: {:?}", result);
    let stmt = result.unwrap();
    if let sqlrustgo_parser::Statement::CreateTable(create) = stmt {
        assert_eq!(create.columns.len(), 2);
        assert!(create.columns[0].primary_key);
    } else {
        panic!("Expected CreateTable statement");
    }
}

#[test]
fn test_foreign_key_column() {
    let result = parse("CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id))");
    assert!(result.is_ok(), "Failed to parse FOREIGN KEY: {:?}", result);
    let stmt = result.unwrap();
    if let sqlrustgo_parser::Statement::CreateTable(create) = stmt {
        assert_eq!(create.columns.len(), 2);
        assert!(create.columns[1].references.is_some());
    } else {
        panic!("Expected CreateTable statement");
    }
}

#[test]
fn test_not_null_column() {
    let result = parse("CREATE TABLE users (id INTEGER NOT NULL, name TEXT)");
    assert!(result.is_ok(), "Failed to parse NOT NULL: {:?}", result);
    let stmt = result.unwrap();
    if let sqlrustgo_parser::Statement::CreateTable(create) = stmt {
        assert_eq!(create.columns.len(), 2);
        assert!(!create.columns[0].nullable);
    } else {
        panic!("Expected CreateTable statement");
    }
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

#[test]
fn test_upsert_syntax() {
    let result = parse(
        "INSERT INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name='Alice'",
    );
    assert!(result.is_ok(), "Failed to parse UPSERT: {:?}", result);
    let stmt = result.unwrap();
    if let sqlrustgo_parser::Statement::Insert(insert) = stmt {
        assert_eq!(insert.table, "users");
        assert!(insert.on_duplicate.is_some());
    } else {
        panic!("Expected Insert statement");
    }
}
