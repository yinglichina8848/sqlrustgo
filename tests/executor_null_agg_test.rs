use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_count_all_null_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT COUNT(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_count_star_vs_count_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (1), (NULL), (2)").unwrap();

    let result_star = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    let result_col = engine.execute("SELECT COUNT(v) FROM t").unwrap();

    assert_eq!(result_star.rows[0][0], Value::Integer(3));
    assert_eq!(result_col.rows[0][0], Value::Integer(2));
}

#[test]
fn test_sum_all_null_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT SUM(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}

#[test]
fn test_sum_empty_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();

    let result = engine.execute("SELECT SUM(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}

#[test]
fn test_avg_all_null_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT AVG(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}

#[test]
fn test_min_all_null_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT MIN(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}

#[test]
fn test_max_all_null_column() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT MAX(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}

#[test]
fn test_count_distinct_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT COUNT(DISTINCT v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_aggregate_mixed_null_and_values() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (1), (NULL), (2), (NULL)").unwrap();

    let cnt = engine.execute("SELECT COUNT(v) FROM t").unwrap();
    assert_eq!(cnt.rows[0][0], Value::Integer(2));

    let sum = engine.execute("SELECT SUM(v) FROM t").unwrap();
    assert_eq!(sum.rows[0][0], Value::Integer(3));

    let avg = engine.execute("SELECT AVG(v) FROM t").unwrap();
    assert!(matches!(avg.rows[0][0], Value::Integer(1) | Value::Float(_)));
}

#[test]
fn test_first_last_aggregates_with_nulls() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (5), (NULL), (3), (NULL)").unwrap();

    let min = engine.execute("SELECT MIN(v) FROM t").unwrap();
    assert_eq!(min.rows[0][0], Value::Integer(3));

    let max = engine.execute("SELECT MAX(v) FROM t").unwrap();
    assert_eq!(max.rows[0][0], Value::Integer(5));
}

#[test]
fn test_count_empty_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();

    let result = engine.execute("SELECT COUNT(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(0));

    let result_star = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result_star.rows[0][0], Value::Integer(0));
}

#[test]
fn test_avg_empty_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();

    let result = engine.execute("SELECT AVG(v) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Null);
}
