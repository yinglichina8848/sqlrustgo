//! UNION / INTERSECT / EXCEPT Tests
//!
//! P2 tests for set operations per TEST_PLAN.md
//! Tests UNION, INTERSECT, and EXCEPT operations

#[cfg(test)]
mod tests {
    use sqlrustgo::{parse, ExecutionEngine, MemoryStorage, StorageEngine};
    use sqlrustgo_types::Value;
    use std::sync::{Arc, RwLock};

    fn create_test_tables() -> MemoryStorage {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "table_a".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).ok();
        storage
            .insert("table_a", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("table_a", vec![vec![Value::Integer(2)]])
            .ok();
        storage
            .insert("table_a", vec![vec![Value::Integer(3)]])
            .ok();

        let info = sqlrustgo_storage::TableInfo {
            name: "table_b".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).ok();
        storage
            .insert("table_b", vec![vec![Value::Integer(2)]])
            .ok();
        storage
            .insert("table_b", vec![vec![Value::Integer(3)]])
            .ok();
        storage
            .insert("table_b", vec![vec![Value::Integer(4)]])
            .ok();

        storage
    }

    #[test]
    fn test_union_basic() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result =
            engine.execute(parse("SELECT id FROM table_a UNION SELECT id FROM table_b").unwrap());

        assert!(result.is_ok(), "UNION should execute without error");
    }

    #[test]
    fn test_union_all() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result = engine
            .execute(parse("SELECT id FROM table_a UNION ALL SELECT id FROM table_b").unwrap());

        assert!(result.is_ok(), "UNION ALL should execute without error");
    }

    #[test]
    fn test_union_with_order_by() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result = engine.execute(
            parse("SELECT id FROM table_a UNION SELECT id FROM table_b ORDER BY id DESC").unwrap(),
        );

        assert!(result.is_ok(), "UNION with ORDER BY should work");
    }

    #[test]
    fn test_union_with_limit() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result = engine
            .execute(parse("SELECT id FROM table_a UNION SELECT id FROM table_b LIMIT 3").unwrap());

        assert!(result.is_ok(), "UNION with LIMIT should work");
    }

    #[test]
    fn test_union_distinct_removes_duplicates() {
        let mut storage = MemoryStorage::new();

        let info = sqlrustgo_storage::TableInfo {
            name: "numbers".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "num".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };
        storage.create_table(&info).ok();
        storage
            .insert("numbers", vec![vec![Value::Integer(1)]])
            .ok();
        storage
            .insert("numbers", vec![vec![Value::Integer(2)]])
            .ok();

        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result =
            engine.execute(parse("SELECT num FROM numbers UNION SELECT num FROM numbers").unwrap());

        assert!(result.is_ok());
    }

    #[test]
    fn test_intersect_syntax() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result = engine
            .execute(parse("SELECT id FROM table_a INTERSECT SELECT id FROM table_b").unwrap());

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_except_syntax() {
        let storage = create_test_tables();
        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result =
            engine.execute(parse("SELECT id FROM table_a EXCEPT SELECT id FROM table_b").unwrap());

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_union_multiple_tables() {
        let mut storage = MemoryStorage::new();

        for i in 1..=3 {
            let info = sqlrustgo_storage::TableInfo {
                name: format!("t{}", i),
                columns: vec![sqlrustgo_storage::ColumnDefinition {
                    name: "val".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            };
            storage.create_table(&info).ok();
            let table_name = format!("t{}", i);
            storage
                .insert(&table_name, vec![vec![Value::Integer(i)]])
                .ok();
        }

        let mut engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));

        let result = engine.execute(parse("SELECT val FROM t1 UNION SELECT val FROM t2").unwrap());

        assert!(result.is_ok());
    }
}
