

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_employees() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, salary INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice', 5000), (2, 'Bob', 6000), (3, 'Charlie', 4500), (4, 'Diana', 5500), (5, 'Eve', 7000)")
        .unwrap();
    engine
}

fn setup_nulls() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE items (id INTEGER, value INTEGER, tag TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO items VALUES (1, 10, 'a'), (2, NULL, 'b'), (3, 30, NULL), (4, NULL, NULL)")
        .unwrap();
    engine
}

#[test]
fn test_filter_basic_equality() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id, name FROM employees WHERE id = 2")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
    assert_eq!(result.rows[0][1], Value::Text("Bob".to_string()));
}

#[test]
fn test_filter_greater_than() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE salary > 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&2));
    assert!(ids.contains(&4));
    assert!(ids.contains(&5));
}

#[test]
fn test_filter_less_than_or_equal() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE salary <= 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
}

#[test]
fn test_filter_and_condition() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id, name FROM employees WHERE salary > 4500 AND salary < 6000")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let names: Vec<&str> = result
        .rows
        .iter()
        .map(|r| match &r[1] {
            Value::Text(s) => s.as_str(),
            _ => panic!("expected text"),
        })
        .collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Diana"));
}

#[test]
fn test_filter_or_condition() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE id = 1 OR id = 4")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&4));
}

#[test]
fn test_filter_combined_and_or() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE (id = 1 OR id = 2) AND salary >= 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
}

#[test]
fn test_filter_all_rows_match() {
    let mut engine = setup_employees();
    let result = engine.execute("SELECT id FROM employees WHERE 1 = 1").unwrap();
    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_filter_no_rows_match() {
    let mut engine = setup_employees();
    let result = engine.execute("SELECT id FROM employees WHERE 1 = 0").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_null_equals_null_zero_rows() {
        let mut engine = setup_nulls();
    let result = engine
        .execute("SELECT id FROM items WHERE value = NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_null_not_equals_null_zero_rows() {
        let mut engine = setup_nulls();
    let result = engine
        .execute("SELECT id FROM items WHERE value <> NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_is_null() {
    let mut engine = setup_nulls();
    let result = engine.execute("SELECT id FROM items WHERE value IS NULL").unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&2));
    assert!(ids.contains(&4));
}

#[test]
fn test_filter_is_not_null() {
    let mut engine = setup_nulls();
    let result = engine
        .execute("SELECT id FROM items WHERE value IS NOT NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
}

#[test]
fn test_filter_text_is_null() {
    let mut engine = setup_nulls();
    let result = engine.execute("SELECT id FROM items WHERE tag IS NULL").unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&3));
    assert!(ids.contains(&4));
}

#[test]
fn test_filter_column_compared_to_null() {
        let mut engine = setup_nulls();
    let result = engine
        .execute("SELECT id FROM items WHERE tag = NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_on_empty_table() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE empty (id INTEGER)")
        .unwrap();
    let result = engine
        .execute("SELECT id FROM empty WHERE id > 0")
        .unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_filter_with_order_by_count() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT name FROM employees WHERE salary > 5000 ORDER BY name")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_filter_not_equals() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE id <> 2")
        .unwrap();
    assert_eq!(result.rows.len(), 4);
    for row in &result.rows {
        assert_ne!(row[0], Value::Integer(2));
    }
}

#[test]
fn test_filter_multiple_columns() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE salary > 5000 AND (id = 2 OR id = 5)")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| match r[0] {
            Value::Integer(v) => v,
            _ => panic!("expected integer"),
        })
        .collect();
    assert!(ids.contains(&2));
    assert!(ids.contains(&5));
}

#[test]
fn test_filter_name_equality() {
    let mut engine = setup_employees();
    let result = engine
        .execute("SELECT id FROM employees WHERE name = 'Bob'")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
}
