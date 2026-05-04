use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_create_table_then_immediate_select() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();

    let result = engine.execute("SELECT * FROM t").unwrap();
    assert_eq!(result.rows.len(), 0);

    let result = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_drop_table_then_query() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (1), (2)").unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(result.rows[0][0], Value::Integer(2));

    engine.execute("DROP TABLE t").unwrap();

    let result = engine.execute("SELECT * FROM t");
    assert!(result.is_err());
}

#[test]
fn test_multiple_tables() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1)").unwrap();
    engine.execute("INSERT INTO t2 VALUES (10), (20)").unwrap();

    let r1 = engine.execute("SELECT COUNT(*) FROM t1").unwrap();
    assert_eq!(r1.rows[0][0], Value::Integer(1));

    let r2 = engine.execute("SELECT COUNT(*) FROM t2").unwrap();
    assert_eq!(r2.rows[0][0], Value::Integer(2));
}

#[test]
fn test_create_and_recreate_table() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (1)").unwrap();
    engine.execute("DROP TABLE t").unwrap();

    engine.execute("CREATE TABLE t (id INTEGER, val TEXT)").unwrap();
    engine.execute("INSERT INTO t VALUES (1, 'hello')").unwrap();

    let r = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    assert_eq!(r.rows[0][0], Value::Integer(1));
}
