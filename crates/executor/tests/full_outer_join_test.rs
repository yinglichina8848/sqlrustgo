use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_full_outer_join_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (1, 100), (2, 200), (4, 400)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 4);
}

#[test]
fn test_full_outer_join_all_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (1, 100), (2, 200)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_full_outer_join_no_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (3, 100), (4, 200)")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 4);
}
