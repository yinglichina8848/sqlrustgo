//! Boundary Condition Tests
//!
//! P3 tests for edge cases and boundary conditions
//! Tests null handling, empty tables, large values, etc.

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
    use sqlrustgo_storage::{MemoryStorage, StorageEngine};
    use sqlrustgo_types::Value;

    #[test]
    fn test_empty_table_scan() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "empty_table".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        let result = storage.scan("empty_table").unwrap();

        assert!(result.is_empty(), "Empty table should return empty vector");
    }

    #[test]
    fn test_null_value_handling() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "nullable_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "col".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert("nullable_test", vec![vec![Value::Null]])
            .ok();

        let result = storage.scan("nullable_test").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], Value::Null);
    }

    #[test]
    fn test_large_integer_values() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "large_values".to_string(),
            columns: vec![ColumnDefinition {
                name: "val".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert("large_values", vec![vec![Value::Integer(i64::MAX)]])
            .ok();
        storage
            .insert("large_values", vec![vec![Value::Integer(i64::MIN)]])
            .ok();

        let result = storage.scan("large_values").unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_large_text_values() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "large_text".to_string(),
            columns: vec![ColumnDefinition {
                name: "text".to_string(),
                data_type: "VARCHAR".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        let large_text = "x".repeat(10000);
        storage
            .insert("large_text", vec![vec![Value::Text(large_text)]])
            .ok();

        let result = storage.scan("large_text").unwrap();

        assert!(!result.is_empty());
    }

    #[test]
    fn test_zero_value() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "zero_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "val".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert("zero_test", vec![vec![Value::Integer(0)]])
            .ok();

        let result = storage.scan("zero_test").unwrap();

        assert_eq!(result[0][0], Value::Integer(0));
    }

    #[test]
    fn test_negative_values() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "negative_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "val".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        for i in -10..=-1 {
            storage
                .insert("negative_test", vec![vec![Value::Integer(i)]])
                .ok();
        }

        let result = storage.scan("negative_test").unwrap();

        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_float_precision() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "float_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "val".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        let pi = std::f64::consts::PI;
        storage
            .insert("float_test", vec![vec![Value::Float(pi)]])
            .ok();

        let result = storage.scan("float_test").unwrap();

        if let Value::Float(f) = &result[0][0] {
            assert!((f - pi).abs() < 0.0001);
        }
    }

    #[test]
    fn test_many_columns() {
        let mut storage = MemoryStorage::new();

        let columns: Vec<ColumnDefinition> = (0..50)
            .map(|i| ColumnDefinition {
                name: format!("col{}", i),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            })
            .collect();

        let info = TableInfo {
            name: "many_cols".to_string(),
            columns,
        };

        storage.create_table(&info).unwrap();

        let values: Vec<Value> = (0..50).map(|i| Value::Integer(i as i64)).collect();
        storage.insert("many_cols", vec![values]).ok();

        let result = storage.scan("many_cols").unwrap();

        assert_eq!(result[0].len(), 50);
    }

    #[test]
    fn test_unicode_text() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "unicode_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "text".to_string(),
                data_type: "VARCHAR".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert(
                "unicode_test",
                vec![vec![Value::Text("你好世界🌍".to_string())]],
            )
            .ok();
        storage
            .insert(
                "unicode_test",
                vec![vec![Value::Text("Hello 🌍".to_string())]],
            )
            .ok();

        let result = storage.scan("unicode_test").unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_special_characters() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "special_chars".to_string(),
            columns: vec![ColumnDefinition {
                name: "text".to_string(),
                data_type: "VARCHAR".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert(
                "special_chars",
                vec![vec![Value::Text("quote\"test".to_string())]],
            )
            .ok();
        storage
            .insert(
                "special_chars",
                vec![vec![Value::Text("backslash\\test".to_string())]],
            )
            .ok();
        storage
            .insert(
                "special_chars",
                vec![vec![Value::Text("newline\ntest".to_string())]],
            )
            .ok();

        let result = storage.scan("special_chars").unwrap();

        assert_eq!(result.len(), 3);
    }
}
