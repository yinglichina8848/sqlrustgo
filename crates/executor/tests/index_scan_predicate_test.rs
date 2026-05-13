#![allow(deprecated)]

//! IndexScan predicate integration tests
//! Tests all predicate types: Eq (=), Gt (>), Lt (<), GtEq (>=), LtEq (<=)

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

mod index_scan_eq_tests {
    use super::*;

    #[test]
    fn test_index_scan_eq_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
            .unwrap();

        // SELECT * FROM t WHERE id = 2
        let result = e.execute("SELECT * FROM t WHERE id = 2").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(2));
        assert_eq!(result.rows[0][1], Value::Text("Bob".to_string()));
    }

    #[test]
    fn test_index_scan_eq_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
            .unwrap();

        // SELECT * FROM t WHERE id = 99 (no match)
        let result = e.execute("SELECT * FROM t WHERE id = 99").unwrap();
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_index_scan_eq_with_text_column() {
        let mut e = engine();
        e.execute("CREATE TABLE users (id INTEGER, email TEXT)").unwrap();
        e.execute("CREATE INDEX idx_email ON users (id)").unwrap();
        e.execute("INSERT INTO users VALUES (1, 'alice@test.com'), (2, 'bob@test.com')")
            .unwrap();

        let result = e.execute("SELECT email FROM users WHERE id = 1").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Text("alice@test.com".to_string()));
    }
}

mod index_scan_gt_tests {
    use super::*;

    #[test]
    fn test_index_scan_gt_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie'), (4, 'David')")
            .unwrap();

        // SELECT * FROM t WHERE id > 2
        let result = e.execute("SELECT * FROM t WHERE id > 2").unwrap();
        assert_eq!(result.rows.len(), 2);
        // Should return rows with id = 3 and id = 4
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&3));
        assert!(ids.contains(&4));
    }

    #[test]
    fn test_index_scan_gt_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2)").unwrap();

        // SELECT * FROM t WHERE id > 10 (no match)
        let result = e.execute("SELECT * FROM t WHERE id > 10").unwrap();
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_index_scan_gt_all_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();

        // SELECT * FROM t WHERE id > 0 (all match)
        let result = e.execute("SELECT * FROM t WHERE id > 0").unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}

mod index_scan_lt_tests {
    use super::*;

    #[test]
    fn test_index_scan_lt_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie'), (4, 'David')")
            .unwrap();

        // SELECT * FROM t WHERE id < 3
        let result = e.execute("SELECT * FROM t WHERE id < 3").unwrap();
        assert_eq!(result.rows.len(), 2);
        // Should return rows with id = 1 and id = 2
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_index_scan_lt_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (5), (6), (7)").unwrap();

        // SELECT * FROM t WHERE id < 1 (no match)
        let result = e.execute("SELECT * FROM t WHERE id < 1").unwrap();
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_index_scan_lt_all_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();

        // SELECT * FROM t WHERE id < 100 (all match)
        let result = e.execute("SELECT * FROM t WHERE id < 100").unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}

mod index_scan_gteq_tests {
    use super::*;

    #[test]
    fn test_index_scan_gteq_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie'), (4, 'David')")
            .unwrap();

        // SELECT * FROM t WHERE id >= 3
        let result = e.execute("SELECT * FROM t WHERE id >= 3").unwrap();
        assert_eq!(result.rows.len(), 2);
        // Should return rows with id = 3 and id = 4
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&3));
        assert!(ids.contains(&4));
    }

    #[test]
    fn test_index_scan_gteq_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2)").unwrap();

        // SELECT * FROM t WHERE id >= 100 (no match)
        let result = e.execute("SELECT * FROM t WHERE id >= 100").unwrap();
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_index_scan_gteq_all_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();

        // SELECT * FROM t WHERE id >= 1 (all match)
        let result = e.execute("SELECT * FROM t WHERE id >= 1").unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}

mod index_scan_lteq_tests {
    use super::*;

    #[test]
    fn test_index_scan_lteq_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie'), (4, 'David')")
            .unwrap();

        // SELECT * FROM t WHERE id <= 2
        let result = e.execute("SELECT * FROM t WHERE id <= 2").unwrap();
        assert_eq!(result.rows.len(), 2);
        // Should return rows with id = 1 and id = 2
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn test_index_scan_lteq_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (5), (6), (7)").unwrap();

        // SELECT * FROM t WHERE id <= 0 (no match)
        let result = e.execute("SELECT * FROM t WHERE id <= 0").unwrap();
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_index_scan_lteq_all_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();

        // SELECT * FROM t WHERE id <= 100 (all match)
        let result = e.execute("SELECT * FROM t WHERE id <= 100").unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}

mod index_scan_boundary_tests {
    use super::*;

    #[test]
    fn test_index_scan_eq_boundary_i64_max() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (9223372036854775807), (-9223372036854775808)")
            .unwrap();

        let result = e.execute("SELECT * FROM t WHERE id = 9223372036854775807").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(9223372036854775807));
    }

    #[test]
    fn test_index_scan_eq_boundary_i64_min() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (9223372036854775807), (-9223372036854775808)")
            .unwrap();

        let result = e.execute("SELECT * FROM t WHERE id = -9223372036854775808").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(-9223372036854775808));
    }

    #[test]
    fn test_index_scan_gt_negative_values() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (-10), (-5), (0), (5), (10)").unwrap();

        let result = e.execute("SELECT * FROM t WHERE id > -5").unwrap();
        assert_eq!(result.rows.len(), 3);
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&0));
        assert!(ids.contains(&5));
        assert!(ids.contains(&10));
    }

    #[test]
    fn test_index_scan_lt_negative_values() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (-10), (-5), (0), (5), (10)").unwrap();

        let result = e.execute("SELECT * FROM t WHERE id < 0").unwrap();
        assert_eq!(result.rows.len(), 2);
        let ids: Vec<i64> = result.rows.iter().map(|r| r[0].as_integer().unwrap()).collect();
        assert!(ids.contains(&-10));
        assert!(ids.contains(&-5));
    }
}

mod index_scan_with_projection {
    use super::*;

    #[test]
    fn test_index_scan_with_projection() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER, name TEXT)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 100, 'Alice'), (2, 200, 'Bob'), (3, 300, 'Charlie')")
            .unwrap();

        // SELECT val, name FROM t WHERE id = 2
        let result = e.execute("SELECT val, name FROM t WHERE id = 2").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(200));
        assert_eq!(result.rows[0][1], Value::Text("Bob".to_string()));
    }

    #[test]
    fn test_index_scan_with_aggregate() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)").unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 100), (2, 200), (3, 300), (4, 400)")
            .unwrap();

        // SELECT COUNT(*), SUM(val) FROM t WHERE id > 2
        let result = e.execute("SELECT COUNT(*), SUM(val) FROM t WHERE id > 2").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(2)); // COUNT: 2 rows (id=3,4)
        assert_eq!(result.rows[0][1], Value::Integer(700)); // SUM: 300+400
    }
}

mod index_scan_multiple_rows_same_key {
    use super::*;

    #[test]
    fn test_index_scan_eq_multiple_rows_same_key() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, category TEXT)").unwrap();
        e.execute("CREATE INDEX idx_category ON t (id)").unwrap();
        // Multiple rows can have the same indexed value
        e.execute("INSERT INTO t VALUES (1, 'A'), (1, 'B'), (2, 'C'), (1, 'D')")
            .unwrap();

        let result = e.execute("SELECT * FROM t WHERE id = 1").unwrap();
        assert_eq!(result.rows.len(), 3);
    }
}
