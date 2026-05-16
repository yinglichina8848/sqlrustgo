#[cfg(test)]
mod tests {
    use sqlrustgo::execution_engine::EngineConfig;
    use sqlrustgo::MemoryExecutionEngine;
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;
    use std::sync::{Arc, RwLock};

    fn engine() -> MemoryExecutionEngine {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        MemoryExecutionEngine::new_with_config(storage, EngineConfig::default())
    }

    #[test]
    fn test_recursive_cte_basic() {
        let mut e = engine();
        let result = e.execute(
            "WITH RECURSIVE cte AS (SELECT 1 AS n UNION ALL SELECT n + 1 FROM cte WHERE n < 10) SELECT * FROM cte"
        );
        assert!(result.is_ok(), "Recursive CTE should execute: {:?}", result);
        let rows = result.unwrap().rows;
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0][0], Value::Integer(1));
        assert_eq!(rows[9][0], Value::Integer(10));
    }

    #[test]
    fn test_recursive_cte_with_table_reference() {
        let mut e = engine();
        e.execute("CREATE TABLE org(id INTEGER, name TEXT, manager_id INTEGER)").unwrap();
        e.execute("INSERT INTO org VALUES (1, 'CEO', NULL)").unwrap();
        e.execute("INSERT INTO org VALUES (2, 'VP', 1)").unwrap();
        e.execute("INSERT INTO org VALUES (3, 'Director', 2)").unwrap();

        let result = e.execute(
            "WITH RECURSIVE subordinates AS (SELECT id, name, manager_id FROM org WHERE manager_id = 1 UNION ALL SELECT o.id, o.name, o.manager_id FROM org o JOIN subordinates s ON o.manager_id = s.id) SELECT COUNT(*) FROM subordinates"
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows[0][0], Value::Integer(2));
    }
}
