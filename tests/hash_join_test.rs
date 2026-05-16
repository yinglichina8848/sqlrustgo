#![allow(deprecated)]

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_inner_join_hash() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, dept_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice', 10), (2, 'Bob', 20), (3, 'Carol', 10)")
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (10, 'Engineering'), (20, 'Sales')")
        .unwrap();

    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees INNER JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_left_join_hash() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, customer_id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE customers (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 100, 500), (2, 101, 300), (3, 100, 200)")
        .unwrap();
    engine
        .execute("INSERT INTO customers VALUES (100, 'Alice'), (101, 'Bob')")
        .unwrap();

    let result = engine
        .execute("SELECT orders.id, customers.name FROM orders LEFT JOIN customers ON orders.customer_id = customers.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}
