//! Catalog Consistency Verification Tests
//!
//! P0 tests for catalog consistency per ISSUE #846
//! Verifies that system catalogs remain consistent across operations

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{
        ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint, StorageEngine, TableInfo,
    };
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;
    use std::collections::HashSet;

    fn create_users_table() -> TableInfo {
        TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                ColumnDefinition {
                    name: "email".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        }
    }

    fn create_orders_table() -> TableInfo {
        TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                ColumnDefinition {
                    name: "user_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "users".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(ForeignKeyAction::Cascade),
                        on_update: Some(ForeignKeyAction::Cascade),
                    }),
                },
                ColumnDefinition {
                    name: "amount".to_string(),
                    data_type: "REAL".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                },
            ],
        }
    }

    #[test]
    fn test_table_metadata_consistency_after_create() {
        let mut storage = MemoryStorage::new();
        let info = create_users_table();

        storage.create_table(&info).unwrap();

        let retrieved = storage.get_table_info("users").unwrap();

        assert_eq!(retrieved.name, info.name);
        assert_eq!(retrieved.columns.len(), info.columns.len());

        for (col, retrieved_col) in info.columns.iter().zip(retrieved.columns.iter()) {
            assert_eq!(col.name, retrieved_col.name);
            assert_eq!(col.data_type, retrieved_col.data_type);
            assert_eq!(col.nullable, retrieved_col.nullable);
            assert_eq!(col.is_unique, retrieved_col.is_unique);
        }
    }

    #[test]
    fn test_table_list_consistency() {
        let mut storage = MemoryStorage::new();

        let tables = vec![
            create_users_table(),
            create_orders_table(),
            TableInfo {
                name: "products".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: true,
                        references: None,
                    },
                    ColumnDefinition {
                        name: "name".to_string(),
                        data_type: "VARCHAR".to_string(),
                        nullable: false,
                        is_unique: false,
                        references: None,
                    },
                ],
            },
        ];

        for table in &tables {
            storage.create_table(table).unwrap();
        }

        let listed = storage.list_tables();
        assert_eq!(listed.len(), 3);

        let table_set: HashSet<_> = listed.into_iter().collect();
        assert!(table_set.contains("users"));
        assert!(table_set.contains("orders"));
        assert!(table_set.contains("products"));
    }

    #[test]
    fn test_foreign_key_consistency() {
        let mut storage = MemoryStorage::new();

        storage.create_table(&create_users_table()).unwrap();
        storage.create_table(&create_orders_table()).unwrap();

        let orders_info = storage.get_table_info("orders").unwrap();

        let user_id_col = orders_info
            .columns
            .iter()
            .find(|c| c.name == "user_id")
            .expect("user_id column should exist");

        let fk = user_id_col
            .references
            .as_ref()
            .expect("user_id should have foreign key");

        assert_eq!(fk.referenced_table, "users");
        assert_eq!(fk.referenced_column, "id");

        let users_exist = storage.has_table(&fk.referenced_table);
        assert!(users_exist, "Referenced table should exist");
    }

    #[test]
    fn test_foreign_key_referenced_column_exists() {
        let mut storage = MemoryStorage::new();

        storage.create_table(&create_users_table()).unwrap();
        storage.create_table(&create_orders_table()).unwrap();

        let users_info = storage.get_table_info("users").unwrap();
        let user_column_names: HashSet<_> =
            users_info.columns.iter().map(|c| c.name.clone()).collect();

        let orders_info = storage.get_table_info("orders").unwrap();
        for col in &orders_info.columns {
            if let Some(fk) = &col.references {
                assert!(
                    user_column_names.contains(&fk.referenced_column),
                    "Foreign key references non-existent column: {}",
                    fk.referenced_column
                );
            }
        }
    }

    #[test]
    fn test_unique_constraint_consistency() {
        let mut storage = MemoryStorage::new();
        let info = create_users_table();

        storage.create_table(&info).unwrap();

        let retrieved = storage.get_table_info("users").unwrap();

        let unique_columns: Vec<_> = retrieved
            .columns
            .iter()
            .filter(|c| c.is_unique)
            .map(|c| c.name.clone())
            .collect();

        assert!(unique_columns.contains(&"id".to_string()));
        assert!(unique_columns.contains(&"email".to_string()));

        let non_unique: Vec<_> = retrieved
            .columns
            .iter()
            .filter(|c| !c.is_unique)
            .map(|c| c.name.clone())
            .collect();
        assert!(non_unique.contains(&"name".to_string()));
    }

    #[test]
    fn test_nullable_constraint_consistency() {
        let mut storage = MemoryStorage::new();
        let info = create_users_table();

        storage.create_table(&info).unwrap();

        let retrieved = storage.get_table_info("users").unwrap();

        let non_nullable: Vec<_> = retrieved
            .columns
            .iter()
            .filter(|c| !c.nullable)
            .map(|c| c.name.clone())
            .collect();

        assert!(non_nullable.contains(&"id".to_string()));
        assert!(non_nullable.contains(&"email".to_string()));

        let nullable: Vec<_> = retrieved
            .columns
            .iter()
            .filter(|c| c.nullable)
            .map(|c| c.name.clone())
            .collect();

        assert!(nullable.contains(&"name".to_string()));
    }

    #[test]
    fn test_drop_table_consistency() {
        let mut storage = MemoryStorage::new();

        storage.create_table(&create_users_table()).unwrap();
        storage.create_table(&create_orders_table()).unwrap();

        storage.drop_table("users").unwrap();

        assert!(!storage.has_table("users"));
        assert!(storage.has_table("orders"));

        let tables = storage.list_tables();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0], "orders");
    }

    #[test]
    fn test_table_count_consistency() {
        let mut storage = MemoryStorage::new();

        let initial_count = storage.list_tables().len();
        assert_eq!(initial_count, 0);

        storage.create_table(&create_users_table()).unwrap();
        assert_eq!(storage.list_tables().len(), 1);

        storage.create_table(&create_orders_table()).unwrap();
        assert_eq!(storage.list_tables().len(), 2);

        storage.drop_table("users").unwrap();
        assert_eq!(storage.list_tables().len(), 1);

        storage.drop_table("orders").unwrap();
        assert_eq!(storage.list_tables().len(), 0);
    }

    #[test]
    fn test_column_data_type_consistency() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "test_types".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "int_col".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "varchar_col".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "real_col".to_string(),
                    data_type: "REAL".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).unwrap();

        let retrieved = storage.get_table_info("test_types").unwrap();

        assert_eq!(retrieved.columns[0].data_type, "INTEGER");
        assert_eq!(retrieved.columns[1].data_type, "VARCHAR");
        assert_eq!(retrieved.columns[2].data_type, "REAL");
    }

    #[test]
    fn test_analyze_table_stats_consistency() {
        let mut storage = MemoryStorage::new();

        storage.create_table(&create_users_table()).unwrap();

        for i in 1..=10 {
            storage
                .insert(
                    "users",
                    vec![vec![
                        Value::Integer(i),
                        Value::Text(format!("user{}@example.com", i)),
                        Value::Text(format!("User {}", i)),
                    ]],
                )
                .unwrap();
        }

        let stats = storage.analyze_table("users").unwrap();

        assert_eq!(stats.table_name, "users");
        assert_eq!(stats.row_count, 10);
        assert!(!stats.column_stats.is_empty());
    }

    #[test]
    fn test_table_info_persistence_after_operations() {
        let mut storage = MemoryStorage::new();

        storage.create_table(&create_users_table()).unwrap();

        let original_info = storage.get_table_info("users").unwrap();

        storage
            .insert(
                "users",
                vec![vec![
                    Value::Integer(1),
                    Value::Text("test@example.com".to_string()),
                    Value::Text("Test User".to_string()),
                ]],
            )
            .unwrap();

        let after_insert_info = storage.get_table_info("users").unwrap();

        assert_eq!(original_info.name, after_insert_info.name);
        assert_eq!(original_info.columns.len(), after_insert_info.columns.len());

        for (orig, after) in original_info
            .columns
            .iter()
            .zip(after_insert_info.columns.iter())
        {
            assert_eq!(orig.name, after.name);
            assert_eq!(orig.data_type, after.data_type);
        }
    }

    #[test]
    fn test_case_sensitive_table_names() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "Users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            }],
        };

        storage.create_table(&info).unwrap();

        assert!(storage.has_table("Users"));
        assert!(!storage.has_table("users"));
        assert!(!storage.has_table("USERS"));
    }

    #[test]
    fn test_multiple_tables_column_isolation() {
        let mut storage = MemoryStorage::new();

        let table1 = TableInfo {
            name: "table1".to_string(),
            columns: vec![ColumnDefinition {
                name: "col_a".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                references: None,
            }],
        };

        let table2 = TableInfo {
            name: "table2".to_string(),
            columns: vec![ColumnDefinition {
                name: "col_b".to_string(),
                data_type: "VARCHAR".to_string(),
                nullable: true,
                is_unique: false,
                references: None,
            }],
        };

        storage.create_table(&table1).unwrap();
        storage.create_table(&table2).unwrap();

        let info1 = storage.get_table_info("table1").unwrap();
        let info2 = storage.get_table_info("table2").unwrap();

        assert_eq!(info1.columns[0].name, "col_a");
        assert_eq!(info2.columns[0].name, "col_b");

        assert_ne!(info1.columns[0].data_type, info2.columns[0].data_type);
    }
}
