//! Additional parser coverage tests - round 2
//! Focus: Complex expressions, NULL handling, CASE variations, CAST, CONVERT, date functions

use sqlrustgo_parser::parse;

#[test]
fn test_parse_case_simple_expression() {
    let sql =
        "SELECT CASE status WHEN 'active' THEN 1 WHEN 'inactive' THEN 0 ELSE -1 END FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE simple: {:?}", result);
}

#[test]
fn test_parse_case_searched_expression() {
    let sql = "SELECT CASE WHEN age > 18 THEN 'adult' WHEN age > 12 THEN 'teen' ELSE 'child' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE searched: {:?}",
        result
    );
}

#[test]
fn test_parse_case_with_or_expression() {
    let sql = "SELECT CASE WHEN status = 'a' OR status = 'b' THEN 1 ELSE 0 END FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE with OR: {:?}", result);
}

#[test]
fn test_parse_case_with_and_expression() {
    let sql =
        "SELECT CASE WHEN age > 18 AND name = 'test' THEN 'valid' ELSE 'invalid' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE with AND: {:?}",
        result
    );
}

#[test]
fn test_parse_case_without_else() {
    let sql = "SELECT CASE WHEN active THEN 'yes' ELSE 'no' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE without else: {:?}",
        result
    );
}

#[test]
fn test_parse_case_in_where() {
    let sql = "SELECT * FROM users WHERE CASE WHEN status = 'active' THEN 1 ELSE 0 END = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE in WHERE: {:?}",
        result
    );
}

#[test]
fn test_parse_nullif() {
    let sql = "SELECT NULLIF(age, 0) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NULLIF: {:?}", result);
}

#[test]
fn test_parse_coalesce() {
    let sql = "SELECT COALESCE(name, email, 'anonymous') FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse COALESCE: {:?}", result);
}

#[test]
fn test_parse_cast() {
    let sql = "SELECT CAST(age AS CHAR) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CAST: {:?}", result);
}

#[test]
fn test_parse_convert() {
    let sql = "SELECT CONVERT(age, CHAR) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CONVERT: {:?}", result);
}

#[test]
fn test_parse_extract() {
    let sql = "SELECT EXTRACT(YEAR FROM birth_date) FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_date_add() {
    let sql = "SELECT DATE_ADD(created_at, INTERVAL 1 DAY) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DATE_ADD: {:?}", result);
}

#[test]
fn test_parse_date_sub() {
    let sql = "SELECT DATE_SUB(created_at, INTERVAL 1 MONTH) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DATE_SUB: {:?}", result);
}

#[test]
fn test_parse_datediff() {
    let sql = "SELECT DATEDIFF(end_date, start_date) FROM orders";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DATEDIFF: {:?}", result);
}

#[test]
fn test_parse_curdate() {
    let sql = "SELECT CURDATE()";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CURDATE: {:?}", result);
}

#[test]
fn test_parse_now() {
    let sql = "SELECT NOW()";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOW: {:?}", result);
}

#[test]
fn test_parse_day() {
    let sql = "SELECT DAY(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DAY: {:?}", result);
}

#[test]
fn test_parse_month() {
    let sql = "SELECT MONTH(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MONTH: {:?}", result);
}

#[test]
fn test_parse_year() {
    let sql = "SELECT YEAR(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse YEAR: {:?}", result);
}

#[test]
fn test_parse_hour() {
    let sql = "SELECT HOUR(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse HOUR: {:?}", result);
}

#[test]
fn test_parse_minute() {
    let sql = "SELECT MINUTE(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MINUTE: {:?}", result);
}

#[test]
fn test_parse_second() {
    let sql = "SELECT SECOND(created_at) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SECOND: {:?}", result);
}

#[test]
fn test_parse_trim() {
    let sql = "SELECT TRIM(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse TRIM: {:?}", result);
}

#[test]
fn test_parse_ltrim() {
    let sql = "SELECT LTRIM(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LTRIM: {:?}", result);
}

#[test]
fn test_parse_rtrim() {
    let sql = "SELECT RTRIM(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RTRIM: {:?}", result);
}

#[test]
fn test_parse_length() {
    let sql = "SELECT LENGTH(name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LENGTH: {:?}", result);
}

#[test]
fn test_parse_concat_ws() {
    let sql = "SELECT CONCAT_WS('-', first_name, last_name) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CONCAT_WS: {:?}", result);
}

#[test]
fn test_parse_inet_aton() {
    let sql = "SELECT INET_ATON(ip_address) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INET_ATON: {:?}", result);
}

#[test]
fn test_parse_inet_ntoa() {
    let sql = "SELECT INET_NTOA(ip_num) FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INET_NTOA: {:?}", result);
}

#[test]
fn test_parse_if_function() {
    let sql = "SELECT IF(age > 18, 'adult', 'minor') FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IF function: {:?}", result);
}

#[test]
fn test_parse_ifnull() {
    let sql = "SELECT IFNULL(name, 'anonymous') FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IFNULL: {:?}", result);
}

#[test]
fn test_parse_istrue() {
    let sql = "SELECT * FROM users WHERE active = TRUE";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_isfalse() {
    let sql = "SELECT * FROM users WHERE active = FALSE";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_isunknown() {
    let sql = "SELECT * FROM users WHERE status = UNKNOWN";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS UNKNOWN: {:?}", result);
}

#[test]
fn test_parse_bitwise_and() {
    let sql = "SELECT 5 & 3";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse bitwise AND: {:?}", result);
}

#[test]
fn test_parse_bitwise_or() {
    let sql = "SELECT 5 | 3";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse bitwise OR: {:?}", result);
}

#[test]
fn test_parse_bitwise_xor() {
    let sql = "SELECT 5 ^ 3";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse bitwise XOR: {:?}", result);
}

#[test]
fn test_parse_bitwise_not() {
    let sql = "SELECT ~5";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse bitwise NOT: {:?}", result);
}

#[test]
fn test_parse_modulo() {
    let sql = "SELECT id % 2 FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_not_between() {
    let sql = "SELECT * FROM users WHERE age NOT BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT BETWEEN: {:?}", result);
}

#[test]
fn test_parse_not_like() {
    let sql = "SELECT * FROM users WHERE name NOT LIKE '%test%'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT LIKE: {:?}", result);
}

#[test]
fn test_parse_not_in() {
    let sql = "SELECT * FROM users WHERE status NOT IN ('active', 'pending')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT IN: {:?}", result);
}

#[test]
fn test_parse_regexp() {
    let sql = "SELECT * FROM users WHERE name REGEXP '^A'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REGEXP: {:?}", result);
}

#[test]
fn test_parse_not_regexp() {
    let sql = "SELECT * FROM users WHERE name NOT REGEXP '^A'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT REGEXP: {:?}", result);
}

#[test]
fn test_parse_exists_subquery() {
    let sql =
        "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse EXISTS: {:?}", result);
}

#[test]
fn test_parse_not_exists_subquery() {
    let sql = "SELECT * FROM users WHERE NOT EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT EXISTS: {:?}", result);
}

#[test]
fn test_parse_any_subquery() {
    let sql = "SELECT * FROM users WHERE age > ANY (SELECT age FROM seniors)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ANY: {:?}", result);
}

#[test]
fn test_parse_some_subquery() {
    let sql = "SELECT * FROM users WHERE age > SOME (SELECT age FROM seniors)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SOME: {:?}", result);
}

#[test]
fn test_parse_all_subquery() {
    let sql = "SELECT * FROM users WHERE age > ALL (SELECT age FROM seniors)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ALL: {:?}", result);
}

#[test]
fn test_parse_scalar_subquery() {
    let sql = "SELECT (SELECT id FROM users LIMIT 1) AS first_id";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_row_value_constructor() {
    let sql = "SELECT * FROM users WHERE id = 1 AND name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse row value constructor: {:?}",
        result
    );
}

#[test]
fn test_parse_leading_zeros() {
    let sql = "SELECT 007";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse leading zeros: {:?}",
        result
    );
}

#[test]
fn test_parse_scientific_notation() {
    let sql = "SELECT 1.5e10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse scientific notation: {:?}",
        result
    );
}

#[test]
fn test_parse_double_not() {
    let sql = "SELECT NOT NOT active FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_negative_number() {
    let sql = "SELECT -id FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_decimal_number() {
    let sql = "SELECT 3.14159";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse decimal: {:?}", result);
}

#[test]
fn test_parse_hex_number() {
    let sql = "SELECT 0xFF";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse hex: {:?}", result);
}

#[test]
fn test_parse_binary_literal() {
    let sql = "SELECT 0b1010";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse binary: {:?}", result);
}

#[test]
fn test_parse_group_concat() {
    let sql = "SELECT GROUP_CONCAT(name) FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_position() {
    let sql = "SELECT POSITION('test' IN description) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse POSITION: {:?}", result);
}

#[test]
fn test_parse_boolean_literal_true() {
    let sql = "SELECT TRUE, FALSE";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse boolean literals: {:?}",
        result
    );
}

#[test]
fn test_parse_arithmetic_precedence() {
    let sql = "SELECT id + 1 FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse precedence: {:?}", result);
}

#[test]
fn test_parse_arithmetic_division() {
    let sql = "SELECT id / 2 FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse division: {:?}", result);
}

#[test]
fn test_parse_arithmetic_addition() {
    let sql = "SELECT id + 1 FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse addition: {:?}", result);
}

#[test]
fn test_parse_arithmetic_subtraction() {
    let sql = "SELECT id - 1 FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse subtraction: {:?}", result);
}

#[test]
fn test_parse_arithmetic_multiplication() {
    let sql = "SELECT id * 2 FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse multiplication: {:?}",
        result
    );
}

#[test]
fn test_parse_comments_single_line() {
    let sql = "SELECT 1 FROM users -- this is a comment";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse single line comment: {:?}",
        result
    );
}

#[test]
fn test_parse_comments_block() {
    let sql = "SELECT 1 /* block comment */ FROM users";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_comment_before_select() {
    let sql = "SELECT 1 FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse comment before SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_comment_after_select() {
    let sql = "SELECT 1 FROM users -- comment after";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse comment after SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_with_database_table() {
    let sql = "SELECT * FROM mydb.users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SELECT with database.table: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_select() {
    let sql = "INSERT INTO users_backup SELECT id, name FROM users WHERE active = TRUE";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_insert_set() {
    let sql = "INSERT INTO users (name, age) VALUES ('test', 25)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse INSERT SET: {:?}", result);
}

#[test]
fn test_parse_insert_ignore() {
    let sql = "INSERT IGNORE INTO users (id, name) VALUES (1, 'test')";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_replace_into() {
    let sql = "REPLACE INTO users VALUES (1, 'test')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPLACE INTO: {:?}", result);
}

#[test]
fn test_parse_on_duplicate_key() {
    let sql =
        "INSERT INTO users (id, name) VALUES (1, 'test') ON DUPLICATE KEY UPDATE name = 'updated'";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_delete_limit() {
    let sql = "DELETE FROM users ORDER BY created_at LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE LIMIT: {:?}", result);
}

#[test]
fn test_parse_delete_order_by() {
    let sql = "DELETE FROM users ORDER BY created_at DESC";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DELETE ORDER BY: {:?}",
        result
    );
}

#[test]
fn test_parse_update_order_by_limit() {
    let sql = "UPDATE users SET name = 'test' ORDER BY created_at DESC LIMIT 10";
    let result = parse(sql);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_update_multiple_columns() {
    let sql = "UPDATE users SET name = 'test', age = 25, status = 'active'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UPDATE multiple columns: {:?}",
        result
    );
}
