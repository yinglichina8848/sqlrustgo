#![allow(deprecated)]

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

mod insert_tests {
    use super::*;

    #[test]
    fn test_insert_single_row() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice')").unwrap();
        let r = e.execute("SELECT * FROM t").unwrap();
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.rows[0][0], Value::Integer(1));
        assert_eq!(r.rows[0][1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_insert_multiple_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(3));
    }

    #[test]
    fn test_insert_select_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE source (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("CREATE TABLE target (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("INSERT INTO source VALUES (1, 10), (2, 20)")
            .unwrap();
        e.execute("INSERT INTO target SELECT * FROM source")
            .unwrap();
        let r = e.execute("SELECT COUNT(*) FROM target").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_insert_into_existing_table() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1)").unwrap();
        e.execute("INSERT INTO t VALUES (2)").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(2));
    }
}

mod update_tests {
    use super::*;

    #[test]
    fn test_update_single_row() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice')").unwrap();
        e.execute("UPDATE t SET name = 'Bob' WHERE id = 1").unwrap();
        let r = e.execute("SELECT name FROM t WHERE id = 1").unwrap();
        assert_eq!(r.rows[0][0], Value::Text("Bob".to_string()));
    }
}

mod delete_tests {
    use super::*;

    #[test]
    fn test_delete_single_row() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob')")
            .unwrap();
        e.execute("DELETE FROM t WHERE id = 1").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_delete_multiple_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3), (4)")
            .unwrap();
        e.execute("DELETE FROM t WHERE id > 2").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_delete_all_rows() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2)").unwrap();
        e.execute("DELETE FROM t").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(0));
    }

    #[test]
    fn test_delete_with_and_condition() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 10), (2, 20), (3, 30)")
            .unwrap();
        e.execute("DELETE FROM t WHERE id > 1 AND val < 30")
            .unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_delete_with_or_condition() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
        e.execute("DELETE FROM t WHERE id = 1 OR id = 3").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(1));
    }
}

mod replace_tests {
    use super::*;

    #[test]
    fn test_replace_into() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice')").unwrap();
        e.execute("REPLACE INTO t VALUES (1, 'Bob')").unwrap();
        let r = e.execute("SELECT name FROM t WHERE id = 1").unwrap();
        assert_eq!(r.rows[0][0], Value::Text("Bob".to_string()));
    }

    #[test]
    fn test_replace_increments_count() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, val INTEGER)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 10)").unwrap();
        e.execute("REPLACE INTO t VALUES (2, 20)").unwrap();
        let r = e.execute("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(r.rows[0][0], Value::Integer(2));
    }
}
