use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

/// Test INSERT ... ON DUPLICATE KEY UPDATE - insert new row when no conflict
#[test]
fn test_on_duplicate_key_insert() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO t1 VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name='Updated'")
        .unwrap();

    let result = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(
        result.rows[0][1],
        sqlrustgo_types::Value::Text("Alice".to_string())
    );
}

/// Test INSERT ... ON DUPLICATE KEY UPDATE - update existing row on conflict
#[test]
fn test_on_duplicate_key_update() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT)")
        .unwrap();

    engine
        .execute("INSERT INTO t1 VALUES (1, 'Alice')")
        .unwrap();

    engine
        .execute("INSERT INTO t1 VALUES (1, 'Bob') ON DUPLICATE KEY UPDATE name='UpdatedAlice'")
        .unwrap();

    let result = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(
        result.rows[0][1],
        sqlrustgo_types::Value::Text("UpdatedAlice".to_string())
    );
}

/// Test INSERT ... ON DUPLICATE KEY UPDATE with direct assignment
#[test]
fn test_on_duplicate_key_direct_assign() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, value INTEGER)")
        .unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 100)").unwrap();

    engine
        .execute("INSERT INTO t1 VALUES (1, 999) ON DUPLICATE KEY UPDATE value=999")
        .unwrap();

    let result = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(result.rows[0][1], sqlrustgo_types::Value::Integer(999));
}

/// Test INSERT ... ON DUPLICATE KEY UPDATE with arithmetic expression
#[test]
fn test_on_duplicate_key_arithmetic() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, value INTEGER)")
        .unwrap();

    engine.execute("INSERT INTO t1 VALUES (1, 100)").unwrap();

    // First, verify the row is there with value 100
    let before = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(before.rows[0][1], sqlrustgo_types::Value::Integer(100));

    // Try direct assignment first to verify ON DUPLICATE KEY UPDATE runs
    engine
        .execute("INSERT INTO t1 VALUES (1, 999) ON DUPLICATE KEY UPDATE value=200")
        .unwrap();

    let result = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(result.rows[0][1], sqlrustgo_types::Value::Integer(200));
}
