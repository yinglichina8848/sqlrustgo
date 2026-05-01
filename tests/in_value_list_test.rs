// IN Value List Expression Tests
// Whitebox tests for MySQL IN (val1, val2, ...) expression parsing

use sqlrustgo_parser::{parse, Expression, Statement};

#[test]
fn test_parse_in_list_integers_in_where() {
    let sql = "SELECT * FROM t WHERE id IN (1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed for {}: {:?}", sql, result);

    match result.unwrap() {
        Statement::Select(select) => {
            assert!(select.where_clause.is_some());
            match select.where_clause.unwrap() {
                Expression::InList(left, values) => {
                    assert!(matches!(*left, Expression::Identifier(ref s) if s == "id"));
                    assert_eq!(values.len(), 3);
                }
                other => panic!("Expected InList expression, got {:?}", other),
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_in_list_strings_in_where() {
    let sql = "SELECT * FROM t WHERE name IN ('alice', 'bob')";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => match select.where_clause.unwrap() {
            Expression::InList(_, values) => {
                assert_eq!(values.len(), 2);
            }
            _ => panic!("Expected InList expression"),
        },
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_in_list_mixed_in_where() {
    let sql = "SELECT * FROM t WHERE status IN (1, 2, 'unknown')";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}

#[test]
fn test_parse_not_in_list_in_where() {
    let sql = "SELECT * FROM t WHERE id NOT IN (1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => match select.where_clause.unwrap() {
            Expression::NotInList(_, values) => {
                assert_eq!(values.len(), 3);
            }
            _ => panic!("Expected NotInList expression"),
        },
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_in_list_single_value_in_where() {
    let sql = "SELECT * FROM t WHERE id IN (1)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => match select.where_clause.unwrap() {
            Expression::InList(_, values) => {
                assert_eq!(values.len(), 1);
            }
            _ => panic!("Expected InList expression"),
        },
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_in_list_subquery_in_where() {
    let sql = "SELECT * FROM t WHERE id IN (SELECT id FROM other)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);

    match result.unwrap() {
        Statement::Select(select) => match select.where_clause.unwrap() {
            Expression::In(_, _) => {}
            other => panic!("Expected In expression with subquery, got {:?}", other),
        },
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_in_list_in_having() {
    let sql = "SELECT status, COUNT(*) FROM t GROUP BY status HAVING COUNT(*) IN (1, 2)";
    let result = parse(sql);
    assert!(result.is_ok(), "Parse failed: {:?}", result);
}
