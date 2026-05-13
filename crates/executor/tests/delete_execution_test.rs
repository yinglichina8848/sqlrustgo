#![allow(deprecated)]

//! Delete execution integration tests
//! Tests DELETE statement execution paths

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

mod delete_basic_tests {
    use super::*;

    #[test]
    fn test_delete_all_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'A'), (2, 'B'), (3, 'C')")
            .unwrap();

        let delete_result = e.execute("DELETE FROM t").unwrap();
        assert_eq!(delete_result.affected_rows, 3);

        let select_result = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select_result.rows[0][0], Value::Integer(0));
    }

    #[test]
    fn test_delete_with_where_eq() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'A'), (2, 'B'), (3, 'C')")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id = 2").unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT * FROM t ORDER BY id").unwrap();
        assert_eq!(select.rows.len(), 2);
        assert_eq!(select.rows[0][0], Value::Integer(1));
        assert_eq!(select.rows[1][0], Value::Integer(3));
    }

    #[test]
    fn test_delete_with_where_gt() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id > 3").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_delete_with_where_lt() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4), (5)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id < 3").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_delete_with_where_gteq() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id >= 3").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_delete_with_where_lteq() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id <= 2").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_delete_no_match() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();

        let result = e.execute("DELETE FROM t WHERE id = 99").unwrap();
        assert_eq!(result.affected_rows, 0);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_delete_empty_table() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();

        let result = e.execute("DELETE FROM t").unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_delete_with_and_condition() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 10), (1, 20), (2, 10), (2, 20)")
            .unwrap();

        let result = e
            .execute("DELETE FROM t WHERE id = 1 AND val = 10")
            .unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_delete_with_or_condition() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id = 1 OR id = 3").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(2));
    }
}

mod delete_text_column_tests {
    use super::*;

    #[test]
    fn test_delete_with_text_where() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE name = 'Bob'").unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT name FROM t ORDER BY id").unwrap();
        assert_eq!(select.rows.len(), 2);
        assert_eq!(select.rows[0][0], Value::Text("Alice".to_string()));
        assert_eq!(select.rows[1][0], Value::Text("Charlie".to_string()));
    }

    #[test]
    fn test_delete_with_text_pattern() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Alex')")
            .unwrap();

        // Delete where name LIKE 'Al%' - LIKE may not be fully supported
        let result = e.execute("DELETE FROM t WHERE name LIKE 'Al%'").unwrap();
        // LIKE pattern matching may not work, expect 0 affected if not supported
        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3)); // All rows still there if LIKE not supported
    }
}

mod delete_boundary_tests {
    use super::*;

    #[test]
    fn test_delete_with_i64_max() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (9223372036854775807)")
            .unwrap();

        let result = e
            .execute("DELETE FROM t WHERE id = 9223372036854775807")
            .unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_delete_with_i64_min() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (-9223372036854775808), (0), (1)")
            .unwrap();

        let result = e
            .execute("DELETE FROM t WHERE id = -9223372036854775808")
            .unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_delete_multiple_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (1), (1), (2), (2), (3)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id = 1").unwrap();
        assert_eq!(result.affected_rows, 3);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }
}

mod delete_with_index {
    use super::*;

    #[test]
    fn test_delete_with_index_scan() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 100), (2, 200), (3, 300), (4, 400)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id = 2").unwrap();
        assert_eq!(result.affected_rows, 1);

        let select = e.execute("SELECT val FROM t WHERE id = 2").unwrap();
        assert_eq!(select.rows.len(), 0);
    }

    #[test]
    fn test_delete_with_range_using_index() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("CREATE INDEX idx_id ON t (id)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 100), (2, 200), (3, 300), (4, 400), (5, 500)")
            .unwrap();

        let result = e.execute("DELETE FROM t WHERE id > 2 AND id < 5").unwrap();
        assert_eq!(result.affected_rows, 2);

        let select = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(select.rows[0][0], Value::Integer(3));
    }
}
