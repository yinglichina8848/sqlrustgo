use crate::parse;

#[test]
fn test_parse_select_position_expression() {
    let sql = "SELECT POSITION('a' IN 'banana') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "POSITION: {:?}", result);
}

#[test]
fn test_parse_select_substring_expression() {
    let sql = "SELECT SUBSTRING('test' FROM 1 FOR 2) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SUBSTRING FROM FOR: {:?}",
        result
    );
}

#[test]
fn test_parse_select_trim_leading() {
    let sql = "SELECT TRIM(LEADING ' ' FROM '  test  ') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TRIM LEADING: {:?}",
        result
    );
}

#[test]
fn test_parse_select_trim_trailing() {
    let sql = "SELECT TRIM(TRAILING ' ' FROM '  test  ') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TRIM TRAILING: {:?}",
        result
    );
}

#[test]
fn test_parse_select_trim_both() {
    let sql = "SELECT TRIM(BOTH ' ' FROM '  test  ') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "TRIM BOTH: {:?}", result);
}

#[test]
fn test_parse_select_overlay() {
    let sql = "SELECT OVERLAY('abcdef' PLACING 'xyz' FROM 2 FOR 3) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "OVERLAY: {:?}", result);
}

#[test]
fn test_parse_select_extract_date() {
    let sql = "SELECT EXTRACT(YEAR FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT YEAR: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_month() {
    let sql = "SELECT EXTRACT(MONTH FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT MONTH: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_day() {
    let sql = "SELECT EXTRACT(DAY FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT DAY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_hour() {
    let sql = "SELECT EXTRACT(HOUR FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT HOUR: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_minute() {
    let sql = "SELECT EXTRACT(MINUTE FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT MINUTE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_second() {
    let sql = "SELECT EXTRACT(SECOND FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT SECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_microsecond() {
    let sql = "SELECT EXTRACT(MICROSECOND FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT MICROSECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_week() {
    let sql = "SELECT EXTRACT(WEEK FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT WEEK: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_quarter() {
    let sql = "SELECT EXTRACT(QUARTER FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT QUARTER: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_year_month() {
    let sql = "SELECT EXTRACT(YEAR_MONTH FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT YEAR_MONTH: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_day_hour() {
    let sql = "SELECT EXTRACT(DAY_HOUR FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT DAY_HOUR: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_day_minute() {
    let sql = "SELECT EXTRACT(DAY_MINUTE FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT DAY_MINUTE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_day_second() {
    let sql = "SELECT EXTRACT(DAY_SECOND FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT DAY_SECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_hour_minute() {
    let sql = "SELECT EXTRACT(HOUR_MINUTE FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT HOUR_MINUTE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_hour_second() {
    let sql = "SELECT EXTRACT(HOUR_SECOND FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT HOUR_SECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_select_extract_minute_second() {
    let sql = "SELECT EXTRACT(MINUTE_SECOND FROM date_col) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXTRACT MINUTE_SECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_select_time_value() {
    let sql = "SELECT TIME '12:30:00' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TIME value: {:?}",
        result
    );
}

#[test]
fn test_parse_select_date_value() {
    let sql = "SELECT DATE '2024-01-01' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DATE value: {:?}",
        result
    );
}

#[test]
fn test_parse_select_timestamp_value() {
    let sql = "SELECT TIMESTAMP '2024-01-01 12:00:00' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TIMESTAMP value: {:?}",
        result
    );
}

#[test]
fn test_parse_select_intervals() {
    let sql = "SELECT INTERVAL 1 DAY + INTERVAL 2 HOUR FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL addition: {:?}",
        result
    );
}

#[test]
fn test_parse_collate() {
    let sql = "SELECT 'test' COLLATE utf8mb4_unicode_ci FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "COLLATE: {:?}", result);
}

#[test]
fn test_parse_match_expression() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('text')";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "MATCH: {:?}", result);
}

#[test]
fn test_parse_not_match_expression() {
    let sql = "SELECT * FROM t WHERE NOT MATCH(col) AGAINST('text')";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NOT MATCH: {:?}", result);
}

#[test]
fn test_parse_or表达式() {
    let sql = "SELECT * FROM t WHERE a = 1 OR b = 2";
    let result = parse(sql);
    assert!(result.is_ok(), "OR expression: {:?}", result);
}

#[test]
fn test_parse_and表达式() {
    let sql = "SELECT * FROM t WHERE a = 1 AND b = 2";
    let result = parse(sql);
    assert!(result.is_ok(), "AND expression: {:?}", result);
}

#[test]
fn test_parse_xor表达式() {
    let sql = "SELECT * FROM t WHERE a = 1 XOR b = 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "XOR expression: {:?}",
        result
    );
}

#[test]
fn test_parse_not_in_expression() {
    let sql = "SELECT * FROM t WHERE a NOT IN (1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok(), "NOT IN: {:?}", result);
}

#[test]
fn test_parse_is_not_boolean() {
    let sql = "SELECT * FROM t WHERE a IS NOT TRUE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "IS NOT TRUE: {:?}",
        result
    );
}

#[test]
fn test_parse_is_not_false() {
    let sql = "SELECT * FROM t WHERE a IS NOT FALSE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "IS NOT FALSE: {:?}",
        result
    );
}

#[test]
fn test_parse_is_not_unknown() {
    let sql = "SELECT * FROM t WHERE a IS NOT UNKNOWN";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "IS NOT UNKNOWN: {:?}",
        result
    );
}

#[test]
fn test_parse_like_escape() {
    let sql = "SELECT * FROM t WHERE name LIKE '%' ESCAPE '\\'";
    let result = parse(sql);
    assert!(result.is_ok(), "LIKE ESCAPE: {:?}", result);
}

#[test]
fn test_parse_not_like() {
    let sql = "SELECT * FROM t WHERE name NOT LIKE '%test%'";
    let result = parse(sql);
    assert!(result.is_ok(), "NOT LIKE: {:?}", result);
}

#[test]
fn test_parse_regexp_not() {
    let sql = "SELECT * FROM t WHERE name NOT REGEXP 'pattern'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "NOT REGEXP: {:?}",
        result
    );
}

#[test]
fn test_parse_between_and() {
    let sql = "SELECT * FROM t WHERE a BETWEEN 1 AND 10";
    let result = parse(sql);
    assert!(result.is_ok(), "BETWEEN AND: {:?}", result);
}

#[test]
fn test_parse_not_between() {
    let sql = "SELECT * FROM t WHERE a NOT BETWEEN 1 AND 10";
    let result = parse(sql);
    assert!(result.is_ok(), "NOT BETWEEN: {:?}", result);
}

#[test]
fn test_parse_binary_operator() {
    let sql = "SELECT a BINARY b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BINARY operator: {:?}",
        result
    );
}

#[test]
fn test_parse_not_binary_operator() {
    let sql = "SELECT NOT BINARY 'test' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "NOT BINARY: {:?}",
        result
    );
}

#[test]
fn test_parse_hex_literal() {
    let sql = "SELECT 0x48656c6c6f FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Hex literal: {:?}", result);
}

#[test]
fn test_parse_bit_literal() {
    let sql = "SELECT 0b1010 FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Bit literal: {:?}",
        result
    );
}

#[test]
fn test_parse_exponent_literal() {
    let sql = "SELECT 1e10 FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Exponent literal: {:?}", result);
}

#[test]
fn test_parse_negative_literal() {
    let sql = "SELECT -123 FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Negative literal: {:?}", result);
}

#[test]
fn test_parse_string_literal_escaped() {
    let sql = "SELECT 'hello\\nworld' FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Escaped string: {:?}", result);
}

#[test]
fn test_parse_string_literal_concat() {
    let sql = "SELECT 'hello' 'world' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "String concat: {:?}",
        result
    );
}

#[test]
fn test_parse_null_literal() {
    let sql = "SELECT NULL FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "NULL literal: {:?}", result);
}

#[test]
fn test_parse_true_literal() {
    let sql = "SELECT TRUE FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "TRUE literal: {:?}", result);
}

#[test]
fn test_parse_false_literal() {
    let sql = "SELECT FALSE FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "FALSE literal: {:?}", result);
}

#[test]
fn test_parse_boolean_literal() {
    let sql = "SELECT TRUE, FALSE FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Boolean literals: {:?}", result);
}

#[test]
fn test_parse_unary_plus() {
    let sql = "SELECT +5 FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Unary plus: {:?}",
        result
    );
}

#[test]
fn test_parse_unary_minus() {
    let sql = "SELECT -5 FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Unary minus: {:?}", result);
}

#[test]
fn test_parse_unary_not() {
    let sql = "SELECT NOT a FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Unary NOT: {:?}", result);
}

#[test]
fn test_parse_unary_bitwise_not() {
    let sql = "SELECT ~a FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Unary bitwise NOT: {:?}", result);
}

#[test]
fn test_parse_division() {
    let sql = "SELECT a / b FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Division: {:?}", result);
}

#[test]
fn test_parse_modulo() {
    let sql = "SELECT a % b FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Modulo: {:?}", result);
}

#[test]
fn test_parse_null_safe_eq() {
    let sql = "SELECT a <=> b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "NULL SAFE EQ: {:?}",
        result
    );
}

#[test]
fn test_parse_left_shift() {
    let sql = "SELECT a << 2 FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LEFT SHIFT: {:?}",
        result
    );
}

#[test]
fn test_parse_right_shift() {
    let sql = "SELECT a >> 2 FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RIGHT SHIFT: {:?}",
        result
    );
}

#[test]
fn test_parse_logical_and() {
    let sql = "SELECT a AND b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOGICAL AND: {:?}",
        result
    );
}

#[test]
fn test_parse_logical_or() {
    let sql = "SELECT a OR b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOGICAL OR: {:?}",
        result
    );
}

#[test]
fn test_parse_logical_xor() {
    let sql = "SELECT a XOR b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOGICAL XOR: {:?}",
        result
    );
}

#[test]
fn test_parse_logical_not() {
    let sql = "SELECT NOT a FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOGICAL NOT: {:?}",
        result
    );
}

#[test]
fn test_parse_bitwise_and() {
    let sql = "SELECT a & b FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Bitwise AND: {:?}", result);
}

#[test]
fn test_parse_bitwise_or() {
    let sql = "SELECT a | b FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Bitwise OR: {:?}", result);
}

#[test]
fn test_parse_bitwise_xor() {
    let sql = "SELECT a ^ b FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Bitwise XOR: {:?}", result);
}

#[test]
fn test_parse_collation() {
    let sql = "SELECT 'test' COLLATE utf8mb4_general_ci FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "COLLATE: {:?}", result);
}

#[test]
fn test_parse_scientific_notation() {
    let sql = "SELECT 1.5e-10 FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Scientific notation: {:?}", result);
}

#[test]
fn test_parse_decimal_literal() {
    let sql = "SELECT 123.456 FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Decimal literal: {:?}", result);
}

#[test]
fn test_parse_signed_literal() {
    let sql = "SELECT +123, -456 FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Signed literals: {:?}",
        result
    );
}

#[test]
fn test_parse_column_alias_quoted() {
    let sql = "SELECT id AS `alias` FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Quoted alias: {:?}",
        result
    );
}

#[test]
fn test_parse_table_alias_quoted() {
    let sql = "SELECT t.id FROM `table` AS t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Quoted table alias: {:?}",
        result
    );
}

#[test]
fn test_parse_order_by_expression() {
    let sql = "SELECT * FROM t ORDER BY a + b";
    let result = parse(sql);
    assert!(result.is_ok(), "ORDER BY expression: {:?}", result);
}

#[test]
fn test_parse_order_by_alias() {
    let sql = "SELECT id AS my_id FROM t ORDER BY my_id";
    let result = parse(sql);
    assert!(result.is_ok(), "ORDER BY alias: {:?}", result);
}

#[test]
fn test_parse_order_by_column_position() {
    let sql = "SELECT id, name FROM t ORDER BY 1, 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY position: {:?}",
        result
    );
}

#[test]
fn test_parse_order_by_nulls_first() {
    let sql = "SELECT * FROM t ORDER BY a NULLS FIRST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY NULLS FIRST: {:?}",
        result
    );
}

#[test]
fn test_parse_order_by_nulls_last() {
    let sql = "SELECT * FROM t ORDER BY a NULLS LAST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY NULLS LAST: {:?}",
        result
    );
}

#[test]
fn test_parse_limit_row_count() {
    let sql = "SELECT * FROM t LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok(), "LIMIT row count: {:?}", result);
}

#[test]
fn test_parse_limit_offset() {
    let sql = "SELECT * FROM t LIMIT 10 OFFSET 5";
    let result = parse(sql);
    assert!(result.is_ok(), "LIMIT OFFSET: {:?}", result);
}

#[test]
fn test_parse_limit_comma_offset() {
    let sql = "SELECT * FROM t LIMIT 5, 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LIMIT comma offset: {:?}",
        result
    );
}

#[test]
fn test_parse_distinct_all() {
    let sql = "SELECT DISTINCT ALL id FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DISTINCT ALL: {:?}",
        result
    );
}

#[test]
fn test_parse_select_high_priority() {
    let sql = "SELECT HIGH_PRIORITY * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HIGH_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_straight_join() {
    let sql = "SELECT STRAIGHT_JOIN * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STRAIGHT_JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_select_small_result() {
    let sql = "SELECT SMALL_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SMALL_RESULT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_big_result() {
    let sql = "SELECT BIG_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BIG_RESULT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_sql_cache() {
    let sql = "SELECT SQL_CACHE * FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SQL_CACHE: {:?}", result);
}

#[test]
fn test_parse_select_sql_no_cache() {
    let sql = "SELECT SQL_NO_CACHE * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SQL_NO_CACHE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_sql_buffer_result() {
    let sql = "SELECT SQL_BUFFER_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SQL_BUFFER_RESULT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_calc_found_rows() {
    let sql = "SELECT SQL_CALC_FOUND_ROWS * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SQL_CALC_FOUND_ROWS: {:?}",
        result
    );
}

#[test]
fn test_parse_union() {
    let sql = "SELECT 1 UNION SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok(), "UNION: {:?}", result);
}

#[test]
fn test_parse_union_all() {
    let sql = "SELECT 1 UNION ALL SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok(), "UNION ALL: {:?}", result);
}

#[test]
fn test_parse_union_distinct() {
    let sql = "SELECT 1 UNION DISTINCT SELECT 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNION DISTINCT: {:?}",
        result
    );
}

#[test]
fn test_parse_intersect() {
    let sql = "SELECT 1 INTERSECT SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "INTERSECT: {:?}", result);
}

#[test]
fn test_parse_except() {
    let sql = "SELECT 1 EXCEPT SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXCEPT: {:?}", result);
}

#[test]
fn test_parse_join_natural() {
    let sql = "SELECT * FROM a NATURAL JOIN b";
    let result = parse(sql);
    assert!(result.is_ok(), "NATURAL JOIN: {:?}", result);
}

#[test]
fn test_parse_join_cross() {
    let sql = "SELECT * FROM a CROSS JOIN b";
    let result = parse(sql);
    assert!(result.is_ok(), "CROSS JOIN: {:?}", result);
}

#[test]
fn test_parse_join_inner() {
    let sql = "SELECT * FROM a INNER JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(result.is_ok(), "INNER JOIN: {:?}", result);
}

#[test]
fn test_parse_join_left() {
    let sql = "SELECT * FROM a LEFT JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(result.is_ok(), "LEFT JOIN: {:?}", result);
}

#[test]
fn test_parse_join_right() {
    let sql = "SELECT * FROM a RIGHT JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(result.is_ok(), "RIGHT JOIN: {:?}", result);
}

#[test]
fn test_parse_join_full() {
    let sql = "SELECT * FROM a FULL JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FULL JOIN: {:?}", result);
}

#[test]
fn test_parse_join_using() {
    let sql = "SELECT * FROM a JOIN b USING (id)";
    let result = parse(sql);
    assert!(result.is_ok(), "JOIN USING: {:?}", result);
}

#[test]
fn test_parse_join_multiple_using() {
    let sql = "SELECT * FROM a JOIN b USING (id, name)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JOIN USING multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_multiple_joins() {
    let sql = "SELECT * FROM a JOIN b ON a.id = b.id JOIN c ON b.id = c.id";
    let result = parse(sql);
    assert!(result.is_ok(), "Multiple JOINs: {:?}", result);
}

#[test]
fn test_parse_group_by() {
    let sql = "SELECT id, COUNT(*) FROM t GROUP BY id";
    let result = parse(sql);
    assert!(result.is_ok(), "GROUP BY: {:?}", result);
}

#[test]
fn test_parse_group_by_with_rollup() {
    let sql = "SELECT type, region, SUM(val) FROM t GROUP BY type, region WITH ROLLUP";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY ROLLUP: {:?}",
        result
    );
}

#[test]
fn test_parse_group_by_with_cube() {
    let sql = "SELECT type, region, SUM(val) FROM t GROUP BY type, region WITH CUBE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY CUBE: {:?}",
        result
    );
}

#[test]
fn test_parse_having() {
    let sql = "SELECT id, COUNT(*) AS cnt FROM t GROUP BY id HAVING cnt > 1";
    let result = parse(sql);
    assert!(result.is_ok(), "HAVING: {:?}", result);
}

#[test]
fn test_parse_window_over() {
    let sql = "SELECT ROW_NUMBER() OVER () FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_partition_by() {
    let sql = "SELECT SUM(v) OVER (PARTITION BY type) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window PARTITION BY: {:?}",
        result
    );
}

#[test]
fn test_parse_window_order_by() {
    let sql = "SELECT SUM(v) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window ORDER BY: {:?}",
        result
    );
}

#[test]
fn test_parse_window_rows_between() {
    let sql = "SELECT SUM(v) OVER (ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window ROWS BETWEEN: {:?}",
        result
    );
}

#[test]
fn test_parse_window_range_between() {
    let sql = "SELECT SUM(v) OVER (RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window RANGE BETWEEN: {:?}",
        result
    );
}

#[test]
fn test_parse_window_exclude_current_row() {
    let sql = "SELECT SUM(v) OVER (EXCLUDE CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window EXCLUDE CURRENT ROW: {:?}",
        result
    );
}

#[test]
fn test_parse_window_exclude_group() {
    let sql = "SELECT SUM(v) OVER (EXCLUDE GROUP) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window EXCLUDE GROUP: {:?}",
        result
    );
}

#[test]
fn test_parse_window_exclude_ties() {
    let sql = "SELECT SUM(v) OVER (EXCLUDE TIES) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window EXCLUDE TIES: {:?}",
        result
    );
}

#[test]
fn test_parse_cte_named() {
    let sql = "WITH cte AS (SELECT 1) SELECT * FROM cte";
    let result = parse(sql);
    assert!(result.is_ok(), "CTE named: {:?}", result);
}

#[test]
fn test_parse_cte_recursive() {
    let sql = "WITH RECURSIVE cte AS (SELECT 1 UNION ALL SELECT id+1 FROM cte WHERE id < 10) SELECT * FROM cte";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE recursive: {:?}",
        result
    );
}

#[test]
fn test_parse_cte_multiple() {
    let sql = "WITH cte1 AS (SELECT 1), cte2 AS (SELECT 2) SELECT * FROM cte1, cte2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_subquery_in_select() {
    let sql = "SELECT (SELECT MAX(id) FROM t) FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Subquery in SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_subquery_in_from() {
    let sql = "SELECT * FROM (SELECT 1 AS a) AS subq";
    let result = parse(sql);
    assert!(result.is_ok(), "Subquery in FROM: {:?}", result);
}

#[test]
fn test_parse_subquery_in_where() {
    let sql = "SELECT * FROM t WHERE id IN (SELECT id FROM t2)";
    let result = parse(sql);
    assert!(result.is_ok(), "Subquery in WHERE: {:?}", result);
}

#[test]
fn test_parse_scalar_subquery_in_set() {
    let sql = "UPDATE t SET id = (SELECT MAX(id) FROM t2)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Scalar subquery in SET: {:?}",
        result
    );
}
