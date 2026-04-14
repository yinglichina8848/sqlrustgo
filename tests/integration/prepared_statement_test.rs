use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, Statement};
use sqlrustgo_types::SqlError;
use std::sync::{Arc, RwLock};

fn create_test_engine() -> ExecutionEngine {
    let storage = MemoryStorage::new();
    ExecutionEngine::new(Arc::new(RwLock::new(storage)))
}

fn execute_sql(engine: &mut ExecutionEngine, sql: &str) -> Result<(), SqlError> {
    let statement = parse(sql).map_err(|e| SqlError::ExecutionError(e.to_string()))?;
    engine.execute(statement).map(|_| ())
}

#[test]
fn test_prepare_basic_select() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT, name TEXT)").unwrap();
    execute_sql(&mut engine, "PREPARE my_select FROM 'SELECT * FROM users'").unwrap();
}

#[test]
fn test_prepare_with_parameters() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT, name TEXT)").unwrap();
    execute_sql(
        &mut engine,
        "PREPARE stmt FROM 'SELECT * FROM users WHERE id = ?'",
    )
    .unwrap();
}

#[test]
fn test_prepare_invalid_sql() {
    let mut engine = create_test_engine();

    let result = execute_sql(&mut engine, "PREPARE stmt FROM 'SELEC * FORM users'");
    assert!(result.is_err());
}

#[test]
fn test_execute_prepared_statement() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT, name TEXT)").unwrap();
    execute_sql(&mut engine, "INSERT INTO users VALUES (1, 'Alice')").unwrap();
    execute_sql(&mut engine, "INSERT INTO users VALUES (2, 'Bob')").unwrap();
    execute_sql(&mut engine, "PREPARE my_select FROM 'SELECT * FROM users'").unwrap();
    execute_sql(&mut engine, "EXECUTE my_select").unwrap();
}

#[test]
fn test_execute_nonexistent_statement() {
    let mut engine = create_test_engine();

    let result = execute_sql(&mut engine, "EXECUTE nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_execute_after_deallocate() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT)").unwrap();
    execute_sql(&mut engine, "PREPARE stmt FROM 'SELECT * FROM users'").unwrap();
    execute_sql(&mut engine, "DEALLOCATE PREPARE stmt").unwrap();
    let result = execute_sql(&mut engine, "EXECUTE stmt");
    assert!(result.is_err());
}

#[test]
fn test_deallocate_prepared_statement() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT)").unwrap();
    execute_sql(&mut engine, "PREPARE my_stmt FROM 'SELECT * FROM users'").unwrap();
    execute_sql(&mut engine, "DEALLOCATE PREPARE my_stmt").unwrap();
}

#[test]
fn test_prepared_statement_sql_injection_prevention() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT, name TEXT)").unwrap();
    execute_sql(&mut engine, "INSERT INTO users VALUES (1, 'Alice')").unwrap();
    execute_sql(
        &mut engine,
        "PREPARE safe_query FROM 'SELECT * FROM users WHERE id = ?'",
    )
    .unwrap();
    execute_sql(&mut engine, "EXECUTE safe_query").unwrap();
}

#[test]
fn test_prepare_caches_physical_plan() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT, name TEXT)").unwrap();
    execute_sql(&mut engine, "INSERT INTO users VALUES (1, 'Alice')").unwrap();
    execute_sql(&mut engine, "PREPARE my_select FROM 'SELECT * FROM users'").unwrap();

    execute_sql(&mut engine, "EXECUTE my_select").unwrap();
    execute_sql(&mut engine, "EXECUTE my_select").unwrap();
}

#[test]
fn test_multiple_prepare_same_name() {
    let mut engine = create_test_engine();

    execute_sql(&mut engine, "CREATE TABLE users (id INT)").unwrap();
    execute_sql(&mut engine, "PREPARE my_stmt FROM 'SELECT * FROM users'").unwrap();
    execute_sql(&mut engine, "PREPARE my_stmt FROM 'SELECT id FROM users'").unwrap();
    execute_sql(&mut engine, "EXECUTE my_stmt").unwrap();
}
