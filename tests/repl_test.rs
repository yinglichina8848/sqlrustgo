//! Tests for REPL main functions

use sqlrustgo::{ExecutionEngine, ExecutionResult, parse};

/// Process user input - copied from main.rs for testing
fn process_input(
    input: &str,
    engine: &mut ExecutionEngine,
) -> Result<Option<ExecutionResult>, String> {
    // Handle special commands
    if input.starts_with('.') {
        return handle_command(input);
    }

    // Handle exit
    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
        std::process::exit(0);
    }

    // Execute SQL statement
    match parse(input) {
        Ok(statement) => match engine.execute(statement) {
            Ok(result) => Ok(Some(result)),
            Err(e) => Err(format!("Execution error: {}", e)),
        },
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Handle special commands - copied from main.rs for testing
fn handle_command(input: &str) -> Result<Option<ExecutionResult>, String> {
    match input {
        ".help" => {
            println!("Available commands:");
            println!("  .help      Show this help message");
            println!("  .tables    List all tables");
            println!("  .schema    Show schema info");
            println!("  .exit      Exit the REPL");
            println!("  .quit      Exit the REPL");
            Ok(None)
        }
        ".tables" => {
            println!("Tables: (catalog not yet implemented)");
            Ok(None)
        }
        ".schema" => {
            println!("Schema: (catalog not yet implemented)");
            Ok(None)
        }
        ".exit" | ".quit" => {
            std::process::exit(0);
        }
        _ => Err("Unknown command. Type .help for available commands.".to_string()),
    }
}

/// Print execution result - copied from main.rs for testing
fn print_result(result: ExecutionResult) {
    if result.rows.is_empty() {
        println!("OK, {} row(s) affected", result.rows_affected);
    } else {
        println!("{} row(s) in set", result.rows.len());
    }
}

#[test]
fn test_handle_command_help() {
    let result = handle_command(".help");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_handle_command_tables() {
    let result = handle_command(".tables");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_handle_command_schema() {
    let result = handle_command(".schema");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_handle_command_unknown() {
    let result = handle_command(".unknown");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown command"));
}

#[test]
fn test_print_result_empty_rows() {
    let result = ExecutionResult {
        rows_affected: 5,
        columns: vec![],
        rows: vec![],
    };
    print_result(result);
}

#[test]
fn test_print_result_with_rows() {
    use sqlrustgo::Value;
    let result = ExecutionResult {
        rows_affected: 0,
        columns: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ],
    };
    print_result(result);
}

#[test]
fn test_process_input_create_table() {
    let mut engine = ExecutionEngine::new();
    let result = process_input(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)",
        &mut engine,
    );
    assert!(result.is_ok());
}

#[test]
fn test_process_input_parse_error() {
    let mut engine = ExecutionEngine::new();
    let result = process_input("SELECT", &mut engine);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Parse error"));
}

#[test]
fn test_handle_command_case_sensitive() {
    // Commands are case-sensitive
    let result = handle_command(".HELP");
    assert!(result.is_err());
}

#[test]
fn test_process_input_select() {
    let mut engine = ExecutionEngine::new();
    // First create a table
    let _ = process_input("CREATE TABLE test (id INTEGER)", &mut engine);
    // Then insert
    let _ = process_input("INSERT INTO test VALUES (1)", &mut engine);
    // Then select
    let result = process_input("SELECT * FROM test", &mut engine);
    assert!(result.is_ok());
}

#[test]
fn test_process_input_insert() {
    let mut engine = ExecutionEngine::new();
    let _ = process_input("CREATE TABLE test (id INTEGER)", &mut engine);
    let result = process_input("INSERT INTO test VALUES (1)", &mut engine);
    assert!(result.is_ok());
}

#[test]
fn test_process_input_update() {
    let mut engine = ExecutionEngine::new();
    let _ = process_input("CREATE TABLE test (id INTEGER, value INTEGER)", &mut engine);
    let _ = process_input("INSERT INTO test VALUES (1, 10)", &mut engine);
    let result = process_input("UPDATE test SET value = 20 WHERE id = 1", &mut engine);
    assert!(result.is_ok());
}

#[test]
fn test_process_input_delete() {
    let mut engine = ExecutionEngine::new();
    let _ = process_input("CREATE TABLE test (id INTEGER)", &mut engine);
    let _ = process_input("INSERT INTO test VALUES (1)", &mut engine);
    let result = process_input("DELETE FROM test WHERE id = 1", &mut engine);
    assert!(result.is_ok());
}
