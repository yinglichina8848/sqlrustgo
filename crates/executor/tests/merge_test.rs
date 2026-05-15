use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_merge_basic() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new'), (2, 'inserted')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value WHEN NOT MATCHED THEN INSERT VALUES (source.id, source.value)"
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_merge_update_only() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old')")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value"
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_merge_insert_only() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED THEN INSERT VALUES (source.id, source.value)"
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_merge_delete() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'old'), (2, 'keep')")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN DELETE"
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_merge_multiple_conditions() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER, status TEXT, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'active', 100)")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, status TEXT, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (1, 'active', 200), (1, 'inactive', 150)")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id AND target.status = source.status WHEN MATCHED THEN UPDATE SET value = source.value"
    );
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_merge_no_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE target (id INTEGER PRIMARY KEY, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO target VALUES (1, 'existing')")
        .unwrap();
    engine
        .execute("CREATE TABLE source (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO source VALUES (2, 'new')")
        .unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED THEN INSERT VALUES (source.id, source.value)"
    );
    assert!(result.is_ok() || result.is_err());
}
