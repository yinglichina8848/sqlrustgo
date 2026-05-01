// Test for LIMIT clause - TDD demonstration
// Bug: src/execution_engine.rs does not handle LIMIT in SELECT statements

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

#[test]
fn test_select_limit() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // Setup: insert 10 rows
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    for i in 1..=10 {
        engine
            .execute(&format!("INSERT INTO t1 VALUES ({})", i))
            .unwrap();
    }

    // SELECT with LIMIT 3 - should return only 3 rows
    let result = engine.execute("SELECT * FROM t1 LIMIT 3");
    assert!(result.is_ok(), "SELECT with LIMIT should succeed");

    let rows = result.unwrap();
    assert_eq!(rows.rows.len(), 3, "LIMIT 3 should return exactly 3 rows");
    assert_eq!(rows.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(rows.rows[1][0], sqlrustgo_types::Value::Integer(2));
    assert_eq!(rows.rows[2][0], sqlrustgo_types::Value::Integer(3));
}

#[test]
fn test_select_limit_offset() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    for i in 1..=10 {
        engine
            .execute(&format!("INSERT INTO t1 VALUES ({})", i))
            .unwrap();
    }

    // SELECT with LIMIT 3 OFFSET 5 - should skip 5 rows, return 3
    let result = engine.execute("SELECT * FROM t1 LIMIT 3 OFFSET 5");
    assert!(result.is_ok(), "SELECT with LIMIT OFFSET should succeed");

    let rows = result.unwrap();
    assert_eq!(rows.rows.len(), 3, "LIMIT 3 OFFSET 5 should return 3 rows");
    assert_eq!(rows.rows[0][0], sqlrustgo_types::Value::Integer(6));
    assert_eq!(rows.rows[1][0], sqlrustgo_types::Value::Integer(7));
    assert_eq!(rows.rows[2][0], sqlrustgo_types::Value::Integer(8));
}

#[test]
fn test_select_limit_zero() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (2)").unwrap();

    // LIMIT 0 should return empty
    let result = engine.execute("SELECT * FROM t1 LIMIT 0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows.len(), 0);
}
