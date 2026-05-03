//! Executor 覆盖率提升专项测试
//!
//! 目标: executor 模块覆盖率从 44% 提升至 60%+
//! 策略: 通过 MemoryExecutionEngine 执行 SQL，覆盖边界条件
//!
//! 覆盖范围:
//! 1. JOIN — NULL key, 空表, 半连接, 反连接, FULL, 交叉连接
//! 2. Aggregation — 空输入, NULL 聚合, HAVING, DISTINCT, GROUP BY
//! 3. Filter — NULL predicate, 三值逻辑, 恒真/恒假
//! 4. Expression — NULL 传播, 算术, 比较
//! 5. DDL + DML 边界
//! 6. 嵌套计划

use sqlrustgo::{ExecutionEngine, ExecutorResult, MemoryExecutionEngine};
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

fn engine() -> MemoryExecutionEngine {
    ExecutionEngine::with_memory()
}

fn run(engine: &mut MemoryExecutionEngine, sql: &str) -> ExecutorResult {
    engine.execute(sql).expect(sql)
}

fn assert_rows(result: &ExecutorResult, expected: usize, msg: &str) {
    assert_eq!(result.rows.len(), expected, "{}: got {} rows, expected {}",
        msg, result.rows.len(), expected);
}

// ============================================================================
// 测试 1: JOIN 边界路径
// ============================================================================

mod join_tests {
    use super::*;

    /// INNER JOIN — 两表都为空
    #[test]
    fn test_inner_join_empty_both() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id");
        assert_rows(&r, 0, "inner join two empty tables");
    }

    /// INNER JOIN — 右表为空
    #[test]
    fn test_inner_join_empty_right() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 100)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id");
        assert_rows(&r, 0, "inner join with empty right");
    }

    /// INNER JOIN — 左表为空
    #[test]
    fn test_inner_join_empty_left() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap(); // empty
        e.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1, 100)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id");
        assert_rows(&r, 0, "inner join with empty left");
    }

    /// LEFT JOIN — 右表无匹配（必须 NULL padding）
    #[test]
    fn test_left_join_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 100), (2, 200)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT t1.id, t2.id FROM t1 LEFT JOIN t2 ON t1.id = t2.id");
        // left join must return all left rows, with NULL for unmatched
        assert_rows(&r, 2, "left join no match");
    }

    /// RIGHT JOIN — 左表无匹配
    #[test]
    fn test_right_join_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap(); // empty
        e.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1, 100), (2, 200)").unwrap();
        let r = run(&mut e, "SELECT t1.id, t2.id FROM t1 RIGHT JOIN t2 ON t1.id = t2.id");
        // right join returns all right rows
        assert_rows(&r, 2, "right join no match");
    }

    /// FULL OUTER JOIN — 两边都不匹配
    #[test]
    fn test_full_outer_join_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (2)").unwrap();
        let r = run(&mut e, "SELECT t1.id, t2.id FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id");
        // full outer join: matched rows + unmatched left (NULL,2) + unmatched right (1,NULL)
        assert_eq!(r.rows.len(), 3, "full outer join no match");
    }

    /// CROSS JOIN — 笛卡尔积（空表）
    #[test]
    fn test_cross_join_empty() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("CREATE TABLE t2 (b INTEGER)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 CROSS JOIN t2");
        assert_rows(&r, 0, "cross join two empty tables");
    }

    /// CROSS JOIN — 一表有数据
    #[test]
    fn test_cross_join_with_data() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        e.execute("CREATE TABLE t2 (b INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (10)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 CROSS JOIN t2");
        // Cartesian product: 2 * 1 = 2
        assert_rows(&r, 2, "cross join with data");
    }

    /// LEFT SEMI JOIN — 只返回左表（EXISTS 语义）
    #[test]
    fn test_left_semi_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (2), (3), (4)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE EXISTS (SELECT 1 FROM t2 WHERE t2.id = t1.id)");
        assert_rows(&r, 2, "left semi join via EXISTS");
    }

    /// LEFT ANTI JOIN — 返回无匹配的左行（NOT EXISTS 语义）
    #[test]
    fn test_left_anti_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (2), (3), (4)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE NOT EXISTS (SELECT 1 FROM t2 WHERE t2.id = t1.id)");
        assert_rows(&r, 1, "left anti join via NOT EXISTS"); // only id=1
    }

    /// JOIN with NULL key — NULL = NULL 不匹配
    #[test]
    fn test_join_null_key() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (NULL), (1)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 JOIN t2 ON t1.id = t2.id");
        // NULL = NULL is not TRUE in SQL, so only (1,1) matches
        assert_rows(&r, 1, "join with NULL key");
    }

    /// Self JOIN
    #[test]
    fn test_self_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')").unwrap();
        let r = run(&mut e, "SELECT a.id, b.id FROM t1 a JOIN t1 b ON a.id = b.id");
        // self join: (1,1) and (2,2)
        assert_rows(&r, 2, "self join");
    }

    /// Multiple JOIN conditions
    #[test]
    fn test_join_multiple_conditions() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 10), (2, 20)").unwrap();
        e.execute("CREATE TABLE t2 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1, 10), (1, 20), (2, 10)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 JOIN t2 ON t1.a = t2.a AND t1.b = t2.b");
        // matches: (1,10)-(1,10) and (2,20)-(2,20)?? no, t2 has (1,10),(1,20),(2,10)
        // only (1,10)-(1,10) and (2,20)-?? no (2,20) in t2, so only (1,10)
        assert_rows(&r, 1, "join multiple conditions");
    }
}

// ============================================================================
// 测试 2: Aggregation 边界路径
// ============================================================================

mod aggregate_tests {
    use super::*;

    /// 空表聚合 COUNT(*)
    #[test]
    fn test_agg_empty_table_count_star() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT COUNT(*) FROM t1");
        assert_rows(&r, 1, "COUNT(*) on empty table");
        // COUNT(*) = 0
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(0));
    }

    /// 空表聚合 SUM
    #[test]
    fn test_agg_empty_table_sum() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT SUM(x) FROM t1");
        assert_rows(&r, 1, "SUM on empty table");
        // SUM of empty = NULL
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Null);
    }

    /// 全 NULL 聚合
    #[test]
    fn test_agg_all_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (NULL), (NULL)").unwrap();
        let r = run(&mut e, "SELECT COUNT(x), SUM(x), AVG(x), MIN(x), MAX(x) FROM t1");
        assert_rows(&r, 1, "aggregate all NULL");
        // COUNT(col) skips NULLs = 0
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(0));
    }

    /// COUNT(*) vs COUNT(col) 区别
    #[test]
    fn test_count_star_vs_count_col() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1), (2)").unwrap();
        let r1 = run(&mut e, "SELECT COUNT(*) FROM t1");
        let r2 = run(&mut e, "SELECT COUNT(x) FROM t1");
        // COUNT(*) = 3, COUNT(x) = 2 (skips NULL)
        assert_eq!(r1.rows[0][0], sqlrustgo_types::Value::Integer(3));
        assert_eq!(r2.rows[0][0], sqlrustgo_types::Value::Integer(2));
    }

    /// GROUP BY NULL key
    #[test]
    fn test_group_by_null_key() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (k INTEGER, v INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL, 1), (NULL, 2), (1, 3)").unwrap();
        let r = run(&mut e, "SELECT k, COUNT(*) FROM t1 GROUP BY k");
        assert_rows(&r, 2, "group by null key"); // NULL group + 1 group
    }

    /// HAVING 过滤后为空
    #[test]
    fn test_having_filters_all() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        let r = run(&mut e, "SELECT x, COUNT(*) FROM t1 GROUP BY x HAVING COUNT(*) > 100");
        assert_rows(&r, 0, "HAVING filters all");
    }

    /// GROUP BY + ORDER BY 交互
    #[test]
    fn test_group_by_order_by() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a TEXT, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES ('c', 1), ('a', 2), ('b', 3)").unwrap();
        let r = run(&mut e, "SELECT a, SUM(b) FROM t1 GROUP BY a ORDER BY a");
        assert_rows(&r, 3, "group by order by");
        // should be ordered: a, b, c
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Text("a".to_string()));
        assert_eq!(r.rows[2][0], sqlrustgo_types::Value::Text("c".to_string()));
    }

    /// DISTINCT with NULLs
    #[test]
    fn test_distinct_with_nulls() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1), (NULL), (1), (2)").unwrap();
        let r = run(&mut e, "SELECT DISTINCT x FROM t1");
        // NULL, 1, 2 = 3 distinct values
        assert_eq!(r.rows.len(), 3, "distinct with nulls");
    }

    /// 多列 GROUP BY
    #[test]
    fn test_group_by_multiple_columns() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER, c INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 1, 10), (1, 1, 20), (1, 2, 30)").unwrap();
        let r = run(&mut e, "SELECT a, b, SUM(c) FROM t1 GROUP BY a, b");
        assert_rows(&r, 2, "group by multiple columns");
    }

    /// 聚合 + JOIN
    #[test]
    fn test_agg_with_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER, v INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 10), (2, 20)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1), (1), (2)").unwrap();
        let r = run(&mut e, "SELECT t1.id, SUM(t1.v) FROM t1 JOIN t2 ON t1.id = t2.id GROUP BY t1.id");
        // id=1: 10 * 2 rows = 20; id=2: 20 * 1 row = 20
        assert_rows(&r, 2, "agg with join");
    }
}

// ============================================================================
// 测试 3: Filter 边界路径
// ============================================================================

mod filter_tests {
    use super::*;

    /// WHERE NULL — 恒假
    #[test]
    fn test_filter_where_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE NULL");
        assert_rows(&r, 0, "WHERE NULL");
    }

    /// WHERE 1=1 — 恒真
    #[test]
    fn test_filter_always_true() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE 1=1");
        assert_rows(&r, 2, "WHERE 1=1");
    }

    /// WHERE 1=0 — 恒假
    #[test]
    fn test_filter_always_false() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE 1=0");
        assert_rows(&r, 0, "WHERE 1=0");
    }

    /// NULL = NULL — 不是 TRUE
    #[test]
    fn test_filter_null_equality() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x = NULL");
        assert_rows(&r, 0, "x = NULL filters all");
    }

    /// NULL IS NULL — TRUE
    #[test]
    fn test_filter_is_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x IS NULL");
        assert_rows(&r, 1, "IS NULL");
    }

    /// x IS NOT NULL
    #[test]
    fn test_filter_is_not_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x IS NOT NULL");
        assert_rows(&r, 1, "IS NOT NULL");
    }

    /// NULL > 10 — 不是 TRUE
    #[test]
    fn test_filter_null_comparison() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (15)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x > 10");
        assert_rows(&r, 1, "NULL > 10 not TRUE");
    }

    /// 复杂 AND/OR
    #[test]
    fn test_filter_and_or() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (15, 3), (5, 3), (15, 10), (5, 10)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE (a > 10 AND b < 5) OR (a < 10 AND b > 5)");
        // (15,3) matches first clause; (5,10) matches second clause
        assert_rows(&r, 2, "complex AND/OR");
    }

    /// NOT 表达式
    #[test]
    fn test_filter_not() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (0), (1), (2)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE NOT (x = 1)");
        assert_rows(&r, 2, "NOT expression");
    }

    /// BETWEEN
    #[test]
    fn test_filter_between() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (5), (10), (15), (20)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x BETWEEN 10 AND 20");
        assert_rows(&r, 3, "BETWEEN");
    }

    /// NOT BETWEEN
    #[test]
    fn test_filter_not_between() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (5), (10), (15), (20)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x NOT BETWEEN 10 AND 20");
        assert_rows(&r, 1, "NOT BETWEEN");
    }

    /// LIKE 表达式
    #[test]
    fn test_filter_like() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x TEXT)").unwrap();
        e.execute("INSERT INTO t1 VALUES ('hello'), ('world'), ('hell'), ('hallo')").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x LIKE 'hel%'");
        assert_rows(&r, 2, "LIKE pattern");
    }

    /// IN with NULL
    #[test]
    fn test_filter_in_with_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (NULL), (3)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x IN (1, 2, NULL)");
        assert_rows(&r, 2, "IN with NULL");
    }

    /// IN — 无匹配
    #[test]
    fn test_filter_in_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x IN (10, 20)");
        assert_rows(&r, 0, "IN no match");
    }

    /// NOT IN with NULL
    #[test]
    fn test_filter_not_in_with_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
        let r = run(&mut e, "SELECT * FROM t1 WHERE x NOT IN (1, 2, NULL)");
        assert_rows(&r, 0, "NOT IN with NULL");
    }
}

// ============================================================================
// 测试 4: Expression NULL 传播
// ============================================================================

mod expression_tests {
    use super::*;

    /// NULL + 数值 = NULL
    #[test]
    fn test_expr_null_arithmetic() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL, 10), (5, 10)").unwrap();
        let r = run(&mut e, "SELECT a + b FROM t1 WHERE a + b > 0");
        assert_rows(&r, 1, "NULL + b filtered out");
    }

    /// 算术表达式除零
    #[test]
    fn test_expr_divide_by_zero() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (0), (5)").unwrap();
        let r = run(&mut e, "SELECT 10 / a FROM t1 WHERE a != 0");
        assert_rows(&r, 1, "divide by zero protected");
    }

    /// 三值逻辑 AND
    #[test]
    fn test_expr_three_valued_and() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (5), (15)").unwrap();
        // a > 10 AND NULL — if a>10 is true, then TRUE AND NULL = NULL (not true)
        let r = run(&mut e, "SELECT * FROM t1 WHERE a > 10 AND NULL");
        assert_rows(&r, 0, "TRUE AND NULL = NULL (filtered)");
    }

    /// 三值逻辑 OR
    #[test]
    fn test_expr_three_valued_or() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (5)").unwrap();
        // a > 10 OR NULL — a>10 is false, so FALSE OR NULL = NULL (not true)
        let r = run(&mut e, "SELECT * FROM t1 WHERE a > 10 OR NULL");
        assert_rows(&r, 0, "FALSE OR NULL = NULL (filtered)");
    }

    /// CASE WHEN NULL
    #[test]
    fn test_expr_case_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1)").unwrap();
        let r = run(&mut e, "SELECT CASE WHEN x IS NULL THEN 0 ELSE x END FROM t1");
        assert_rows(&r, 2, "CASE WHEN NULL");
    }

    /// COALESCE
    #[test]
    fn test_expr_coalesce() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (NULL), (1), (NULL)").unwrap();
        let r = run(&mut e, "SELECT COALESCE(x, 999) FROM t1");
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(999));
        assert_eq!(r.rows[1][0], sqlrustgo_types::Value::Integer(1));
        assert_eq!(r.rows[2][0], sqlrustgo_types::Value::Integer(999));
    }

    /// NULLIF
    #[test]
    fn test_expr_nullif() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (0), (1)").unwrap();
        let r = run(&mut e, "SELECT NULLIF(x, 0) FROM t1");
        // NULLIF(0,0) = NULL; NULLIF(1,0) = 1
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Null);
        assert_eq!(r.rows[1][0], sqlrustgo_types::Value::Integer(1));
    }
}

// ============================================================================
// 测试 5: DDL + DML 边界
// ============================================================================

mod ddl_dml_tests {
    use super::*;

    /// INSERT 后 SELECT
    #[test]
    fn test_insert_then_select() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 'a'), (2, 'b')").unwrap();
        let r = run(&mut e, "SELECT * FROM t1");
        assert_rows(&r, 2, "insert then select");
    }

    /// UPDATE 影响 0 行
    #[test]
    fn test_update_zero_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1)").unwrap();
        let r = run(&mut e, "UPDATE t1 SET x = 100 WHERE x = 999");
        assert_rows(&r, 0, "update 0 rows");
    }

    /// DELETE 影响 0 行
    #[test]
    fn test_delete_zero_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1)").unwrap();
        let r = run(&mut e, "DELETE FROM t1 WHERE x = 999");
        assert_rows(&r, 0, "delete 0 rows");
    }

    /// TRUNCATE
    #[test]
    fn test_truncate() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
        e.execute("TRUNCATE TABLE t1").unwrap();
        let r = run(&mut e, "SELECT COUNT(*) FROM t1");
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(0));
    }

    /// ALTER TABLE ADD COLUMN
    #[test]
    fn test_alter_add_column() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1)").unwrap();
        e.execute("ALTER TABLE t1 ADD COLUMN b TEXT").unwrap();
        let r = run(&mut e, "SELECT * FROM t1");
        assert_rows(&r, 1, "alter add column");
        assert_eq!(r.rows[0].len(), 2, "should have 2 columns");
    }

    /// DROP TABLE 然后查
    #[test]
    fn test_drop_table() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("DROP TABLE t1").unwrap();
        let r = e.execute("SELECT * FROM t1");
        assert!(r.is_err(), "dropped table should error");
    }

    /// INSERT 多行
    #[test]
    fn test_insert_multiple_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3), (4), (5)").unwrap();
        let r = run(&mut e, "SELECT COUNT(*) FROM t1");
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(5));
    }
}

// ============================================================================
// 测试 6: 子查询边界
// ============================================================================

mod subquery_tests {
    use super::*;

    /// 子查询返回空
    #[test]
    fn test_subquery_empty() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        e.execute("CREATE TABLE t2 (y INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT * FROM t1 WHERE x IN (SELECT y FROM t2)");
        assert_rows(&r, 0, "IN with empty subquery");
    }

    /// 标量子查询
    #[test]
    fn test_scalar_subquery() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        e.execute("CREATE TABLE t2 (y INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (100)").unwrap();
        let r = run(&mut e, "SELECT (SELECT y FROM t2 LIMIT 1) AS scalar FROM t1");
        assert_rows(&r, 2, "scalar subquery");
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(100));
    }

    /// EXISTS with empty
    #[test]
    fn test_exists_empty() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1)").unwrap();
        e.execute("CREATE TABLE t2 (y INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT * FROM t1 WHERE EXISTS (SELECT 1 FROM t2)");
        assert_rows(&r, 0, "EXISTS empty subquery");
    }

    /// NOT EXISTS
    #[test]
    fn test_not_exists() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        e.execute("CREATE TABLE t2 (y INTEGER)").unwrap(); // empty
        let r = run(&mut e, "SELECT * FROM t1 WHERE NOT EXISTS (SELECT 1 FROM t2 WHERE y = t1.x)");
        assert_rows(&r, 2, "NOT EXISTS with empty");
    }
}

// ============================================================================
// 测试 7: 深度嵌套计划
// ============================================================================

mod deep_nesting_tests {
    use super::*;

    /// 深度嵌套: JOIN -> AGG -> FILTER -> SORT -> LIMIT
    #[test]
    fn test_deep_nesting_join_agg_filter_sort() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 10), (2, 20), (1, 15), (2, 25)").unwrap();
        e.execute("CREATE TABLE t2 (a INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1), (2)").unwrap();
        let r = run(&mut e,
            "SELECT t1.a, SUM(t1.b) FROM t1 JOIN t2 ON t1.a = t2.a \
             GROUP BY t1.a HAVING SUM(t1.b) > 15 ORDER BY t1.a LIMIT 10");
        assert_rows(&r, 1, "deep nesting: join->agg->having->sort->limit");
        // a=1: 10+15=25 > 15; a=2: 20+25=45 > 15; ordered by a, limit 10
        assert_eq!(r.rows[0][0], sqlrustgo_types::Value::Integer(1));
    }

    /// 嵌套子查询 + JOIN
    #[test]
    fn test_nested_subquery_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (x INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2), (3)").unwrap();
        e.execute("CREATE TABLE t2 (y INTEGER)").unwrap();
        e.execute("INSERT INTO t2 VALUES (10), (20)").unwrap();
        let r = run(&mut e,
            "SELECT * FROM (SELECT * FROM t1 WHERE x > 1) AS sub1 \
             JOIN (SELECT * FROM t2 WHERE y > 10) AS sub2 ON sub1.x = sub2.y - 9");
        // sub1: 2,3; sub2: 20; 2 = 20-18? no; 3 = 20-17? no
        assert_rows(&r, 0, "nested subquery join");
    }

    /// 多重聚合
    #[test]
    fn test_multiple_aggregates() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1, 10), (1, 20), (2, 30)").unwrap();
        let r = run(&mut e,
            "SELECT a, COUNT(*), SUM(b), AVG(b), MIN(b), MAX(b) FROM t1 GROUP BY a");
        assert_rows(&r, 2, "multiple aggregates");
        // a=1: count=2, sum=30, avg=15, min=10, max=20
        assert_eq!(r.rows[0][1], sqlrustgo_types::Value::Integer(2));
        assert_eq!(r.rows[0][2], sqlrustgo_types::Value::Integer(30));
    }
}
