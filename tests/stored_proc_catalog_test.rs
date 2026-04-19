// Stored Procedure Catalog Integration Tests
//
// These tests verify the stored procedure and trigger catalog integration
// that was added for Issue #1636 (存储过程与触发器 Catalog 集成)

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use sqlrustgo_catalog::Catalog;
use std::sync::{Arc, RwLock};

#[test]
fn test_create_and_call_procedure_with_catalog() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob')")
        .unwrap();

    let create_result =
        engine.execute("CREATE PROCEDURE get_count() BEGIN SELECT COUNT(*) FROM users; END");
    assert!(
        create_result.is_ok(),
        "CREATE PROCEDURE should succeed with catalog"
    );

    let call_result = engine.execute("CALL get_count()");
    assert!(
        call_result.is_ok(),
        "CALL should succeed after procedure creation"
    );
}

#[test]
fn test_call_requires_catalog() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    let result = engine.execute("CALL my_proc(1, 2)");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("catalog"),
        "Error should mention catalog requirement"
    );
}

#[test]
fn test_create_procedure_requires_catalog() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    let result = engine.execute("CREATE PROCEDURE test_proc() BEGIN END");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("catalog"),
        "Error should mention catalog requirement"
    );
}

#[test]
fn test_procedure_not_found() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog);

    let result = engine.execute("CALL nonexistent_proc()");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Error should indicate procedure not found"
    );
}

#[test]
fn test_duplicate_procedure_error() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    let create1 = engine.execute("CREATE PROCEDURE test_dup() BEGIN SELECT 1; END");
    assert!(create1.is_ok());

    let create2 = engine.execute("CREATE PROCEDURE test_dup() BEGIN SELECT 2; END");
    assert!(create2.is_err(), "Duplicate procedure should return error");
}

#[test]
fn test_create_trigger_with_catalog() {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    let mut engine = ExecutionEngine::with_memory_and_catalog(catalog.clone());

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, created_ts TEXT)")
        .unwrap();

    let create_trigger = engine.execute(
        "CREATE TRIGGER before_insert_ts BEFORE INSERT ON users FOR EACH ROW BEGIN SET NEW.created_ts = 'triggered'; END"
    );
    assert!(create_trigger.is_ok(), "CREATE TRIGGER should succeed");

    let insert_result = engine.execute("INSERT INTO users VALUES (1, 'Alice')");
    assert!(insert_result.is_ok());
}
