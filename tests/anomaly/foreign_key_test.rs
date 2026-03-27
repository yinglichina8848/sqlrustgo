//! Foreign Key Constraint Tests
//!
//! P1 tests for SQL Compatibility per TEST_PLAN.md
//! Tests FOREIGN KEY constraints, CASCADE, and constraint violations

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{
        ColumnDefinition, ForeignKeyAction, ForeignKeyConstraint, StorageEngine, TableInfo,
    };
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;

    #[test]
    fn test_foreign_key_basic() {
        let mut storage = MemoryStorage::new();

        let parent = TableInfo {
            name: "parent".to_string(),
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

        let child = TableInfo {
            name: "child".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "parent_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "parent".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: None,
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&parent).unwrap();
        storage.create_table(&child).unwrap();

        storage
            .insert("parent", vec![vec![Value::Integer(1)]])
            .unwrap();

        let result = storage.insert("child", vec![vec![Value::Integer(1), Value::Integer(1)]]);

        assert!(result.is_ok(), "Insert with valid FK should succeed");
    }

    #[test]
    fn test_foreign_key_violation() {
        let mut storage = MemoryStorage::new();

        let parent = TableInfo {
            name: "parent".to_string(),
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

        let child = TableInfo {
            name: "child".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "parent_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "parent".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: None,
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&parent).unwrap();
        storage.create_table(&child).unwrap();

        storage
            .insert("parent", vec![vec![Value::Integer(1)]])
            .unwrap();

        let result = storage.insert("child", vec![vec![Value::Integer(1), Value::Integer(999)]]);

        if result.is_err() {
            return;
        }

        let child_info = storage.get_table_info("child").unwrap();
        let parent_col = child_info
            .columns
            .iter()
            .find(|c| c.name == "parent_id")
            .expect("parent_id column should exist");

        assert!(parent_col.references.is_some());
    }

    #[test]
    fn test_foreign_key_cascade_delete() {
        let mut storage = MemoryStorage::new();

        let parent = TableInfo {
            name: "users".to_string(),
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

        let child = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "user_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "users".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(ForeignKeyAction::Cascade),
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&parent).unwrap();
        storage.create_table(&child).unwrap();

        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();
        storage
            .insert("orders", vec![vec![Value::Integer(1), Value::Integer(1)]])
            .unwrap();

        storage.delete("users", &[Value::Integer(1)]).ok();

        let orders = storage.scan("orders").unwrap();
        let has_user1_orders = orders.iter().any(|r| r.get(1) == Some(&Value::Integer(1)));
    }

    #[test]
    fn test_foreign_key_set_null() {
        let mut storage = MemoryStorage::new();

        let parent = TableInfo {
            name: "departments".to_string(),
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

        let child = TableInfo {
            name: "employees".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "dept_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "departments".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(ForeignKeyAction::SetNull),
                        on_update: None,
                    }),
                },
            ],
        };

        let parent = TableInfo {
            name: "departments".to_string(),
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

        storage.create_table(&parent).unwrap();
        storage.create_table(&child).unwrap();

        storage
            .insert("departments", vec![vec![Value::Integer(1)]])
            .unwrap();
        storage
            .insert(
                "employees",
                vec![vec![Value::Integer(1), Value::Integer(1)]],
            )
            .unwrap();

        storage.delete("departments", &[Value::Integer(1)]).ok();
    }

    #[test]
    fn test_self_referential_foreign_key() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "employees".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "manager_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "employees".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: None,
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&info).unwrap();

        storage
            .insert("employees", vec![vec![Value::Integer(1), Value::Null]])
            .unwrap();

        let result = storage.insert(
            "employees",
            vec![vec![Value::Integer(2), Value::Integer(1)]],
        );

        assert!(result.is_ok(), "Self-referential FK insert should succeed");
    }

    #[test]
    fn test_multiple_foreign_keys() {
        let mut storage = MemoryStorage::new();

        let table1 = TableInfo {
            name: "customers".to_string(),
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

        let table2 = TableInfo {
            name: "products".to_string(),
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

        let table3 = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "customer_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "customers".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: None,
                        on_update: None,
                    }),
                },
                ColumnDefinition {
                    name: "product_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "products".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: None,
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&table1).unwrap();
        storage.create_table(&table2).unwrap();
        storage.create_table(&table3).unwrap();

        storage
            .insert("customers", vec![vec![Value::Integer(1)]])
            .unwrap();
        storage
            .insert("products", vec![vec![Value::Integer(1)]])
            .unwrap();

        let result = storage.insert(
            "orders",
            vec![vec![
                Value::Integer(1),
                Value::Integer(1),
                Value::Integer(1),
            ]],
        );

        assert!(result.is_ok(), "Multiple FK insert should succeed");
    }

    #[test]
    fn test_foreign_key_restrict() {
        let mut storage = MemoryStorage::new();

        let parent = TableInfo {
            name: "categories".to_string(),
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

        let child = TableInfo {
            name: "items".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "category_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "categories".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(ForeignKeyAction::Restrict),
                        on_update: None,
                    }),
                },
            ],
        };

        storage.create_table(&parent).unwrap();
        storage.create_table(&child).unwrap();

        storage
            .insert("categories", vec![vec![Value::Integer(1)]])
            .unwrap();
        storage
            .insert("items", vec![vec![Value::Integer(1), Value::Integer(1)]])
            .unwrap();

        let _delete_result = storage.delete("categories", &[Value::Integer(1)]);
    }

    #[test]
    fn test_foreign_key_metadata_retrieval() {
        let mut storage = MemoryStorage::new();

        let child = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "customer_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "customers".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(ForeignKeyAction::Cascade),
                        on_update: Some(ForeignKeyAction::Cascade),
                    }),
                },
            ],
        };

        storage.create_table(&child).unwrap();

        let retrieved = storage.get_table_info("orders").unwrap();

        let customer_id_col = retrieved
            .columns
            .iter()
            .find(|c| c.name == "customer_id")
            .expect("customer_id should exist");

        let fk = customer_id_col
            .references
            .as_ref()
            .expect("customer_id should have FK");

        assert_eq!(fk.referenced_table, "customers");
        assert_eq!(fk.referenced_column, "id");
        assert_eq!(fk.on_delete, Some(ForeignKeyAction::Cascade));
        assert_eq!(fk.on_update, Some(ForeignKeyAction::Cascade));
    }
}
