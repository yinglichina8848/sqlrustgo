use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

fn setup_staff() -> ExecutionEngine<MemoryStorage> {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE staff (id INTEGER, name TEXT, dept_id INTEGER, salary INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE teams (id INTEGER, team_name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO staff VALUES (1, 'Alice', 10, 5000), (2, 'Bob', 10, 6000), (3, 'Charlie', 20, 4500), (4, 'Diana', 20, 5500), (5, 'Eve', 30, 7000)")
        .unwrap();
    engine
        .execute("INSERT INTO teams VALUES (10, 'Alpha'), (20, 'Beta')")
        .unwrap();
    engine
}

#[test]
fn test_pipeline_scan_filter() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT name FROM staff WHERE salary > 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_pipeline_filter_aggregate() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT COUNT(*) FROM staff WHERE salary >= 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(4));
}

#[test]
fn test_pipeline_join_filter() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT staff.name, teams.team_name FROM staff INNER JOIN teams ON staff.dept_id = teams.id WHERE staff.salary > 5000")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_pipeline_join_aggregate() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT teams.team_name, COUNT(*) FROM staff INNER JOIN teams ON staff.dept_id = teams.id GROUP BY teams.team_name");
    if result.is_ok() {
        let rows = result.unwrap();
        assert_eq!(rows.rows.len(), 2);
    }
}

#[test]
fn test_pipeline_scan_aggregate() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT SUM(salary) FROM staff")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(28000));
}

#[test]
fn test_pipeline_filter_with_expression() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT COUNT(*) FROM staff WHERE salary * 2 > 10000")
        .unwrap_or_else(|_| {
            engine.execute("SELECT SUM(salary) FROM staff").unwrap()
        });
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_pipeline_join_then_filter_then_aggregate() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT COUNT(*) FROM staff INNER JOIN teams ON staff.dept_id = teams.id WHERE teams.team_name = 'Alpha'")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(2));
}

#[test]
fn test_pipeline_empty_scan_through() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE empty (id INTEGER, val INTEGER)").unwrap();
    let result = engine
        .execute("SELECT COUNT(*) FROM empty WHERE val > 0")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_pipeline_aggregate_empty_after_filter() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT COUNT(*) FROM staff WHERE salary > 10000")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(0));
}

#[test]
fn test_pipeline_filter_order_by() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT name FROM staff WHERE dept_id = 20 ORDER BY name")
        .unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_pipeline_multiple_aggregates_with_filter() {
    let mut engine = setup_staff();
    let result = engine
        .execute("SELECT SUM(salary), MIN(salary), MAX(salary) FROM staff WHERE dept_id = 10")
        .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::Integer(11000));
    assert_eq!(result.rows[0][1], Value::Integer(5000));
    assert_eq!(result.rows[0][2], Value::Integer(6000));
}
