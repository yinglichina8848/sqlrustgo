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

#[test]
#[ignore = "Parser does not support NOT (expr) syntax - NOT implementation is Phase 2"]
fn test_filter_not_null_comparison() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 'Alice'), (NULL, 'Bob'), (3, 'Charlie')")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE NOT (id = NULL)")
        .unwrap();

    assert_eq!(result.rows.len(), 0, "NOT(UNKNOWN) should be UNKNOWN, not TRUE");
}

#[test]
fn test_filter_and_with_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (a INTEGER, b INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (1, 10), (NULL, 20), (3, NULL), (NULL, NULL)")
        .unwrap();

    let result = engine
        .execute("SELECT * FROM t WHERE a > 10 AND b = NULL")
        .unwrap();

    assert_eq!(result.rows.len(), 0, "TRUE AND UNKNOWN should filter out");
}

// semantic_guard: join_null_filter
// Tests JOIN + IS NULL combination - locks NULL handling across join and filter paths
#[test]
fn test_join_with_is_null_filter() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER, data TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1), (2), (3)")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (2, 'B')")
        .unwrap();

    // t1 LEFT JOIN t2: rows 1, 2, 3 (row 1 and 3 have NULL t2.id)
    // WHERE t2.id IS NULL: only rows where t2.id is NULL (rows 1, 3)
    let result = engine
        .execute(
            "SELECT t1.id FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NULL",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 2);
    let ids: Vec<i64> = result
        .rows
        .iter()
        .map(|r| {
            if let Value::Integer(i) = r[0] {
                i
            } else {
                panic!("Expected Integer")
            }
        })
        .collect();
    assert!(ids.contains(&1));
    assert!(ids.contains(&3));
}

// semantic_guard: filter_join_combination
// Tests filter + JOIN combination with NULL handling in both paths
#[test]
fn test_filter_with_null_and_join() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t1 (id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE t2 (id INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t1 VALUES (1), (2), (3), (NULL)")
        .unwrap();
    engine
        .execute("INSERT INTO t2 VALUES (2), (3), (NULL)")
        .unwrap();

    // t1 LEFT JOIN t2 ON t1.id = t2.id
    // WHERE t1.id > 1 AND t2.id IS NULL
    // t1 rows: 2, 3 match t2; NULL and 1 don't match
    // After join: 2, 3 have matches; NULL and 1 have NULL t2.id
    // WHERE t1.id > 1: keeps 2, 3
    // WHERE t2.id IS NULL: keeps rows where t2.id is NULL
    // Result: 1 row (t1.id=3, t2.id=NULL because 3 doesn't match t2's 3)
    // Wait - t1.id=3 has t2.id=3 match, not NULL
    // So result should be: t1.id=1 with t2.id=NULL
    let result = engine
        .execute(
            "SELECT t1.id, t2.id FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t1.id > 1 AND t2.id IS NULL",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 0, "t1.id=2,3 matched t2, only t1.id=1 has NULL t2.id but fails t1.id > 1");
}

// semantic_guard: aggregate_null
// Tests COUNT with NULL - locks aggregate NULL handling strategy
#[test]
fn test_count_with_null() {
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE t (val INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO t VALUES (NULL), (10), (20)")
        .unwrap();

    let result_star = engine
        .execute("SELECT COUNT(*) FROM t")
        .unwrap();

    let count_star = if let Value::Integer(i) = result_star.rows[0][0] {
        i
    } else {
        panic!("Expected Integer for COUNT(*)")
    };
    assert_eq!(count_star, 3, "COUNT(*) should count all rows including NULL");

    let result_val = engine
        .execute("SELECT COUNT(val) FROM t")
        .unwrap();

    let count_val = if let Value::Integer(i) = result_val.rows[0][0] {
        i
    } else {
        panic!("Expected Integer for COUNT(val)")
    };
    assert_eq!(count_val, 2, "COUNT(col) should ignore NULL");
}

// Top 5 Semantic Risk Tests (Phase 1 Completeness)

// Risk 1: JOIN + WHERE + AGGREGATE combination
// Tests: SELECT COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NOT NULL
#[test]
#[ignore = "execute_select_with_join does not handle aggregates - known gap"]
fn test_semantic_join_where_aggregate() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
    engine.execute("INSERT INTO t2 VALUES (2), (3)").unwrap();

    let result = engine
        .execute("SELECT COUNT(t2.id) FROM t1 LEFT JOIN t2 ON t1.id = t2.id WHERE t2.id IS NOT NULL")
        .unwrap();

    assert_eq!(result.rows.len(), 1);
    let count = if let Value::Integer(i) = result.rows[0][0] {
        i
    } else {
        panic!("Expected Integer")
    };
    assert_eq!(count, 2, "JOIN+WHERE+Aggregate: t2.id=2,3 matched and pass IS NOT NULL");
}

// Risk 2: HAVING with GROUP BY (schema mismatch check)
#[test]
fn test_semantic_having_with_group_by() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (dept INTEGER, val INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (1, 10), (1, 20), (2, 30)").unwrap();

    let result = engine
        .execute("SELECT dept, COUNT(*) as cnt FROM t GROUP BY dept HAVING COUNT(*) > 1")
        .unwrap();

    assert_eq!(result.rows.len(), 1);
    let dept = if let Value::Integer(i) = result.rows[0][0] {
        i
    } else {
        panic!("Expected Integer for dept")
    };
    assert_eq!(dept, 1, "Only dept=1 has COUNT(*) > 1 (2 rows)");
}

// Risk 3: Aggregate with all NULL values
#[test]
fn test_semantic_aggregate_all_null() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (val INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();

    let result = engine.execute("SELECT SUM(val), AVG(val), MIN(val), MAX(val), COUNT(val) FROM t").unwrap();

    assert_eq!(result.rows.len(), 1);
    // All aggregates of all-NULL should return NULL
    assert!(matches!(result.rows[0][0], Value::Null), "SUM of all NULL should be NULL");
    assert!(matches!(result.rows[0][1], Value::Null), "AVG of all NULL should be NULL");
    assert!(matches!(result.rows[0][2], Value::Null), "MIN of all NULL should be NULL");
    assert!(matches!(result.rows[0][3], Value::Null), "MAX of all NULL should be NULL");
    assert_eq!(result.rows[0][4], Value::Integer(0), "COUNT of all NULL should be 0");
}

// Risk 4: GROUP BY with NULL keys
#[test]
fn test_semantic_group_by_null_key() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (k INTEGER, v INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (NULL, 1), (NULL, 2), (1, 3)").unwrap();

    let result = engine.execute("SELECT k, COUNT(*) FROM t GROUP BY k").unwrap();

    assert_eq!(result.rows.len(), 2);
    // Should have: NULL group (count=2), 1 group (count=1)
    let has_null_group = result.rows.iter().any(|r| matches!(r[0], Value::Null) && r[1] == Value::Integer(2));
    let has_one_group = result.rows.iter().any(|r| r[0] == Value::Integer(1) && r[1] == Value::Integer(1));
    assert!(has_null_group, "Should have NULL group with count=2");
    assert!(has_one_group, "Should have group k=1 with count=1");
}

// Risk 5: Filter + Aggregate order (WHERE applied before aggregate)
#[test]
fn test_semantic_filter_before_aggregate() {
    let mut engine = create_engine();
    engine.execute("CREATE TABLE t (val INTEGER)").unwrap();
    engine.execute("INSERT INTO t VALUES (5), (15), (25), (35)").unwrap();

    let result = engine.execute("SELECT COUNT(*) FROM t WHERE val > 10").unwrap();

    assert_eq!(result.rows.len(), 1);
    let count = if let Value::Integer(i) = result.rows[0][0] {
        i
    } else {
        panic!("Expected Integer")
    };
    // val > 10: 15, 25, 35 = 3 rows
    assert_eq!(count, 3, "COUNT after WHERE filter should count filtered rows only");
}
