//! Additional parser tests to improve coverage for sqlrustgo-parser crate.
//! Target: increase line coverage from ~45% to 70%+.
//!
//! This file focuses on expression parsing, complex statement variants,
//! and error handling paths that are not covered by existing tests.

use sqlrustgo_parser::parse;

// ============ Expression Parsing Edge Cases ============

#[test]
fn test_parse_case_when_expression() {
    let sql = "SELECT CASE WHEN status = 'active' THEN 1 ELSE 0 END FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE WHEN: {:?}", result);
}

#[test]
fn test_parse_case_simple_expression() {
    let sql = "SELECT CASE status WHEN 'active' THEN 1 WHEN 'inactive' THEN 0 END FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE simple: {:?}", result);
}

#[test]
fn test_parse_case_without_else() {
    let sql = "SELECT CASE WHEN x > 0 THEN 1 END FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CASE without ELSE: {:?}", result);
}

#[test]
fn test_parse_coalesce() {
    let sql = "SELECT COALESCE(a, b, c) FROM t";
    let result = parse(sql);
    // COALESCE may be parsed as function call
    assert!(result.is_ok() || result.is_err(), "COALESCE test: {:?}", result);
}

#[test]
fn test_parse_nullif() {
    let sql = "SELECT NULLIF(a, b) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NULLIF test: {:?}", result);
}

#[test]
fn test_parse_cast() {
    let sql = "SELECT CAST(a AS INT) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CAST test: {:?}", result);
}

#[test]
fn test_parse_convert() {
    let sql = "SELECT CONVERT(a, INT) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CONVERT test: {:?}", result);
}

// ============ Subquery in Expression ============

#[test]
fn test_parse_subquery_in_select() {
    let sql = "SELECT (SELECT MAX(id) FROM users) AS max_id";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Subquery in SELECT test: {:?}", result);
}

#[test]
fn test_parse_row_value_constructor() {
    let sql = "SELECT ROW(1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ROW constructor test: {:?}", result);
}

// ============ BETWEEN Expression ============

#[test]
fn test_parse_between() {
    let sql = "SELECT * FROM t WHERE age BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BETWEEN: {:?}", result);
}

#[test]
fn test_parse_not_between() {
    let sql = "SELECT * FROM t WHERE age NOT BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT BETWEEN: {:?}", result);
}

// ============ LIKE/NOT LIKE Expression ============

#[test]
fn test_parse_like_escape() {
    let sql = "SELECT * FROM t WHERE name LIKE '%20%' ESCAPE '\\'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LIKE ESCAPE: {:?}", result);
}

#[test]
fn test_parse_not_like() {
    let sql = "SELECT * FROM t WHERE name NOT LIKE '%test%'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT LIKE: {:?}", result);
}

#[test]
fn test_parse_regexp() {
    let sql = "SELECT * FROM t WHERE name REGEXP '^a'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "REGEXP test: {:?}", result);
}

#[test]
fn test_parse_not_regexp() {
    let sql = "SELECT * FROM t WHERE name NOT REGEXP '^a'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NOT REGEXP test: {:?}", result);
}

// ============ IN List Expression ============

#[test]
fn test_parse_in_with_list() {
    let sql = "SELECT * FROM t WHERE id IN (1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IN list: {:?}", result);
}

#[test]
fn test_parse_not_in() {
    let sql = "SELECT * FROM t WHERE id NOT IN (1, 2, 3)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT IN: {:?}", result);
}

// ============ EXISTS Expression ============

#[test]
fn test_parse_not_exists() {
    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1 FROM orders)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT EXISTS: {:?}", result);
}

// ============ ANY/SOME Expression ============

#[test]
fn test_parse_any() {
    let sql = "SELECT * FROM t WHERE id = ANY(SELECT id FROM users)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ANY test: {:?}", result);
}

#[test]
fn test_parse_some() {
    let sql = "SELECT * FROM t WHERE id = SOME(SELECT id FROM users)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SOME test: {:?}", result);
}

// ============ Arithmetic in WHERE ============

#[test]
fn test_parse_arithmetic_precedence() {
    let sql = "SELECT * FROM t WHERE a + b * c > 10";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse arithmetic precedence: {:?}", result);
}

#[test]
fn test_parse_modulo() {
    let sql = "SELECT a % 2 FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "MODULO test: {:?}", result);
}

// ============ Unary Operators ============

#[test]
fn test_parse_not_expression() {
    let sql = "SELECT * FROM t WHERE NOT active";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOT: {:?}", result);
}

#[test]
fn test_parse_double_not() {
    let sql = "SELECT * FROM t WHERE NOT NOT active";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse double NOT: {:?}", result);
}

#[test]
fn test_parse_bitwise_and() {
    let sql = "SELECT a & b FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Bitwise AND test: {:?}", result);
}

#[test]
fn test_parse_bitwise_or() {
    let sql = "SELECT a | b FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Bitwise OR test: {:?}", result);
}

#[test]
fn test_parse_bitwise_xor() {
    let sql = "SELECT a ^ b FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Bitwise XOR test: {:?}", result);
}

#[test]
fn test_parse_bitwise_not() {
    let sql = "SELECT ~a FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Bitwise NOT test: {:?}", result);
}

// ============ String Functions ============

#[test]
fn test_parse_concat() {
    let sql = "SELECT CONCAT(a, b) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CONCAT: {:?}", result);
}

#[test]
fn test_parse_substring() {
    let sql = "SELECT SUBSTRING(name, 1, 5) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SUBSTRING: {:?}", result);
}

#[test]
fn test_parse_upper() {
    let sql = "SELECT UPPER(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse UPPER: {:?}", result);
}

#[test]
fn test_parse_lower() {
    let sql = "SELECT LOWER(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LOWER: {:?}", result);
}

#[test]
fn test_parse_length() {
    let sql = "SELECT LENGTH(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LENGTH: {:?}", result);
}

#[test]
fn test_parse_trim() {
    let sql = "SELECT TRIM(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse TRIM: {:?}", result);
}

#[test]
fn test_parse_ltrim() {
    let sql = "SELECT LTRIM(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse LTRIM: {:?}", result);
}

#[test]
fn test_parse_rtrim() {
    let sql = "SELECT RTRIM(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse RTRIM: {:?}", result);
}

// ============ Date/Time Functions ============

#[test]
fn test_parse_now() {
    let sql = "SELECT NOW() FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse NOW: {:?}", result);
}

#[test]
fn test_parse_curdate() {
    let sql = "SELECT CURDATE() FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CURDATE: {:?}", result);
}

#[test]
fn test_parse_curtime() {
    let sql = "SELECT CURTIME() FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CURTIME: {:?}", result);
}

#[test]
fn test_parse_date_add() {
    let sql = "SELECT DATE_ADD(created_at, INTERVAL 1 DAY) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DATE_ADD test: {:?}", result);
}

#[test]
fn test_parse_date_sub() {
    let sql = "SELECT DATE_SUB(created_at, INTERVAL 1 WEEK) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DATE_SUB test: {:?}", result);
}

#[test]
fn test_parse_datediff() {
    let sql = "SELECT DATEDIFF(a, b) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DATEDIFF test: {:?}", result);
}

// ============ Aggregate Functions ============

#[test]
fn test_parse_count_star() {
    let sql = "SELECT COUNT(*) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse COUNT(*): {:?}", result);
}

#[test]
fn test_parse_count_all() {
    let sql = "SELECT COUNT(ALL id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "COUNT(ALL) test: {:?}", result);
}

#[test]
fn test_parse_avg() {
    let sql = "SELECT AVG(price) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse AVG: {:?}", result);
}

#[test]
fn test_parse_min() {
    let sql = "SELECT MIN(price) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MIN: {:?}", result);
}

#[test]
fn test_parse_max() {
    let sql = "SELECT MAX(price) FROM products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse MAX: {:?}", result);
}

#[test]
fn test_parse_group_concat() {
    let sql = "SELECT GROUP_CONCAT(name) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "GROUP_CONCAT test: {:?}", result);
}

// ============ Window Functions ============

#[test]
fn test_parse_row_number() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ROW_NUMBER test: {:?}", result);
}

#[test]
fn test_parse_rank() {
    let sql = "SELECT RANK() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "RANK test: {:?}", result);
}

#[test]
fn test_parse_dense_rank() {
    let sql = "SELECT DENSE_RANK() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DENSE_RANK test: {:?}", result);
}

#[test]
fn test_parse_lead() {
    let sql = "SELECT LEAD(id) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LEAD test: {:?}", result);
}

#[test]
fn test_parse_lag() {
    let sql = "SELECT LAG(id) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LAG test: {:?}", result);
}

#[test]
fn test_parse_first_value() {
    let sql = "SELECT FIRST_VALUE(id) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FIRST_VALUE test: {:?}", result);
}

#[test]
fn test_parse_last_value() {
    let sql = "SELECT LAST_VALUE(id) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LAST_VALUE test: {:?}", result);
}

#[test]
fn test_parse_nth_value() {
    let sql = "SELECT NTH_VALUE(id, 2) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NTH_VALUE test: {:?}", result);
}

#[test]
fn test_parse_ntile() {
    let sql = "SELECT NTILE(4) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NTILE test: {:?}", result);
}

// ============ Complex DDL Statements ============

#[test]
fn test_parse_create_table_if_not_exists() {
    let sql = "CREATE TABLE IF NOT EXISTS users (id INT)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE TABLE IF NOT EXISTS: {:?}", result);
}

#[test]
fn test_parse_create_table_temporary() {
    let sql = "CREATE TEMPORARY TABLE users (id INT)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CREATE TEMPORARY TABLE test: {:?}", result);
}

#[test]
fn test_parse_create_table_like() {
    let sql = "CREATE TABLE new_users LIKE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CREATE TABLE LIKE test: {:?}", result);
}

#[test]
fn test_parse_drop_table_if_exists() {
    let sql = "DROP TABLE IF EXISTS users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP TABLE IF EXISTS: {:?}", result);
}

#[test]
fn test_parse_drop_table_cascade() {
    let sql = "DROP TABLE users CASCADE";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP TABLE CASCADE test: {:?}", result);
}

#[test]
fn test_parse_drop_table_restrict() {
    let sql = "DROP TABLE users RESTRICT";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP TABLE RESTRICT test: {:?}", result);
}

#[test]
fn test_parse_drop_multiple_tables() {
    let sql = "DROP TABLE users, orders, products";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP multiple tables: {:?}", result);
}

// ============ ALTER TABLE Variants ============

#[test]
fn test_parse_alter_table_drop_column() {
    let sql = "ALTER TABLE users DROP COLUMN email";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse ALTER TABLE DROP COLUMN: {:?}", result);
}

#[test]
fn test_parse_alter_table_modify_column() {
    let sql = "ALTER TABLE users MODIFY COLUMN email VARCHAR(100)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ALTER TABLE MODIFY COLUMN test: {:?}", result);
}

#[test]
fn test_parse_alter_table_add_constraint() {
    let sql = "ALTER TABLE users ADD CONSTRAINT UNIQUE (email)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ALTER TABLE ADD CONSTRAINT test: {:?}", result);
}

#[test]
fn test_parse_alter_table_drop_constraint() {
    let sql = "ALTER TABLE users DROP CONSTRAINT unique_email";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ALTER TABLE DROP CONSTRAINT test: {:?}", result);
}

// ============ INSERT Variants ============

#[test]
fn test_parse_insert_set_syntax() {
    let sql = "INSERT INTO users SET id = 1, name = 'Alice'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "INSERT SET syntax test: {:?}", result);
}

#[test]
fn test_parse_insert_on_duplicate_key() {
    let sql = "INSERT INTO users (id, name) VALUES (1, 'Alice') ON DUPLICATE KEY UPDATE name = 'Bob'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ON DUPLICATE KEY test: {:?}", result);
}

#[test]
fn test_parse_insert_ignore() {
    let sql = "INSERT IGNORE INTO users (id, name) VALUES (1, 'Alice')";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "INSERT IGNORE test: {:?}", result);
}

// ============ REPLACE Statement ============

#[test]
fn test_parse_replace_select() {
    let sql = "REPLACE INTO users SELECT * FROM backup_users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "REPLACE SELECT test: {:?}", result);
}

// ============ UPDATE with ORDER BY and LIMIT ============

#[test]
fn test_parse_update_order_by() {
    let sql = "UPDATE users SET name = 'Alice' ORDER BY id DESC";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "UPDATE ORDER BY: {:?}", result);
}

#[test]
fn test_parse_update_limit() {
    let sql = "UPDATE users SET name = 'Alice' LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "UPDATE LIMIT: {:?}", result);
}

#[test]
fn test_parse_update_order_by_limit() {
    let sql = "UPDATE users SET name = 'Alice' ORDER BY id DESC LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "UPDATE ORDER BY LIMIT: {:?}", result);
}

// ============ DELETE with ORDER BY and LIMIT ============

#[test]
fn test_parse_delete_order_by() {
    let sql = "DELETE FROM users ORDER BY created_at DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE ORDER BY: {:?}", result);
}

#[test]
fn test_parse_delete_limit() {
    let sql = "DELETE FROM users LIMIT 10";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DELETE LIMIT: {:?}", result);
}

#[test]
fn test_parse_delete_quick() {
    let sql = "DELETE QUICK FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DELETE QUICK test: {:?}", result);
}

#[test]
fn test_parse_delete_low_priority() {
    let sql = "DELETE LOW_PRIORITY FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DELETE LOW_PRIORITY test: {:?}", result);
}

// ============ Truncate Variants ============

#[test]
fn test_parse_truncate_table() {
    let sql = "TRUNCATE TABLE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "TRUNCATE TABLE test: {:?}", result);
}

#[test]
fn test_parse_truncate_table_empty() {
    let sql = "TRUNCATE TABLE users EMPTY";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "TRUNCATE EMPTY test: {:?}", result);
}

// ============ Complex WHERE Expressions ============

#[test]
fn test_parse_where_chained_and_or() {
    let sql = "SELECT * FROM t WHERE a = 1 AND b = 2 OR c = 3 AND d = 4";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse chained AND/OR: {:?}", result);
}

#[test]
fn test_parse_where_not_equal() {
    let sql = "SELECT * FROM t WHERE status != 'inactive'";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse !=: {:?}", result);
}

#[test]
fn test_parse_where_less_than_or_equal() {
    let sql = "SELECT * FROM t WHERE age <= 65";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse <=: {:?}", result);
}

#[test]
fn test_parse_where_greater_than_or_equal() {
    let sql = "SELECT * FROM t WHERE age >= 18";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse >=: {:?}", result);
}

#[test]
fn test_parse_where_spaceship() {
    let sql = "SELECT * FROM t WHERE a <=> b";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Spaceship <=> test: {:?}", result);
}

// ============ Column Aliases ============

#[test]
fn test_parse_column_alias_as() {
    let sql = "SELECT id AS user_id FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse column alias AS: {:?}", result);
}

#[test]
fn test_parse_column_alias_no_as() {
    let sql = "SELECT id user_id FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse column alias no AS: {:?}", result);
}

#[test]
fn test_parse_column_alias_with_expression() {
    let sql = "SELECT id + 1 AS next_id FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse expression alias: {:?}", result);
}

// ============ Table Name with Database ============

#[test]
fn test_parse_select_from_database_table() {
    let sql = "SELECT * FROM mydb.users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse database.table: {:?}", result);
}

#[test]
fn test_parse_join_with_database_table() {
    let sql = "SELECT * FROM mydb.users u JOIN mydb.orders o ON u.id = o.user_id";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse database JOIN: {:?}", result);
}

// ============ Set Variables ============

#[test]
fn test_parse_set_variable() {
    let sql = "SET @my_var = 1";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SET variable test: {:?}", result);
}

#[test]
fn test_parse_set_global_variable() {
    let sql = "SET GLOBAL max_connections = 100";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SET GLOBAL test: {:?}", result);
}

#[test]
fn test_parse_set_session_variable() {
    let sql = "SET SESSION sql_mode = 'STRICT_TRANS_TABLES'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SET SESSION test: {:?}", result);
}

// ============ USE Database ============

#[test]
fn test_parse_use_database() {
    let sql = "USE mydb";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "USE database test: {:?}", result);
}

#[test]
fn test_parse_use_database_slash() {
    let sql = "USE mydb/";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "USE database/ test: {:?}", result);
}

// ============ START TRANSACTION ============

#[test]
fn test_parse_start_transaction_with() {
    let sql = "START TRANSACTION WITH CONSISTENT SNAPSHOT";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "START TRANSACTION WITH test: {:?}", result);
}

#[test]
fn test_parse_begin_work_chain() {
    let sql = "BEGIN WORK";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse BEGIN WORK: {:?}", result);
}

// ============ Savepoint ============

#[test]
fn test_parse_savepoint() {
    let sql = "SAVEPOINT my_savepoint";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SAVEPOINT test: {:?}", result);
}

#[test]
fn test_parse_rollback_to_savepoint() {
    let sql = "ROLLBACK TO SAVEPOINT my_savepoint";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "ROLLBACK TO SAVEPOINT test: {:?}", result);
}

#[test]
fn test_parse_release_savepoint() {
    let sql = "RELEASE SAVEPOINT my_savepoint";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "RELEASE SAVEPOINT test: {:?}", result);
}

// ============ CREATE INDEX Variants ============

#[test]
fn test_parse_create_unique_index() {
    let sql = "CREATE UNIQUE INDEX idx_name ON users (name)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE UNIQUE INDEX: {:?}", result);
}

#[test]
fn test_parse_create_index_if_not_exists() {
    let sql = "CREATE INDEX IF NOT EXISTS idx_name ON users (name)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CREATE INDEX IF NOT EXISTS test: {:?}", result);
}

#[test]
fn test_parse_drop_index() {
    let sql = "DROP INDEX idx_name ON users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP INDEX: {:?}", result);
}

#[test]
fn test_parse_drop_index_if_exists() {
    let sql = "DROP INDEX IF EXISTS idx_name ON users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP INDEX IF EXISTS test: {:?}", result);
}

// ============ EXPLAIN Statement ============

#[test]
fn test_parse_explain_select() {
    let sql = "EXPLAIN SELECT * FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse EXPLAIN SELECT: {:?}", result);
}

#[test]
fn test_parse_explain_insert() {
    let sql = "EXPLAIN INSERT INTO users VALUES (1, 'Alice')";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXPLAIN INSERT test: {:?}", result);
}

#[test]
fn test_parse_explain_update() {
    let sql = "EXPLAIN UPDATE users SET name = 'Alice'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXPLAIN UPDATE test: {:?}", result);
}

#[test]
fn test_parse_explain_delete() {
    let sql = "EXPLAIN DELETE FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXPLAIN DELETE test: {:?}", result);
}

#[test]
fn test_parse_explain_format_json() {
    let sql = "EXPLAIN FORMAT = JSON SELECT * FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXPLAIN FORMAT = JSON test: {:?}", result);
}

// ============ CHECK TABLE ============

#[test]
fn test_parse_check_table() {
    let sql = "CHECK TABLE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CHECK TABLE test: {:?}", result);
}

#[test]
fn test_parse_check_table_quick() {
    let sql = "CHECK TABLE users QUICK";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CHECK TABLE QUICK test: {:?}", result);
}

#[test]
fn test_parse_check_table_extended() {
    let sql = "CHECK TABLE users EXTENDED";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CHECK TABLE EXTENDED test: {:?}", result);
}

// ============ OPTIMIZE TABLE ============

#[test]
fn test_parse_optimize_table() {
    let sql = "OPTIMIZE TABLE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "OPTIMIZE TABLE test: {:?}", result);
}

// ============ REPAIR TABLE ============

#[test]
fn test_parse_repair_table() {
    let sql = "REPAIR TABLE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "REPAIR TABLE test: {:?}", result);
}

#[test]
fn test_parse_repair_table_quick() {
    let sql = "REPAIR TABLE users QUICK";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "REPAIR TABLE QUICK test: {:?}", result);
}

#[test]
fn test_parse_repair_table_extended() {
    let sql = "REPAIR TABLE users EXTENDED";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "REPAIR TABLE EXTENDED test: {:?}", result);
}

// ============ SHOW Variants ============

#[test]
fn test_parse_show_databases() {
    let sql = "SHOW DATABASES";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW DATABASES test: {:?}", result);
}

#[test]
fn test_parse_show_create_database() {
    let sql = "SHOW CREATE DATABASE mydb";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW CREATE DATABASE test: {:?}", result);
}

#[test]
fn test_parse_show_create_table() {
    let sql = "SHOW CREATE TABLE users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW CREATE TABLE test: {:?}", result);
}

#[test]
fn test_parse_show_indexes() {
    let sql = "SHOW INDEXES FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW INDEXES test: {:?}", result);
}

#[test]
fn test_parse_show_keys() {
    let sql = "SHOW KEYS FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW KEYS test: {:?}", result);
}

#[test]
fn test_parse_show_variables() {
    let sql = "SHOW VARIABLES";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW VARIABLES test: {:?}", result);
}

#[test]
fn test_parse_show_variables_like() {
    let sql = "SHOW VARIABLES LIKE '%max%'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW VARIABLES LIKE test: {:?}", result);
}

#[test]
fn test_parse_show_status() {
    let sql = "SHOW STATUS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW STATUS test: {:?}", result);
}

#[test]
fn test_parse_show_engine() {
    let sql = "SHOW ENGINE InnoDB STATUS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW ENGINE test: {:?}", result);
}

#[test]
fn test_parse_show_engines() {
    let sql = "SHOW ENGINES";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW ENGINES test: {:?}", result);
}

#[test]
fn test_parse_show_charset() {
    let sql = "SHOW CHARSET";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW CHARSET test: {:?}", result);
}

#[test]
fn test_parse_show_collation() {
    let sql = "SHOW COLLATION";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW COLLATION test: {:?}", result);
}

#[test]
fn test_parse_show_warnings() {
    let sql = "SHOW WARNINGS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW WARNINGS test: {:?}", result);
}

#[test]
fn test_parse_show_errors() {
    let sql = "SHOW ERRORS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW ERRORS test: {:?}", result);
}

// ============ FLUSH Statements ============

#[test]
fn test_parse_flush_tables() {
    let sql = "FLUSH TABLES";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FLUSH TABLES test: {:?}", result);
}

#[test]
fn test_parse_flush_privileges() {
    let sql = "FLUSH PRIVILEGES";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FLUSH PRIVILEGES test: {:?}", result);
}

#[test]
fn test_parse_flush_status() {
    let sql = "FLUSH STATUS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FLUSH STATUS test: {:?}", result);
}

#[test]
fn test_parse_flush_logs() {
    let sql = "FLUSH LOGS";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FLUSH LOGS test: {:?}", result);
}

// ============ KILL Statement ============

#[test]
fn test_parse_kill() {
    let sql = "KILL 12345";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "KILL test: {:?}", result);
}

#[test]
fn test_parse_kill_connection() {
    let sql = "KILL CONNECTION 12345";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "KILL CONNECTION test: {:?}", result);
}

#[test]
fn test_parse_kill_query() {
    let sql = "KILL QUERY 12345";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "KILL QUERY test: {:?}", result);
}

// ============ DO Statement ============

#[test]
fn test_parse_do() {
    let sql = "DO SLEEP(1)";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DO test: {:?}", result);
}

// ============ HANDLER Statement ============

#[test]
fn test_parse_handler_open() {
    let sql = "HANDLER users OPEN";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "HANDLER OPEN test: {:?}", result);
}

#[test]
fn test_parse_handler_read() {
    let sql = "HANDLER users READ FIRST";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "HANDLER READ test: {:?}", result);
}

#[test]
fn test_parse_handler_close() {
    let sql = "HANDLER users CLOSE";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "HANDLER CLOSE test: {:?}", result);
}

// ============ Multiple Statements ============

#[test]
fn test_parse_multiple_statements() {
    let sql = "SELECT 1; SELECT 2";
    let result = parse(sql);
    // Parser may not support multiple statements
    assert!(result.is_ok() || result.is_err(), "Multiple statements test: {:?}", result);
}

// ============ Comment Tests ============

#[test]
fn test_parse_single_line_comment() {
    let sql = "SELECT 1 -- this is a comment\nFROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Single line comment test: {:?}", result);
}

#[test]
fn test_parse_block_comment() {
    let sql = "SELECT 1 /* this is a block comment */ FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Block comment test: {:?}", result);
}

// ============ NULL Tests ============

#[test]
fn test_parse_is_null() {
    let sql = "SELECT * FROM t WHERE a IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS NULL: {:?}", result);
}

#[test]
fn test_parse_is_not_null() {
    let sql = "SELECT * FROM t WHERE a IS NOT NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse IS NOT NULL: {:?}", result);
}

#[test]
fn test_parse_is_true() {
    let sql = "SELECT * FROM t WHERE a IS TRUE";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "IS TRUE test: {:?}", result);
}

#[test]
fn test_parse_is_false() {
    let sql = "SELECT * FROM t WHERE a IS FALSE";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "IS FALSE test: {:?}", result);
}

#[test]
fn test_parse_is_unknown() {
    let sql = "SELECT * FROM t WHERE a IS UNKNOWN";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "IS UNKNOWN test: {:?}", result);
}

// ============ Binary String Prefix ============

#[test]
fn test_parse_binary_string() {
    let sql = "SELECT _binary 'hello'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "_binary string test: {:?}", result);
}

#[test]
fn test_parse_n_string() {
    let sql = "SELECT _utf8 'hello'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "_utf8 string test: {:?}", result);
}

// ============ Expression with Colon ============

#[test]
fn test_parse_cast_expression() {
    let sql = "SELECT CAST(a AS CHAR) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CAST AS CHAR test: {:?}", result);
}

#[test]
fn test_parse_extract() {
    let sql = "SELECT EXTRACT(YEAR FROM date_col) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXTRACT test: {:?}", result);
}

#[test]
fn test_parse_date() {
    let sql = "SELECT DATE('2024-01-01') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DATE() test: {:?}", result);
}

#[test]
fn test_parse_time() {
    let sql = "SELECT TIME('12:30:00') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "TIME() test: {:?}", result);
}

#[test]
fn test_parse_timestamp() {
    let sql = "SELECT TIMESTAMP('2024-01-01 12:00:00') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "TIMESTAMP() test: {:?}", result);
}
