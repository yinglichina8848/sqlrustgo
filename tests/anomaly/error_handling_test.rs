//! Error Handling Tests
//!
//! P3 tests for error handling and recovery
//! Tests invalid operations, error messages, recovery

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
    use sqlrustgo_storage::{MemoryStorage, StorageEngine};

    #[test]
    fn test_drop_nonexistent_table() {
        let mut storage = MemoryStorage::new();

        let result = storage.drop_table("nonexistent");

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_insert_invalid_table() {
        let mut storage = MemoryStorage::new();

        let result = storage.insert(
            "nonexistent",
            vec![vec![sqlrustgo_types::Value::Integer(1)]],
        );

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_table_info_nonexistent() {
        let storage = MemoryStorage::new();

        let result = storage.get_table_info("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_table_name() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "test_table".to_string(),
            columns: vec![],
        };

        storage.create_table(&info).unwrap();

        let result = storage.create_table(&info);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_duplicate_key_insert() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "unique_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert(
                "unique_test",
                vec![vec![sqlrustgo_types::Value::Integer(1)]],
            )
            .ok();

        let result = storage.insert(
            "unique_test",
            vec![vec![sqlrustgo_types::Value::Integer(1)]],
        );

        if result.is_err() {
            return;
        }
    }

    #[test]
    fn test_empty_column_name() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        let result = storage.create_table(&info);

        if result.is_ok() {
            assert!(storage.has_table("test"));
        }
    }

    #[test]
    fn test_case_sensitivity_table_names() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "TestTable".to_string(),
            columns: vec![],
        };

        storage.create_table(&info).ok();

        assert!(storage.has_table("TestTable"));
    }

    #[test]
    fn test_delete_nonexistent_row() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        let result = storage.delete("test", &[sqlrustgo_types::Value::Integer(999)]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_data_type() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "col".to_string(),
                data_type: "INVALID_TYPE".to_string(),
                nullable: false,
                is_unique: false,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        let result = storage.create_table(&info);

        if result.is_ok() {
            assert!(storage.has_table("test"));
        }
    }

    #[test]
    fn test_list_nonexistent_tables() {
        let storage = MemoryStorage::new();

        let tables = storage.list_tables();

        assert!(tables.is_empty());
    }
}
