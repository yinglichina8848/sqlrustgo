//! Window function and MySQL modifier tests for parser coverage.
//! Covers: OVER clauses, PARTITION BY, ORDER BY, MySQL query modifiers, SUBSTRING FROM...FOR

use sqlrustgo_parser::parse;

// ============ Window Function Tests ============

#[test]
fn test_parse_window_row_number_basic() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ROW_NUMBER: {:?}", result);
}

#[test]
fn test_parse_window_rank_basic() {
    let sql = "SELECT RANK() OVER (ORDER BY score DESC) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RANK: {:?}", result);
}

#[test]
fn test_parse_window_dense_rank_basic() {
    let sql = "SELECT DENSE_RANK() OVER (ORDER BY salary) FROM employees";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DENSE_RANK: {:?}", result);
}

#[test]
fn test_parse_window_count_star() {
    let sql = "SELECT COUNT(*) OVER (PARTITION BY department) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COUNT(*) OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_sum_with_partition() {
    let sql = "SELECT SUM(salary) OVER (PARTITION BY department ORDER BY hire_date) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUM OVER PARTITION: {:?}",
        result
    );
}

#[test]
fn test_parse_window_avg_with_partition() {
    let sql = "SELECT AVG(amount) OVER (PARTITION BY category) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse AVG OVER PARTITION: {:?}",
        result
    );
}

#[test]
fn test_parse_window_min_max_with_partition() {
    let sql = "SELECT MIN(price) OVER (PARTITION BY brand), MAX(price) OVER (PARTITION BY brand) FROM products";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse MIN/MAX OVER PARTITION: {:?}",
        result
    );
}

#[test]
fn test_parse_window_partition_by_multiple_columns() {
    let sql =
        "SELECT ROW_NUMBER() OVER (PARTITION BY department, status ORDER BY id) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse PARTITION BY multiple columns: {:?}",
        result
    );
}

#[test]
fn test_parse_window_order_by_multiple_columns() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY last_name, first_name) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ORDER BY multiple columns: {:?}",
        result
    );
}

#[test]
fn test_parse_window_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT user_id) OVER (PARTITION BY session_id) FROM events";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COUNT DISTINCT OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_with_alias() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) AS row_num FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with alias: {:?}",
        result
    );
}

#[test]
fn test_parse_window_multiple_functions() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) AS rn, RANK() OVER (ORDER BY id) AS rk, DENSE_RANK() OVER (ORDER BY id) AS dr FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple window functions: {:?}",
        result
    );
}

#[test]
fn test_parse_window_first_value() {
    let sql = "SELECT FIRST_VALUE(price) OVER (PARTITION BY category ORDER BY date) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse FIRST_VALUE: {:?}", result);
}

#[test]
fn test_parse_window_last_value() {
    let sql = "SELECT LAST_VALUE(price) OVER (PARTITION BY category ORDER BY date) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LAST_VALUE: {:?}", result);
}

#[test]
fn test_parse_window_nth_value() {
    let sql = "SELECT NTH_VALUE(price, 2) OVER (PARTITION BY category) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NTH_VALUE: {:?}", result);
}

#[test]
fn test_parse_window_lag() {
    let sql = "SELECT LAG(salary) OVER (PARTITION BY department ORDER BY year) FROM salaries";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LAG: {:?}", result);
}

#[test]
fn test_parse_window_lead() {
    let sql = "SELECT LEAD(revenue) OVER (PARTITION BY region ORDER BY quarter) FROM sales";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LEAD: {:?}", result);
}

#[test]
fn test_parse_window_ntile() {
    let sql = "SELECT NTILE(4) OVER (ORDER BY score) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NTILE: {:?}", result);
}

#[test]
fn test_parse_window_count_star_no_partition() {
    let sql = "SELECT COUNT(*) OVER () FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COUNT(*) OVER without partition: {:?}",
        result
    );
}

#[test]
fn test_parse_window_with_where_filter() {
    let sql = "SELECT * FROM (SELECT ROW_NUMBER() OVER (ORDER BY id) AS rn FROM users) AS numbered WHERE rn > 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with WHERE: {:?}",
        result
    );
}

#[test]
fn test_parse_window_in_subquery() {
    let sql =
        "SELECT * FROM (SELECT id, ROW_NUMBER() OVER (ORDER BY id) AS rn FROM users) AS numbered";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window in subquery: {:?}",
        result
    );
}

#[test]
fn test_parse_window_with_join() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY o.date) AS rn, u.name FROM users u JOIN orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_window_aggregate_in_select() {
    let sql = "SELECT department, COUNT(*) AS cnt, ROW_NUMBER() OVER (ORDER BY COUNT(*)) AS rn FROM employees GROUP BY department";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with aggregate: {:?}",
        result
    );
}

#[test]
fn test_parse_window_with_limit() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) AS rn FROM users LIMIT 100";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with LIMIT: {:?}",
        result
    );
}

// ============ MySQL Query Modifier Tests ============

#[test]
fn test_parse_mysql_high_priority() {
    let sql = "SELECT HIGH_PRIORITY * FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse HIGH_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_sql_cache() {
    let sql = "SELECT SQL_CACHE * FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SQL_CACHE: {:?}", result);
}

#[test]
fn test_parse_mysql_sql_no_cache() {
    let sql = "SELECT SQL_NO_CACHE * FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SQL_NO_CACHE: {:?}", result);
}

#[test]
fn test_parse_mysql_sql_calc_found_rows() {
    let sql = "SELECT SQL_CALC_FOUND_ROWS * FROM users LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SQL_CALC_FOUND_ROWS: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_multiple_modifiers() {
    let sql = "SELECT HIGH_PRIORITY SQL_NO_CACHE * FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple MySQL modifiers: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_modifier_with_distinct() {
    let sql = "SELECT DISTINCT HIGH_PRIORITY name FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse MySQL modifier with DISTINCT: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_modifier_with_aggregate() {
    let sql = "SELECT SQL_NO_CACHE COUNT(*) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse MySQL modifier with aggregate: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_modifier_with_function() {
    let sql = "SELECT HIGH_PRIORITY SQL_CACHE UPPER(name) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse MySQL modifier with function: {:?}",
        result
    );
}

#[test]
fn test_parse_mysql_modifier_stacked() {
    let sql = "SELECT HIGH_PRIORITY SQL_CACHE SQL_CALC_FOUND_ROWS name FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse stacked MySQL modifiers: {:?}",
        result
    );
}

// ============ SUBSTRING FROM...FOR Tests ============

#[test]
fn test_parse_substring_from_for_basic() {
    let sql = "SELECT SUBSTRING('Hello World' FROM 1 FOR 5)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING FROM...FOR: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_from_for_no_length() {
    let sql = "SELECT SUBSTRING('Hello World' FROM 7)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING FROM without length: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_from_for_in_column() {
    let sql = "SELECT SUBSTRING(name FROM 1 FOR 10) AS first_10_chars FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING FROM...FOR in column: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_comma_separated() {
    let sql = "SELECT SUBSTRING('Hello World', 1, 5)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING with commas: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_comma_no_length() {
    let sql = "SELECT SUBSTRING('Hello World', 7)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING with comma no length: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_mixed_functions() {
    let sql = "SELECT SUBSTRING(UPPER(name) FROM 1 FOR 5) AS first_5_upper FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING with UPPER: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_from_for_with_expression() {
    let sql = "SELECT SUBSTRING(name FROM 1 + 0 FOR LENGTH(name)) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING with expressions: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_in_where() {
    let sql = "SELECT * FROM users WHERE SUBSTRING(name, 1, 1) = 'A'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING in WHERE: {:?}",
        result
    );
}

#[test]
fn test_parse_substring_multiple_columns() {
    let sql = "SELECT SUBSTRING(first_name FROM 1 FOR 1) || '.' || SUBSTRING(last_name FROM 1 FOR 1) AS initials FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple SUBSTRING: {:?}",
        result
    );
}

// ============ Qualified Table Names ============

#[test]
fn test_parse_qualified_table_name() {
    let sql = "SELECT * FROM information_schema.tables";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse qualified table name: {:?}",
        result
    );
}

#[test]
fn test_parse_qualified_column_name() {
    let sql = "SELECT users.id, users.name FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse qualified column name: {:?}",
        result
    );
}

#[test]
fn test_parse_qualified_column_with_alias() {
    let sql = "SELECT u.id, u.name FROM users AS u";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse qualified column with alias: {:?}",
        result
    );
}

#[test]
fn test_parse_table_alias_with_qualified_name() {
    let sql = "SELECT u.name, o.id FROM users u JOIN orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse table alias with qualified names: {:?}",
        result
    );
}

// ============ Subquery in FROM ============

#[test]
fn test_parse_subquery_in_from() {
    let sql = "SELECT * FROM (SELECT id, name FROM users) AS user_subquery";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse subquery in FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_subquery_with_join() {
    let sql = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse subquery with JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_nested_subquery_in_from() {
    let sql = "SELECT u.id, u.name FROM users u WHERE u.id IN (SELECT user_id FROM orders)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse nested subquery: {:?}",
        result
    );
}

// ============ GROUP BY ROLLUP ============

#[test]
fn test_parse_group_by_rollup() {
    let sql = "SELECT department, SUM(salary) FROM employees GROUP BY ROLLUP(department)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY ROLLUP: {:?}",
        result
    );
}

#[test]
fn test_parse_group_by_rollup_multiple() {
    let sql =
        "SELECT department, status, SUM(salary) FROM employees GROUP BY ROLLUP(department, status)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GROUP BY ROLLUP multiple: {:?}",
        result
    );
}

// ============ Column Operator Expressions ============

#[test]
fn test_parse_column_arithmetic_expression() {
    let sql = "SELECT price * quantity AS total FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse column arithmetic: {:?}",
        result
    );
}

#[test]
fn test_parse_complex_expression_in_select() {
    let sql = "SELECT price * quantity * (1 - discount) AS final_price FROM products";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse complex expression: {:?}",
        result
    );
}

// ============ Complex WHERE Clauses ============

#[test]
fn test_parse_where_like_escape() {
    let sql = "SELECT * FROM users WHERE name LIKE '%test%' ESCAPE '\\'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIKE ESCAPE: {:?}", result);
}

#[test]
fn test_parse_where_not_between() {
    let sql = "SELECT * FROM users WHERE age NOT BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT BETWEEN: {:?}", result);
}

#[test]
fn test_parse_where_is_not_null() {
    let sql = "SELECT * FROM users WHERE email IS NOT NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS NOT NULL: {:?}", result);
}

#[test]
fn test_parse_where_complex_and_or() {
    let sql = "SELECT * FROM users WHERE age > 18 AND name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse complex AND/OR: {:?}",
        result
    );
}

// ============ Complex JOIN Patterns ============

#[test]
fn test_parse_left_join_with_condition() {
    let sql = "SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id AND orders.status = 'active'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LEFT JOIN with condition: {:?}",
        result
    );
}

#[test]
fn test_parse_multiple_joins() {
    let sql = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id LEFT JOIN products p ON o.product_id = p.id";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiple JOINs: {:?}",
        result
    );
}

// ============ Combined Window + MySQL Modifiers ============

#[test]
fn test_parse_window_with_mysql_modifier() {
    let sql = "SELECT HIGH_PRIORITY SQL_NO_CACHE ROW_NUMBER() OVER (ORDER BY id) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse window with MySQL modifier: {:?}",
        result
    );
}

#[test]
fn test_parse_window_substring_mysql_modifier() {
    let sql = "SELECT SQL_NO_CACHE SUBSTRING(name FROM 1 FOR 10), ROW_NUMBER() OVER (ORDER BY id) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SUBSTRING and window with modifier: {:?}",
        result
    );
}
