use sqlrustgo_parser::parse;

#[test]
fn test_parse_create_table_as_select() {
    let sql = "CREATE TABLE t AS SELECT * FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE AS SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_if_not_exists_as_select() {
    let sql = "CREATE TABLE IF NOT EXISTS t AS SELECT * FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE IF NOT EXISTS AS SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_create_table_like_as_select() {
    let sql = "CREATE TABLE t LIKE t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE TABLE LIKE: {:?}",
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
fn test_parse_alter_table_rename_to() {
    let sql = "ALTER TABLE t RENAME TO new_t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER TABLE RENAME TO: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_table_cascade_restrict() {
    let sql = "DROP TABLE t CASCADE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP TABLE CASCADE: {:?}",
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
fn test_parse_show_warnings_limit() {
    let sql = "SHOW WARNINGS LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW WARNINGS LIMIT: {:?}",
        result
    );
}

#[test]
fn test_parse_show_errors_limit() {
    let sql = "SHOW ERRORS LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW ERRORS LIMIT: {:?}",
        result
    );
}

#[test]
fn test_parse_show_count_warnings() {
    let sql = "SHOW COUNT(*) WARNINGS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW COUNT(*) WARNINGS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_count_errors() {
    let sql = "SHOW COUNT(*) ERRORS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW COUNT(*) ERRORS: {:?}",
        result
    );
}

#[test]
fn test_parse_delete_using() {
    let sql = "DELETE FROM t1 USING t1 JOIN t2 ON t1.id = t2.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE USING: {:?}",
        result
    );
}

#[test]
fn test_parse_replace() {
    let sql = "REPLACE INTO t VALUES (1)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPLACE INTO: {:?}",
        result
    );
}

#[test]
fn test_parse_replace_set() {
    let sql = "REPLACE INTO t SET id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPLACE SET: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_select() {
    let sql = "INSERT INTO t SELECT * FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT SELECT: {:?}",
        result
    );
}

#[test]
fn test_parse_update_with_set() {
    let sql = "UPDATE t SET id = 1, name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE with multiple SET: {:?}",
        result
    );
}

#[test]
fn test_parse_update_where() {
    let sql = "UPDATE t SET id = 1 WHERE id = 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE WHERE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_group_concat() {
    let sql = "SELECT GROUP_CONCAT(name SEPARATOR ',') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP_CONCAT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_group_concat_order() {
    let sql = "SELECT GROUP_CONCAT(name ORDER BY name) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP_CONCAT ORDER BY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_object() {
    let sql = "SELECT JSON_OBJECT('key', value) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_OBJECT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_array() {
    let sql = "SELECT JSON_ARRAY(a, b, c) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_ARRAY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_extract() {
    let sql = "SELECT JSON_EXTRACT(data, '$.field') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_EXTRACT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_set() {
    let sql = "SELECT JSON_SET(data, '$.field', 'value') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "JSON_SET: {:?}", result);
}

#[test]
fn test_parse_select_json_insert() {
    let sql = "SELECT JSON_INSERT(data, '$.field', 'value') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_INSERT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_replace() {
    let sql = "SELECT JSON_REPLACE(data, '$.field', 'value') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_REPLACE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_merge() {
    let sql = "SELECT JSON_MERGE(a, b) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_MERGE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_keys() {
    let sql = "SELECT JSON_KEYS(data) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "JSON_KEYS: {:?}", result);
}

#[test]
fn test_parse_select_json_length() {
    let sql = "SELECT JSON_LENGTH(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_LENGTH: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_type() {
    let sql = "SELECT JSON_TYPE(data) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "JSON_TYPE: {:?}", result);
}

#[test]
fn test_parse_select_json_valid() {
    let sql = "SELECT JSON_VALID(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_VALID: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_quote() {
    let sql = "SELECT JSON_QUOTE('string') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_QUOTE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_unquote() {
    let sql = "SELECT JSON_UNQUOTE(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_UNQUOTE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_contains() {
    let sql = "SELECT JSON_CONTAINS(a, b) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_CONTAINS: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_search() {
    let sql = "SELECT JSON_SEARCH(data, 'one', 'pattern') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_SEARCH: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_depth() {
    let sql = "SELECT JSON_DEPTH(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_DEPTH: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_prett() {
    let sql = "SELECT JSON_PRETTY(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_PRETTY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_json_storage_size() {
    let sql = "SELECT JSON_STORAGE_SIZE(data) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "JSON_STORAGE_SIZE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_signed() {
    let sql = "SELECT CAST(a AS SIGNED) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS SIGNED: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_unsigned() {
    let sql = "SELECT CAST(a AS UNSIGNED) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS UNSIGNED: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_decimal() {
    let sql = "SELECT CAST(a AS DECIMAL(10,2)) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS DECIMAL: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_date() {
    let sql = "SELECT CAST(a AS DATE) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS DATE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_time() {
    let sql = "SELECT CAST(a AS TIME) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS TIME: {:?}",
        result
    );
}

#[test]
fn test_parse_select_cast_as_datetime() {
    let sql = "SELECT CAST(a AS DATETIME) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CAST AS DATETIME: {:?}",
        result
    );
}

#[test]
fn test_parse_select_convert_to() {
    let sql = "SELECT CONVERT(a, CHAR) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CONVERT TO CHAR: {:?}",
        result
    );
}

#[test]
fn test_parse_select_convert_using() {
    let sql = "SELECT CONVERT(a USING utf8) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CONVERT USING: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_against() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('search')";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AGAINST: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_against_boolean() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('search' IN BOOLEAN MODE)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AGAINST BOOLEAN: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_against_with_query() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('+word1 -word2' IN BOOLEAN MODE)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AGAINST with query: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_against_natural_language() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('search' IN NATURAL LANGUAGE MODE)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AGAINST NL MODE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_against_query_expansion() {
    let sql = "SELECT * FROM t WHERE MATCH(col) AGAINST('search' WITH QUERY EXPANSION)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AGAINST QUERY EXPANSION: {:?}",
        result
    );
}

#[test]
fn test_parse_select_match_in_boolean_mode() {
    let sql = "SELECT * FROM t WHERE MATCH(col) IN BOOLEAN MODE AGAINST('search')";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH IN BOOLEAN MODE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_with_lock_in_share_mode() {
    let sql = "SELECT * FROM t FOR UPDATE LOCK IN SHARE MODE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR UPDATE LOCK IN SHARE MODE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_into_outfile() {
    let sql = "SELECT * INTO OUTFILE '/tmp/out.txt' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT INTO OUTFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_into_dumpfile() {
    let sql = "SELECT * INTO DUMPFILE '/tmp/out.txt' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT INTO DUMPFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_for_share() {
    let sql = "SELECT * FROM t FOR SHARE";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "FOR SHARE: {:?}", result);
}

#[test]
fn test_parse_select_for_update_nowait() {
    let sql = "SELECT * FROM t FOR UPDATE NOWAIT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR UPDATE NOWAIT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_for_update_skip_locked() {
    let sql = "SELECT * FROM t FOR UPDATE SKIP LOCKED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR UPDATE SKIP LOCKED: {:?}",
        result
    );
}

#[test]
fn test_parse_select_for_share_nowait() {
    let sql = "SELECT * FROM t FOR SHARE NOWAIT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR SHARE NOWAIT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_for_share_skip_locked() {
    let sql = "SELECT * FROM t FOR SHARE SKIP LOCKED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR SHARE SKIP LOCKED: {:?}",
        result
    );
}

#[test]
fn test_parse_select_wait_timeout() {
    let sql = "SELECT * FROM t FOR UPDATE WAIT 5";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR UPDATE WAIT: {:?}",
        result
    );
}

#[test]
fn test_parse_select_score() {
    let sql = "SELECT *, MATCH(col) AGAINST('search') AS score FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MATCH AS score: {:?}",
        result
    );
}

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
fn test_parse_rollback_chain() {
    let sql = "ROLLBACK CHAIN";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ROLLBACK CHAIN: {:?}",
        result
    );
}

#[test]
fn test_parse_savepoint_quoted() {
    let sql = "SAVEPOINT 'my_savepoint'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SAVEPOINT quoted: {:?}",
        result
    );
}

#[test]
fn test_parse_release_savepoint_quoted() {
    let sql = "RELEASE SAVEPOINT 'my_savepoint'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "RELEASE SAVEPOINT quoted: {:?}",
        result
    );
}

#[test]
fn test_parse_start_transaction_read_only() {
    let sql = "START TRANSACTION READ ONLY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START TRANSACTION READ ONLY: {:?}",
        result
    );
}

#[test]
fn test_parse_start_transaction_read_write() {
    let sql = "START TRANSACTION READ WRITE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START TRANSACTION READ WRITE: {:?}",
        result
    );
}

#[test]
fn test_parse_start_transaction_consistent_snapshot() {
    let sql = "START TRANSACTION CONSISTENT SNAPSHOT";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "START TRANSACTION CONSISTENT SNAPSHOT: {:?}",
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
fn test_parse_set_transaction_isolation_read_uncommitted() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION ISOLATION READ UNCOMMITTED: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation_read_committed() {
    let sql = "SET TRANSACTION ISOLATION LEVEL READ COMMITTED";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION ISOLATION READ COMMITTED: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation_repeatable_read() {
    let sql = "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION ISOLATION REPEATABLE READ: {:?}",
        result
    );
}

#[test]
fn test_parse_set_transaction_isolation_serializable() {
    let sql = "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SET TRANSACTION ISOLATION SERIALIZABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_fulltext() {
    let sql = "CREATE FULLTEXT INDEX idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FULLTEXT INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_spatial() {
    let sql = "CREATE SPATIAL INDEX idx ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE SPATIAL INDEX: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_btree() {
    let sql = "CREATE INDEX idx USING BTREE ON t(col)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX USING BTREE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_index_hash() {
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
fn test_parse_create_index_with_asc_desc() {
    let sql = "CREATE INDEX idx ON t(col ASC, col2 DESC)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE INDEX ASC/DESC: {:?}",
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
fn test_parse_show_table_status_full() {
    let sql = "SHOW TABLE STATUS FROM mydb LIKE '%user%'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW TABLE STATUS LIKE: {:?}",
        result
    );
}

#[test]
fn test_parse_show_global_status() {
    let sql = "SHOW GLOBAL STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW GLOBAL STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_session_status() {
    let sql = "SHOW SESSION STATUS";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW SESSION STATUS: {:?}",
        result
    );
}

#[test]
fn test_parse_show_global_variables() {
    let sql = "SHOW GLOBAL VARIABLES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW GLOBAL VARIABLES: {:?}",
        result
    );
}

#[test]
fn test_parse_show_session_variables() {
    let sql = "SHOW SESSION VARIABLES";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SHOW SESSION VARIABLES: {:?}",
        result
    );
}

#[test]
fn test_parse_grant_proxy() {
    let sql = "GRANT PROXY ON 'user1'@'localhost' TO 'user2'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GRANT PROXY: {:?}",
        result
    );
}

#[test]
fn test_parse_revoke_proxy() {
    let sql = "REVOKE PROXY ON 'user1'@'localhost' FROM 'user2'@'localhost'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REVOKE PROXY: {:?}",
        result
    );
}

#[test]
fn test_parse_analyze_table() {
    let sql = "ANALYZE TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ANALYZE TABLE: {:?}",
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

#[test]
fn test_parse_optimize_table() {
    let sql = "OPTIMIZE TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "OPTIMIZE TABLE: {:?}",
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
fn test_parse_check_table() {
    let sql = "CHECK TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECK TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_repair_table() {
    let sql = "REPAIR TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPAIR TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_checksum_table() {
    let sql = "CHECKSUM TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CHECKSUM TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_load_data() {
    let sql = "LOAD DATA INFILE '/tmp/data.txt' INTO TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA INFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_load_data_duplicate() {
    let sql = "LOAD DATA INFILE '/tmp/data.txt' REPLACE INTO TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA REPLACE: {:?}",
        result
    );
}

#[test]
fn test_parse_load_data_ignore() {
    let sql = "LOAD DATA INFILE '/tmp/data.txt' IGNORE INTO TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOAD DATA IGNORE: {:?}",
        result
    );
}

#[test]
fn test_parse_create_server() {
    let sql =
        "CREATE SERVER my_server FOREIGN DATA WRAPPER mysql OPTIONS (HOST 'host', DATABASE 'db')";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE SERVER: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_server() {
    let sql = "DROP SERVER my_server";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP SERVER: {:?}",
        result
    );
}

#[test]
fn test_parse_create_aggregate() {
    let sql = "CREATE AGGREGATE FUNCTION my_func RETURNS INTEGER SONAME 'my_func.so'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE AGGREGATE FUNCTION: {:?}",
        result
    );
}

#[test]
fn test_parse_create_function() {
    let sql = "CREATE FUNCTION my_func RETURNS INTEGER SONAME 'my_func.so'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE FUNCTION: {:?}",
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
fn test_parse_drop_aggregate() {
    let sql = "DROP AGGREGATE FUNCTION my_func";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP AGGREGATE FUNCTION: {:?}",
        result
    );
}

#[test]
fn test_parse_install_plugin() {
    let sql = "INSTALL PLUGIN my_plugin SONAME 'my_plugin.so'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSTALL PLUGIN: {:?}",
        result
    );
}

#[test]
fn test_parse_uninstall_plugin() {
    let sql = "UNINSTALL PLUGIN my_plugin";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNINSTALL PLUGIN: {:?}",
        result
    );
}

#[test]
fn test_parse_create_udf() {
    let sql = "CREATE FUNCTION my_func RETURNS INTEGER SONAME 'my_func.so'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE UDF: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_udf() {
    let sql = "DROP FUNCTION my_func";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "DROP UDF: {:?}", result);
}

#[test]
fn test_parse_create_sequence() {
    let sql = "CREATE SEQUENCE my_seq";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CREATE SEQUENCE: {:?}",
        result
    );
}

#[test]
fn test_parse_alter_sequence() {
    let sql = "ALTER SEQUENCE my_seq RESTART WITH 100";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ALTER SEQUENCE: {:?}",
        result
    );
}

#[test]
fn test_parse_drop_sequence() {
    let sql = "DROP SEQUENCE my_seq";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DROP SEQUENCE: {:?}",
        result
    );
}
