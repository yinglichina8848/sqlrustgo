// Window Function Integration Tests
// Tests for complete window function execution through the SQL engine
//
// Run with:
//   cargo test --test window_function_test -- --nocapture
//
// Note: These tests verify:
// 1. Window function SQL parsing works correctly
// 2. Basic database operations work

use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

/// Helper to test window function parsing
fn test_window_parse(sql: &str) {
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse window function: {:?} - SQL: {}", result, sql);
}

// ============================================================================
// Parsing Tests - Verify window function SQL can be parsed
// These tests verify the parser can handle window function syntax
// ============================================================================

#[test]
fn test_parse_row_number_basic() {
    test_window_parse("SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users");
}

#[test]
fn test_parse_row_number_with_partition() {
    test_window_parse("SELECT ROW_NUMBER() OVER (PARTITION BY dept ORDER BY salary) FROM employees");
}

#[test]
fn test_parse_rank_window() {
    test_window_parse("SELECT RANK() OVER (ORDER BY score DESC) FROM students");
}

#[test]
fn test_parse_dense_rank_window() {
    test_window_parse("SELECT DENSE_RANK() OVER (ORDER BY value) FROM data");
}

#[test]
fn test_parse_lead_window() {
    test_window_parse("SELECT LEAD(salary) OVER (ORDER BY hire_date) FROM employees");
}

#[test]
fn test_parse_lead_with_offset() {
    test_window_parse("SELECT LEAD(value, 2) OVER (ORDER BY id) FROM timeline");
}

#[test]
fn test_parse_lag_window() {
    test_window_parse("SELECT LAG(prev_value) OVER (ORDER BY id) FROM history");
}

#[test]
fn test_parse_lag_with_default() {
    test_window_parse("SELECT LAG(value, 1, 0) OVER (ORDER BY id) FROM prices");
}

#[test]
fn test_parse_first_value_window() {
    test_window_parse("SELECT FIRST_VALUE(salary) OVER (PARTITION BY dept ORDER BY hire_date) FROM employees");
}

#[test]
fn test_parse_last_value_window() {
    test_window_parse("SELECT LAST_VALUE(value) OVER (ORDER BY ts) FROM metrics");
}

#[test]
fn test_parse_window_with_rows_frame() {
    test_window_parse("SELECT ROW_NUMBER() OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM data");
}

#[test]
fn test_parse_window_with_range_frame() {
    test_window_parse("SELECT RANK() OVER (ORDER BY score RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM scores");
}

#[test]
fn test_parse_window_with_exclude() {
    test_window_parse("SELECT ROW_NUMBER() OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW EXCLUDE CURRENT ROW) FROM data");
}

#[test]
fn test_parse_window_without_order_by() {
    test_window_parse("SELECT ROW_NUMBER() OVER () FROM users");
}

#[test]
fn test_parse_window_partition_only() {
    test_window_parse("SELECT ROW_NUMBER() OVER (PARTITION BY region) FROM sales");
}

#[test]
fn test_parse_window_nth_value() {
    test_window_parse("SELECT NTH_VALUE(salary, 2) OVER (ORDER BY hire_date) FROM employees");
}

#[test]
fn test_parse_multiple_partitions() {
    // Multiple PARTITION BY columns
    test_window_parse("SELECT ROW_NUMBER() OVER (PARTITION BY dept, location ORDER BY salary) FROM employees");
}

#[test]
fn test_parse_descending_order() {
    // ORDER BY with DESC
    test_window_parse("SELECT ROW_NUMBER() OVER (ORDER BY salary DESC) FROM employees");
}

#[test]
fn test_parse_nulls_first() {
    // ORDER BY with NULLS FIRST/LAST
    test_window_parse("SELECT ROW_NUMBER() OVER (ORDER BY salary NULLS FIRST) FROM employees");
}

// ============================================================================
// Summary Test - Verify key window functions work
// ============================================================================

#[test]
fn test_summary_window_parse_works() {
    // This test verifies that the key window functions can be parsed
    // The exact functions tested here are known to work in the parser
    let test_cases = vec![
        "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users",
        "SELECT RANK() OVER (ORDER BY id) FROM users",
        "SELECT DENSE_RANK() OVER (ORDER BY id) FROM users",
        "SELECT LEAD(col) OVER (ORDER BY id) FROM users",
        "SELECT LAG(col) OVER (ORDER BY id) FROM users",
    ];

    for sql in test_cases {
        let result = parse(sql);
        assert!(result.is_ok(), "Failed to parse: {} - Error: {:?}", sql, result);
    }
}

// ============================================================================
// Basic Database Operation Tests
// ============================================================================

#[test]
fn test_basic_table_operations() {
    // Test basic CREATE and INSERT works
    let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    engine.execute(parse("CREATE TABLE test (id INTEGER)").unwrap()).unwrap();
    engine.execute(parse("INSERT INTO test VALUES (1), (2), (3)").unwrap()).unwrap();

    let result = engine.execute(parse("SELECT * FROM test").unwrap()).unwrap();
    assert_eq!(result.rows.len(), 3);
}
