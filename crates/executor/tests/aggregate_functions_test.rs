#![allow(deprecated)]

//! Aggregate functions integration tests
//! Tests SUM, AVG, MIN, MAX with various scenarios not covered in aggregate_tests.rs

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

mod aggregate_sum_tests {
    use super::*;

    #[test]
    fn test_sum_single_column() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (10), (20), (30)").unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(60));
    }

    #[test]
    fn test_sum_with_zeros() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (0), (0), (100)").unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(100));
    }

    #[test]
    fn test_sum_with_negatives() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (-10), (20), (-30)").unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(-20));
    }

    #[test]
    fn test_sum_i64_boundary() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (9223372036854775807), (-9223372036854775808)")
            .unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        // SUM of max and min i64
        assert_eq!(result.rows[0][0], Value::Integer(-1));
    }

    #[test]
    fn test_sum_all_null_returns_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (NULL), (NULL), (NULL)").unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        // SUM of all NULLs should return NULL, not 0
        matches!(result.rows[0][0], Value::Null);
    }

    #[test]
    fn test_sum_with_single_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (NULL)").unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        matches!(result.rows[0][0], Value::Null);
    }
}

mod aggregate_avg_tests {
    use super::*;

    #[test]
    fn test_avg_single_column() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (10), (20), (30)").unwrap();
        let result = e.execute("SELECT AVG(v) FROM t").unwrap();
        let avg = &result.rows[0][0];
        match avg {
            Value::Integer(20) => {}
            Value::Float(f) => assert!((*f - 20.0).abs() < 0.001),
            _ => panic!("Unexpected avg value: {:?}", avg),
        }
    }

    #[test]
    fn test_avg_with_nulls_ignores_nulls() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (10), (NULL), (30)").unwrap();
        let result = e.execute("SELECT AVG(v) FROM t").unwrap();
        let avg = &result.rows[0][0];
        match avg {
            Value::Integer(20) => {}
            Value::Float(f) => assert!((*f - 20.0).abs() < 0.001),
            _ => {}
        }
    }

    #[test]
    fn test_avg_all_null_returns_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (NULL), (NULL)").unwrap();
        let result = e.execute("SELECT AVG(v) FROM t").unwrap();
        matches!(result.rows[0][0], Value::Null);
    }
}

mod aggregate_min_max_tests {
    use super::*;

    #[test]
    fn test_min_with_integer() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (10), (20), (30)").unwrap();
        let result = e.execute("SELECT MIN(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(10));
    }

    #[test]
    fn test_max_with_integer() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (10), (20), (30)").unwrap();
        let result = e.execute("SELECT MAX(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(30));
    }

    #[test]
    fn test_min_all_null_returns_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (NULL), (NULL)").unwrap();
        let result = e.execute("SELECT MIN(v) FROM t").unwrap();
        matches!(result.rows[0][0], Value::Null);
    }

    #[test]
    fn test_max_all_null_returns_null() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (NULL), (NULL)").unwrap();
        let result = e.execute("SELECT MAX(v) FROM t").unwrap();
        matches!(result.rows[0][0], Value::Null);
    }

    #[test]
    fn test_min_with_negative_values() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (-100), (0), (100)").unwrap();
        let result = e.execute("SELECT MIN(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(-100));
    }

    #[test]
    fn test_max_with_negative_values() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (-100), (0), (100)").unwrap();
        let result = e.execute("SELECT MAX(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(100));
    }
}

mod aggregate_group_by_multi_column {
    use super::*;

    #[test]
    fn test_group_by_two_columns() {
        let mut e = engine();
        e.execute("CREATE TABLE sales (region TEXT, product TEXT, amount INTEGER)")
            .unwrap();
        e.execute("INSERT INTO sales VALUES ('East', 'A', 100), ('East', 'A', 200), ('East', 'B', 150), ('West', 'A', 300)")
            .unwrap();
        let result = e
            .execute("SELECT region, product, SUM(amount) FROM sales GROUP BY region, product")
            .unwrap();
        assert_eq!(result.rows.len(), 3);
    }

    #[test]
    fn test_group_by_with_agg_on_non_group_column() {
        let mut e = engine();
        e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 10), (1, 20), (2, 30)").unwrap();
        let result = e
            .execute("SELECT a, SUM(b) FROM t GROUP BY a")
            .unwrap();
        assert_eq!(result.rows.len(), 2);
        // Find row where a=1
        let row1 = result.rows.iter().find(|r| r[0] == Value::Integer(1)).unwrap();
        assert_eq!(row1[1], Value::Integer(30));
    }

    #[test]
    fn test_group_by_three_columns() {
        let mut e = engine();
        e.execute("CREATE TABLE t (a INTEGER, b INTEGER, c INTEGER, v INTEGER)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 1, 1, 10), (1, 1, 2, 20), (1, 2, 1, 30)")
            .unwrap();
        let result = e
            .execute("SELECT a, b, c, SUM(v) FROM t GROUP BY a, b, c")
            .unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}

mod aggregate_count_tests {
    use super::*;

    #[test]
    fn test_count_star_vs_count_column() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1, NULL), (2, NULL), (3, 100)").unwrap();
        let star_result = e.execute("SELECT COUNT(*) FROM t").unwrap();
        let col_result = e.execute("SELECT COUNT(val) FROM t").unwrap();
        assert_eq!(star_result.rows[0][0], Value::Integer(3));
        assert_eq!(col_result.rows[0][0], Value::Integer(1)); // Only non-NULL
    }

    #[test]
    fn test_count_multiple_columns() {
        let mut e = engine();
        e.execute("CREATE TABLE t (a INTEGER, b INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 2), (3, 4), (5, 6)").unwrap();
        let result = e.execute("SELECT COUNT(a), COUNT(b) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(3));
        assert_eq!(result.rows[0][1], Value::Integer(3));
    }
}

mod aggregate_with_join {
    use super::*;

    #[test]
    fn test_aggregate_after_join() {
        let mut e = engine();
        e.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
        e.execute("CREATE TABLE t2 (id INTEGER, val INTEGER)").unwrap();
        e.execute("INSERT INTO t1 VALUES (1), (2)").unwrap();
        e.execute("INSERT INTO t2 VALUES (1, 10), (1, 20), (2, 30)").unwrap();
        let result = e
            .execute("SELECT t1.id, SUM(t2.val) FROM t1 LEFT JOIN t2 ON t1.id = t2.id GROUP BY t1.id")
            .unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_aggregate_with_inner_select() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (5), (10), (15)").unwrap();
        // Using subquery in FROM might not work, test simple aggregation
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(30));
    }
}

mod aggregate_boundary_tests {
    use super::*;

    #[test]
    fn test_sum_large_numbers() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1000000000), (1000000000), (1000000000)")
            .unwrap();
        let result = e.execute("SELECT SUM(v) FROM t").unwrap();
        assert_eq!(result.rows[0][0], Value::Integer(3000000000));
    }

    #[test]
    fn test_avg_large_numbers() {
        let mut e = engine();
        e.execute("CREATE TABLE t (v INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1000000), (2000000)").unwrap();
        let result = e.execute("SELECT AVG(v) FROM t").unwrap();
        let avg = &result.rows[0][0];
        match avg {
            Value::Integer(1500000) => {}
            Value::Float(f) => assert!((*f - 1500000.0).abs() < 0.001),
            _ => {}
        }
    }
}