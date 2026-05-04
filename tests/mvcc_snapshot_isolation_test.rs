use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn create_engine_with_storage(
    storage: Arc<RwLock<MemoryStorage>>,
) -> ExecutionEngine<MemoryStorage> {
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
    engine.execute("COMMIT").unwrap();

    let result = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(200));
}

#[test]
fn test_start_transaction() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    engine.execute("START TRANSACTION").unwrap();
    engine
        .execute("UPDATE t SET value = 300 WHERE id = 1")
        .unwrap();
    engine.execute("COMMIT").unwrap();

    let result = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(300));
}

#[test]
fn test_serializable_isolation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    engine.execute("BEGIN SERIALIZABLE").unwrap();

    let result = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(100));

    engine.execute("COMMIT").unwrap();
}

#[test]
fn test_set_transaction_isolation() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();

    let result = engine.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE");
    assert!(result.is_ok());

    engine.execute("BEGIN").unwrap();

    let result = engine.execute("SELECT value FROM t WHERE id = 1").unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(100));

    engine.execute("COMMIT").unwrap();
}

#[test]
fn test_concurrent_transactions_with_shared_storage() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage1 = storage.clone();
    let handle1 = std::thread::spawn(move || {
        let engine = create_engine_with_storage(storage1);
        let mut engine = engine;
        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();
    });

    handle1.join().unwrap();

    let storage2 = storage.clone();
    let handle2 = std::thread::spawn(move || {
        let engine = create_engine_with_storage(storage2);
        let mut engine = engine;
        let result = engine.execute("SELECT value FROM t WHERE id = 1");
        assert!(result.is_ok());
    });

    handle2.join().unwrap();
}

#[test]
fn test_multiple_concurrent_readers() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    let storage_setup = storage.clone();
    std::thread::spawn(move || {
        let engine = create_engine_with_storage(storage_setup);
        let mut engine = engine;
        engine
            .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
            .unwrap();
        engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();
        engine.execute("INSERT INTO t VALUES (2, 200)").unwrap();
    })
    .join()
    .unwrap();

    let storage_reader = storage.clone();
    let handle1 = std::thread::spawn(move || {
        let engine = create_engine_with_storage(storage_reader);
        let mut engine = engine;
        engine.execute("BEGIN").unwrap();
        let r = engine.execute("SELECT SUM(value) FROM t");
        assert!(r.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    let storage_reader2 = storage.clone();
    let handle2 = std::thread::spawn(move || {
        let engine = create_engine_with_storage(storage_reader2);
        let mut engine = engine;
        engine.execute("BEGIN").unwrap();
        let r = engine.execute("SELECT COUNT(*) FROM t");
        assert!(r.is_ok());
        engine.execute("COMMIT").unwrap();
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

#[test]
fn test_transaction_reads_initial_state() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, value INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t VALUES (1, 100)").unwrap();
    engine.execute("INSERT INTO t VALUES (2, 200)").unwrap();

    engine.execute("BEGIN").unwrap();

    let r1 = engine.execute("SELECT SUM(value) FROM t").unwrap();
    let sum1 = r1.rows[0][0].clone();

    let r2 = engine.execute("SELECT COUNT(*) FROM t").unwrap();
    let count = r2.rows[0][0].clone();

    assert_eq!(sum1, sqlrustgo::Value::Integer(300));
    assert_eq!(count, sqlrustgo::Value::Integer(2));

    engine.execute("COMMIT").unwrap();
}
