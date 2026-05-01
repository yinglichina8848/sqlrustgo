use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_begin_commit_transaction() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    engine.execute("BEGIN").unwrap();

    engine
        .execute("UPDATE t SET value = 200 WHERE id = 1")
        .unwrap();

    let commit_result = engine.execute("COMMIT");
    assert!(commit_result.is_ok(), "COMMIT should succeed");

    let select_result = engine
        .execute("SELECT id, value FROM t WHERE id = 1")
        .unwrap();
    assert_eq!(select_result.rows[0][1], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_begin_rollback_transaction() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    engine.execute("BEGIN").unwrap();

    engine
        .execute("UPDATE t SET value = 999 WHERE id = 1")
        .unwrap();

    let rollback_result = engine.execute("ROLLBACK");
    assert!(rollback_result.is_ok(), "ROLLBACK should succeed");
}

#[test]
fn test_begin_serializable() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    let begin_result = engine.execute("BEGIN SERIALIZABLE");
    assert!(begin_result.is_ok(), "BEGIN SERIALIZABLE should succeed");

    let select_result = engine
        .execute("SELECT id, value FROM t WHERE id = 1")
        .unwrap();
    assert_eq!(select_result.rows[0][1], sqlrustgo::Value::Integer(100));

    engine.execute("COMMIT").unwrap();
}

#[test]
fn test_set_transaction_isolation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    let set_result = engine.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE");
    assert!(
        set_result.is_ok(),
        "SET TRANSACTION ISOLATION LEVEL should succeed"
    );

    let begin_result = engine.execute("BEGIN");
    assert!(
        begin_result.is_ok(),
        "BEGIN should succeed after SET TRANSACTION"
    );

    engine.execute("COMMIT").unwrap();
}

#[test]
fn test_start_transaction() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    let start_result = engine.execute("START TRANSACTION");
    assert!(start_result.is_ok(), "START TRANSACTION should succeed");

    engine
        .execute("UPDATE t SET value = 300 WHERE id = 1")
        .unwrap();

    let commit_result = engine.execute("COMMIT");
    assert!(commit_result.is_ok(), "COMMIT should succeed");

    let select_result = engine
        .execute("SELECT id, value FROM t WHERE id = 1")
        .unwrap();
    assert_eq!(select_result.rows[0][1], sqlrustgo::Value::Integer(300));
}

#[test]
fn test_start_transaction_serializable() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    let start_result = engine.execute("START TRANSACTION ISOLATION LEVEL SERIALIZABLE");
    assert!(
        start_result.is_ok(),
        "START TRANSACTION ISOLATION LEVEL SERIALIZABLE should succeed"
    );

    let select_result = engine
        .execute("SELECT id, value FROM t WHERE id = 1")
        .unwrap();
    assert_eq!(select_result.rows[0][1], sqlrustgo::Value::Integer(100));

    engine.execute("COMMIT").unwrap();
}
