//! Parser boundary/edge case tests (Layer 5).
//! Tests for edge conditions: empty, very long, many items, etc.

use sqlrustgo_parser::parse;

#[test]
fn test_single_char_identifier() {
    let result = parse("SELECT a FROM t");
    assert!(result.is_ok(), "Single char identifier should work: {:?}", result);
}

#[test]
fn test_long_identifier() {
    let sql = format!("SELECT {} FROM t", "a".repeat(100));
    let result = parse(&sql);
    assert!(result.is_ok(), "Long identifier should work: {:?}", result);
}

#[test]
fn test_very_long_identifier() {
    let sql = format!("SELECT {} FROM t", "a".repeat(1000));
    let result = parse(&sql);
    assert!(result.is_ok(), "Very long identifier should work: {:?}", result);
}

#[test]
fn test_many_columns() {
    let cols: Vec<String> = (0..50).map(|i| format!("col{}", i)).collect();
    let sql = format!("SELECT {} FROM t", cols.join(", "));
    let result = parse(&sql);
    assert!(result.is_ok(), "Many columns should work: {:?}", result);
}

#[test]
fn test_deeply_nested_parens() {
    let sql = "SELECT (((((1)))))";
    let result = parse(sql);
    assert!(result.is_err(), "Deeply nested parens not fully supported: {:?}", result);
}

#[test]
fn test_long_string_literal() {
    let sql = format!("SELECT '{}'", "x".repeat(1000));
    let result = parse(&sql);
    assert!(result.is_ok(), "Long string literal should work: {:?}", result);
}

#[test]
fn test_many_digits_in_number() {
    let sql = "SELECT 123456789012345678901234567890";
    let result = parse(sql);
    assert!(result.is_ok(), "Long number should work: {:?}", result);
}

#[test]
fn test_negative_number() {
    let result = parse("SELECT -1 FROM t");
    assert!(result.is_err(), "Negative numbers not supported: {:?}", result);
}

#[test]
fn test_decimal_number() {
    let result = parse("SELECT 3.14159");
    assert!(result.is_ok(), "Decimal number should work: {:?}", result);
}

#[test]
fn test_negative_decimal() {
    let result = parse("SELECT -3.14159 FROM t");
    assert!(result.is_err(), "Negative decimals not supported: {:?}", result);
}

#[test]
fn test_hex_number() {
    let result = parse("SELECT 0xFF");
    assert!(result.is_ok(), "Hex number should work: {:?}", result);
}

#[test]
fn test_leading_zeros() {
    let result = parse("SELECT 007");
    assert!(result.is_ok(), "Leading zeros should work: {:?}", result);
}

#[test]
fn test_scientific_notation() {
    let result = parse("SELECT 1e10");
    assert!(result.is_ok(), "Scientific notation should work: {:?}", result);
}

#[test]
fn test_double_star_exponent() {
    let result = parse("SELECT 1e-5");
    assert!(result.is_ok(), "Negative exponent should work: {:?}", result);
}

#[test]
fn test_quoted_identifier() {
    let result = parse("SELECT `column name` FROM t");
    assert!(result.is_err(), "Quoted identifiers not supported: {:?}", result);
}

#[test]
fn test_quoted_identifier_with_special_chars() {
    let result = parse("SELECT `column-name.with.dots` FROM t");
    assert!(result.is_err(), "Quoted identifiers not supported: {:?}", result);
}

#[test]
fn test_alias_with_as() {
    let result = parse("SELECT col AS alias FROM t");
    assert!(result.is_ok(), "Alias with AS should work: {:?}", result);
}

#[test]
fn test_alias_without_as() {
    let result = parse("SELECT col alias FROM t");
    assert!(result.is_ok(), "Alias without AS should work: {:?}", result);
}

#[test]
fn test_expression_in_select() {
    let result = parse("SELECT a + b * c FROM t");
    assert!(result.is_ok(), "Expression in select should work: {:?}", result);
}

#[test]
fn test_function_in_select() {
    let result = parse("SELECT COUNT(*) FROM t");
    assert!(result.is_ok(), "Function in select should work: {:?}", result);
}

#[test]
fn test_distinct_in_select() {
    let result = parse("SELECT DISTINCT a FROM t");
    assert!(result.is_ok(), "DISTINCT in select should work: {:?}", result);
}

#[test]
fn test_all_in_select() {
    let result = parse("SELECT ALL a FROM t");
    assert!(result.is_err(), "ALL keyword not supported: {:?}", result);
}

#[test]
fn test_multiple_statements() {
    let result = parse("SELECT 1; SELECT 2");
    assert!(result.is_err(), "Multiple statements not supported: {:?}", result);
}

#[test]
fn test_comment_before_select() {
    let result = parse("-- comment\nSELECT 1");
    assert!(result.is_err(), "Comments not supported: {:?}", result);
}

#[test]
fn test_comment_after_select() {
    let result = parse("SELECT 1 -- comment");
    assert!(result.is_err(), "Comments not supported: {:?}", result);
}

#[test]
fn test_block_comment() {
    let result = parse("/* comment */ SELECT 1");
    assert!(result.is_err(), "Block comments not supported: {:?}", result);
}

#[test]
fn test_cte_basic() {
    let result = parse("WITH cte AS (SELECT 1) SELECT * FROM cte");
    assert!(result.is_err(), "CTE not supported: {:?}", result);
}

#[test]
fn test_multiple_ctes() {
    let result = parse("WITH cte1 AS (SELECT 1), cte2 AS (SELECT 2) SELECT * FROM cte1, cte2");
    assert!(result.is_err(), "CTE not supported: {:?}", result);
}

#[test]
fn test_scalar_subquery() {
    let result = parse("SELECT (SELECT MAX(x) FROM t) FROM t2");
    assert!(result.is_err(), "Scalar subquery not supported: {:?}", result);
}

#[test]
fn test_exists_subquery() {
    let result = parse("SELECT * FROM t WHERE EXISTS (SELECT 1)");
    assert!(result.is_err(), "EXISTS subquery not supported: {:?}", result);
}

#[test]
fn test_in_subquery() {
    let result = parse("SELECT * FROM t WHERE a IN (SELECT x FROM t2)");
    assert!(result.is_ok(), "IN subquery should work: {:?}", result);
}

#[test]
fn test_join_with_using() {
    let result = parse("SELECT * FROM t1 JOIN t2 USING (id)");
    assert!(result.is_ok(), "JOIN USING should work: {:?}", result);
}

#[test]
fn test_natural_join() {
    let result = parse("SELECT * FROM t1 NATURAL JOIN t2");
    assert!(result.is_ok(), "NATURAL JOIN should work: {:?}", result);
}

#[test]
fn test_cross_join() {
    let result = parse("SELECT * FROM t1 CROSS JOIN t2");
    assert!(result.is_ok(), "CROSS JOIN should work: {:?}", result);
}

#[test]
fn test_insert_single_value() {
    let result = parse("INSERT INTO t VALUES (1)");
    assert!(result.is_ok(), "Single value insert should work: {:?}", result);
}

#[test]
fn test_insert_multiple_values() {
    let result = parse("INSERT INTO t VALUES (1), (2), (3)");
    assert!(result.is_ok(), "Multiple values insert should work: {:?}", result);
}

#[test]
fn test_insert_with_columns() {
    let result = parse("INSERT INTO t (a, b) VALUES (1, 2)");
    assert!(result.is_ok(), "Insert with columns should work: {:?}", result);
}

#[test]
fn test_update_with_where() {
    let result = parse("UPDATE t SET a = 1 WHERE b = 2");
    assert!(result.is_ok(), "UPDATE with WHERE should work: {:?}", result);
}

#[test]
fn test_delete_with_where() {
    let result = parse("DELETE FROM t WHERE a = 1");
    assert!(result.is_ok(), "DELETE with WHERE should work: {:?}", result);
}

#[test]
fn test_create_table_with_primary_key() {
    let result = parse("CREATE TABLE t (a INT PRIMARY KEY, b TEXT)");
    assert!(result.is_ok(), "CREATE TABLE with PRIMARY KEY should work: {:?}", result);
}

#[test]
fn test_create_table_with_not_null() {
    let result = parse("CREATE TABLE t (a INT NOT NULL)");
    assert!(result.is_ok(), "CREATE TABLE with NOT NULL should work: {:?}", result);
}

#[test]
fn test_create_table_with_default() {
    let result = parse("CREATE TABLE t (a INT DEFAULT 0)");
    assert!(result.is_ok(), "CREATE TABLE with DEFAULT should work: {:?}", result);
}

#[test]
fn test_create_table_with_unique() {
    let result = parse("CREATE TABLE t (a INT UNIQUE)");
    assert!(result.is_ok(), "CREATE TABLE with UNIQUE should work: {:?}", result);
}

#[test]
fn test_create_table_with_check() {
    let result = parse("CREATE TABLE t (a INT CHECK (a > 0))");
    assert!(result.is_ok(), "CREATE TABLE with CHECK should work: {:?}", result);
}

#[test]
fn test_create_table_with_foreign_key() {
    let result = parse("CREATE TABLE t (a INT REFERENCES other(id))");
    assert!(result.is_ok(), "CREATE TABLE with FOREIGN KEY should work: {:?}", result);
}

#[test]
fn test_drop_table_if_exists() {
    let result = parse("DROP TABLE IF EXISTS t");
    assert!(result.is_ok(), "DROP TABLE IF EXISTS should work: {:?}", result);
}

#[test]
fn test_truncate_table() {
    let result = parse("TRUNCATE TABLE t");
    assert!(result.is_err(), "TRUNCATE not supported: {:?}", result);
}

#[test]
fn test_alter_table_add_column() {
    let result = parse("ALTER TABLE t ADD COLUMN a INT");
    assert!(result.is_ok(), "ALTER TABLE ADD COLUMN should work: {:?}", result);
}

#[test]
fn test_alter_table_drop_column() {
    let result = parse("ALTER TABLE t DROP COLUMN a");
    assert!(result.is_ok(), "ALTER TABLE DROP COLUMN should work: {:?}", result);
}

#[test]
fn test_create_index() {
    let result = parse("CREATE INDEX idx ON t (a)");
    assert!(result.is_ok(), "CREATE INDEX should work: {:?}", result);
}

#[test]
fn test_create_unique_index() {
    let result = parse("CREATE UNIQUE INDEX idx ON t (a)");
    assert!(result.is_ok(), "CREATE UNIQUE INDEX should work: {:?}", result);
}

#[test]
fn test_drop_index() {
    let result = parse("DROP INDEX idx");
    assert!(result.is_ok(), "DROP INDEX should work: {:?}", result);
}

#[test]
fn test_begin_transaction() {
    let result = parse("BEGIN");
    assert!(result.is_ok(), "BEGIN should work: {:?}", result);
}

#[test]
fn test_commit() {
    let result = parse("COMMIT");
    assert!(result.is_ok(), "COMMIT should work: {:?}", result);
}

#[test]
fn test_rollback() {
    let result = parse("ROLLBACK");
    assert!(result.is_ok(), "ROLLBACK should work: {:?}", result);
}

#[test]
fn test_set_variable() {
    let result = parse("SET a = 1");
    assert!(result.is_err(), "SET not supported: {:?}", result);
}

#[test]
fn test_show_tables() {
    let result = parse("SHOW TABLES");
    assert!(result.is_ok(), "SHOW TABLES should work: {:?}", result);
}

#[test]
fn test_describe_table() {
    let result = parse("DESCRIBE t");
    assert!(result.is_ok(), "DESCRIBE should work: {:?}", result);
}

#[test]
fn test_explain_select() {
    let result = parse("EXPLAIN SELECT * FROM t");
    assert!(result.is_err(), "EXPLAIN not supported: {:?}", result);
}
