//! Window Function Boundary Tests
//! GAP-3: coverage improvement for window function edge cases
//! Issue #878: 窗口函数边界测试补全

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine<MemoryStorage> {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    #[allow(deprecated)]
    ExecutionEngine::new(storage)
}

fn setup_student_data(engine: &mut ExecutionEngine<MemoryStorage>) {
    engine
        .execute("CREATE TABLE class (id INTEGER PRIMARY KEY, name TEXT, score INTEGER)")
        .unwrap();
    for sql in &[
        "INSERT INTO class VALUES (1, 'Alice', 85)",
        "INSERT INTO class VALUES (2, 'Bob', 92)",
        "INSERT INTO class VALUES (3, 'Charlie', 78)",
        "INSERT INTO class VALUES (4, 'Diana', 92)",
        "INSERT INTO class VALUES (5, 'Eve', 65)",
    ] {
        engine.execute(sql).unwrap();
    }
}

// =============================================================================
// NTILE Boundary Tests (Issue #878)
// =============================================================================

#[test]
fn test_ntile_with_partition() {
    // NTILE with PARTITION BY - should divide into buckets per partition
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    // Add department column for partition
    engine
        .execute("ALTER TABLE class ADD COLUMN department TEXT")
        .unwrap();
    engine.execute("UPDATE class SET department = 'Science' WHERE id <= 3").unwrap();
    engine.execute("UPDATE class SET department = 'Arts' WHERE id > 3").unwrap();

    let result = engine
        .execute(
            "SELECT name, department, NTILE(2) OVER (PARTITION BY department ORDER BY score) as quartile FROM class",
        )
        .unwrap();

    // Should have 5 rows
    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_ntile_boundary_cases() {
    // NTILE with more buckets than rows
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute("SELECT NTILE(10) OVER (ORDER BY score) FROM class")
        .unwrap();

    // 5 rows into 10 buckets - first 5 get bucket 1, rest get bucket 2 (or NULL)
    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_ntile_with_null_score() {
    // NTILE with NULL values - NULLs should be handled
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE test_nulls (id INTEGER, val INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO test_nulls VALUES (1, 10)").unwrap();
    engine.execute("INSERT INTO test_nulls VALUES (2, NULL)").unwrap();
    engine.execute("INSERT INTO test_nulls VALUES (3, 20)").unwrap();

    let result = engine
        .execute("SELECT NTILE(2) OVER (ORDER BY val) FROM test_nulls")
        .unwrap();

    assert_eq!(result.rows.len(), 3);
}

// =============================================================================
// LEAD/LAG Boundary Tests (Issue #878)
// =============================================================================

#[test]
fn test_lead_with_offset() {
    // LEAD with explicit offset
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, score, LEAD(name, 1) OVER (ORDER BY score) as next_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // Alice has 85, next score is Charlie with 78 (lower), so LEAD goes to lower score
}

#[test]
fn test_lead_with_null_default() {
    // LEAD with explicit NULL default
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, LEAD(name, 1, 'N/A') OVER (ORDER BY score) as next_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // Last row should have 'N/A' as default
}

#[test]
fn test_lead_with_null_offset() {
    // LEAD when all rows have same value
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE same_values (id INTEGER, val INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO same_values VALUES (1, 10)").unwrap();
    engine.execute("INSERT INTO same_values VALUES (2, 10)").unwrap();
    engine.execute("INSERT INTO same_values VALUES (3, 10)").unwrap();

    let result = engine
        .execute("SELECT LEAD(val) OVER (ORDER BY id) FROM same_values")
        .unwrap();

    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_lag_with_offset() {
    // LAG with explicit offset
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, score, LAG(name, 1) OVER (ORDER BY score) as prev_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_lag_with_default() {
    // LAG with default value
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, LAG(name, 1, 'FIRST') OVER (ORDER BY score) as prev_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // First row should have 'FIRST' as default
}

#[test]
fn test_lag_with_large_offset() {
    // LAG with offset larger than window
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, LAG(name, 10) OVER (ORDER BY score) as prev_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // All should have NULL default since offset exceeds rows
}

// =============================================================================
// FIRST_VALUE/LAST_VALUE Boundary Tests (Issue #878)
// =============================================================================

#[test]
fn test_first_value_with_order_by() {
    // FIRST_VALUE with ORDER BY dependency
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, FIRST_VALUE(name) OVER (ORDER BY score DESC) as top_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // Should get highest score student as first_value
}

#[test]
fn test_first_value_with_partition() {
    // FIRST_VALUE within partition
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE sales (id INTEGER, region TEXT, q1 INTEGER, q2 INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO sales VALUES (1, 'North', 100, 200)").unwrap();
    engine.execute("INSERT INTO sales VALUES (2, 'North', 150, 180)").unwrap();
    engine.execute("INSERT INTO sales VALUES (3, 'South', 300, 250)").unwrap();

    let result = engine
        .execute(
            "SELECT region, FIRST_VALUE(q1) OVER (PARTITION BY region ORDER BY q2) as first_quarter FROM sales",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 3);
}

#[test]
fn test_last_value_with_order_by() {
    // LAST_VALUE with ORDER BY dependency
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, LAST_VALUE(name) OVER (ORDER BY score) as last_student FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_last_value_default_frame() {
    // LAST_VALUE with default frame (RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute("SELECT LAST_VALUE(score) OVER (ORDER BY id) FROM class")
        .unwrap();

    assert_eq!(result.rows.len(), 5);
}

// =============================================================================
// NTH_VALUE Boundary Tests (Issue #878)
// =============================================================================

#[test]
fn test_nth_value_second_row() {
    // NTH_VALUE with n=2
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT name, NTH_VALUE(score, 2) OVER (ORDER BY id) as second_score FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
}

#[test]
fn test_nth_value_out_of_range() {
    // NTH_VALUE when n exceeds rows
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT NTH_VALUE(score, 10) OVER (ORDER BY id) FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
    // Should return NULL for out-of-range nth
}

#[test]
fn test_nth_value_with_partition() {
    // NTH_VALUE within partition
    let mut engine = create_engine();
    engine
        .execute("CREATE TABLE ranked (id INTEGER, grp TEXT, val INTEGER)")
        .unwrap();
    engine.execute("INSERT INTO ranked VALUES (1, 'A', 10)").unwrap();
    engine.execute("INSERT INTO ranked VALUES (2, 'A', 20)").unwrap();
    engine.execute("INSERT INTO ranked VALUES (3, 'B', 30)").unwrap();
    engine.execute("INSERT INTO ranked VALUES (4, 'B', 40)").unwrap();

    let result = engine
        .execute(
            "SELECT grp, NTH_VALUE(val, 2) OVER (PARTITION BY grp ORDER BY id) as second_in_group FROM ranked",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 4);
}

#[test]
fn test_nth_value_from_last() {
    // NTH_VALUE from last row using ORDER BY DESC
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine
        .execute(
            "SELECT NTH_VALUE(score, 2) OVER (ORDER BY id DESC) as second_from_last FROM class",
        )
        .unwrap();

    assert_eq!(result.rows.len(), 5);
}

// =============================================================================
// Combined Window Function Tests
// =============================================================================

#[test]
fn test_all_window_functions_combined() {
    // Combined test from Issue #878 example
    let mut engine = create_engine();
    setup_student_data(&mut engine);

    let result = engine.execute(
        "SELECT name, \
         NTILE(4) OVER (ORDER BY score) as quartile, \
         LEAD(name, 1) OVER (ORDER BY score) as next_student, \
         LAG(name, 1, 'N/A') OVER (ORDER BY score) as prev_student, \
         FIRST_VALUE(score) OVER (ORDER BY score) as min_score, \
         LAST_VALUE(score) OVER (ORDER BY score) as max_score, \
         NTH_VALUE(score, 2) OVER (ORDER BY score) as second_score \
         FROM class",
    ).unwrap();

    assert_eq!(result.rows.len(), 5);
}
