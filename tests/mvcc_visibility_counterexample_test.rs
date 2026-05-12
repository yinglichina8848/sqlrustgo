use sqlrustgo::execution_engine::EngineConfig;
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
}

#[test]
fn test_self_read_is_visible() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, value INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 100)");

    let _ = engine.execute("BEGIN");
    let _ = engine.execute("UPDATE t SET value = 200 WHERE id = 1");

    let result = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.rows[0][0], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_version_chain_monotonicity() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, value INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 100)");

    let _ = engine.execute("BEGIN");
    let _ = engine.execute("COMMIT");

    let r1 = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(r1.is_ok());
    assert_eq!(r1.unwrap().rows[0][0], sqlrustgo::Value::Integer(100));

    let _ = engine.execute("BEGIN");
    let _ = engine.execute("UPDATE t SET value = 200 WHERE id = 1");
    let _ = engine.execute("COMMIT");

    let r2 = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(r2.is_ok());
    assert_eq!(r2.unwrap().rows[0][0], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_read_committed_version() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, value INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 100)");

    let _ = engine.execute("BEGIN");
    let _ = engine.execute("UPDATE t SET value = 200 WHERE id = 1");
    let _ = engine.execute("COMMIT");

    let _ = engine.execute("BEGIN");
    let result = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows[0][0], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_snapshot_sees_committed_only() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, value INTEGER)");
    let _ = engine.execute("INSERT INTO t VALUES (1, 100)");

    let _ = engine.execute("BEGIN");
    let snapshot_result = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(snapshot_result.is_ok());
    let snapshot_value = snapshot_result.unwrap().rows[0][0].clone();
    assert_eq!(snapshot_value, sqlrustgo::Value::Integer(100));

    let _ = engine.execute("UPDATE t SET value = 200 WHERE id = 1");
    let _ = engine.execute("COMMIT");

    let after_result = engine.execute("SELECT value FROM t WHERE id = 1");
    assert!(after_result.is_ok());
    assert_eq!(after_result.unwrap().rows[0][0], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_multiple_versions_created() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, value INTEGER)");

    let _ = engine.execute("INSERT INTO t VALUES (1, 100)");
    let r1 = engine.execute("SELECT value FROM t WHERE id = 1");
    assert_eq!(r1.unwrap().rows[0][0], sqlrustgo::Value::Integer(100));

    let _ = engine.execute("UPDATE t SET value = 200 WHERE id = 1");
    let r2 = engine.execute("SELECT value FROM t WHERE id = 1");
    assert_eq!(r2.unwrap().rows[0][0], sqlrustgo::Value::Integer(200));

    let _ = engine.execute("UPDATE t SET value = 300 WHERE id = 1");
    let r3 = engine.execute("SELECT value FROM t WHERE id = 1");
    assert_eq!(r3.unwrap().rows[0][0], sqlrustgo::Value::Integer(300));
}
