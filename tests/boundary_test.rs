use sqlrustgo::parse;

#[test]
fn test_null_handling_in_select() {
    let sql = "SELECT NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "NULL in SELECT should parse");
}

#[test]
fn test_null_handling_in_where() {
    let sql = "SELECT * FROM t WHERE x IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "IS NULL should parse");
}

#[test]
fn test_null_handling_in_join() {
    let sql = "SELECT * FROM a LEFT JOIN b ON a.id = b.id WHERE b.id IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "NULL in JOIN should parse");
}

#[test]
fn test_large_integer_positive() {
    let sql = "SELECT 9223372036854775807";
    let result = parse(sql);
    assert!(result.is_ok(), "Max i64 should parse");
}

#[test]
fn test_large_integer_negative() {
    let sql = "SELECT -9223372036854775808";
    let result = parse(sql);
    assert!(result.is_ok(), "Min i64 should parse");
}

#[test]
fn test_large_float() {
    let sql = "SELECT 1.7976931348623157e308";
    let result = parse(sql);
    assert!(result.is_ok(), "Max f64 should parse");
}

#[test]
fn test_special_characters_chinese() {
    let sql = "SELECT '中文测试'";
    let result = parse(sql);
    assert!(result.is_ok(), "Chinese characters should parse");
}

#[test]
fn test_special_characters_emoji() {
    let sql = "SELECT '😀🎉🔥'";
    let result = parse(sql);
    assert!(result.is_ok(), "Emoji should parse");
}

#[test]
fn test_special_characters_sql_injection_attempt() {
    let sql = "SELECT * FROM users WHERE name = 'admin'--'";
    let result = parse(sql);
    assert!(result.is_ok(), "SQL comment should parse");
}

#[test]
fn test_empty_string() {
    let sql = "SELECT ''";
    let result = parse(sql);
    assert!(result.is_ok(), "Empty string should parse");
}

#[test]
fn test_whitespace_handling() {
    let sql = "SELECT   1   ,   2   ,   3   ";
    let result = parse(sql);
    assert!(result.is_ok(), "Extra whitespace should parse");
}

#[test]
fn test_tab_and_newline_in_string() {
    let sql = "SELECT 'line1\nline2\ttab'";
    let result = parse(sql);
    assert!(result.is_ok(), "Newline and tab in string should parse");
}

#[test]
fn test_zero_division_parsing() {
    let sql = "SELECT 1 / 0";
    let result = parse(sql);
    assert!(result.is_ok(), "Division by zero should parse");
}

#[test]
fn test_max_column_name_length() {
    let long_name = "a".repeat(64);
    let sql = format!("SELECT {} FROM t", long_name);
    let result = parse(&sql);
    assert!(result.is_ok(), "Max length column name should parse");
}

#[test]
fn test_max_table_name_length() {
    let long_name = "t".repeat(64);
    let sql = format!("SELECT * FROM {}", long_name);
    let result = parse(&sql);
    assert!(result.is_ok(), "Max length table name should parse");
}

#[test]
fn test_deeply_nested_subquery() {
    let sql = "SELECT * FROM (SELECT * FROM (SELECT * FROM t) AS a) AS b";
    let result = parse(sql);
    assert!(result.is_ok(), "Deeply nested subquery should parse");
}

#[test]
fn test_many_columns_in_select() {
    let cols: Vec<_> = (0..100).map(|i| format!("col{}", i)).collect();
    let sql = format!("SELECT {} FROM t", cols.join(", "));
    let result = parse(&sql);
    assert!(result.is_ok(), "Many columns should parse");
}

#[test]
fn test_many_values_in_insert() {
    let values: Vec<_> = (0..100).map(|_| "1").collect();
    let sql = format!("INSERT INTO t VALUES ({})", values.join(", "));
    let result = parse(&sql);
    assert!(result.is_ok(), "Many values should parse");
}

#[test]
fn test_bool_true_false() {
    let sql = "SELECT TRUE, FALSE, true, false";
    let result = parse(sql);
    assert!(result.is_ok(), "Boolean values should parse");
}

#[test]
fn test_hex_value() {
    let sql = "SELECT 0xFF, 0xabcdef";
    let result = parse(sql);
    assert!(result.is_ok(), "Hex values should parse");
}

#[test]
fn test_bit_value() {
    let sql = "SELECT b'1010', b'11111111'";
    let result = parse(sql);
    assert!(result.is_ok(), "Bit values should parse");
}
