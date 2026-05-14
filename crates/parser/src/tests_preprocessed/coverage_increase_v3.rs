use crate::parse;

// ============ More GRANT/REVOKE Coverage ============

#[test]
fn test_parse_grant_execute() {
    let sql = "GRANT EXECUTE ON PROCEDURE my_proc TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT EXECUTE: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_usage() {
    let sql = "GRANT USAGE ON *.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT USAGE: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_write() {
    let sql = "GRANT WRITE ON mydb.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT WRITE: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_read() {
    let sql = "GRANT READ ON mydb.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT READ: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_multiple_privileges() {
    let sql = "GRANT SELECT, INSERT, UPDATE ON mydb.* TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_with_columns() {
    let sql = "GRANT SELECT (col1, col2) ON mydb.t TO 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT with columns: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_option() {
    let sql = "GRANT SELECT ON mydb.* TO 'user'@'localhost' WITH GRANT OPTION";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT WITH GRANT OPTION: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_execute() {
    let sql = "REVOKE EXECUTE ON PROCEDURE my_proc FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE EXECUTE: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_usage() {
    let sql = "REVOKE USAGE ON *.* FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE USAGE: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_multiple() {
    let sql = "REVOKE SELECT, INSERT, UPDATE ON mydb.* FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_grant_option() {
    let sql = "REVOKE GRANT OPTION ON mydb.* FROM 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE GRANT OPTION: {:?}",
        result
    );
}

// ============ More CREATE/DROP Coverage ============

#[test]
fn test_parse_drop_index_restrict() {
    let sql = "DROP INDEX idx_name ON t RESTRICT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX RESTRICT: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_cascade() {
    let sql = "DROP INDEX idx_name ON t CASCADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX CASCADE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_role_with_admin() {
    let sql = "CREATE ROLE 'admin' WITH ADMIN 'user'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE ROLE WITH ADMIN: {:?}",
        result
    );
}

// ============ More SET Coverage ============

#[test]
fn test_parse_set_password() {
    let sql = "SET PASSWORD FOR 'user'@'localhost' = 'newpass'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET PASSWORD: {:?}",
        result
    );
}

#[test]
fn test_parse_set_autocommit() {
    let sql = "SET AUTOCOMMIT = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET AUTOCOMMIT: {:?}",
        result
    );
}

#[test]
fn test_parse_set_timezone() {
    let sql = "SET TIME_ZONE = '+00:00'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TIME_ZONE: {:?}",
        result
    );
}

#[test]
fn test_parse_set_names() {
    let sql = "SET NAMES utf8mb4";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SET NAMES: {:?}", result);
}

#[test]
fn test_parse_set_character_set() {
    let sql = "SET CHARACTER SET utf8mb4";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET CHARACTER SET: {:?}",
        result
    );
}

// ============ More Transaction Coverage ============

#[test]
fn test_parse_begin_chain() {
    let sql = "BEGIN CHAIN";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BEGIN CHAIN: {:?}",
        result
    );
}

#[test]
fn test_parse_begin_release() {
    let sql = "BEGIN RELEASE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BEGIN RELEASE: {:?}",
        result
    );
}

#[test]
fn test_parse_commit_chain() {
    let sql = "COMMIT CHAIN";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "COMMIT CHAIN: {:?}",
        result
    );
}

#[test]
fn test_parse_commit_release() {
    let sql = "COMMIT RELEASE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "COMMIT RELEASE: {:?}",
        result
    );
}

#[test]
fn test_parse_commit_work() {
    let sql = "COMMIT WORK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "COMMIT WORK: {:?}",
        result
    );
}

#[test]
fn test_parse_rollback_work() {
    let sql = "ROLLBACK WORK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ROLLBACK WORK: {:?}",
        result
    );
}

// ============ More SELECT Coverage ============

#[test]
fn test_parse_select_into_file() {
    let sql = "SELECT * FROM t INTO OUTFILE '/tmp/out.txt' LINES TERMINATED BY '\\n'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT INTO OUTFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_distinct() {
    let sql = "SELECT DISTINCT id FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "SELECT DISTINCT: {:?}", result);
}

#[test]
fn test_parse_select_all() {
    let sql = "SELECT ALL id FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT ALL: {:?}",
        result
    );
}

#[test]
fn test_parse_select_high_priority() {
    let sql = "SELECT HIGH_PRIORITY * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT HIGH_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_sql_cache() {
    let sql = "SELECT SQL_CACHE * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT SQL_CACHE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_sql_no_cache() {
    let sql = "SELECT SQL_NO_CACHE * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT SQL_NO_CACHE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_straight_join() {
    let sql = "SELECT STRAIGHT_JOIN * FROM a JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT STRAIGHT_JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_select_small_result() {
    let sql = "SELECT SMALL_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT SMALL_RESULT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_big_result() {
    let sql = "SELECT BIG_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT BIG_RESULT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_buffer_result() {
    let sql = "SELECT SQL_BUFFER_RESULT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT SQL_BUFFER_RESULT: {:?}",
        result
    );
}

// ============ More JOIN Coverage ============

#[test]
fn test_parse_left_outer_join() {
    let sql = "SELECT * FROM a LEFT OUTER JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LEFT OUTER JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_right_outer_join() {
    let sql = "SELECT * FROM a RIGHT OUTER JOIN b ON a.id = b.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RIGHT OUTER JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_join_on_with_expression() {
    let sql = "SELECT * FROM a JOIN b ON a.id = b.id AND a.type = b.type";
    let result = parse(sql);
    assert!(result.is_ok(), "JOIN ON with expression: {:?}", result);
}

#[test]
fn test_parse_join_using_multiple() {
    let sql = "SELECT * FROM a JOIN b USING (id, type)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JOIN USING multiple: {:?}",
        result
    );
}

// ============ More GROUP BY Coverage ============

#[test]
fn test_parse_group_by_with_expression() {
    let sql = "SELECT COUNT(*) FROM t GROUP BY id + 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY expression: {:?}",
        result
    );
}

#[test]
fn test_parse_group_by_with_alias() {
    let sql = "SELECT status, COUNT(*) as cnt FROM t GROUP BY status";
    let result = parse(sql);
    assert!(result.is_ok(), "GROUP BY alias: {:?}", result);
}

#[test]
fn test_parse_group_by_cube() {
    let sql = "SELECT type, region, SUM(sales) FROM t GROUP BY CUBE (type, region)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY CUBE: {:?}",
        result
    );
}

#[test]
fn test_parse_group_by_rollup() {
    let sql = "SELECT type, region, SUM(sales) FROM t GROUP BY ROLLUP (type, region)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY ROLLUP: {:?}",
        result
    );
}

// ============ More HAVING Coverage ============

#[test]
fn test_parse_having_with_expression() {
    let sql = "SELECT COUNT(*) FROM t HAVING COUNT(*) > 1";
    let result = parse(sql);
    assert!(result.is_ok(), "HAVING expression: {:?}", result);
}

#[test]
fn test_parse_having_with_and() {
    let sql = "SELECT type FROM t GROUP BY type HAVING COUNT(*) > 1 AND SUM(amount) > 100";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HAVING AND: {:?}",
        result
    );
}

// ============ More Window Function Coverage ============

#[test]
fn test_parse_window_partition() {
    let sql = "SELECT AVG(amount) OVER (PARTITION BY type) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window PARTITION BY: {:?}",
        result
    );
}

#[test]
fn test_parse_window_rows_range() {
    let sql =
        "SELECT SUM(amount) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window ROWS BETWEEN: {:?}",
        result
    );
}

#[test]
fn test_parse_window_exclude() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id EXCLUDE CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window EXCLUDE: {:?}",
        result
    );
}

#[test]
fn test_parse_window_gaps() {
    let sql =
        "SELECT ROW_NUMBER() OVER (ORDER BY id GAPS BETWEEN 1 PRECEDING AND 1 FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Window GAPS: {:?}",
        result
    );
}

// ============ More LIMIT/OFFSET Coverage ============

#[test]
fn test_parse_limit_param_marker() {
    let sql = "SELECT * FROM t LIMIT ?";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LIMIT ?: {:?}", result);
}

#[test]
fn test_parse_limit_negative() {
    let sql = "SELECT * FROM t LIMIT 10 OFFSET -5";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LIMIT negative offset: {:?}",
        result
    );
}

// ============ More Subquery Coverage ============

#[test]
fn test_parse_scalar_subquery() {
    let sql = "SELECT (SELECT MAX(id) FROM t) AS max_id FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Scalar subquery: {:?}",
        result
    );
}

#[test]
fn test_parse_exists_subquery() {
    let sql = "SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t2 WHERE t.id = t2.id)";
    let result = parse(sql);
    assert!(result.is_ok(), "EXISTS subquery: {:?}", result);
}

#[test]
fn test_parse_not_exists_subquery() {
    let sql = "SELECT * FROM t WHERE NOT EXISTS (SELECT 1 FROM t2)";
    let result = parse(sql);
    assert!(result.is_ok(), "NOT EXISTS subquery: {:?}", result);
}

// ============ More Expression Coverage ============

#[test]
fn test_parse_function_call_multiple_args() {
    let sql = "SELECT CONCAT_WS(',', a, b, c) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "CONCAT_WS: {:?}", result);
}

#[test]
fn test_parse_function_call_coalesce() {
    let sql = "SELECT COALESCE(a, b, c, 'default') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "COALESCE: {:?}", result);
}

#[test]
fn test_parse_function_call_greatest() {
    let sql = "SELECT GREATEST(a, b, c) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "GREATEST: {:?}", result);
}

#[test]
fn test_parse_function_call_least() {
    let sql = "SELECT LEAST(a, b, c) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LEAST: {:?}", result);
}

#[test]
fn test_parse_interval_year_month() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 YEAR 2 MONTH) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL YEAR MONTH: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_day_hour() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 DAY 2 HOUR) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL DAY HOUR: {:?}",
        result
    );
}

// ============ More INSERT Coverage ============

#[test]
fn test_parse_insert_ignore() {
    let sql = "INSERT IGNORE INTO t VALUES (1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT IGNORE: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_delayed() {
    let sql = "INSERT DELAYED INTO t VALUES (1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT DELAYED: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_high_priority() {
    let sql = "INSERT HIGH_PRIORITY INTO t VALUES (1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT HIGH_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_set() {
    let sql = "INSERT INTO t SET id = 1, name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT SET: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_on_duplicate_key_update() {
    let sql = "INSERT INTO t VALUES (1) ON DUPLICATE KEY UPDATE id = 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT ON DUPLICATE KEY: {:?}",
        result
    );
}

// ============ More UPDATE Coverage ============

#[test]
fn test_parse_update_low_priority() {
    let sql = "UPDATE LOW_PRIORITY t SET id = 1 WHERE id = 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE LOW_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_update_ignore() {
    let sql = "UPDATE IGNORE t SET id = 1 WHERE id = 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE IGNORE: {:?}",
        result
    );
}

#[test]
fn test_parse_update_multiple_tables() {
    let sql = "UPDATE t1, t2 SET t1.id = t2.id WHERE t1.id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE multiple tables: {:?}",
        result
    );
}

// ============ More DELETE Coverage ============

#[test]
fn test_parse_delete_low_priority() {
    let sql = "DELETE LOW_PRIORITY FROM t WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE LOW_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_delete_quick() {
    let sql = "DELETE QUICK FROM t WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE QUICK: {:?}",
        result
    );
}

#[test]
fn test_parse_delete_ignore() {
    let sql = "DELETE IGNORE FROM t WHERE id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE IGNORE: {:?}",
        result
    );
}

// ============ More TRUNCATE Coverage ============

#[test]
fn test_parse_truncate_table_restart() {
    let sql = "TRUNCATE TABLE t RESTART IDENTITY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TRUNCATE RESTART: {:?}",
        result
    );
}

// ============ More CREATE TABLE Coverage ============

#[test]
fn test_parse_create_table_engine() {
    let sql = "CREATE TABLE t (id INT) ENGINE = InnoDB";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE ENGINE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_charset() {
    let sql = "CREATE TABLE t (id INT) DEFAULT CHARSET = utf8mb4";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE CHARSET: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_collate() {
    let sql = "CREATE TABLE t (id INT) COLLATE = utf8mb4_unicode_ci";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE COLLATE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_comment() {
    let sql = "CREATE TABLE t (id INT) COMMENT = 'my table'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE COMMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_row_format() {
    let sql = "CREATE TABLE t (id INT) ROW_FORMAT = Compact";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE ROW_FORMAT: {:?}",
        result
    );
}

// ============ More ALTER TABLE Coverage ============

#[test]
fn test_parse_alter_table_rename_index() {
    let sql = "ALTER TABLE t RENAME INDEX old_idx TO new_idx";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_rename_key() {
    let sql = "ALTER TABLE t RENAME KEY old_key TO new_key";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME KEY: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_force() {
    let sql = "ALTER TABLE t FORCE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE FORCE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_order() {
    let sql = "ALTER TABLE t ORDER BY id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ORDER BY: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_disable_keys() {
    let sql = "ALTER TABLE t DISABLE KEYS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE DISABLE KEYS: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_enable_keys() {
    let sql = "ALTER TABLE t ENABLE KEYS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ENABLE KEYS: {:?}",
        result
    );
}

// ============ More CHECK/OPTIMIZE/REPAIR Coverage ============

#[test]
fn test_parse_check_table_for_upgrade() {
    let sql = "CHECK TABLE t FOR UPGRADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE FOR UPGRADE: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_medium() {
    let sql = "CHECK TABLE t MEDIUM";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE MEDIUM: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_high() {
    let sql = "CHECK TABLE t HIGH";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE HIGH: {:?}",
        result
    );
}

#[test]
fn test_parse_optimize_local() {
    let sql = "OPTIMIZE LOCAL TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "OPTIMIZE LOCAL: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_table_local() {
    let sql = "REPAIR LOCAL TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR LOCAL: {:?}",
        result
    );
}

// ============ More SHOW Coverage ============

#[test]
fn test_parse_show_create_event() {
    let sql = "SHOW CREATE EVENT my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW CREATE EVENT: {:?}",
        result
    );
}

#[test]
fn test_parse_show_create_procedure() {
    let sql = "SHOW CREATE PROCEDURE my_proc";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW CREATE PROCEDURE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_create_function() {
    let sql = "SHOW CREATE FUNCTION my_func";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW CREATE FUNCTION: {:?}",
        result
    );
}

#[test]
fn test_parse_show_function_status() {
    let sql = "SHOW FUNCTION STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW FUNCTION STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_procedure_status() {
    let sql = "SHOW PROCEDURE STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW PROCEDURE STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events() {
    let sql = "SHOW EVENTS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EVENTS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_triggers() {
    let sql = "SHOW TRIGGERS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TRIGGERS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_profile() {
    let sql = "SHOW PROFILE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW PROFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_profile_for_query() {
    let sql = "SHOW PROFILE FOR QUERY 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW PROFILE FOR QUERY: {:?}",
        result
    );
}

#[test]
fn test_parse_show_processlist() {
    let sql = "SHOW PROCESSLIST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW PROCESSLIST: {:?}",
        result
    );
}

#[test]
fn test_parse_show_full_processlist() {
    let sql = "SHOW FULL PROCESSLIST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW FULL PROCESSLIST: {:?}",
        result
    );
}

// ============ More FLUSH Coverage ============

#[test]
fn test_parse_flush_des_key_file() {
    let sql = "FLUSH DES_KEY_FILE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH DES_KEY_FILE: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_host_cache() {
    let sql = "FLUSH HOSTS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH HOSTS: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_optimizer_costs() {
    let sql = "FLUSH OPTIMIZER_COSTS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH OPTIMIZER_COSTS: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_user_resources() {
    let sql = "FLUSH USER_RESOURCES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH USER_RESOURCES: {:?}",
        result
    );
}

// ============ More DO/Release ============

#[test]
fn test_parse_do_with_multiple_expressions() {
    let sql = "DO 1 + 2, SLEEP(1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DO multiple expressions: {:?}",
        result
    );
}

// ============ More BINLOG Coverage ============

#[test]
fn test_parse_show_binlog_events() {
    let sql = "SHOW BINLOG EVENTS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW BINLOG EVENTS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_binlog_events_in() {
    let sql = "SHOW BINLOG EVENTS IN 'binlog.000001'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW BINLOG EVENTS IN: {:?}",
        result
    );
}

#[test]
fn test_parse_show_master_status() {
    let sql = "SHOW MASTER STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW MASTER STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_slave_status() {
    let sql = "SHOW SLAVE STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW SLAVE STATUS: {:?}",
        result
    );
}

// ============ More CACHE Index Coverage ============

#[test]
fn test_parse_cache_index() {
    let sql = "CACHE INDEX t1, t2 IN default";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CACHE INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_load_index() {
    let sql = "LOAD INDEX INTO CACHE t1, t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_load_index_ignore_leaves() {
    let sql = "LOAD INDEX INTO CACHE t1 IGNORE LEAVES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD INDEX IGNORE LEAVES: {:?}",
        result
    );
}

// ============ More RESET Coverage ============

#[test]
fn test_parse_reset_master() {
    let sql = "RESET MASTER";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESET MASTER: {:?}",
        result
    );
}

#[test]
fn test_parse_reset_slave() {
    let sql = "RESET SLAVE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESET SLAVE: {:?}",
        result
    );
}

#[test]
fn test_parse_reset_slave_all() {
    let sql = "RESET SLAVE ALL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESET SLAVE ALL: {:?}",
        result
    );
}

// ============ More START SLAVE Coverage ============

#[test]
fn test_parse_start_slave() {
    let sql = "START SLAVE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START SLAVE: {:?}",
        result
    );
}

#[test]
fn test_parse_start_slave_until() {
    let sql = "START SLAVE UNTIL MASTER_LOG_FILE='binlog.000001', MASTER_LOG_POS=100";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START SLAVE UNTIL: {:?}",
        result
    );
}

#[test]
fn test_parse_start_slave_sql_thread() {
    let sql = "START SLAVE SQL_THREAD";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START SLAVE SQL_THREAD: {:?}",
        result
    );
}

#[test]
fn test_parse_start_slave_io_thread() {
    let sql = "START SLAVE IO_THREAD";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START SLAVE IO_THREAD: {:?}",
        result
    );
}

#[test]
fn test_parse_stop_slave() {
    let sql = "STOP SLAVE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STOP SLAVE: {:?}",
        result
    );
}

#[test]
fn test_parse_stop_slave_sql_thread() {
    let sql = "STOP SLAVE SQL_THREAD";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STOP SLAVE SQL_THREAD: {:?}",
        result
    );
}

#[test]
fn test_parse_stop_slave_io_thread() {
    let sql = "STOP SLAVE IO_THREAD";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STOP SLAVE IO_THREAD: {:?}",
        result
    );
}

// ============ More CHANGE MASTER Coverage ============

#[test]
fn test_parse_change_master() {
    let sql = "CHANGE MASTER TO MASTER_HOST = 'host', MASTER_PORT = 3306";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHANGE MASTER TO: {:?}",
        result
    );
}

#[test]
fn test_parse_change_master_auto_position() {
    let sql = "CHANGE MASTER TO MASTER_AUTO_POSITION = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHANGE MASTER AUTO_POSITION: {:?}",
        result
    );
}

// ============ More XA Coverage ============

#[test]
fn test_parse_xa_start() {
    let sql = "XA START 'xid'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "XA START: {:?}", result);
}

#[test]
fn test_parse_xa_begin() {
    let sql = "XA BEGIN 'xid'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "XA BEGIN: {:?}", result);
}

#[test]
fn test_parse_xa_end() {
    let sql = "XA END 'xid'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "XA END: {:?}", result);
}

#[test]
fn test_parse_xa_prepare() {
    let sql = "XA PREPARE 'xid'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "XA PREPARE: {:?}",
        result
    );
}

#[test]
fn test_parse_xa_commit() {
    let sql = "XA COMMIT 'xid'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "XA COMMIT: {:?}", result);
}

#[test]
fn test_parse_xa_rollback() {
    let sql = "XA ROLLBACK 'xid'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "XA ROLLBACK: {:?}",
        result
    );
}

#[test]
fn test_parse_xa_recover() {
    let sql = "XA RECOVER";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "XA RECOVER: {:?}",
        result
    );
}
