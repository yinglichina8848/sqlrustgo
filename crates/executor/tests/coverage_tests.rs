use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_aggregate_count_all() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM users").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(3));
}

#[test]
fn test_aggregate_count_column() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine.execute("SELECT COUNT(id) FROM users").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(3));
}

#[test]
fn test_aggregate_sum() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100), (2, 200), (3, 300)")
        .unwrap();

    let result = engine.execute("SELECT SUM(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(600));
}

#[test]
fn test_aggregate_min() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100), (2, 200), (3, 300)")
        .unwrap();

    let result = engine.execute("SELECT MIN(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(100));
}

#[test]
fn test_aggregate_max() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100), (2, 200), (3, 300)")
        .unwrap();

    let result = engine.execute("SELECT MAX(amount) FROM orders").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(300));
}

#[test]
fn test_aggregate_count_with_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE items (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO items VALUES (1, 100), (2, NULL), (3, 300), (4, NULL)")
        .unwrap();

    let result = engine.execute("SELECT COUNT(value) FROM items").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_aggregate_sum_with_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE items (id INTEGER, value INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO items VALUES (1, 100), (2, NULL), (3, 300)")
        .unwrap();

    let result = engine.execute("SELECT SUM(value) FROM items").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(400));
}

#[test]
fn test_limit_basic() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE numbers (n INTEGER)").unwrap();
    engine
        .execute("INSERT INTO numbers VALUES (1), (2), (3), (4), (5)")
        .unwrap();

    let result = engine.execute("SELECT * FROM numbers LIMIT 3").unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_limit_zero() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE numbers (n INTEGER)").unwrap();
    engine
        .execute("INSERT INTO numbers VALUES (1), (2), (3)")
        .unwrap();

    let result = engine.execute("SELECT * FROM numbers LIMIT 0").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_null_is_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE test (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO test VALUES (1), (NULL), (3)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM test WHERE a IS NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_null_is_not_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE test (a INTEGER)").unwrap();
    engine
        .execute("INSERT INTO test VALUES (1), (NULL), (3)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM test WHERE a IS NOT NULL")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_select_from_empty_table() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE empty_table (id INTEGER, name TEXT)")
        .unwrap();

    let result = engine.execute("SELECT * FROM empty_table").unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_aggregate_on_empty_table() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE empty_table (id INTEGER)")
        .unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM empty_table").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_simple_filter() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine.execute("SELECT * FROM users WHERE id > 1").unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_filter_with_and() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER, status TEXT)")
        .unwrap();
    engine
        .execute(
            "INSERT INTO orders VALUES (1, 100, 'active'), (2, 200, 'active'), (3, 300, 'closed')",
        )
        .unwrap();

    let result = engine
        .execute("SELECT * FROM orders WHERE amount > 100 AND status = 'active'")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_filter_with_or() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM users WHERE id = 1 OR id = 3")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_projection() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 'alice@test.com')")
        .unwrap();

    let result = engine.execute("SELECT id, name FROM users").unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_projection_with_expression() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100), (2, 200)")
        .unwrap();

    let result = engine
        .execute("SELECT id, amount * 2 AS double_amount FROM orders")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_order_by_ascending() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE numbers (n INTEGER)").unwrap();
    engine
        .execute("INSERT INTO numbers VALUES (3), (1), (2)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM numbers ORDER BY n ASC")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_order_by_descending() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE numbers (n INTEGER)").unwrap();
    engine
        .execute("INSERT INTO numbers VALUES (3), (1), (2)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM numbers ORDER BY n DESC")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_inner_join() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, dept_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute(
            "INSERT INTO employees VALUES (1, 'Alice', 10), (2, 'Bob', 20), (3, 'Charlie', 30)",
        )
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (10, 'Engineering'), (20, 'Sales')")
        .unwrap();

    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees INNER JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_left_join() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, dept_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute(
            "INSERT INTO employees VALUES (1, 'Alice', 10), (2, 'Bob', 20), (3, 'Charlie', 30)",
        )
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (10, 'Engineering'), (20, 'Sales')")
        .unwrap();

    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees LEFT JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}
