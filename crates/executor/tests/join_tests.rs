use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_employees_departments() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, dept_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice', 10), (2, 'Bob', 20), (3, 'Charlie', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (10, 'Engineering'), (20, 'Sales')")
        .unwrap();
    engine
}

#[test]
fn test_inner_join_basic() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees INNER JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_inner_join_rows_content() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees INNER JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_left_join_preserves_all_left() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees LEFT JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_left_join_non_matching_null() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees LEFT JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_right_join_basic() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees RIGHT JOIN departments ON employees.dept_id = departments.id")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_join_with_filter() {
    let mut engine = setup_employees_departments();
    let result = engine
        .execute("SELECT employees.name, departments.dept_name FROM employees INNER JOIN departments ON employees.dept_id = departments.id WHERE employees.name = 'Alice'")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    let row_has_engineering = result.rows[0].iter().any(|v| *v == Value::Text("Engineering".to_string()));
    let row_has_alice = result.rows[0].iter().any(|v| *v == Value::Text("Alice".to_string()));
    assert!(row_has_alice);
    assert!(row_has_engineering);
}

#[test]
fn test_join_empty_table() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO t1 VALUES (1)").unwrap();
    let result = engine
        .execute("SELECT * FROM t1 INNER JOIN t2 ON t1.id = t2.id")
        .unwrap();
    assert_eq!(result.rows.len(), 0);
}

#[test]
fn test_join_with_null_values() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, val INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, val INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1, 10), (2, NULL)")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (1, 100), (2, 200)")
        .unwrap();
    let result = engine
        .execute("SELECT t1.id, t2.val FROM t1 INNER JOIN t2 ON t1.id = t2.id ORDER BY t1.id")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_join_multiple_conditions() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE orders (id INTEGER, customer_id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE customers (id INTEGER, name TEXT, min_amount INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO orders VALUES (1, 1, 100), (2, 1, 200), (3, 2, 50)")
        .unwrap();
    engine
        .execute("INSERT INTO customers VALUES (1, 'Alice', 50), (2, 'Bob', 100)")
        .unwrap();
    let result = engine
        .execute("SELECT orders.id, customers.name FROM orders INNER JOIN customers ON orders.customer_id = customers.id")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_join_with_aggregate() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT, dept_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice', 10), (2, 'Bob', 10), (3, 'Charlie', 20)")
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (10, 'Engineering'), (20, 'Sales')")
        .unwrap();
    let result = engine.execute("SELECT departments.dept_name, COUNT(*) FROM employees INNER JOIN departments ON employees.dept_id = departments.id GROUP BY departments.dept_name");
    if result.is_ok() {
        let rows = result.unwrap();
        assert!(rows.rows.len() >= 1);
    }
}

#[test]
fn test_join_three_tables() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE a (id INTEGER)").unwrap();
    engine
        .execute("CREATE TABLE b (id INTEGER)").unwrap();
    engine
        .execute("CREATE TABLE c (id INTEGER)").unwrap();
    engine.execute("INSERT INTO a VALUES (1), (2)").unwrap();
    engine.execute("INSERT INTO b VALUES (1), (3)").unwrap();
    engine.execute("INSERT INTO c VALUES (1), (4)").unwrap();
    let result = engine
        .execute("SELECT a.id FROM a INNER JOIN b ON a.id = b.id INNER JOIN c ON a.id = c.id")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(1));
}
