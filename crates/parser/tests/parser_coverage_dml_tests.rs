use sqlrustgo_parser::parse;

#[test]
fn test_create_trigger_before_insert() {
    let sql = "CREATE TRIGGER trigger1 BEFORE INSERT ON users FOR EACH ROW BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER BEFORE INSERT: {:?}",
        result
    );
}

#[test]
fn test_create_trigger_after_update() {
    let sql = "CREATE TRIGGER trigger2 AFTER UPDATE ON users FOR EACH ROW BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER AFTER UPDATE: {:?}",
        result
    );
}

#[test]
fn test_create_trigger_multiple_statements() {
    let sql = "CREATE TRIGGER trigger3 BEFORE DELETE ON orders FOR EACH ROW BEGIN INSERT INTO log VALUES (OLD.id); UPDATE stats SET count = count + 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER with multiple statements: {:?}",
        result
    );
}

#[test]
fn test_create_trigger_referencing_old() {
    let sql =
        "CREATE TRIGGER trigger4 BEFORE UPDATE ON users FOR EACH ROW BEGIN SELECT OLD.name; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER with OLD: {:?}",
        result
    );
}

#[test]
fn test_create_trigger_referencing_new() {
    let sql =
        "CREATE TRIGGER trigger5 BEFORE INSERT ON users FOR EACH ROW BEGIN SELECT NEW.name; END";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TRIGGER with NEW: {:?}",
        result
    );
}

#[test]
fn test_drop_view() {
    let sql = "DROP VIEW view1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP VIEW: {:?}", result);
}

#[test]
fn test_drop_view_if_exists() {
    let sql = "DROP VIEW IF EXISTS view1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP VIEW IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_drop_index() {
    let sql = "DROP INDEX idx1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP INDEX: {:?}", result);
}

#[test]
fn test_drop_table_if_exists() {
    let sql = "DROP TABLE IF EXISTS users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DROP TABLE IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_analyze_table() {
    let sql = "ANALYZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ANALYZE TABLE: {:?}",
        result
    );
}

#[test]
fn test_check_table() {
    let sql = "CHECK TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CHECK TABLE: {:?}", result);
}

#[test]
fn test_optimize_table() {
    let sql = "OPTIMIZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse OPTIMIZE TABLE: {:?}",
        result
    );
}

#[test]
fn test_repair_table() {
    let sql = "REPAIR TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPAIR TABLE: {:?}", result);
}

#[test]
fn test_show_tables() {
    let sql = "SHOW TABLES";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW TABLES: {:?}", result);
}

#[test]
fn test_show_tables_like() {
    let sql = "SHOW TABLES LIKE 'user%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW TABLES LIKE: {:?}",
        result
    );
}

#[test]
fn test_show_columns() {
    let sql = "SHOW COLUMNS FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW COLUMNS: {:?}", result);
}

#[test]
fn test_show_columns_from() {
    let sql = "SHOW COLUMNS FROM mydb.users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW COLUMNS FROM: {:?}",
        result
    );
}

#[test]
fn test_show_index() {
    let sql = "SHOW INDEX FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse SHOW INDEX: {:?}", result);
}

#[test]
fn test_show_table_status() {
    let sql = "SHOW TABLE STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse SHOW TABLE STATUS: {:?}",
        result
    );
}

#[test]
fn test_explain_select() {
    let sql = "EXPLAIN SELECT * FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse EXPLAIN SELECT: {:?}",
        result
    );
}

#[test]
fn test_explain_update() {
    let sql = "EXPLAIN UPDATE users SET name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse EXPLAIN UPDATE: {:?}",
        result
    );
}

#[test]
fn test_describe_table() {
    let sql = "DESCRIBE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DESCRIBE: {:?}", result);
}

#[test]
fn test_desc_table() {
    let sql = "DESC users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DESC: {:?}", result);
}

#[test]
fn test_grant_basic() {
    let sql = "GRANT SELECT ON users TO user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GRANT: {:?}", result);
}

#[test]
fn test_grant_with_option() {
    let sql = "GRANT SELECT, INSERT ON users TO user1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse GRANT with multiple privileges: {:?}",
        result
    );
}

#[test]
fn test_grant_role() {
    let sql = "GRANT admin TO user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse GRANT role: {:?}", result);
}

#[test]
fn test_revoke_basic() {
    let sql = "REVOKE SELECT ON users FROM user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REVOKE: {:?}", result);
}

#[test]
fn test_revoke_role() {
    let sql = "REVOKE admin FROM user1";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REVOKE role: {:?}", result);
}

#[test]
fn test_create_role() {
    let sql = "CREATE ROLE admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE ROLE: {:?}", result);
}

#[test]
fn test_drop_role() {
    let sql = "DROP ROLE admin";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse DROP ROLE: {:?}", result);
}

#[test]
fn test_create_index() {
    let sql = "CREATE INDEX idx1 ON users (name)";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE INDEX: {:?}", result);
}

#[test]
fn test_create_unique_index() {
    let sql = "CREATE UNIQUE INDEX idx1 ON users (name)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE UNIQUE INDEX: {:?}",
        result
    );
}

#[test]
fn test_alter_table_add_column() {
    let sql = "ALTER TABLE users ADD COLUMN age INT";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE ADD COLUMN: {:?}",
        result
    );
}

#[test]
fn test_alter_table_drop_column() {
    let sql = "ALTER TABLE users DROP COLUMN age";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE DROP COLUMN: {:?}",
        result
    );
}

#[test]
fn test_alter_table_rename_to() {
    let sql = "ALTER TABLE users RENAME TO new_users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ALTER TABLE RENAME TO: {:?}",
        result
    );
}

#[test]
fn test_backup_database() {
    let sql = "BACKUP DATABASE TO '/backup/db.dump'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BACKUP DATABASE: {:?}",
        result
    );
}

#[test]
fn test_restore_database() {
    let sql = "RESTORE DATABASE FROM '/backup/db.dump'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RESTORE DATABASE: {:?}",
        result
    );
}

#[test]
fn test_vacuum_table() {
    let sql = "VACUUM TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse VACUUM TABLE: {:?}", result);
}

#[test]
fn test_call_procedure() {
    let sql = "CALL my_procedure()";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CALL: {:?}", result);
}

#[test]
fn test_call_procedure_with_args() {
    let sql = "CALL my_procedure(1, 'test')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CALL with args: {:?}",
        result
    );
}

#[test]
fn test_create_view_simple() {
    let sql = "CREATE VIEW v1 AS SELECT * FROM users";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse CREATE VIEW: {:?}", result);
}

#[test]
fn test_window_function_row_number() {
    let sql = "SELECT ROW_NUMBER() OVER (PARTITION BY dept ORDER BY salary DESC) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse ROW_NUMBER window function: {:?}",
        result
    );
}

#[test]
fn test_window_function_rank() {
    let sql = "SELECT RANK() OVER (ORDER BY salary DESC) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse RANK window function: {:?}",
        result
    );
}

#[test]
fn test_window_function_dense_rank() {
    let sql = "SELECT DENSE_RANK() OVER (ORDER BY salary DESC) FROM employees";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DENSE_RANK window function: {:?}",
        result
    );
}

#[test]
fn test_with_clause_simple() {
    let sql = "WITH cte AS (SELECT id FROM users) SELECT * FROM cte";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse WITH clause: {:?}", result);
}

#[test]
fn test_with_clause_multiple() {
    let sql = "WITH cte1 AS (SELECT id FROM users), cte2 AS (SELECT id FROM orders) SELECT * FROM cte1, cte2";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse WITH multiple CTEs: {:?}",
        result
    );
}

#[test]
fn test_replace() {
    let sql = "REPLACE INTO users VALUES (1, 'test')";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse REPLACE: {:?}", result);
}

#[test]
fn test_delete_with_where() {
    let sql = "DELETE FROM users WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DELETE with WHERE: {:?}",
        result
    );
}

#[test]
fn test_delete_with_order_by_limit() {
    let sql = "DELETE FROM users ORDER BY created_at DESC LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse DELETE with ORDER BY LIMIT: {:?}",
        result
    );
}

#[test]
fn test_update_with_where() {
    let sql = "UPDATE users SET name = 'test' WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UPDATE with WHERE: {:?}",
        result
    );
}

#[test]
fn test_update_multiple_columns() {
    let sql = "UPDATE users SET name = 'test', age = 25 WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse UPDATE multiple columns: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_primary_key() {
    let sql = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with PRIMARY KEY: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_foreign_key() {
    let sql = "CREATE TABLE orders (id INT, user_id INT REFERENCES users(id))";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with FOREIGN KEY: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_not_null() {
    let sql = "CREATE TABLE users (id INT NOT NULL, name VARCHAR(100) NOT NULL)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with NOT NULL: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_unique() {
    let sql = "CREATE TABLE users (id INT UNIQUE, email VARCHAR(100) UNIQUE)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with UNIQUE: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_default() {
    let sql = "CREATE TABLE users (id INT, name VARCHAR(100) DEFAULT 'Anonymous')";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with DEFAULT: {:?}",
        result
    );
}

#[test]
fn test_create_table_with_auto_increment() {
    let sql = "CREATE TABLE users (id INT AUTO_INCREMENT PRIMARY KEY)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CREATE TABLE with AUTO_INCREMENT: {:?}",
        result
    );
}

#[test]
fn test_cast_expression() {
    let sql = "SELECT CAST(name AS VARCHAR(100)) FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CAST expression: {:?}",
        result
    );
}

#[test]
fn test_nullif_expression() {
    let sql = "SELECT NULLIF(a, b) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse NULLIF expression: {:?}",
        result
    );
}

#[test]
fn test_coalesce_expression() {
    let sql = "SELECT COALESCE(a, b, 'default') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse COALESCE expression: {:?}",
        result
    );
}

#[test]
fn test_subquery_in_select() {
    let sql = "SELECT * FROM (SELECT id FROM users) AS subq";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse subquery in SELECT: {:?}",
        result
    );
}

#[test]
fn test_subquery_in_where() {
    let sql = "SELECT * FROM users WHERE id IN (SELECT id FROM admins)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse subquery in WHERE: {:?}",
        result
    );
}

#[test]
fn test_exists_subquery() {
    let sql =
        "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse EXISTS subquery: {:?}",
        result
    );
}

#[test]
fn test_in_expression() {
    let sql = "SELECT * FROM users WHERE id IN (1, 2, 3)";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IN expression: {:?}",
        result
    );
}

#[test]
fn test_between_expression() {
    let sql = "SELECT * FROM users WHERE age BETWEEN 18 AND 65";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse BETWEEN expression: {:?}",
        result
    );
}

#[test]
fn test_like_expression() {
    let sql = "SELECT * FROM users WHERE name LIKE 'John%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse LIKE expression: {:?}",
        result
    );
}

#[test]
fn test_not_like_expression() {
    let sql = "SELECT * FROM users WHERE name NOT LIKE 'John%'";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse NOT LIKE expression: {:?}",
        result
    );
}

#[test]
fn test_is_null_expression() {
    let sql = "SELECT * FROM users WHERE name IS NULL";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IS NULL expression: {:?}",
        result
    );
}

#[test]
fn test_is_not_null_expression() {
    let sql = "SELECT * FROM users WHERE name IS NOT NULL";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse IS NOT NULL expression: {:?}",
        result
    );
}

#[test]
fn test_case_when_expression() {
    let sql = "SELECT CASE WHEN status = 1 THEN 'active' ELSE 'inactive' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE WHEN expression: {:?}",
        result
    );
}

#[test]
fn test_case_when_with_multiple() {
    let sql = "SELECT CASE WHEN status = 1 THEN 'active' WHEN status = 2 THEN 'pending' ELSE 'unknown' END FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok(),
        "Failed to parse CASE WHEN with multiple conditions: {:?}",
        result
    );
}
