use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_catalog::Catalog;
use std::sync::{Arc, RwLock};

fn create_engine() -> MemoryExecutionEngine {
    let catalog = Arc::new(RwLock::new(Catalog::new("test")));
    MemoryExecutionEngine::with_memory_and_catalog(catalog)
}

#[test]
fn test_information_schema_triggers_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.triggers");
    assert!(
        result.is_ok(),
        "TRIGGERS should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_routines_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.routines");
    assert!(
        result.is_ok(),
        "ROUTINES should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_parameters_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.parameters");
    assert!(
        result.is_ok(),
        "PARAMETERS should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_user_privileges_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.user_privileges");
    assert!(
        result.is_ok(),
        "USER_PRIVILEGES should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_schema_privileges_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.schema_privileges");
    assert!(
        result.is_ok(),
        "SCHEMA_PRIVILEGES should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_table_privileges_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.table_privileges");
    assert!(
        result.is_ok(),
        "TABLE_PRIVILEGES should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_information_schema_column_privileges_queryable() {
    let mut engine = create_engine();
    let result = engine.execute("SELECT * FROM information_schema.column_privileges");
    assert!(
        result.is_ok(),
        "COLUMN_PRIVILEGES should be queryable: {:?}",
        result.err()
    );
}

#[test]
fn test_existing_information_schema_tables_still_work() {
    let mut engine = create_engine();
    let _ = engine.execute("CREATE TABLE t (id INTEGER, name TEXT)");

    let result = engine.execute("SELECT * FROM information_schema.schemata");
    if let Err(e) = result {
        panic!("schemata query failed: {:?}", e);
    }
    let result = engine.execute("SELECT * FROM information_schema.tables");
    if let Err(e) = result {
        panic!("tables query failed: {:?}", e);
    }
    let result = engine.execute("SELECT * FROM information_schema.columns");
    if let Err(e) = result {
        panic!("columns query failed: {:?}", e);
    }
    let result = engine.execute("SELECT * FROM information_schema.indexes");
    if let Err(e) = result {
        panic!("indexes query failed: {:?}", e);
    }
}
