#![allow(deprecated)]

//! DDL execution tests
//! Tests CREATE INDEX, DROP TABLE basics

use sqlrustgo::ExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> ExecutionEngine<MemoryStorage> {
    ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())))
}

mod create_index_tests {
    use super::*;

    #[test]
    fn test_create_index_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice'), (2, 'Bob')").unwrap();

        let result = e.execute("CREATE INDEX idx_name ON t (name)").unwrap();
        assert_eq!(result.affected_rows >= 0, true);

        // Index should be used for queries
        let select = e.execute("SELECT * FROM t WHERE name = 'Bob'").unwrap();
        assert_eq!(select.rows.len(), 1);
    }

    #[test]
    fn test_create_unique_index() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, email TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'a@test.com'), (2, 'b@test.com')").unwrap();

        let result = e.execute("CREATE UNIQUE INDEX idx_email ON t (email)").unwrap();
        assert_eq!(result.affected_rows >= 0, true);
    }
}

mod drop_table_tests {
    use super::*;

    #[test]
    fn test_drop_table_basic() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (1)").unwrap();

        let result = e.execute("DROP TABLE t").unwrap();
        assert_eq!(result.affected_rows >= 0, true);

        let select = e.execute("SELECT * FROM t");
        assert!(select.is_err());
    }

    #[test]
    fn test_drop_table_if_exists() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();

        let result = e.execute("DROP TABLE IF EXISTS t").unwrap();
        assert_eq!(result.affected_rows >= 0, true);

        // Should not error on second drop
        let result2 = e.execute("DROP TABLE IF EXISTS t").unwrap();
        assert_eq!(result2.affected_rows >= 0, true);
    }
}