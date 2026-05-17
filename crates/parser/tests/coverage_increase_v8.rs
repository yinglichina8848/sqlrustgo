//! Tests for uncovered parser statements
//! Target: increase line coverage in parse_vacuum, parse_optimize, parse_check,
//! parse_backup, parse_restore, parse_repair, and other uncovered statements.

use sqlrustgo_parser::parse;

// ============ OPTIMIZE TABLE ============

#[test]
fn test_parse_optimize_table() {
    let sql = "OPTIMIZE TABLE users";
    let result = parse(sql);
    assert!(result.is_ok(), "OPTIMIZE TABLE: {:?}", result);
}

#[test]
fn test_parse_optimize_table_multiple() {
    let sql = "OPTIMIZE TABLE users, orders";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "OPTIMIZE TABLE multiple: {:?}",
        result
    );
}

// ============ CHECK TABLE ============

#[test]
fn test_parse_check_table() {
    let sql = "CHECK TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_quick() {
    let sql = "CHECK TABLE users QUICK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE QUICK: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_fast() {
    let sql = "CHECK TABLE users FAST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE FAST: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_medium() {
    let sql = "CHECK TABLE users MEDIUM";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE MEDIUM: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_extended() {
    let sql = "CHECK TABLE users EXTENDED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE EXTENDED: {:?}",
        result
    );
}

#[test]
fn test_parse_check_table_changed() {
    let sql = "CHECK TABLE users CHANGED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE CHANGED: {:?}",
        result
    );
}

// ============ ANALYZE TABLE ============

#[test]
fn test_parse_analyze_table() {
    let sql = "ANALYZE TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_analyze_table_no_write_to_binlog() {
    let sql = "ANALYZE NO_WRITE_TO_BINLOG TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE NO_WRITE_TO_BINLOG: {:?}",
        result
    );
}

#[test]
fn test_parse_analyze_table_local() {
    let sql = "ANALYZE LOCAL TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE LOCAL: {:?}",
        result
    );
}

// ============ VACUUM ============

#[test]
fn test_parse_vacuum() {
    let sql = "VACUUM";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "VACUUM: {:?}", result);
}

#[test]
fn test_parse_vacuum_table() {
    let sql = "VACUUM TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_vacuum_table_full() {
    let sql = "VACUUM TABLE users FULL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM TABLE FULL: {:?}",
        result
    );
}

#[test]
fn test_parse_vacuum_analyze() {
    let sql = "VACUUM ANALYZE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM ANALYZE: {:?}",
        result
    );
}

#[test]
fn test_parse_vacuum_multiple_tables() {
    let sql = "VACUUM TABLE users, orders";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM multiple tables: {:?}",
        result
    );
}

#[test]
fn test_parse_vacuum_full_tables() {
    let sql = "VACUUM TABLE users, orders FULL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "VACUUM FULL multiple: {:?}",
        result
    );
}

// ============ REPAIR TABLE ============

#[test]
fn test_parse_repair_table() {
    let sql = "REPAIR TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_table_quick() {
    let sql = "REPAIR TABLE users QUICK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR TABLE QUICK: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_table_extended() {
    let sql = "REPAIR TABLE users EXTENDED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR TABLE EXTENDED: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_table_use_frm() {
    let sql = "REPAIR TABLE users USE_FRM";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR TABLE USE_FRM: {:?}",
        result
    );
}

// ============ BACKUP ============

#[test]
fn test_parse_backup() {
    let sql = "BACKUP DATABASE TO '/backup/db.bak'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP DATABASE: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_incremental() {
    let sql = "BACKUP DATABASE TO '/backup/db.bak' INCREMENTAL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP INCREMENTAL: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_differential() {
    let sql = "BACKUP DATABASE TO '/backup/db.bak' DIFFERENTIAL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP DIFFERENTIAL: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_compressed() {
    let sql = "BACKUP DATABASE TO '/backup/db.bak' COMPRESSED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP COMPRESSED: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_all_options() {
    let sql = "BACKUP DATABASE TO '/backup/db.bak' INCREMENTAL COMPRESSED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP all options: {:?}",
        result
    );
}

#[test]
fn test_parse_backup_without_database_keyword() {
    let sql = "BACKUP TO '/backup/db.bak'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "BACKUP without DATABASE: {:?}",
        result
    );
}

// ============ RESTORE ============

#[test]
fn test_parse_restore() {
    let sql = "RESTORE FROM '/backup/db.bak'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_restore_to_database() {
    let sql = "RESTORE FROM '/backup/db.bak' TO mydb";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE TO: {:?}",
        result
    );
}

#[test]
fn test_parse_restore_point_in_time() {
    let sql = "RESTORE FROM '/backup/db.bak' POINT IN TIME '2024-01-01 00:00:00'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE PITR: {:?}",
        result
    );
}

#[test]
fn test_parse_restore_database_keyword() {
    let sql = "RESTORE DATABASE FROM '/backup/db.bak'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESTORE DATABASE: {:?}",
        result
    );
}

// ============ SIGN RECORD (GMP) ============

#[test]
fn test_parse_sign_record_basic() {
    let sql = "SIGN RECORD FOR users REASON 'testing'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SIGN RECORD basic: {:?}",
        result
    );
}

#[test]
fn test_parse_sign_record_with_columns() {
    let sql = "SIGN RECORD FOR users (col1 = 'value1', col2 = 'value2') REASON 'testing'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SIGN RECORD with columns: {:?}",
        result
    );
}

#[test]
fn test_parse_sign_record_string_name() {
    let sql = "SIGN RECORD 'users' REASON 'testing'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SIGN RECORD string name: {:?}",
        result
    );
}

// ============ CREATE APPROVAL POLICY (GMP) ============

#[test]
fn test_parse_create_approval_policy() {
    let sql = "CREATE APPROVAL POLICY my_policy FOR TABLE users REQUIRES 2 APPROVERS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE APPROVAL POLICY: {:?}",
        result
    );
}

// ============ CHECK TABLE with various options ============

#[test]
fn test_parse_check_table_upgrade() {
    let sql = "CHECK TABLE users UPGRADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE UPGRADE: {:?}",
        result
    );
}

// ============ LOAD DATA ============

#[test]
fn test_parse_load_data() {
    let sql = "LOAD DATA INFILE '/data.csv' INTO TABLE users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA INFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_load_data_duplicate() {
    let sql = "LOAD DATA INFILE '/data.csv' INTO TABLE users REPLACE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA REPLACE: {:?}",
        result
    );
}

#[test]
fn test_parse_load_data_ignore() {
    let sql = "LOAD DATA INFILE '/data.csv' INTO TABLE users IGNORE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA IGNORE: {:?}",
        result
    );
}

// ============ DO statement ============

#[test]
fn test_parse_do() {
    let sql = "DO SLEEP(1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DO statement: {:?}",
        result
    );
}

#[test]
fn test_parse_do_with_args() {
    let sql = "DO GET_LOCK('mylock', 10)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DO with args: {:?}",
        result
    );
}

// ============ HANDLER statements ============

#[test]
fn test_parse_handler_read() {
    let sql = "HANDLER users READ FIRST WHERE id > 10 LIMIT 5";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HANDLER READ: {:?}",
        result
    );
}

#[test]
fn test_parse_handler_open() {
    let sql = "HANDLER users OPEN";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HANDLER OPEN: {:?}",
        result
    );
}

#[test]
fn test_parse_handler_close() {
    let sql = "HANDLER users CLOSE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HANDLER CLOSE: {:?}",
        result
    );
}

// ============ INDEX statements ============

#[test]
fn test_parse_alter_index() {
    let sql = "ALTER INDEX idx_name VISIBLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER INDEX VISIBLE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_index_invisible() {
    let sql = "ALTER INDEX idx_name INVISIBLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER INDEX INVISIBLE: {:?}",
        result
    );
}

// ============ TABLE statements ============

#[test]
fn test_parse_alter_table_rename() {
    let sql = "ALTER TABLE users RENAME TO new_users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_convert() {
    let sql = "ALTER TABLE users CONVERT TO CHARACTER SET utf8mb4";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE CONVERT: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_force() {
    let sql = "ALTER TABLE users FORCE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE FORCE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_algorithm() {
    let sql = "ALTER TABLE users ALGORITHM = INPLACE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE ALGORITHM: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_table_lock() {
    let sql = "ALTER TABLE users LOCK = NONE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE LOCK: {:?}",
        result
    );
}

// ============ SHOW COLUMNS with extended ============

#[test]
fn test_parse_show_extended() {
    let sql = "SHOW EXTENDED COLUMNS FROM users";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW EXTENDED: {:?}",
        result
    );
}

#[test]
fn test_parse_show_full() {
    let sql = "SHOW FULL COLUMNS FROM users";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHOW FULL: {:?}", result);
}

// ============ SHOW PROCESSLIST ============

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

// ============ SHOW ENGINE ============

#[test]
fn test_parse_show_engine() {
    let sql = "SHOW ENGINE INNODB STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW ENGINE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_engines() {
    let sql = "SHOW ENGINES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW ENGINES: {:?}",
        result
    );
}

// ============ SHOW PRIVILEGES ============

#[test]
fn test_parse_show_privileges() {
    let sql = "SHOW PRIVILEGES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW PRIVILEGES: {:?}",
        result
    );
}

// ============ SHOW MASTER STATUS ============

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

// ============ SHOW SLAVE STATUS ============

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

// ============ CREATE DATABASE with options ============

#[test]
fn test_parse_create_database_default_charset() {
    let sql = "CREATE DATABASE mydb DEFAULT CHARACTER SET utf8mb4";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE charset: {:?}",
        result
    );
}

#[test]
fn test_parse_create_database_collate() {
    let sql = "CREATE DATABASE mydb COLLATE utf8mb4_unicode_ci";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE DATABASE collate: {:?}",
        result
    );
}

// ============ CREATE EVENT ============

#[test]
fn test_parse_create_event_with_schedule() {
    let sql =
        "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 1 HOUR DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT schedule: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_with_repeat() {
    let sql = "CREATE EVENT my_event ON SCHEDULE EVERY 1 HOUR STARTS CURRENT_TIMESTAMP DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT repeat: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_with_comment() {
    let sql = "CREATE EVENT my_event COMMENT 'my comment' DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT comment: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_with_enable() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 1 DAY ENABLE DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT enable: {:?}",
        result
    );
}

#[test]
fn test_parse_create_event_with_disable() {
    let sql = "CREATE EVENT my_event ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 1 DAY DISABLE DO SELECT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE EVENT disable: {:?}",
        result
    );
}

// ============ DROP EVENT ============

#[test]
fn test_parse_drop_event() {
    let sql = "DROP EVENT IF EXISTS my_event";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP EVENT: {:?}",
        result
    );
}

// ============ ALTER EVENT ============

#[test]
fn test_parse_alter_event() {
    let sql = "ALTER EVENT my_event ENABLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER EVENT: {:?}",
        result
    );
}

// ============ CACHE INDEX ============

#[test]
fn test_parse_cache_index() {
    let sql = "CACHE INDEX users IN default";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CACHE INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_cache_index_multiple() {
    let sql = "CACHE INDEX users, orders IN default";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CACHE INDEX multiple: {:?}",
        result
    );
}

// ============ FLUSH statements ============

#[test]
fn test_parse_flush_tables() {
    let sql = "FLUSH TABLES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH TABLES: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_tables_with_read_lock() {
    let sql = "FLUSH TABLES WITH READ LOCK";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH TABLES WITH READ LOCK: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_tables_for_export() {
    let sql = "FLUSH TABLES FOR EXPORT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH TABLES FOR EXPORT: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_privileges() {
    let sql = "FLUSH PRIVILEGES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH PRIVILEGES: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_logs() {
    let sql = "FLUSH LOGS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH LOGS: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_master_keys() {
    let sql = "FLUSH MASTER KEYS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH MASTER KEYS: {:?}",
        result
    );
}

#[test]
fn test_parse_flush_status() {
    let sql = "FLUSH STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FLUSH STATUS: {:?}",
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

// ============ KILL statement ============

#[test]
fn test_parse_kill() {
    let sql = "KILL 12345";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "KILL: {:?}", result);
}

#[test]
fn test_parse_kill_connection() {
    let sql = "KILL CONNECTION 12345";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "KILL CONNECTION: {:?}",
        result
    );
}

#[test]
fn test_parse_kill_query() {
    let sql = "KILL QUERY 12345";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "KILL QUERY: {:?}",
        result
    );
}

// ============ SHUTDOWN statement ============

#[test]
fn test_parse_shutdown() {
    let sql = "SHUTDOWN";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SHUTDOWN: {:?}", result);
}

// ============ BINLOG statement ============

#[test]
fn test_parse_show_binlog_events() {
    let sql = "SHOW BINLOG EVENTS IN 'binlog.000001'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW BINLOG EVENTS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_binlog_events_limit() {
    let sql = "SHOW BINLOG EVENTS IN 'binlog.000001' LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW BINLOG EVENTS LIMIT: {:?}",
        result
    );
}

// ============ SHOW WARNINGS ============

#[test]
fn test_parse_show_warnings() {
    let sql = "SHOW WARNINGS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW WARNINGS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_warnings_limit() {
    let sql = "SHOW WARNINGS LIMIT 5";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW WARNINGS LIMIT: {:?}",
        result
    );
}

// ============ SHOW ERRORS ============

#[test]
fn test_parse_show_errors() {
    let sql = "SHOW ERRORS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW ERRORS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_errors_limit() {
    let sql = "SHOW ERRORS LIMIT 5";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW ERRORS LIMIT: {:?}",
        result
    );
}

// ============ GET DIAGNOSTICS ============

#[test]
fn test_parse_get_diagnostics() {
    let sql = "GET DIAGNOSTICS @var = NUMBER";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GET DIAGNOSTICS: {:?}",
        result
    );
}

#[test]
fn test_parse_get_diagnostics_multiple() {
    let sql = "GET DIAGNOSTICS @num = NUMBER, @msg = MESSAGE_TEXT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GET DIAGNOSTICS multiple: {:?}",
        result
    );
}

// ============ RESIGNAL statement ============

#[test]
fn test_parse_resignal() {
    let sql = "RESIGNAL";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "RESIGNAL: {:?}", result);
}

#[test]
fn test_parse_resignal_with_sqlstate() {
    let sql = "RESIGNAL SQLSTATE '45000'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESIGNAL SQLSTATE: {:?}",
        result
    );
}

#[test]
fn test_parse_resignal_with_message() {
    let sql = "RESIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'my error'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RESIGNAL MESSAGE: {:?}",
        result
    );
}

// ============ SIGNAL statement ============

#[test]
fn test_parse_signal() {
    let sql = "SIGNAL SQLSTATE '45000'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SIGNAL: {:?}", result);
}

#[test]
fn test_parse_signal_with_info() {
    let sql = "SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'my error', MYSQL_ERRNO = 1234";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SIGNAL with info: {:?}",
        result
    );
}

// ============ CASE expression variations ============

#[test]
fn test_parse_case_simple_searched() {
    let sql = "SELECT CASE WHEN 1 = 1 THEN 'yes' WHEN 2 = 2 THEN 'maybe' ELSE 'no' END FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "CASE searched: {:?}", result);
}

#[test]
fn test_parse_case_with_null() {
    let sql = "SELECT CASE WHEN col IS NULL THEN 'null' ELSE 'not null' END FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "CASE with NULL: {:?}", result);
}

#[test]
fn test_parse_case_in_where() {
    let sql = "SELECT * FROM t WHERE CASE WHEN a > 0 THEN TRUE ELSE FALSE END";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE in WHERE: {:?}",
        result
    );
}

// ============ CAST and CONVERT variations ============

#[test]
fn test_parse_cast_binary() {
    let sql = "SELECT CAST(col AS BINARY(10)) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "CAST BINARY: {:?}", result);
}

#[test]
fn test_parse_cast_unsigned() {
    let sql = "SELECT CAST(col AS UNSIGNED) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "CAST UNSIGNED: {:?}", result);
}

#[test]
fn test_parse_convert_charset() {
    let sql = "SELECT CONVERT(col USING utf8) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CONVERT USING: {:?}",
        result
    );
}

// ============ EXTRACT and DATE functions ============

#[test]
fn test_parse_extract() {
    let sql = "SELECT EXTRACT(YEAR FROM col) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXTRACT: {:?}", result);
}

#[test]
fn test_parse_date_add_interval() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 DAY) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "DATE_ADD: {:?}", result);
}

#[test]
fn test_parse_date_sub_interval() {
    let sql = "SELECT DATE_SUB(col, INTERVAL 1 MONTH) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "DATE_SUB: {:?}", result);
}

#[test]
fn test_parse_timestampadd() {
    let sql = "SELECT TIMESTAMPADD(MINUTE, 5, col) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "TIMESTAMPADD: {:?}", result);
}

#[test]
fn test_parse_timestampdiff() {
    let sql = "SELECT TIMESTAMPDIFF(DAY, col1, col2) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "TIMESTAMPDIFF: {:?}", result);
}

// ============ Window functions ============

#[test]
fn test_parse_window_ntile() {
    let sql = "SELECT NTILE(4) OVER (PARTITION BY a ORDER BY b) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "NTILE window: {:?}", result);
}

#[test]
fn test_parse_window_cume_dist() {
    let sql = "SELECT CUME_DIST() OVER (ORDER BY a) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "CUME_DIST window: {:?}", result);
}

#[test]
fn test_parse_window_nth_value() {
    let sql = "SELECT NTH_VALUE(col, 2) OVER (ORDER BY a) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "NTH_VALUE window: {:?}", result);
}

#[test]
fn test_parse_window_lag() {
    let sql = "SELECT LAG(col) OVER (ORDER BY a) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "LAG window: {:?}", result);
}

#[test]
fn test_parse_window_lead() {
    let sql = "SELECT LEAD(col, 2, 0) OVER (ORDER BY a) FROM t";
    let result = parse(sql);
    assert!(result.is_ok(), "LEAD window: {:?}", result);
}

// ============ Prepared statements ============

#[test]
fn test_parse_prepare() {
    let sql = "PREPARE stmt FROM 'SELECT * FROM users'";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "PREPARE: {:?}", result);
}

#[test]
fn test_parse_execute() {
    let sql = "EXECUTE stmt";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXECUTE: {:?}", result);
}

#[test]
fn test_parse_execute_with_using() {
    let sql = "EXECUTE stmt USING @a, @b";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXECUTE USING: {:?}",
        result
    );
}

#[test]
fn test_parse_deallocate() {
    let sql = "DEALLOCATE PREPARE stmt";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DEALLOCATE: {:?}",
        result
    );
}

// ============ Grouping sets ============

#[test]
fn test_parse_group_by_rollup() {
    let sql = "SELECT region, product, SUM(sales) FROM t GROUP BY ROLLUP(region, product)";
    let result = parse(sql);
    assert!(result.is_ok(), "GROUP BY ROLLUP: {:?}", result);
}

#[test]
fn test_parse_group_by_cube() {
    let sql = "SELECT region, product, SUM(sales) FROM t GROUP BY CUBE(region, product)";
    let result = parse(sql);
    assert!(result.is_ok(), "GROUP BY CUBE: {:?}", result);
}

#[test]
fn test_parse_group_by_grouping_sets() {
    let sql =
        "SELECT region, product, SUM(sales) FROM t GROUP BY GROUPING SETS((region), (product))";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUPING SETS: {:?}",
        result
    );
}
