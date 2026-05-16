use sqlrustgo::execution_engine::EngineConfig;
use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use sqlrustgo_types::Value;
use std::sync::{Arc, RwLock};

fn engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
}

mod non_recursive_cte_tests {
    use super::*;

    #[test]
    fn test_simple_cte() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, name TEXT)").unwrap();
        e.execute("INSERT INTO t VALUES (1, 'Alice')").unwrap();
        e.execute("INSERT INTO t VALUES (2, 'Bob')").unwrap();

        let result = e
            .execute("WITH cte AS (SELECT * FROM t) SELECT * FROM cte")
            .unwrap();
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_cte_with_aggregation() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER, val INTEGER)")
            .unwrap();
        e.execute("INSERT INTO t VALUES (1, 10)").unwrap();
        e.execute("INSERT INTO t VALUES (2, 20)").unwrap();

        let result = e
            .execute("WITH cte AS (SELECT SUM(val) AS total FROM t) SELECT * FROM cte")
            .unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(30));
    }
}

mod recursive_cte_tests {
    use super::*;

    #[test]
    fn test_recursive_cte_basic() {
        let mut e = engine();
        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM cte WHERE n < 5) SELECT * FROM cte"
        );
        assert!(result.is_ok(), "Recursive CTE should execute: {:?}", result);
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0], Value::Integer(1));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[2][0], Value::Integer(3));
        assert_eq!(rows[3][0], Value::Integer(4));
        assert_eq!(rows[4][0], Value::Integer(5));
    }

    #[test]
    fn test_recursive_cte_countdown() {
        let mut e = engine();
        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT 5 AS n UNION ALL SELECT n - 1 FROM cte WHERE n > 1) SELECT * FROM cte"
        );
        assert!(
            result.is_ok(),
            "Recursive countdown should execute: {:?}",
            result
        );
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0], Value::Integer(5));
        assert_eq!(rows[4][0], Value::Integer(1));
    }

    #[test]
    fn test_recursive_cte_with_table_data() {
        let mut e = engine();
        e.execute("CREATE TABLE numbers (n INTEGER)").unwrap();
        e.execute("INSERT INTO numbers VALUES (10)").unwrap();

        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT n FROM numbers UNION ALL SELECT n + 10 FROM cte WHERE n < 50) SELECT * FROM cte"
        );
        assert!(
            result.is_ok(),
            "Recursive CTE with table should execute: {:?}",
            result
        );
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0], Value::Integer(10));
        assert_eq!(rows[4][0], Value::Integer(50));
    }
}

mod multiple_cte_tests {
    use super::*;

    #[test]
    fn test_multiple_named_ctes() {
        let mut e = engine();
        e.execute("CREATE TABLE a (id INTEGER)").unwrap();
        e.execute("CREATE TABLE b (id INTEGER)").unwrap();
        e.execute("INSERT INTO a VALUES (1), (2)").unwrap();
        e.execute("INSERT INTO b VALUES (3), (4)").unwrap();

        let result = e.execute(
            "WITH cte_a AS (SELECT * FROM a), cte_b AS (SELECT * FROM b) SELECT * FROM cte_a UNION ALL SELECT * FROM cte_b"
        );
        assert!(result.is_ok(), "Multiple CTEs should execute: {:?}", result);
        assert_eq!(result.unwrap().rows.len(), 4);
    }

    #[test]
    fn test_recursive_and_non_recursive_cte() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();
        e.execute("INSERT INTO t VALUES (100)").unwrap();

        let result = e.execute(
            "WITH static_cte AS (SELECT 1 AS n), recursive_cte AS (SELECT n FROM static_cte UNION ALL SELECT n + 1 FROM recursive_cte WHERE n < 3) SELECT * FROM recursive_cte"
        );
        assert!(result.is_ok(), "Mixed CTEs should execute: {:?}", result);
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 3);
    }
}

mod nested_cte_tests {
    use super::*;

    #[test]
    fn test_nested_cte_inner_first() {
        let mut e = engine();
        let result = e.execute(
            "WITH outer_cte AS (WITH inner_cte AS (SELECT 42 AS val) SELECT * FROM inner_cte) SELECT * FROM outer_cte"
        );
        assert!(result.is_ok(), "Nested CTE should execute: {:?}", result);
        assert_eq!(result.unwrap().rows[0][0], Value::Integer(42));
    }
}

mod cte_edge_cases {
    use super::*;

    #[test]
    fn test_cte_empty_result() {
        let mut e = engine();
        e.execute("CREATE TABLE t (id INTEGER)").unwrap();

        let result = e.execute("WITH cte AS (SELECT * FROM t WHERE 1=0) SELECT * FROM cte");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows.len(), 0);
    }

    #[test]
    fn test_recursive_cte_single_row() {
        let mut e = engine();
        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM cte WHERE n < 1) SELECT * FROM cte"
        );
        assert!(result.is_ok());
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_recursive_cte_termination() {
        let mut e = engine();
        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM cte WHERE n < 0) SELECT * FROM cte"
        );
        assert!(
            result.is_ok(),
            "Should handle termination condition not met: {:?}",
            result
        );
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0], Value::Integer(1));
    }
}
