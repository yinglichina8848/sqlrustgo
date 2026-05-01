// Aggregate Functions Tests
// Whitebox tests for aggregate function parsing

use sqlrustgo_parser::{parse, AggregateFunction, Expression, Statement};

#[test]
fn test_parse_count_star() {
    let sql = "SELECT COUNT(*) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates.len(), 1);
            assert_eq!(select.aggregates[0].func, AggregateFunction::Count);
            assert!(select.aggregates[0].args.is_empty());
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_count_column() {
    let sql = "SELECT COUNT(id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates.len(), 1);
            assert_eq!(select.aggregates[0].func, AggregateFunction::Count);
            assert_eq!(select.aggregates[0].args.len(), 1);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_sum() {
    let sql = "SELECT SUM(amount) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates.len(), 1);
            assert_eq!(select.aggregates[0].func, AggregateFunction::Sum);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_avg() {
    let sql = "SELECT AVG(price) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates[0].func, AggregateFunction::Avg);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_min() {
    let sql = "SELECT MIN(price) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates[0].func, AggregateFunction::Min);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_max() {
    let sql = "SELECT MAX(price) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates[0].func, AggregateFunction::Max);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates.len(), 1);
            assert_eq!(select.aggregates[0].func, AggregateFunction::Count);
            assert!(select.aggregates[0].distinct);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_multiple_aggregates() {
    let sql = "SELECT COUNT(*), SUM(amount), AVG(price) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert_eq!(select.aggregates.len(), 3);
            assert_eq!(select.aggregates[0].func, AggregateFunction::Count);
            assert_eq!(select.aggregates[1].func, AggregateFunction::Sum);
            assert_eq!(select.aggregates[2].func, AggregateFunction::Avg);
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_aggregate_in_having() {
    let sql = "SELECT status FROM t GROUP BY status HAVING COUNT(*) > 1";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.having.is_some());
            match select.having.unwrap() {
                Expression::BinaryOp(left, op, _right) => {
                    assert_eq!(op, ">");
                    match *left {
                        Expression::Aggregate(agg) => {
                            assert_eq!(agg.func, AggregateFunction::Count);
                        }
                        _ => panic!("Expected Aggregate expression on left side"),
                    }
                }
                _ => panic!("Expected BinaryOp expression in HAVING"),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}
