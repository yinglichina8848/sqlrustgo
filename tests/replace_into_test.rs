// REPLACE INTO Integration Tests (BP2-1b)
//! Tests for REPLACE INTO operation: upsert semantics
//! BP2 Gate: cargo test --test replace_into_test

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

/// Test basic REPLACE INTO - insert new row
#[test]
fn test_replace_into_insert() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // Create test table with primary key
    engine.execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT)").unwrap();

    // Insert initial row
    engine.execute("INSERT INTO t1 VALUES (1, 'Alice')").unwrap();

    // REPLACE INTO - should insert new row
    engine.execute("REPLACE INTO t1 VALUES (2, 'Bob')").unwrap();

    // Verify both rows exist
    let result = engine.execute("SELECT * FROM t1 ORDER BY id").unwrap();
    assert_eq!(result.rows.len(), 2, "Expected 2 rows, got {}", result.rows.len());
}

/// Test REPLACE INTO - replace existing row
#[test]
fn test_replace_into_update() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // Create test table with primary key
    engine.execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, name TEXT)").unwrap();

    // Insert initial row
    engine.execute("INSERT INTO t1 VALUES (1, 'Alice')").unwrap();

    // REPLACE INTO with same PK - should replace
    engine.execute("REPLACE INTO t1 VALUES (1, 'Alicia')").unwrap();

    // Verify only one row exists with updated value
    let result = engine.execute("SELECT * FROM t1").unwrap();
    assert_eq!(result.rows.len(), 1, "Expected 1 row, got {}", result.rows.len());
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(result.rows[0][1], sqlrustgo_types::Value::Text("Alicia".to_string()));
}
