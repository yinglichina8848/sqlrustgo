//! Multi-Table DML Execution Tests
//! GAP-3: coverage improvement for DML statements

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_single_table_update() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c')")
        .unwrap();

    let result = engine
        .execute("UPDATE t SET value = 'updated' WHERE id > 1")
        .unwrap();
    assert_eq!(result.affected_rows, 2);

    let result = engine.execute("SELECT value FROM t WHERE id = 2").unwrap();
    assert_eq!(result.rows[0][0], Value::Text("updated".to_string()));
}

#[test]
fn test_single_table_delete() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c')")
        .unwrap();

    let result = engine.execute("DELETE FROM t WHERE id = 2").unwrap();
    assert_eq!(result.affected_rows, 1);

    let result = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_delete_with_subquery() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER PRIMARY KEY)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c')")
        .unwrap();
    engine.execute("INSERT INTO t2 VALUES (2)").unwrap();

    let result = engine
        .execute("DELETE FROM t1 WHERE id IN (SELECT id FROM t2)")
        .unwrap();
    assert_eq!(result.affected_rows, 1);

    let result = engine.execute("SELECT COUNT(*) FROM t1").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));
}
