// Unary NOT Expression Tests
// Tests for NOT operator handling

use sqlrustgo_parser::{parse, Expression, Statement};

#[test]
fn test_parse_not_exists() {
    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1 FROM other)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::NotExists(_) => {}
                other => panic!("Expected NotExists expression, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_exists() {
    let sql = "SELECT * FROM t WHERE EXISTS (SELECT 1 FROM other)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::Exists(_) => {}
                other => panic!("Expected Exists expression, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_is_null() {
    let sql = "SELECT * FROM t WHERE col IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::IsNull(_) => {}
                other => panic!("Expected IsNull expression, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_is_not_null() {
    let sql = "SELECT * FROM t WHERE col IS NOT NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::IsNotNull(_) => {}
                other => panic!("Expected IsNotNull expression, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_binary_and() {
    let sql = "SELECT * FROM t WHERE id > 5 AND name = 'test'";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::BinaryOp(_, op, _) if op == "AND" => {}
                other => panic!("Expected BinaryOp with AND, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_binary_or() {
    let sql = "SELECT * FROM t WHERE id > 5 OR name = 'test'";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::BinaryOp(_, op, _) if op == "OR" => {}
                other => panic!("Expected BinaryOp with OR, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_comparison_operators() {
    let test_cases = vec![
        ("SELECT * FROM t WHERE id = 1", "="),
        ("SELECT * FROM t WHERE id != 1", "!="),
        ("SELECT * FROM t WHERE id > 1", ">"),
        ("SELECT * FROM t WHERE id >= 1", ">="),
        ("SELECT * FROM t WHERE id < 1", "<"),
        ("SELECT * FROM t WHERE id <= 1", "<="),
    ];

    for (sql, expected_op) in test_cases {
        let result = parse(sql);
        assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);

        match result.unwrap() {
            Statement::Select(select) => {
                match select.where_clause.unwrap() {
                    Expression::BinaryOp(_, op, _) if op == expected_op => {}
                    other => panic!("Expected BinaryOp with {}, got {:?} for {}", expected_op, other, sql),
                }
            }
            _ => panic!("Expected SELECT statement for {}", sql),
        }
    }
}