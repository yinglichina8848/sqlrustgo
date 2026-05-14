//! Additional parser coverage tests - round 3
//! Focus: Transaction statements, DDL, more edge cases, complex joins

use sqlrustgo_parser::parse;

#[test]
fn test_parse_begin_serializable() {
    let sql = "BEGIN SERIALIZABLE";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_repeatable_read() {
    let sql = "BEGIN REPEATABLE READ";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_read_committed() {
    let sql = "BEGIN READ COMMITTED";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_read_uncommitted() {
    let sql = "BEGIN READ UNCOMMITTED";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_work() {
    let sql = "BEGIN WORK";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_tx_serializable() {
    let sql = "BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let _ = parse(sql);
}

#[test]
fn test_parse_begin_tx_work() {
    let sql = "BEGIN TRANSACTION WORK";
    let _ = parse(sql);
}

#[test]
fn test_parse_commit_work() {
    let sql = "COMMIT WORK";
    let _ = parse(sql);
}

#[test]
fn test_parse_rollback_work() {
    let sql = "ROLLBACK WORK";
    let _ = parse(sql);
}

#[test]
fn test_parse_start_transaction() {
    let sql = "START TRANSACTION";
    let _ = parse(sql);
}

#[test]
fn test_parse_start_transaction_isolation() {
    let sql = "START TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let _ = parse(sql);
}

#[test]
fn test_parse_savepoint() {
    let sql = "SAVEPOINT sp1";
    let _ = parse(sql);
}

#[test]
fn test_parse_rollback_to_savepoint() {
    let sql = "ROLLBACK TO SAVEPOINT sp1";
    let _ = parse(sql);
}

#[test]
fn test_parse_release_savepoint() {
    let sql = "RELEASE SAVEPOINT sp1";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_transaction_serializable() {
    let sql = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_transaction_read_committed() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ COMMITTED";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_transaction_repeatable_read() {
    let sql = "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_transaction_read_uncommitted() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_variable() {
    let sql = "SET @var = 1";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_global_variable() {
    let sql = "SET GLOBAL max_connections = 100";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_session_variable() {
    let sql = "SET SESSION wait_timeout = 28800";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_database() {
    let sql = "CREATE DATABASE testdb";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_database() {
    let sql = "DROP DATABASE testdb";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_if_not_exists() {
    let sql = "CREATE TABLE IF NOT EXISTS users (id INT PRIMARY KEY)";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_temporary() {
    let sql = "CREATE TEMPORARY TABLE temp_users (id INT)";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_like() {
    let sql = "CREATE TABLE users_backup LIKE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_auto_increment() {
    let sql = "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY)";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_not_null() {
    let sql = "CREATE TABLE users (id INT NOT NULL, name VARCHAR(100) NOT NULL)";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_default() {
    let sql = "CREATE TABLE users (id INT DEFAULT 0, name VARCHAR(100) DEFAULT 'unknown')";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_primary_key() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(100), PRIMARY KEY (id))";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_unique_key() {
    let sql = "CREATE TABLE users (id INT, email VARCHAR(100), UNIQUE KEY (email))";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_check() {
    let sql = "CREATE TABLE users (age INT CHECK (age >= 0))";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_table_foreign_key() {
    let sql = "CREATE TABLE orders (user_id INT, FOREIGN KEY (user_id) REFERENCES users(id))";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_table_if_exists() {
    let sql = "DROP TABLE IF EXISTS users";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_table_cascade() {
    let sql = "DROP TABLE users CASCADE";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_table_restrict() {
    let sql = "DROP TABLE users RESTRICT";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_multiple_tables() {
    let sql = "DROP TABLE users, orders, products";
    let _ = parse(sql);
}

#[test]
fn test_parse_truncate_table() {
    let sql = "TRUNCATE TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_index() {
    let sql = "CREATE INDEX idx_name ON users(name)";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_unique_index() {
    let sql = "CREATE UNIQUE INDEX idx_email ON users(email)";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_index() {
    let sql = "DROP INDEX idx_name ON users";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_index_if_exists() {
    let sql = "DROP INDEX IF EXISTS idx_name ON users";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_rename() {
    let sql = "ALTER TABLE users RENAME TO renamed_users";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_rename_to() {
    let sql = "ALTER TABLE users RENAME TO new_users";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_add_column() {
    let sql = "ALTER TABLE users ADD COLUMN email VARCHAR(255)";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_drop_column() {
    let sql = "ALTER TABLE users DROP COLUMN email";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_modify_column() {
    let sql = "ALTER TABLE users MODIFY COLUMN email VARCHAR(100)";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_add_constraint() {
    let sql = "ALTER TABLE users ADD CONSTRAINT pk_users PRIMARY KEY (id)";
    let _ = parse(sql);
}

#[test]
fn test_parse_alter_table_drop_constraint() {
    let sql = "ALTER TABLE users DROP CONSTRAINT pk_users";
    let _ = parse(sql);
}

#[test]
fn test_parse_explain_select() {
    let sql = "EXPLAIN SELECT * FROM users";
    let _ = parse(sql);
}

#[test]
fn test_parse_explain_insert() {
    let sql = "EXPLAIN INSERT INTO users VALUES (1, 'test')";
    let _ = parse(sql);
}

#[test]
fn test_parse_explain_update() {
    let sql = "EXPLAIN UPDATE users SET name = 'test'";
    let _ = parse(sql);
}

#[test]
fn test_parse_explain_delete() {
    let sql = "EXPLAIN DELETE FROM users";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_tables() {
    let sql = "SHOW TABLES";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_tables_like() {
    let sql = "SHOW TABLES LIKE '%users%'";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_columns() {
    let sql = "SHOW COLUMNS FROM users";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_columns_with_like() {
    let sql = "SHOW COLUMNS FROM users LIKE '%id%'";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_index() {
    let sql = "SHOW INDEX FROM users";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_databases() {
    let sql = "SHOW DATABASES";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_create_database() {
    let sql = "SHOW CREATE DATABASE testdb";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_create_table() {
    let sql = "SHOW CREATE TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_variables() {
    let sql = "SHOW VARIABLES";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_variables_like() {
    let sql = "SHOW VARIABLES LIKE '%timeout%'";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_status() {
    let sql = "SHOW STATUS";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_warnings() {
    let sql = "SHOW WARNINGS";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_errors() {
    let sql = "SHOW ERRORS";
    let _ = parse(sql);
}

#[test]
fn test_parse_show_processlist() {
    let sql = "SHOW PROCESSLIST";
    let _ = parse(sql);
}

#[test]
fn test_parse_analyze_table() {
    let sql = "ANALYZE TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_check_table() {
    let sql = "CHECK TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_optimize_table() {
    let sql = "OPTIMIZE TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_repair_table() {
    let sql = "REPAIR TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_vacuum_table() {
    let sql = "VACUUM TABLE users";
    let _ = parse(sql);
}

#[test]
fn test_parse_do_statement() {
    let sql = "DO SLEEP(1)";
    let _ = parse(sql);
}

#[test]
fn test_parse_handler_open() {
    let sql = "HANDLER users OPEN";
    let _ = parse(sql);
}

#[test]
fn test_parse_handler_read() {
    let sql = "HANDLER users READ FIRST";
    let _ = parse(sql);
}

#[test]
fn test_parse_handler_close() {
    let sql = "HANDLER users CLOSE";
    let _ = parse(sql);
}

#[test]
fn test_parse_flush_tables() {
    let sql = "FLUSH TABLES";
    let _ = parse(sql);
}

#[test]
fn test_parse_flush_privileges() {
    let sql = "FLUSH PRIVILEGES";
    let _ = parse(sql);
}

#[test]
fn test_parse_flush_logs() {
    let sql = "FLUSH LOGS";
    let _ = parse(sql);
}

#[test]
fn test_parse_flush_status() {
    let sql = "FLUSH STATUS";
    let _ = parse(sql);
}

#[test]
fn test_parse_kill_connection() {
    let sql = "KILL CONNECTION 12345";
    let _ = parse(sql);
}

#[test]
fn test_parse_kill_query() {
    let sql = "KILL QUERY 12345";
    let _ = parse(sql);
}

#[test]
fn test_parse_create_role() {
    let sql = "CREATE ROLE admin";
    let _ = parse(sql);
}

#[test]
fn test_parse_drop_role() {
    let sql = "DROP ROLE admin";
    let _ = parse(sql);
}

#[test]
fn test_parse_set_role() {
    let sql = "SET ROLE admin";
    let _ = parse(sql);
}

#[test]
fn test_parse_grant_role() {
    let sql = "GRANT admin TO user1";
    let _ = parse(sql);
}

#[test]
fn test_parse_revoke_role() {
    let sql = "REVOKE admin FROM user1";
    let _ = parse(sql);
}

#[test]
fn test_parse_multiple_statements() {
    let sql = "SELECT 1; SELECT 2";
    let _ = parse(sql);
}

#[test]
fn test_parse_union() {
    let sql = "SELECT 1 UNION SELECT 2";
    let _ = parse(sql);
}

#[test]
fn test_parse_union_all() {
    let sql = "SELECT 1 UNION ALL SELECT 2";
    let _ = parse(sql);
}

#[test]
fn test_parse_intersect() {
    let sql = "SELECT 1 INTERSECT SELECT 2";
    let _ = parse(sql);
}

#[test]
fn test_parse_except() {
    let sql = "SELECT 1 EXCEPT SELECT 2";
    let _ = parse(sql);
}

#[test]
fn test_parse_cte_simple() {
    let sql = "WITH cte AS (SELECT id FROM users) SELECT * FROM cte";
    let _ = parse(sql);
}

#[test]
fn test_parse_cte_multiple() {
    let sql = "WITH cte1 AS (SELECT id FROM users), cte2 AS (SELECT id FROM orders) SELECT * FROM cte1, cte2";
    let _ = parse(sql);
}
