//! Tests for MATCH...AGAINST full-text search syntax parsing

use sqlrustgo_parser::parse;
use sqlrustgo_parser::Expression;
use sqlrustgo_parser::Statement;

#[test]
fn test_parse_match_against_basic() {
    let sql = "SELECT * FROM articles WHERE MATCH(title, content) AGAINST('search query')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MATCH...AGAINST: {:?}", result);

    let stmt = result.unwrap();
    match stmt {
        Statement::Select(sel) => {
            // Verify WHERE clause contains MatchAgainst expression
            assert!(sel.where_clause.is_some(), "WHERE clause should exist");
            if let Some(expr) = &sel.where_clause {
                assert!(
                    matches!(expr, Expression::MatchAgainst(_, _)),
                    "WHERE should contain MatchAgainst expression"
                );
            }
        }
        _ => panic!("Expected SELECT statement"),
    }
}

#[test]
fn test_parse_match_against_single_column() {
    let sql = "SELECT * FROM t WHERE MATCH(title) AGAINST('hello')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse single column MATCH: {:?}", result);
}

#[test]
fn test_parse_match_against_multiple_columns() {
    let sql = "SELECT * FROM t WHERE MATCH(col1, col2, col3) AGAINST('search term')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse multi-column MATCH: {:?}", result);
}

#[test]
fn test_parse_match_against_subquery() {
    let sql = "SELECT * FROM (SELECT * FROM articles WHERE MATCH(title, content) AGAINST('sql')) AS results";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MATCH in subquery: {:?}", result);
}

#[test]
fn test_parse_match_against_derived_table() {
    let sql = "WITH recent AS (SELECT * FROM articles WHERE MATCH(title) AGAINST('news')) SELECT * FROM recent";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MATCH in CTE: {:?}", result);
}
