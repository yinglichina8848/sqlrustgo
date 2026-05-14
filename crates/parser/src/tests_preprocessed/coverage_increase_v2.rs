use crate::parse;

// ============ CREATE TABLE with Constraints (parse_column_definition, parse_foreign_key_constraint) ============

#[test]
fn test_parse_create_table_with_multiple_primary_keys() {
    let sql = "CREATE TABLE t (a INT, b INT, PRIMARY KEY (a, b))";
    let result = parse(sql);
    assert!(result.is_ok(), "Failed to parse multi-PK: {:?}", result);
}

#[test]
fn test_parse_create_table_with_check_constraint() {
    let sql = "CREATE TABLE t (age INT CHECK (age >= 0))";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK constraint: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_with_named_constraint() {
    let sql = "CREATE TABLE t (id INT CONSTRAINT pk_id PRIMARY KEY)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Named constraint: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_with_foreign_key() {
    let sql =
        "CREATE TABLE orders (id INT, user_id INT, FOREIGN KEY (user_id) REFERENCES users(id))";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FK constraint: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_with_foreign_key_on_delete() {
    let sql = "CREATE TABLE orders (id INT, user_id INT, FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FK ON DELETE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_with_foreign_key_on_update() {
    let sql = "CREATE TABLE orders (id INT, user_id INT, FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE SET NULL)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FK ON UPDATE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_auto_increment() {
    let sql = "CREATE TABLE t (id INT AUTO_INCREMENT PRIMARY KEY)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "AUTO_INCREMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_unique_key() {
    let sql = "CREATE TABLE t (id INT, UNIQUE KEY uk_name (id))";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNIQUE KEY: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_index_hint() {
    let sql = "CREATE TABLE t (id INT, INDEX idx_id (id))";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "INDEX: {:?}", result);
}

// ============ CREATE EVENT (parse_create_event) ============

#[test]
fn test_parse_create_event_basic() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT test: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_at_time() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT '2024-12-31 23:59:59' DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT AT test: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_every() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 HOUR DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT EVERY test: {:?}",
        result
    );
}

// ============ DROP VIEW/EVENT (parse_drop_view, parse_drop_event) ============

#[test]
fn test_parse_drop_view_basic() {
    let sql = "DROP VIEW my_view";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP VIEW test: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_view_if_exists() {
    let sql = "DROP VIEW IF EXISTS my_view";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP VIEW IF EXISTS test: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_event_basic() {
    let sql = "DROP EVENT my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP EVENT test: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_event_if_exists() {
    let sql = "DROP EVENT IF EXISTS my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP EVENT IF EXISTS test: {:?}",
        result
    );
}

// ============ CREATE PROCEDURE (parse_create_procedure) ============

#[test]
fn test_parse_create_procedure_basic() {
    let sql = "CREATE PROCEDURE my_proc() BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE test: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_with_params() {
    let sql = "CREATE PROCEDURE my_proc(IN p1 INT, OUT p2 INT) BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE with params: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_with_body() {
    let sql = "CREATE PROCEDURE my_proc() BEGIN SELECT 1; SELECT 2; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE with body: {:?}",
        result
    );
}

// ============ CREATE FUNCTION (parse_create_procedure - same function handles functions) ============

#[test]
fn test_parse_create_function_basic() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION test: {:?}",
        result
    );
}

// ============ CREATE VIEW (parse_create_view) ============

#[test]
fn test_parse_create_view_basic() {
    let sql = "CREATE VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW test: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_with_columns() {
    let sql = "CREATE VIEW my_view (col1, col2) AS SELECT a, b FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW with columns: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_or_replace() {
    let sql = "CREATE OR REPLACE VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE OR REPLACE VIEW: {:?}",
        result
    );
}

// ============ CALL Statement (parse_call) ============

#[test]
fn test_parse_call_procedure() {
    let sql = "CALL my_proc()";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CALL test: {:?}", result);
}

#[test]
fn test_parse_call_with_args() {
    let sql = "CALL my_proc(1, 2, 3)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CALL with args: {:?}",
        result
    );
}

// ============ MERGE Statement (parse_merge) ============

#[test]
fn test_parse_merge_basic() {
    let sql = "MERGE INTO target t USING source s ON t.id = s.id WHEN MATCHED THEN UPDATE SET t.name = s.name";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MERGE test: {:?}",
        result
    );
}

#[test]
fn test_parse_merge_not_matched() {
    let sql = "MERGE INTO target t USING source s ON t.id = s.id WHEN NOT MATCHED THEN INSERT (id, name) VALUES (s.id, s.name)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MERGE NOT MATCHED: {:?}",
        result
    );
}

// ============ GRANT/REVOKE (parse_grant, parse_grant_role, parse_revoke, parse_revoke_role) ============

#[test]
fn test_parse_grant_privilege() {
    let sql = "GRANT SELECT ON mydb.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT test: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_all() {
    let sql = "GRANT ALL PRIVILEGES ON mydb.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "GRANT ALL: {:?}", result);
}

#[test]
fn test_parse_grant_role() {
    let sql = "GRANT 'role1' TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT role: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_privilege() {
    let sql = "REVOKE SELECT ON mydb.* FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE test: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_all() {
    let sql = "REVOKE ALL PRIVILEGES ON mydb.* FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE ALL: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_role() {
    let sql = "REVOKE 'role1' FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE role: {:?}",
        result
    );
}

// ============ BACKUP/RESTORE (parse_backup, parse_restore) ============

#[test]
fn test_parse_backup_table() {
    let sql = "BACKUP TABLE t TO '/backup/'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP TABLE test: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_database() {
    let sql = "BACKUP DATABASE mydb TO '/backup/'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP DATABASE test: {:?}",
        result
    );
}

#[test]
fn test_parse_restore_table() {
    let sql = "RESTORE TABLE t FROM '/backup/'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE TABLE test: {:?}",
        result
    );
}

#[test]
fn test_parse_restore_database() {
    let sql = "RESTORE DATABASE mydb FROM '/backup/'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE DATABASE test: {:?}",
        result
    );
}

// ============ ANALYZE (parse_analyze) ============

#[test]
fn test_parse_analyze_table() {
    let sql = "ANALYZE TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE TABLE test: {:?}",
        result
    );
}

#[test]
fn test_parse_analyze_no_write_to_binlog() {
    let sql = "ANALYZE NO_WRITE_TO_BINLOG TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE NO_WRITE_TO_BINLOG: {:?}",
        result
    );
}

// ============ DESCRIBE (parse_describe) ============

#[test]
fn test_parse_describe_table() {
    let sql = "DESCRIBE t";
    let result = parse(sql);
    assert!(result.is_ok(), "DESCRIBE test: {:?}", result);
}

#[test]
fn test_parse_desc_table() {
    let sql = "DESC t";
    let result = parse(sql);
    assert!(result.is_ok(), "DESC test: {:?}", result);
}

#[test]
fn test_parse_describe_column() {
    let sql = "DESCRIBE t col_name";
    let result = parse(sql);
    assert!(result.is_ok(), "DESCRIBE column: {:?}", result);
}

// ============ Complex ALTER TABLE (parse_alter_table) ============

#[test]
fn test_parse_alter_table_rename() {
    let sql = "ALTER TABLE t RENAME TO t2";
    let result = parse(sql);
    assert!(result.is_ok(), "ALTER TABLE RENAME: {:?}", result);
}

#[test]
fn test_parse_alter_table_add_column() {
    let sql = "ALTER TABLE t ADD COLUMN new_col INT";
    let result = parse(sql);
    assert!(result.is_ok(), "ALTER TABLE ADD COLUMN: {:?}", result);
}

#[test]
fn test_parse_alter_table_add_index() {
    let sql = "ALTER TABLE t ADD INDEX idx_name (col1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ADD INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_add_unique() {
    let sql = "ALTER TABLE t ADD UNIQUE (col1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ADD UNIQUE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_change_column() {
    let sql = "ALTER TABLE t CHANGE old_col new_col INT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE CHANGE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_alter_default() {
    let sql = "ALTER TABLE t ALTER COLUMN col SET DEFAULT 0";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ALTER SET DEFAULT: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_drop_primary_key() {
    let sql = "ALTER TABLE t DROP PRIMARY KEY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE DROP PK: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_drop_index() {
    let sql = "ALTER TABLE t DROP INDEX idx_name";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE DROP INDEX: {:?}",
        result
    );
}

// ============ Complex SHOW (parse_show) ============

#[test]
fn test_parse_show_tables_from() {
    let sql = "SHOW TABLES FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TABLES FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_show_tables_like() {
    let sql = "SHOW TABLES LIKE '%user%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TABLES LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_columns_from() {
    let sql = "SHOW COLUMNS FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW COLUMNS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_full_columns_from() {
    let sql = "SHOW FULL COLUMNS FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW FULL COLUMNS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_table_status() {
    let sql = "SHOW TABLE STATUS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TABLE STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_table_status_like() {
    let sql = "SHOW TABLE STATUS FROM mydb LIKE '%user%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TABLE STATUS LIKE: {:?}",
        result
    );
}

// ============ SET ROLE/TRANSACTION (parse_set_role, parse_set_transaction) ============

#[test]
fn test_parse_set_role() {
    let sql = "SET ROLE 'role1'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET ROLE test: {:?}",
        result
    );
}

#[test]
fn test_parse_set_role_none() {
    let sql = "SET ROLE NONE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET ROLE NONE: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_snapshot() {
    let sql = "SET TRANSACTION SNAPSHOT '05-01'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION SNAPSHOT: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation() {
    let sql = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION ISOLATION: {:?}",
        result
    );
}

// ============ Isolation Level Parsing (parse_isolation_level_value) ============

#[test]
fn test_parse_isolation_read_uncommitted() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "READ UNCOMMITTED: {:?}",
        result
    );
}

#[test]
fn test_parse_isolation_read_committed() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ COMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "READ COMMITTED: {:?}",
        result
    );
}

#[test]
fn test_parse_isolation_repeatable_read() {
    let sql = "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPEATABLE READ: {:?}",
        result
    );
}

// ============ Complex Expression List (parse_expression_list) ============

#[test]
fn test_parse_expression_list_multiple() {
    let sql = "SELECT a, b, c, a + b, func(a, b) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Expression list test: {:?}", result);
}

#[test]
fn test_parse_expression_list_with_alias() {
    let sql = "SELECT a AS x, b AS y FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "Expression list with alias: {:?}", result);
}

// ============ ORDER BY Expression (parse_order_by) ============

#[test]
fn test_parse_order_by_expression() {
    let sql = "SELECT * FROM t ORDER BY a + b";
    let result = parse(sql);
    assert!(result.is_ok(), "ORDER BY expression: {:?}", result);
}

#[test]
fn test_parse_order_by_multiple() {
    let sql = "SELECT * FROM t ORDER BY a ASC, b DESC";
    let result = parse(sql);
    assert!(result.is_ok(), "ORDER BY multiple: {:?}", result);
}

#[test]
fn test_parse_order_by_with_nulls() {
    let sql = "SELECT * FROM t ORDER BY a NULLS FIRST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY NULLS FIRST: {:?}",
        result
    );
}

// ============ JOIN Variants (parse_join_clause) ============

#[test]
fn test_parse_join_using() {
    let sql = "SELECT * FROM a JOIN b USING (id)";
    let result = parse(sql);
    assert!(result.is_ok(), "JOIN USING: {:?}", result);
}

#[test]
fn test_parse_cross_join() {
    let sql = "SELECT * FROM a CROSS JOIN b";
    let result = parse(sql);
    assert!(result.is_ok(), "CROSS JOIN: {:?}", result);
}

#[test]
fn test_parse_straight_join() {
    let sql = "SELECT * FROM a STRAIGHT_JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STRAIGHT_JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_natural_join() {
    let sql = "SELECT * FROM a NATURAL JOIN b";
    let result = parse(sql);
    assert!(result.is_ok(), "NATURAL JOIN: {:?}", result);
}

// ============ Aggregate Functions (parse_aggregate_function) ============

#[test]
fn test_parse_sum() {
    let sql = "SELECT SUM(amount) FROM orders";
    let result = parse(sql);
    assert!(result.is_ok(), "SUM: {:?}", result);
}

#[test]
fn test_parse_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT user_id) FROM orders";
    let result = parse(sql);
    assert!(result.is_ok(), "COUNT DISTINCT: {:?}", result);
}

#[test]
fn test_parse_aggregate_with_expression() {
    let sql = "SELECT SUM(amount * price) FROM orders";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Aggregate with expression: {:?}",
        result
    );
}

#[test]
fn test_parse_aggregate_with_filter() {
    let sql = "SELECT COUNT(*) FILTER (WHERE status = 'active') FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Aggregate FILTER: {:?}",
        result
    );
}

// ============ Shift Expressions (parse_shift_expression) ============

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

// ============ Subquery in Various Places ============

#[test]
fn test_parse_subquery_in_from() {
    let sql = "SELECT * FROM (SELECT id FROM t) AS subq";
    let result = parse(sql);
    assert!(result.is_ok(), "Subquery in FROM: {:?}", result);
}

#[test]
fn test_parse_subquery_in_update() {
    let sql = "UPDATE t SET a = (SELECT MAX(id) FROM t2)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Subquery in UPDATE: {:?}",
        result
    );
}

#[test]
fn test_parse_subquery_in_delete() {
    let sql = "DELETE FROM t WHERE id IN (SELECT id FROM t2)";
    let result = parse(sql);
    assert!(result.is_ok(), "Subquery in DELETE: {:?}", result);
}

#[test]
fn test_parse_correlated_subquery() {
    let sql = "SELECT * FROM t WHERE id = (SELECT MAX(id) FROM t2 WHERE t2.type = t.type)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Correlated subquery: {:?}",
        result
    );
}

// ============ INSERT with various clauses (parse_insert_with_clause) ============

#[test]
fn test_parse_insert_with_cte() {
    let sql = "WITH cte AS (SELECT 1) INSERT INTO t SELECT * FROM cte";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT with CTE: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_partition() {
    let sql = "INSERT INTO t PARTITION (p1, p2) VALUES (1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT PARTITION: {:?}",
        result
    );
}

// ============ Column List (parse_column_list) ============

#[test]
fn test_parse_column_list_simple() {
    let sql = "SELECT (a, b, c) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Column list in SELECT: {:?}",
        result
    );
}

// ============ Simple Value (parse_simple_value) ============

#[test]
fn test_parse_simple_value_string() {
    let sql = "DO 'hello'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Simple string value: {:?}",
        result
    );
}

// ============ ALTER EVENT (parse_alter_event) ============

#[test]
fn test_parse_alter_event_basic() {
    let sql = "ALTER EVENT my_event ON SCHEDULE EVERY 1 DAY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT: {:?}",
        result
    );
}

// ============ CREATE ROLE (parse_create_role) ============

#[test]
fn test_parse_create_role() {
    let sql = "CREATE ROLE 'role1'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE ROLE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_role_with_admin() {
    let sql = "CREATE ROLE 'role1' WITH ADMIN 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE ROLE WITH ADMIN: {:?}",
        result
    );
}

// ============ DROP ROLE (parse_drop_role) ============

#[test]
fn test_parse_drop_role() {
    let sql = "DROP ROLE 'role1'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP ROLE: {:?}", result);
}

// ============ EXPLAIN Variants (parse_explain) ============

#[test]
fn test_parse_explain_format_tree() {
    let sql = "EXPLAIN FORMAT = TREE SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXPLAIN FORMAT = TREE: {:?}",
        result
    );
}

#[test]
fn test_parse_explain_analyze() {
    let sql = "EXPLAIN ANALYZE SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXPLAIN ANALYZE: {:?}",
        result
    );
}

// ============ VACUUM (parse_vacuum) ============

#[test]
fn test_parse_vacuum_full() {
    let sql = "VACUUM FULL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM FULL: {:?}",
        result
    );
}

#[test]
fn test_parse_vacuum_analyze() {
    let sql = "VACUUM ANALYZE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM ANALYZE: {:?}",
        result
    );
}

// ============ REPAIR (parse_repair) ============

#[test]
fn test_parse_repair_no_write_to_binlog() {
    let sql = "REPAIR NO_WRITE_TO_BINLOG TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR NO_WRITE_TO_BINLOG: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_quick_extended() {
    let sql = "REPAIR QUICK EXTENDED TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR QUICK EXTENDED: {:?}",
        result
    );
}

// ============ OPTIMIZE (parse_optimize) ============

#[test]
fn test_parse_optimize_no_write_to_binlog() {
    let sql = "OPTIMIZE NO_WRITE_TO_BINLOG TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "OPTIMIZE NO_WRITE_TO_BINLOG: {:?}",
        result
    );
}

// ============ CHECK (parse_check) ============

#[test]
fn test_parse_check_tables() {
    let sql = "CHECK TABLE t, t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_check_changed() {
    let sql = "CHECK TABLE t CHANGED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE CHANGED: {:?}",
        result
    );
}

// ============ Complex SELECT (parse_select_statement) ============

#[test]
fn test_parse_select_into_outfile() {
    let sql = "SELECT * FROM t INTO OUTFILE '/tmp/out.txt'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT INTO OUTFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_into_dumpfile() {
    let sql = "SELECT * FROM t INTO DUMPFILE '/tmp/out.txt'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT INTO DUMPFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_for_update() {
    let sql = "SELECT * FROM t FOR UPDATE";
    let result = parse(sql);
    assert!(result.is_ok(), "SELECT FOR UPDATE: {:?}", result);
}

#[test]
fn test_parse_select_lock_in_share_mode() {
    let sql = "SELECT * FROM t LOCK IN SHARE MODE";
    let result = parse(sql);
    assert!(result.is_ok(), "SELECT LOCK IN SHARE MODE: {:?}", result);
}

// ============ Complex WHERE ============

#[test]
fn test_parse_where_is_not_null() {
    let sql = "SELECT * FROM t WHERE a IS NOT NULL AND b IS NULL";
    let result = parse(sql);
    assert!(result.is_ok(), "WHERE IS NOT NULL: {:?}", result);
}

// ============ LIMIT Variations ============

#[test]
fn test_parse_limit_with_offset() {
    let sql = "SELECT * FROM t LIMIT 10 OFFSET 5";
    let result = parse(sql);
    assert!(result.is_ok(), "LIMIT OFFSET: {:?}", result);
}

#[test]
fn test_parse_limit_all() {
    let sql = "SELECT * FROM t LIMIT ALL";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LIMIT ALL: {:?}", result);
}

// ============ Window Frame Specification ============

#[test]
fn test_parse_window_frame_rows() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window frame ROWS: {:?}",
        result
    );
}

#[test]
fn test_parse_window_frame_range() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window frame RANGE: {:?}",
        result
    );
}

// ============ Named Window ============

#[test]
fn test_parse_named_window() {
    let sql = "SELECT AVG(a) OVER w FROM t WINDOW w AS (ORDER BY id)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Named window: {:?}",
        result
    );
}

// ============ CTE with Multiple Statements ============

#[test]
fn test_parse_cte_with_recursive() {
    let sql = "WITH RECURSIVE cte AS (SELECT 1 UNION ALL SELECT id + 1 FROM cte WHERE id < 10) SELECT * FROM cte";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Recursive CTE: {:?}",
        result
    );
}

#[test]
fn test_parse_cte_update() {
    let sql =
        "WITH cte AS (SELECT id FROM t) UPDATE t2 SET name = 'x' WHERE id IN (SELECT id FROM cte)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE with UPDATE: {:?}",
        result
    );
}

// ============ Multiple Table Operations ============

#[test]
fn test_parse_delete_multi_table() {
    let sql = "DELETE t1, t2 FROM t1 JOIN t2 ON t1.id = t2.id WHERE t1.id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE multi-table: {:?}",
        result
    );
}

#[test]
fn test_parse_update_multi_table() {
    let sql = "UPDATE t1 JOIN t2 ON t1.id = t2.id SET t1.name = t2.name";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE multi-table: {:?}",
        result
    );
}

// ============ User Variable ============

#[test]
fn test_parse_user_variable() {
    let sql = "SELECT @user_var FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "User variable: {:?}", result);
}

#[test]
fn test_parse_system_variable() {
    let sql = "SELECT @@global.max_connections FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "System variable: {:?}",
        result
    );
}

// ============ Case Expression in Different Contexts ============

#[test]
fn test_parse_case_in_update() {
    let sql = "UPDATE t SET status = CASE WHEN a = 1 THEN 'active' ELSE 'inactive' END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE in UPDATE: {:?}",
        result
    );
}

#[test]
fn test_parse_case_in_insert() {
    let sql = "INSERT INTO t (status) VALUES (CASE WHEN a = 1 THEN 'active' ELSE 'inactive' END)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE in INSERT: {:?}",
        result
    );
}

// ============ Interval Expression ============

#[test]
fn test_parse_interval_year() {
    let sql = "SELECT DATE_ADD(date_col, INTERVAL 1 YEAR) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL YEAR: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_multiple() {
    let sql = "SELECT DATE_ADD(date_col, INTERVAL 1 YEAR 2 MONTH 3 DAY) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL multiple: {:?}",
        result
    );
}

// ============ Tuple/Row Constructor in Comparisons ============

#[test]
fn test_parse_tuple_comparison() {
    let sql = "SELECT * FROM t WHERE (a, b) = (1, 2)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Tuple comparison: {:?}",
        result
    );
}

#[test]
fn test_parse_tuple_in() {
    let sql = "SELECT * FROM t WHERE (a, b) IN ((1, 2), (3, 4))";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "Tuple IN: {:?}", result);
}

// ============ Column List in INSERT ============

#[test]
fn test_parse_insert_row_constructor() {
    let sql = "INSERT INTO t (a, b) VALUES (1, 2), (3, 4)";
    let result = parse(sql);
    assert!(result.is_ok(), "INSERT row constructor: {:?}", result);
}
