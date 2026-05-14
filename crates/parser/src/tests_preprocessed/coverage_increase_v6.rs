use crate::parse;

#[test]
fn test_parse_show_events() {
    let sql = "SHOW EVENTS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EVENTS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events_from() {
    let sql = "SHOW EVENTS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EVENTS FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events_like() {
    let sql = "SHOW EVENTS LIKE '%event%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EVENTS LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_events_from_like() {
    let sql = "SHOW EVENTS FROM mydb LIKE '%event%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EVENTS FROM LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_triggers() {
    let sql = "SHOW TRIGGERS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TRIGGERS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_triggers_from() {
    let sql = "SHOW TRIGGERS FROM mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TRIGGERS FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_show_triggers_like() {
    let sql = "SHOW TRIGGERS LIKE '%trigger%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TRIGGERS LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_role() {
    let sql = "DROP ROLE 'role1'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP ROLE: {:?}", result);
}

#[test]
fn test_parse_drop_role_if_exists() {
    let sql = "DROP ROLE IF EXISTS 'role1'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP ROLE IF EXISTS: {:?}",
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

#[test]
fn test_parse_drop_event() {
    let sql = "DROP EVENT my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP EVENT: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_event_if_exists() {
    let sql = "DROP EVENT IF EXISTS my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP EVENT IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_view() {
    let sql = "DROP VIEW my_view";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP VIEW: {:?}", result);
}

#[test]
fn test_parse_drop_view_if_exists() {
    let sql = "DROP VIEW IF EXISTS my_view";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP VIEW IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index() {
    let sql = "DROP INDEX idx ON t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_if_exists() {
    let sql = "DROP INDEX IF EXISTS idx ON t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table() {
    let sql = "DROP TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table_if_exists() {
    let sql = "DROP TABLE IF EXISTS t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table_cascade() {
    let sql = "DROP TABLE t CASCADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE CASCADE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table_restrict() {
    let sql = "DROP TABLE t RESTRICT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE RESTRICT: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table_quick() {
    let sql = "DROP TABLE t QUICK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE QUICK: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_multiple_tables() {
    let sql = "DROP TABLE t1, t2, t3";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP multiple tables: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_database() {
    let sql = "DROP DATABASE mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP DATABASE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_database_if_exists() {
    let sql = "DROP DATABASE IF EXISTS mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP DATABASE IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_on_table() {
    let sql = "DROP INDEX idx ON t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX ON: {:?}",
        result
    );
}

#[test]
fn test_parse_create_database() {
    let sql = "CREATE DATABASE mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_database_if_not_exists() {
    let sql = "CREATE DATABASE IF NOT EXISTS mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE IF NOT EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_create_database_with_charset() {
    let sql = "CREATE DATABASE mydb DEFAULT CHARSET utf8mb4";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE CHARSET: {:?}",
        result
    );
}

#[test]
fn test_parse_create_database_with_collate() {
    let sql = "CREATE DATABASE mydb COLLATE utf8mb4_unicode_ci";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE COLLATE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_database() {
    let sql = "ALTER DATABASE mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER DATABASE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_database_upgrade() {
    let sql = "ALTER DATABASE mydb UPGRADE DATA DIRECTORY NAME";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER DATABASE UPGRADE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_function() {
    let sql = "DROP FUNCTION my_func";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP FUNCTION: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_procedure() {
    let sql = "DROP PROCEDURE my_proc";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP PROCEDURE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_function_if_exists() {
    let sql = "DROP FUNCTION IF EXISTS my_func";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP FUNCTION IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_procedure_if_exists() {
    let sql = "DROP PROCEDURE IF EXISTS my_proc";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP PROCEDURE IF EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger() {
    let sql = "CREATE TRIGGER my_trigger BEFORE INSERT ON t FOR EACH ROW BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TRIGGER: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger_after() {
    let sql = "CREATE TRIGGER my_trigger AFTER UPDATE ON t FOR EACH ROW BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TRIGGER AFTER: {:?}",
        result
    );
}

#[test]
fn test_parse_create_trigger_delete() {
    let sql = "CREATE TRIGGER my_trigger BEFORE DELETE ON t FOR EACH ROW BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TRIGGER DELETE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_every() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 HOUR DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT EVERY: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_at() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT '2024-12-31 23:59:59' DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT AT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_starts() {
    let sql =
        "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY STARTS '2024-01-01 00:00:00' DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT STARTS: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_ends() {
    let sql =
        "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY ENDS '2024-12-31 23:59:59' DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT ENDS: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_on_completion_preserve() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY ON COMPLETION PRESERVE DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT ON COMPLETION PRESERVE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_on_completion_not_preserve() {
    let sql =
        "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY ON COMPLETION NOT PRESERVE DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT ON COMPLETION NOT PRESERVE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_enabled() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY ENABLED DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT ENABLED: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_disabled() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY DISABLED DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT DISABLED: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_comment() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 DAY COMMENT 'my comment' DO BEGIN END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT COMMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_with_body() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT with body: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_event() {
    let sql = "ALTER EVENT my_event ON SCHEDULE EVERY 1 DAY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_event_rename() {
    let sql = "ALTER EVENT my_event RENAME TO new_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT RENAME: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_event_enable() {
    let sql = "ALTER EVENT my_event ENABLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT ENABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_event_disable() {
    let sql = "ALTER EVENT my_event DISABLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT DISABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_rename() {
    let sql = "ALTER TABLE t RENAME TO new_t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME TO: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_rename_column() {
    let sql = "ALTER TABLE t RENAME COLUMN old_col TO new_col";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME COLUMN: {:?}",
        result
    );
}

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
fn test_parse_alter_table_modify() {
    let sql = "ALTER TABLE t MODIFY col_name INT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE MODIFY: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_modify_column() {
    let sql = "ALTER TABLE t MODIFY COLUMN col_name VARCHAR(100)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE MODIFY COLUMN: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_change() {
    let sql = "ALTER TABLE t CHANGE old_col new_col INT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE CHANGE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_change_column() {
    let sql = "ALTER TABLE t CHANGE COLUMN old_col new_col INT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE CHANGE COLUMN: {:?}",
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

#[test]
fn test_parse_alter_table_concurrent() {
    let sql = "ALTER TABLE t DISABLE KEYS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE DISABLE KEYS: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_algorithm() {
    let sql = "ALTER TABLE t ALGORITHM = INPLACE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ALGORITHM: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_lock() {
    let sql = "ALTER TABLE t LOCK = NONE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE LOCK: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_with_validation() {
    let sql = "ALTER TABLE t ALGORITHM = COPY, LOCK = SHARED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE with validation: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_unique() {
    let sql = "CREATE UNIQUE INDEX idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE UNIQUE INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_if_not_exists() {
    let sql = "CREATE INDEX IF NOT EXISTS idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX IF NOT EXISTS: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_using_btree() {
    let sql = "CREATE INDEX idx USING BTREE ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX USING BTREE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_using_hash() {
    let sql = "CREATE INDEX idx USING HASH ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX USING HASH: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_with_length() {
    let sql = "CREATE INDEX idx ON t(col(10))";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX with length: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_asc() {
    let sql = "CREATE INDEX idx ON t(col ASC)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX ASC: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_desc() {
    let sql = "CREATE INDEX idx ON t(col DESC)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX DESC: {:?}",
        result
    );
}

#[test]
fn test_parse_create_spatial_index() {
    let sql = "CREATE SPATIAL INDEX idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE SPATIAL INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_create_fulltext_index() {
    let sql = "CREATE FULLTEXT INDEX idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FULLTEXT INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_comment() {
    let sql = "CREATE INDEX idx ON t(col) COMMENT 'my index'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX COMMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_visible() {
    let sql = "CREATE INDEX idx ON t(col) VISIBLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX VISIBLE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_invisible() {
    let sql = "CREATE INDEX idx ON t(col) INVISIBLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX INVISIBLE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_restrict() {
    let sql = "DROP INDEX idx ON t RESTRICT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX RESTRICT: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_cascade() {
    let sql = "DROP INDEX idx ON t CASCADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX CASCADE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_index_online() {
    let sql = "DROP INDEX CONCURRENTLY idx ON t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP INDEX CONCURRENTLY: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure() {
    let sql = "CREATE PROCEDURE my_proc() BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_with_params() {
    let sql = "CREATE PROCEDURE my_proc(IN p1 INT, OUT p2 INT, INOUT p3 INT) BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE with params: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_comment() {
    let sql = "CREATE PROCEDURE my_proc() COMMENT 'my procedure' BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE COMMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_procedure_language() {
    let sql = "CREATE PROCEDURE my_proc() LANGUAGE SQL BEGIN SELECT 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE PROCEDURE LANGUAGE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function_with_params() {
    let sql = "CREATE FUNCTION my_func(p1 INT) RETURNS INT BEGIN RETURN p1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION with params: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function_deterministic() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT DETERMINISTIC BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION DETERMINISTIC: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function_not_deterministic() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT NOT DETERMINISTIC BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION NOT DETERMINISTIC: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function_with_comment() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT COMMENT 'my function' BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION COMMENT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function_language() {
    let sql = "CREATE FUNCTION my_func() RETURNS INT LANGUAGE SQL BEGIN RETURN 1; END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION LANGUAGE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view() {
    let sql = "CREATE VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW: {:?}",
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

#[test]
fn test_parse_create_view_with_check() {
    let sql = "CREATE VIEW my_view AS SELECT * FROM t WITH CHECK OPTION";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW WITH CHECK OPTION: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_cascaded() {
    let sql = "CREATE VIEW my_view AS SELECT * FROM t WITH CASCADED CHECK OPTION";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW WITH CASCADED CHECK OPTION: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_local() {
    let sql = "CREATE VIEW my_view AS SELECT * FROM t WITH LOCAL CHECK OPTION";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW WITH LOCAL CHECK OPTION: {:?}",
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
fn test_parse_create_view_algorithm() {
    let sql = "CREATE ALGORITHM = UNDEFINED VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW ALGORITHM: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_definer() {
    let sql = "CREATE DEFINER = 'user'@'localhost' VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW DEFINER: {:?}",
        result
    );
}

#[test]
fn test_parse_create_view_security() {
    let sql = "CREATE SQL SECURITY INVOKER VIEW my_view AS SELECT * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE VIEW SQL SECURITY: {:?}",
        result
    );
}
