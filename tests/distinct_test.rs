// DISTINCT Tests
// Whitebox tests for DISTINCT keyword parsing

use sqlrustgo_parser::{parse, Statement};

#[test]
fn test_parse_select_distinct() {
    let sql = "SELECT DISTINCT name FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.distinct, "Expected distinct to be true");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_distinct_multiple_columns() {
    let sql = "SELECT DISTINCT name, age FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.distinct);
            assert_eq!(select.columns.len(), 2);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_not_distinct() {
    let sql = "SELECT name FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(!select.distinct, "Expected distinct to be false");
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_select_distinct_with_where() {
    let sql = "SELECT DISTINCT name FROM t WHERE age > 18";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.distinct);
            assert!(select.where_clause.is_some());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_select_distinct_with_group_by() {
    let sql = "SELECT DISTINCT status FROM t GROUP BY status";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.distinct);
            assert_eq!(select.group_by.len(), 1);
        }
        _ => panic!("Expected SELECT statement"),
    }
}
