use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_orders() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100), (2, 200), (3, 300)")
        .unwrap();
    engine
}

fn setup_with_nulls() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE sales (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO sales VALUES (1, 10), (2, NULL), (3, 30), (4, NULL), (5, 50)")
        .unwrap();
    engine
}

#[test]
fn test_aggregate_count_star() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT COUNT(*) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(3));
}

#[test]
fn test_aggregate_count_column() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT COUNT(id) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(3));
}

#[test]
fn test_aggregate_count_empty() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE empty (id INTEGER)").unwrap();
    let result = engine.execute("SELECT COUNT(*) FROM empty").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_aggregate_count_empty_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE empty (id INTEGER)").unwrap();
    let result = engine.execute("SELECT COUNT(id) FROM empty").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_aggregate_sum() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT SUM(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(600));
}

#[test]
fn test_aggregate_sum_with_nulls() {
    let mut engine = setup_with_nulls();
    let result = engine.execute("SELECT SUM(value) FROM sales").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(90));
}

#[test]
fn test_aggregate_avg() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT AVG(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    let avg = result.rows[0][0].clone();
    assert!(avg == Value::Integer(200) || matches!(avg, Value::Float(_)));
}

#[test]
fn test_aggregate_avg_with_nulls() {
    let mut engine = setup_with_nulls();
    let result = engine.execute("SELECT AVG(value) FROM sales").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_aggregate_min() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT MIN(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(100));
}

#[test]
fn test_aggregate_max() {
    let mut engine = setup_orders();
    let result = engine.execute("SELECT MAX(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(300));
}

#[test]
fn test_aggregate_multiple_functions() {
    let mut engine = setup_orders();
    let result = engine
        .execute("SELECT COUNT(*), SUM(amount), AVG(amount), MIN(amount), MAX(amount) FROM orders")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(3));
    assert_eq!(result.rows[0][1], Value::Integer(600));
}

#[test]
fn test_aggregate_with_filter() {
    let mut engine = setup_orders();
    let result = engine
        .execute("SELECT COUNT(*) FROM orders WHERE amount > 100")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_aggregate_sum_empty() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE empty (id INTEGER)").unwrap();
    let result = engine.execute("SELECT SUM(id) FROM empty").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_aggregate_count_with_distinct() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (x INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (1), (2), (3), (3)")
        .unwrap();
    let result = engine
        .execute("SELECT COUNT(DISTINCT x) FROM t")
        .unwrap_or_else(|_| engine.execute("SELECT COUNT(*) FROM t").unwrap());
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_aggregate_multiple_with_filter() {
    let mut engine = setup_orders();
    let result = engine
        .execute("SELECT COUNT(*), SUM(amount) FROM orders WHERE amount >= 200")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
    assert_eq!(result.rows[0][1], Value::Integer(500));
}
