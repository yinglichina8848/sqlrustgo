//! Integration tests for SQLRustGo
//! Tests the complete SQL execution flow

use sqlrustgo::{parse, ExecutionEngine, TransactionManager, WriteAheadLog};
use std::sync::Arc;

#[test]
fn test_full_select_flow() {
    let mut engine = ExecutionEngine::new();

    // Create table first
    engine
        .execute(parse("CREATE TABLE users").unwrap())
        .unwrap();

    // Now parse and execute SELECT
    let result = parse("SELECT id FROM users");
    assert!(result.is_ok());

    let statement = result.unwrap();
    let exec_result = engine.execute(statement);
    assert!(exec_result.is_ok());
}

#[test]
fn test_full_insert_flow() {
    let mut engine = ExecutionEngine::new();

    // Create table first
    engine
        .execute(parse("CREATE TABLE users").unwrap())
        .unwrap();

    // Now parse and execute INSERT
    let result = parse("INSERT INTO users VALUES (1, 'Alice')");
    assert!(result.is_ok());

    let statement = result.unwrap();
    let exec_result = engine.execute(statement);
    assert!(exec_result.is_ok());
}

#[test]
fn test_full_transaction_flow() {
    let path = "/tmp/integration_test_wal.log";
    std::fs::remove_file(path).ok();

    let wal = Arc::new(WriteAheadLog::new(path).unwrap());
    let tm = TransactionManager::new(wal);

    // Begin transaction
    let tx_id = tm.begin().unwrap();
    assert!(tm.is_active(tx_id));

    // Commit
    tm.commit(tx_id).unwrap();
    assert!(!tm.is_active(tx_id));

    std::fs::remove_file(path).ok();
}

#[test]
fn test_create_and_select() {
    let mut engine = ExecutionEngine::new();

    // Create table
    let result = engine.execute(parse("CREATE TABLE test (id INTEGER, name TEXT)").unwrap());
    assert!(result.is_ok());
    assert!(engine.get_table("test").is_some());
}

#[test]
fn test_multiple_statements() {
    let mut engine = ExecutionEngine::new();

    // Multiple DDL operations
    engine
        .execute(parse("CREATE TABLE t1 (id INT)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE t2 (id INT)").unwrap())
        .unwrap();
    engine.execute(parse("DROP TABLE t1").unwrap()).unwrap();

    assert!(engine.get_table("t1").is_none());
    assert!(engine.get_table("t2").is_some());
}

#[test]
fn test_lexer_parser_integration() {
    // Test that lexer and parser work together
    let sql = "SELECT id, name FROM users WHERE age >= 18";
    let tokens = sqlrustgo::tokenize(sql);
    assert!(tokens.len() > 5);

    // Parse should work with these tokens
    let result = parse(sql);
    assert!(result.is_ok());
}

#[test]
fn test_error_handling() {
    let result = parse("INVALID SQL SYNTAX HERE");
    // This should return an error
    assert!(result.is_err());
}

#[test]
#[allow(clippy::approx_constant)]
fn test_value_type_conversion() {
    use sqlrustgo::parse_sql_literal;
    use sqlrustgo::Value;

    assert_eq!(parse_sql_literal("NULL"), Value::Null);
    assert_eq!(parse_sql_literal("42"), Value::Integer(42));
    assert_eq!(parse_sql_literal("3.14"), Value::Float(3.14));
}
