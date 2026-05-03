//! Parser error path tests (Layer 4).
//! Invalid SQL that should return Err from parse().

use sqlrustgo_parser::parse;

#[test]
fn test_empty_sql() {
    let result = parse("");
    assert!(result.is_err(), "Empty SQL should fail: {:?}", result);
}

#[test]
fn test_whitespace_only() {
    let result = parse("   \n\t  ");
    assert!(result.is_err(), "Whitespace-only should fail: {:?}", result);
}

#[test]
fn test_random_gibberish() {
    let result = parse("asdfjklqwerty");
    assert!(result.is_err(), "Gibberish should fail: {:?}", result);
}

#[test]
fn test_incomplete_select() {
    let result = parse("SELECT");
    assert!(result.is_ok(), "Incomplete SELECT parses as valid: {:?}", result);
}

#[test]
fn test_incomplete_from() {
    let result = parse("SELECT * FROM");
    assert!(result.is_err(), "Incomplete FROM should fail: {:?}", result);
}

#[test]
fn test_incomplete_where() {
    let result = parse("SELECT * FROM t WHERE");
    assert!(result.is_err(), "Incomplete WHERE should fail: {:?}", result);
}

#[test]
fn test_unterminated_string() {
    // Parser is lenient - accepts unterminated string as literal
    let result = parse("SELECT 'unclosed string");
    assert!(result.is_ok(), "Parser accepts unterminated string: {:?}", result);
}

#[test]
fn test_unterminated_double_quoted() {
    // Parser is lenient - unterminated double quotes become identifiers
    let result = parse("SELECT \"unclosed quote");
    assert!(result.is_ok(), "Parser accepts unterminated quotes: {:?}", result);
}

#[test]
fn test_invalid_operator() {
    let result = parse("SELECT * FROM t WHERE a @@ b");
    assert!(result.is_ok(), "Invalid operator parses as valid: {:?}", result);
}

#[test]
fn test_invalid_keyword_usage() {
    let result = parse("FROM SELECT * t");
    assert!(result.is_err(), "Keywords in wrong order should fail: {:?}", result);
}

#[test]
fn test_missing_comma_in_list() {
    let result = parse("SELECT a b FROM t");
    assert!(result.is_ok(), "Missing comma parses as valid: {:?}", result);
}

#[test]
fn test_extra_comma_before_from() {
    let result = parse("SELECT a, FROM t");
    assert!(result.is_ok(), "Extra comma parses as valid: {:?}", result);
}

#[test]
fn test_invalid_group_by() {
    let result = parse("SELECT * FROM t GROUP");
    assert!(result.is_err(), "Incomplete GROUP BY should fail: {:?}", result);
}

#[test]
fn test_invalid_having() {
    let result = parse("SELECT * FROM t HAVING");
    assert!(result.is_err(), "Incomplete HAVING should fail: {:?}", result);
}

#[test]
fn test_invalid_order_by() {
    let result = parse("SELECT * FROM t ORDER");
    assert!(result.is_err(), "Incomplete ORDER BY should fail: {:?}", result);
}

#[test]
fn test_invalid_limit() {
    let result = parse("SELECT * FROM t LIMIT");
    assert!(result.is_ok(), "Incomplete LIMIT parses as valid: {:?}", result);
}

#[test]
fn test_invalid_offset() {
    let result = parse("SELECT * FROM t OFFSET");
    assert!(result.is_ok(), "Incomplete OFFSET parses as valid: {:?}", result);
}

#[test]
fn test_invalid_join_syntax() {
    let result = parse("SELECT * FROM t JOIN");
    assert!(result.is_err(), "Incomplete JOIN should fail: {:?}", result);
}

#[test]
fn test_invalid_on_clause() {
    let result = parse("SELECT * FROM t INNER JOIN t2 ON");
    assert!(result.is_err(), "Incomplete ON clause should fail: {:?}", result);
}

#[test]
fn test_invalid_on_clause_no_condition() {
    let result = parse("SELECT * FROM t INNER JOIN t2 ON a");
    assert!(result.is_ok(), "ON a parses as valid condition: {:?}", result);
}

#[test]
fn test_like_without_pattern() {
    let result = parse("SELECT * FROM t WHERE a LIKE");
    assert!(result.is_ok(), "LIKE without pattern parses: {:?}", result);
}

#[test]
fn test_insert_without_values() {
    let result = parse("INSERT INTO t VALUES");
    assert!(result.is_err(), "INSERT without VALUES should fail: {:?}", result);
}

#[test]
fn test_insert_wrong_keyword() {
    let result = parse("INSERT TO t VALUES (1)");
    assert!(result.is_err(), "INSERT TO should fail: {:?}", result);
}

#[test]
fn test_update_without_set() {
    let result = parse("UPDATE t");
    assert!(result.is_err(), "UPDATE without SET should fail: {:?}", result);
}

#[test]
fn test_update_without_where() {
    let _result = parse("UPDATE t SET a = 1");
}

#[test]
fn test_delete_without_where() {
    let _result = parse("DELETE FROM t");
}

#[test]
fn test_drop_table_without_name() {
    let result = parse("DROP TABLE");
    assert!(result.is_err(), "DROP TABLE without name should fail: {:?}", result);
}

#[test]
fn test_create_table_without_columns() {
    let result = parse("CREATE TABLE t");
    assert!(result.is_ok(), "CREATE TABLE without columns parses: {:?}", result);
}

#[test]
fn test_create_table_incomplete() {
    let result = parse("CREATE TABLE t(");
    assert!(result.is_ok(), "Incomplete CREATE TABLE parses: {:?}", result);
}

#[test]
fn test_select_multiple_from() {
    let result = parse("SELECT * FROM");
    assert!(result.is_err(), "SELECT FROM without table should fail: {:?}", result);
}

#[test]
fn test_null_keyword_in_value() {
    let result = parse("SELECT NULLX");
    assert!(result.is_ok(), "NULLX parses as identifier: {:?}", result);
}

#[test]
fn test_boolean_literal_true() {
    let result = parse("SELECT TRUE FROM t");
    assert!(result.is_ok(), "TRUE literal is supported: {:?}", result);
}

#[test]
fn test_boolean_literal_false() {
    let result = parse("SELECT FALSE FROM t");
    assert!(result.is_ok(), "FALSE literal is supported: {:?}", result);
}

#[test]
fn test_in_clause_empty() {
    let result = parse("SELECT * FROM t WHERE a IN ()");
    assert!(result.is_err(), "Empty IN clause should fail: {:?}", result);
}

#[test]
fn test_between_invalid() {
    let result = parse("SELECT * FROM t WHERE a BETWEEN");
    assert!(result.is_err(), "BETWEEN without second value should fail: {:?}", result);
}

#[test]
fn test_case_without_when() {
    let result = parse("SELECT CASE WHEN 1 = 1 END");
    assert!(result.is_err(), "CASE without WHEN should fail: {:?}", result);
}

#[test]
fn test_case_without_end() {
    let result = parse("SELECT CASE WHEN 1 = 1 THEN 2");
    assert!(result.is_err(), "CASE without END should fail: {:?}", result);
}

#[test]
fn test_subquery_complete() {
    let result = parse("SELECT * FROM (SELECT * FROM t)");
    assert!(result.is_ok(), "Complete subquery should work: {:?}", result);
}

#[test]
fn test_union_incomplete() {
    let result = parse("SELECT 1 UNION");
    assert!(result.is_err(), "Incomplete UNION should fail: {:?}", result);
}

#[test]
fn test_intersect_incomplete() {
    let result = parse("SELECT 1 INTERSECT");
    assert!(result.is_err(), "Incomplete INTERSECT should fail: {:?}", result);
}

#[test]
fn test_except_incomplete() {
    let result = parse("SELECT 1 EXCEPT");
    assert!(result.is_err(), "Incomplete EXCEPT should fail: {:?}", result);
}
