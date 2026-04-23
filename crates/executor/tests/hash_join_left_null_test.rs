use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_left_join_preserves_all_left_rows() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (1, 'Engineering'), (2, 'Sales')")
        .unwrap();

    let result = engine
        .execute(
            "SELECT employees.id, employees.name, departments.id, departments.dept_name FROM employees LEFT JOIN departments ON employees.id = departments.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 3);

    let has_alice = result
        .rows
        .iter()
        .any(|r| r[1] == Value::Text("Alice".to_string()));
    let has_bob = result
        .rows
        .iter()
        .any(|r| r[1] == Value::Text("Bob".to_string()));
    let has_charlie = result
        .rows
        .iter()
        .any(|r| r[1] == Value::Text("Charlie".to_string()));

    assert!(has_alice);
    assert!(has_bob);
    assert!(has_charlie);
}

#[test]
fn test_left_join_non_matching_rows_have_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE departments (id INTEGER, dept_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
        .unwrap();
    engine
        .execute("INSERT INTO departments VALUES (1, 'Engineering'), (2, 'Sales')")
        .unwrap();

    let result = engine
        .execute(
            "SELECT employees.id, employees.name, departments.id, departments.dept_name FROM employees LEFT JOIN departments ON employees.id = departments.id",
        )
        .unwrap();

    let charlie_row = result
        .rows
        .iter()
        .find(|row| row[1] == Value::Text("Charlie".to_string()));

    assert!(charlie_row.is_some());

    if let Some(row) = charlie_row {
        assert!(matches!(&row[2], Value::Null));
        assert!(matches!(&row[3], Value::Null));
    }
}

#[test]
fn test_left_join_no_matches_returns_all_left_with_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE left_table (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE right_table (id INTEGER, data TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO left_table VALUES (1, 'A'), (2, 'B')")
        .unwrap();
    engine
        .execute("INSERT INTO right_table VALUES (999, 'X'), (998, 'Y')")
        .unwrap();

    let result = engine
        .execute(
            "SELECT left_table.id, left_table.value, right_table.id, right_table.data FROM left_table LEFT JOIN right_table ON left_table.id = right_table.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 2);

    for row in &result.rows {
        assert!(matches!(&row[2], Value::Null));
        assert!(matches!(&row[3], Value::Null));
    }
}

#[test]
fn test_left_join_null_keys_do_not_match() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (NULL, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (NULL, 'Engineering')")
        .unwrap();

    let result = engine
        .execute("SELECT t1.id, t1.name, t2.id, t2.value FROM t1 LEFT JOIN t2 ON t1.id = t2.id")
        .unwrap();

    assert_eq!(result.rows.len(), 1);

    let row = &result.rows[0];
    assert!(matches!(&row[0], Value::Null));
    assert!(matches!(&row[2], Value::Null));
    assert!(matches!(&row[3], Value::Null));
}

#[test]
fn test_left_join_multiple_right_matches() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE employees (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE skills (employee_id INTEGER, skill_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO employees VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO skills VALUES (1, 'Rust'), (1, 'Go'), (1, 'Python')")
        .unwrap();

    let result = engine
        .execute(
            "SELECT employees.id, employees.name, skills.employee_id, skills.skill_name FROM employees LEFT JOIN skills ON employees.id = skills.employee_id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 3);

    for row in &result.rows {
        assert_eq!(row[0], Value::Integer(1));
        assert_eq!(row[1], Value::Text("Alice".to_string()));
    }
}

#[test]
fn test_left_join_mixed_null_and_normal_keys() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, value TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (NULL, 'Alice'), (10, 'Bob')")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (NULL, 'X'), (10, 'Y')")
        .unwrap();

    let result = engine
        .execute(
            "SELECT t1.id, t1.name, t2.id, t2.value FROM t1 LEFT JOIN t2 ON t1.id = t2.id",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 2);

    let null_row = result.rows.iter().find(|r| matches!(&r[1], Value::Text(s) if s == "Alice"));
    let normal_row = result.rows.iter().find(|r| matches!(&r[1], Value::Text(s) if s == "Bob"));

    assert!(null_row.is_some(), "Alice row should exist");
    assert!(normal_row.is_some(), "Bob row should exist");

    if let Some(row) = null_row {
        assert!(matches!(&row[0], Value::Null));
        assert!(matches!(&row[2], Value::Null));
        assert!(matches!(&row[3], Value::Null));
    }

    if let Some(row) = normal_row {
        assert_eq!(row[0], Value::Integer(10));
        assert_eq!(row[2], Value::Integer(10));
        assert_eq!(row[3], Value::Text("Y".to_string()));
    }
}

#[test]
#[ignore = "Filter NULL semantics not yet implemented - see issue #1833"]
fn test_filter_with_null_comparison() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'Alice'), (NULL, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE id = NULL")
        .unwrap();

    assert_eq!(result.rows.len(), 0, "WHERE col = NULL should return 0 rows");
}

#[test]
fn test_filter_null_column_vs_value() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 10), (NULL, 20), (3, NULL), (NULL, NULL)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE a = 1")
        .unwrap();

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(1));
    assert_eq!(result.rows[0][1], Value::Integer(10));
}

#[test]
#[ignore = "Filter NULL semantics not yet implemented - see issue #1833"]
fn test_filter_is_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'Alice'), (NULL, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE id IS NULL")
        .unwrap();

    assert_eq!(result.rows.len(), 1);
    assert!(matches!(&result.rows[0][0], Value::Null));
    assert_eq!(result.rows[0][1], Value::Text("Bob".to_string()));
}

#[test]
fn test_filter_is_not_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'Alice'), (NULL, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE id IS NOT NULL")
        .unwrap();

    assert_eq!(result.rows.len(), 2);
    for row in &result.rows {
        assert!(matches!(&row[0], Value::Integer(_)));
    }
}
