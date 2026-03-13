//! Example Tests using the Test Framework
//!
//! This module demonstrates how to use the executor test framework
//! for writing tests.

use crate::executor::test_framework::{
    TestHarness, TestFixture, MockStorage, TestDataGenerator,
    TestTableBuilder, schemas, assertions,
};
use crate::types::Value;

#[test]
fn test_example_select_basic() {
    let mut harness = TestHarness::new();
    
    // Create table and insert data
    harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
    harness.execute_ok("INSERT INTO users VALUES (1, 'Alice')");
    harness.execute_ok("INSERT INTO users VALUES (2, 'Bob')");
    
    // Query data
    let result = harness.select("SELECT * FROM users");
    
    // Verify results
    assertions::assert_row_count(&result, 2);
    assertions::assert_columns(&result, &["id", "name"]);
}

#[test]
fn test_example_select_with_where() {
    let mut harness = TestHarness::new();
    
    harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)");
    harness.execute_ok("INSERT INTO users VALUES (1, 'Alice', 30)");
    harness.execute_ok("INSERT INTO users VALUES (2, 'Bob', 25)");
    harness.execute_ok("INSERT INTO users VALUES (3, 'Charlie', 35)");
    
    let result = harness.select("SELECT * FROM users WHERE age > 28");
    
    assertions::assert_row_count(&result, 2);
    assertions::assert_cell(&result, 0, 1, &Value::Text("Alice".to_string()));
    assertions::assert_cell(&result, 1, 1, &Value::Text("Charlie".to_string()));
}

#[test]
fn test_example_insert_and_select() {
    let mut harness = TestHarness::new();
    
    harness.execute_ok("CREATE TABLE products (id INTEGER, name TEXT, price REAL)");
    harness.execute_ok("INSERT INTO products VALUES (1, 'Apple', 1.50)");
    harness.execute_ok("INSERT INTO products VALUES (2, 'Banana', 0.75)");
    
    let result = harness.select("SELECT name, price FROM products WHERE price < 1.0");
    
    assertions::assert_row_count(&result, 1);
    assertions::assert_cell(&result, 0, 0, &Value::Text("Banana".to_string()));
}

#[test]
fn test_example_update() {
    let mut harness = TestHarness::new();
    
    harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT, active INTEGER)");
    harness.execute_ok("INSERT INTO users VALUES (1, 'Alice', 1)");
    harness.execute_ok("INSERT INTO users VALUES (2, 'Bob', 0)");
    
    // Update
    let result = harness.execute_ok("UPDATE users SET active = 0 WHERE id = 1");
    assertions::assert_affected(&result, 1);
    
    // Verify update
    let result = harness.select("SELECT active FROM users WHERE id = 1");
    assertions::assert_cell(&result, 0, 0, &Value::Integer(0));
}

#[test]
fn test_example_delete() {
    let mut harness = TestHarness::new();
    
    harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
    harness.execute_ok("INSERT INTO users VALUES (1, 'Alice')");
    harness.execute_ok("INSERT INTO users VALUES (2, 'Bob')");
    
    // Delete
    let result = harness.execute_ok("DELETE FROM users WHERE id = 1");
    assertions::assert_affected(&result, 1);
    
    // Verify delete
    let result = harness.select("SELECT * FROM users");
    assertions::assert_row_count(&result, 1);
    assertions::assert_cell(&result, 0, 1, &Value::Text("Bob".to_string()));
}

#[test]
fn test_example_using_fixture() {
    // Use predefined fixture for common test scenarios
    let mut fixture = TestFixture::basic();
    
    let result = fixture.harness.select("SELECT * FROM users WHERE age > 28");
    assertions::assert_row_count(&result, 2);
}

#[test]
fn test_example_using_mock_storage() {
    // Create mock storage with predefined data
    let mut storage = MockStorage::new();
    
    // Create a users table
    let table = TestTableBuilder::new("users")
        .column("id", "INTEGER", false)
        .column("name", "TEXT", false)
        .column("age", "INTEGER", true)
        .row(vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Integer(30),
        ])
        .row(vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Integer(25),
        ])
        .build();
    
    storage.insert_table("users".to_string(), table).ok();
    
    // Verify storage contents
    assert_eq!(storage.table_count(), 1);
    assert_eq!(storage.row_count("users"), 2);
}

#[test]
fn test_example_using_test_data_generator() {
    let mut generator = TestDataGenerator::new(42);
    
    // Generate test data
    let columns = schemas::users_schema();
    let rows = generator.generate_rows(&columns, 10);
    
    assert_eq!(rows.len(), 10);
    
    // Create table with generated data
    let table = TestTableBuilder::new("generated_users")
        .column("id", "INTEGER", false)
        .column("name", "TEXT", false)
        .rows(rows)
        .build();
    
    assert_eq!(table.rows.len(), 10);
}

#[test]
fn test_example_error_cases() {
    let mut harness = TestHarness::new();
    
    // Test table not found
    let err = harness.execute_err("SELECT * FROM nonexistent");
    assert!(matches!(err, crate::types::SqlError::TableNotFound(_)));
    
    // Create table first
    harness.execute_ok("CREATE TABLE users (id INTEGER, name TEXT)");
    
    // Test insert into non-existent table
    let err = harness.execute_err("INSERT INTO nonexistent VALUES (1, 'test')");
    assert!(matches!(err, crate::types::SqlError::TableNotFound(_)));
}
