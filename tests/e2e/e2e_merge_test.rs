use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_merge_basic_syntax() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE target (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO target VALUES (1, 'original')").unwrap();

    let result = engine.execute(
        "MERGE INTO target USING (SELECT 1 as id, 'updated' as value) AS source ON target.id = source.id WHEN MATCHED THEN UPDATE SET value = source.value WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)"
    );
    assert!(result.is_ok(), "MERGE should parse and execute: {:?}", result);
}

#[test]
fn test_merge_when_matched() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE target (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO target VALUES (1, 'original')").unwrap();
    engine.execute("CREATE TABLE source (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO source VALUES (1, 'updated')").unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN MATCHED THEN UPDATE SET target.value = source.value"
    );
    assert!(result.is_ok(), "MERGE WHEN MATCHED should work: {:?}", result);
}

#[test]
fn test_merge_when_not_matched() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE target (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO target VALUES (1, 'original')").unwrap();
    engine.execute("CREATE TABLE source (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO source VALUES (2, 'new_row')").unwrap();

    let result = engine.execute(
        "MERGE INTO target USING source ON target.id = source.id WHEN NOT MATCHED THEN INSERT (id, value) VALUES (source.id, source.value)"
    );
    assert!(result.is_ok(), "MERGE WHEN NOT MATCHED should work: {:?}", result);
}
