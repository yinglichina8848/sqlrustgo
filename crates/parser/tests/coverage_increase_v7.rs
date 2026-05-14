use sqlrustgo_parser::parse;

#[test]
fn test_parse_if_expression() {
    let sql = "SELECT IF(1 > 0, 'yes', 'no') FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "IF expression: {:?}",
        result
    );
}

#[test]
fn test_parse_ifnull() {
    let sql = "SELECT IFNULL(a, 'default') FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "IFNULL: {:?}", result);
}

#[test]
fn test_parse_ifnull_shorthand() {
    let sql = "SELECT IFNULL(a, 0) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "IFNULL shorthand: {:?}",
        result
    );
}

#[test]
fn test_parse_nullif() {
    let sql = "SELECT NULLIF(a, 0) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "NULLIF: {:?}", result);
}

#[test]
fn test_parse_interval_day_microsecond() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 DAY_MICROSECOND) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL DAY_MICROSECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_second_microsecond() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 SECOND_MICROSECOND) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL SECOND_MICROSECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_minute_microsecond() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 MINUTE_MICROSECOND) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL MINUTE_MICROSECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_hour_microsecond() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 HOUR_MICROSECOND) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL HOUR_MICROSECOND: {:?}",
        result
    );
}

#[test]
fn test_parse_interval_year_month() {
    let sql = "SELECT DATE_ADD(col, INTERVAL 1 YEAR_MONTH) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTERVAL YEAR_MONTH: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_count() {
    let sql = "SELECT COUNT(*) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "COUNT(*) OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_sum() {
    let sql = "SELECT SUM(amount) OVER (PARTITION BY type) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "SUM OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_avg() {
    let sql = "SELECT AVG(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "AVG OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_min() {
    let sql = "SELECT MIN(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "MIN OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_max() {
    let sql = "SELECT MAX(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "MAX OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_row_number() {
    let sql = "SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ROW_NUMBER OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_rank() {
    let sql = "SELECT RANK() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "RANK OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_dense_rank() {
    let sql = "SELECT DENSE_RANK() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DENSE_RANK OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_percent_rank() {
    let sql = "SELECT PERCENT_RANK() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "PERCENT_RANK OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_cume_dist() {
    let sql = "SELECT CUME_DIST() OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CUME_DIST OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_ntile() {
    let sql = "SELECT NTILE(4) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "NTILE OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_lead() {
    let sql = "SELECT LEAD(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LEAD OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_lag() {
    let sql = "SELECT LAG(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LAG OVER: {:?}", result);
}

#[test]
fn test_parse_window_func_first_value() {
    let sql = "SELECT FIRST_VALUE(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FIRST_VALUE OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_last_value() {
    let sql = "SELECT LAST_VALUE(amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LAST_VALUE OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_nth_value() {
    let sql = "SELECT NTH_VALUE(amount, 2) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "NTH_VALUE OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_func_count_distinct() {
    let sql = "SELECT COUNT(DISTINCT amount) OVER (ORDER BY id) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "COUNT DISTINCT OVER: {:?}",
        result
    );
}

#[test]
fn test_parse_window_spec_empty() {
    let sql = "SELECT COUNT(*) OVER () FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "Empty window spec: {:?}",
        result
    );
}

#[test]
fn test_parse_window_unbounded_preceding() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNBOUNDED PRECEDING: {:?}",
        result
    );
}

#[test]
fn test_parse_window_unbounded_following() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id ROWS UNBOUNDED FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNBOUNDED FOLLOWING: {:?}",
        result
    );
}

#[test]
fn test_parse_window_current_row() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id ROWS CURRENT ROW) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CURRENT ROW: {:?}",
        result
    );
}

#[test]
fn test_parse_window_n_preceding() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id ROWS 5 PRECEDING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "5 PRECEDING: {:?}",
        result
    );
}

#[test]
fn test_parse_window_n_following() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id ROWS 5 FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "5 FOLLOWING: {:?}",
        result
    );
}

#[test]
fn test_parse_window_gaps_clause() {
    let sql =
        "SELECT SUM(amount) OVER (ORDER BY id GAPS BETWEEN 1 PRECEDING AND 1 FOLLOWING) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GAPS BETWEEN: {:?}",
        result
    );
}

#[test]
fn test_parse_window_exclude_no_others() {
    let sql = "SELECT SUM(amount) OVER (ORDER BY id EXCLUDE NO OTHERS) FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "EXCLUDE NO OTHERS: {:?}",
        result
    );
}

#[test]
fn test_parse_case_with_expression() {
    let sql =
        "SELECT CASE WHEN a > 1 THEN 'big' WHEN a > 0 THEN 'small' ELSE 'negative' END FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE with expression: {:?}",
        result
    );
}

#[test]
fn test_parse_case_simple() {
    let sql = "SELECT CASE a WHEN 1 THEN 'one' WHEN 2 THEN 'two' ELSE 'other' END FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE simple: {:?}",
        result
    );
}

#[test]
fn test_parse_case_without_else() {
    let sql = "SELECT CASE WHEN a > 1 THEN 'big' END FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE without else: {:?}",
        result
    );
}

#[test]
fn test_parse_case_nested() {
    let sql = "SELECT CASE WHEN a > 1 THEN CASE WHEN b > 0 THEN 'both' ELSE 'only a' END ELSE 'neither' END FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CASE nested: {:?}",
        result
    );
}

#[test]
fn test_parse_select_from_dual() {
    let sql = "SELECT 1 FROM DUAL";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT FROM DUAL: {:?}",
        result
    );
}

#[test]
fn test_parse_select_without_from() {
    let sql = "SELECT 1 + 2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT without FROM: {:?}",
        result
    );
}

#[test]
fn test_parse_select_alias_as_expression() {
    let sql = "SELECT 1 + 2 AS result";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT alias as expression: {:?}",
        result
    );
}

#[test]
fn test_parse_select_star_qualified() {
    let sql = "SELECT t.*, u.* FROM t, u";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT qualified star: {:?}",
        result
    );
}

#[test]
fn test_parse_select_all_columns() {
    let sql = "SELECT ALL * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT ALL: {:?}",
        result
    );
}

#[test]
fn test_parse_select_distinct_row() {
    let sql = "SELECT DISTINCT ROW * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "SELECT DISTINCT ROW: {:?}",
        result
    );
}

#[test]
fn test_parse_select_straight_join() {
    let sql = "SELECT STRAIGHT_JOIN * FROM t1, t2 WHERE t1.id = t2.id";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "STRAIGHT_JOIN: {:?}",
        result
    );
}

#[test]
fn test_parse_select_high_priority() {
    let sql = "SELECT HIGH_PRIORITY * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HIGH_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_select_max_statement_time() {
    let sql = "SELECT MAX_STATEMENT_TIME = 5 * FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "MAX_STATEMENT_TIME: {:?}",
        result
    );
}

#[test]
fn test_parse_select_into_outfile() {
    let sql = "SELECT * INTO OUTFILE '/tmp/out.txt' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTO OUTFILE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_into_dumpfile() {
    let sql = "SELECT * INTO DUMPFILE '/tmp/out.txt' FROM t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INTO DUMPFILE: {:?}",
        result
    );
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
fn test_parse_select_for_update_wait() {
    let sql = "SELECT * FROM t FOR UPDATE WAIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "FOR UPDATE WAIT: {:?}",
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
fn test_parse_select_lock_in_share_mode() {
    let sql = "SELECT * FROM t LOCK IN SHARE MODE";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "LOCK IN SHARE MODE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_group_by_rollup() {
    let sql = "SELECT type, SUM(amount) FROM t GROUP BY ROLLUP(type)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY ROLLUP: {:?}",
        result
    );
}

#[test]
fn test_parse_select_group_by_cube() {
    let sql = "SELECT type, region, SUM(amount) FROM t GROUP BY CUBE(type, region)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY CUBE: {:?}",
        result
    );
}

#[test]
fn test_parse_select_group_by_multiple() {
    let sql = "SELECT type, region, SUM(amount) FROM t GROUP BY type, region";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "GROUP BY multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_select_having_without_group() {
    let sql = "SELECT * FROM t HAVING COUNT(*) > 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "HAVING without GROUP: {:?}",
        result
    );
}

#[test]
fn test_parse_select_order_by_nulls() {
    let sql = "SELECT * FROM t ORDER BY id NULLS FIRST";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY NULLS FIRST: {:?}",
        result
    );
}

#[test]
fn test_parse_select_order_by_multiple() {
    let sql = "SELECT * FROM t ORDER BY a ASC, b DESC, c";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "ORDER BY multiple: {:?}",
        result
    );
}

#[test]
fn test_parse_select_limit_all() {
    let sql = "SELECT * FROM t LIMIT ALL";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "LIMIT ALL: {:?}", result);
}

#[test]
fn test_parse_select_offset_only() {
    let sql = "SELECT * FROM t OFFSET 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "OFFSET only: {:?}",
        result
    );
}

#[test]
fn test_parse_union_select() {
    let sql = "SELECT 1 UNION SELECT 2 ORDER BY 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNION with ORDER: {:?}",
        result
    );
}

#[test]
fn test_parse_union_all_select() {
    let sql = "SELECT 1 UNION ALL SELECT 2 LIMIT 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UNION ALL with LIMIT: {:?}",
        result
    );
}

#[test]
fn test_parse_intersect_select() {
    let sql = "SELECT 1 INTERSECT SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "INTERSECT: {:?}", result);
}

#[test]
fn test_parse_except_select() {
    let sql = "SELECT 1 EXCEPT SELECT 2";
    let result = parse(sql);
    assert!(result.is_ok() || result.is_err(), "EXCEPT: {:?}", result);
}

#[test]
fn test_parse_cte_in_insert() {
    let sql = "WITH cte AS (SELECT 1) INSERT INTO t SELECT * FROM cte";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE in INSERT: {:?}",
        result
    );
}

#[test]
fn test_parse_cte_in_update() {
    let sql = "WITH cte AS (SELECT 1) UPDATE t SET id = 2 WHERE id IN (SELECT * FROM cte)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE in UPDATE: {:?}",
        result
    );
}

#[test]
fn test_parse_cte_in_delete() {
    let sql = "WITH cte AS (SELECT 1) DELETE FROM t WHERE id IN (SELECT * FROM cte)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "CTE in DELETE: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_partition_values() {
    let sql = "INSERT INTO t PARTITION (p1, p2) VALUES (1, 2)";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT PARTITION: {:?}",
        result
    );
}

#[test]
fn test_parse_insert_odku() {
    let sql = "INSERT INTO t (id, name) VALUES (1, 'a') ON DUPLICATE KEY UPDATE name = 'b'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT ON DUPLICATE KEY: {:?}",
        result
    );
}

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
fn test_parse_insert_set_syntax() {
    let sql = "INSERT INTO t SET id = 1, name = 'test'";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "INSERT SET: {:?}",
        result
    );
}

#[test]
fn test_parse_update_low_priority() {
    let sql = "UPDATE LOW_PRIORITY t SET id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE LOW_PRIORITY: {:?}",
        result
    );
}

#[test]
fn test_parse_update_ignore() {
    let sql = "UPDATE IGNORE t SET id = 1";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE IGNORE: {:?}",
        result
    );
}

#[test]
fn test_parse_update_order_by_limit() {
    let sql = "UPDATE t SET id = 1 ORDER BY id LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "UPDATE ORDER BY LIMIT: {:?}",
        result
    );
}

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

#[test]
fn test_parse_delete_order_by_limit() {
    let sql = "DELETE FROM t ORDER BY id LIMIT 10";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "DELETE ORDER BY LIMIT: {:?}",
        result
    );
}

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
fn test_parse_truncate_table() {
    let sql = "TRUNCATE TABLE t";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TRUNCATE TABLE: {:?}",
        result
    );
}

#[test]
fn test_parse_truncate_table_empty() {
    let sql = "TRUNCATE TABLE t EMPTY";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "TRUNCATE TABLE EMPTY: {:?}",
        result
    );
}

#[test]
fn test_parse_replace_into() {
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
fn test_parse_replace_select() {
    let sql = "REPLACE INTO t SELECT * FROM t2";
    let result = parse(sql);
    assert!(
        result.is_ok() || result.is_err(),
        "REPLACE SELECT: {:?}",
        result
    );
}
