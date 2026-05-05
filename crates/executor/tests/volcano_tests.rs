use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_volcano_basic_query() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3)")
        .unwrap();
    let result = engine.execute("SELECT id FROM t").unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_volcano_multiple_selects() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (x INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (10), (20)").unwrap();
    let sum = engine.execute("SELECT SUM(x) FROM t").unwrap();
    assert_eq!(sum.rows[0][0], Value::Integer(30));
    let count = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(2));
}

#[test]
fn test_volcano_empty_scan() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    let result = engine.execute("SELECT id FROM t").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_volcano_empty_aggregate() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    let result = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_volcano_insert_select() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (1)").unwrap();
    let row_count = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(row_count.rows[0][0], Value::Integer(1));
}

#[test]
fn test_volcano_select_columns() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 10), (2, 20)")
        .unwrap();
    let count = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(count.rows[0][0], Value::Integer(2));
}

#[test]
fn test_volcano_ten_rows_insert() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (n INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3), (4), (5), (6), (7), (8), (9), (10)")
        .unwrap();
    let result = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(10));
}

#[test]
fn test_volcano_aggregate_filter() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (v INTEGER)").unwrap();
    engine
        .execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)")
        .unwrap();
    let sum = engine.execute("SELECT SUM(v) FROM t").unwrap();
    assert_eq!(sum.rows[0][0], Value::Integer(15));
    let filtered = engine
        .execute("SELECT COUNT(*) FROM t WHERE v = 3")
        .unwrap();
    assert_eq!(filtered.rows[0][0], Value::Integer(1));
    let not_eq = engine
        .execute("SELECT COUNT(*) FROM t WHERE v <> 3")
        .unwrap();
    assert_eq!(not_eq.rows[0][0], Value::Integer(4));
}
