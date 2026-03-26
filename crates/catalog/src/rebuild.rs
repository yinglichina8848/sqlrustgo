//! Catalog rebuild functionality (tests only)
//!
//! Provides utilities to rebuild a Catalog from a StorageEngine for testing.

use crate::DataType;

/// Convert a storage data type string to catalog DataType
fn convert_data_type(data_type: &str) -> Option<DataType> {
    DataType::parse_sql_name(data_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::{
        ColumnDefinition as StorageColumn, ForeignKeyConstraint, ForeignKeyAction as StorageFkAction,
        MemoryStorage, StorageEngine, TableInfo,
    };
    use crate::{
        CatalogResult, ColumnDefinition, ForeignKeyAction, ForeignKeyRef, Table,
    };

    /// Rebuild catalog from a storage engine (test helper)
    ///
    /// This is a test-only function that rebuilds catalog metadata
    /// from an existing storage engine.
    pub fn rebuild_from_storage(storage: &dyn StorageEngine) -> CatalogResult<crate::Catalog> {
        // Clone catalog to work with owned values and avoid borrow issues
        let mut catalog = crate::Catalog::new();
        let default_schema = catalog.default_schema_name().to_string();

        for table_name in storage.list_tables() {
            let info = storage.get_table_info(&table_name).map_err(|e| {
                crate::CatalogError::InvariantViolation(format!(
                    "Failed to get table info for '{}': {}",
                    table_name, e
                ))
            })?;

            let columns: Vec<ColumnDefinition> = info
                .columns
                .into_iter()
                .map(|col| {
                    let data_type = convert_data_type(&col.data_type).ok_or_else(|| {
                        crate::CatalogError::InvariantViolation(format!(
                            "Unknown data type '{}' for column '{}'",
                            col.data_type, col.name
                        ))
                    })?;

                    let mut column_def = ColumnDefinition::new(col.name, data_type);
                    if !col.nullable {
                        column_def = column_def.not_null();
                    }
                    if col.is_unique {
                        column_def = column_def.unique();
                    }
                    Ok(column_def)
                })
                .collect::<CatalogResult<Vec<_>>>()?;

            let mut table = Table::new(info.name.clone(), columns);

            // Collect foreign keys from column references
            let foreign_keys: Vec<ForeignKeyRef> = if let Ok(info) = storage.get_table_info(&table.name) {
                info.columns
                    .iter()
                    .filter_map(|col_info| {
                        col_info.references.as_ref().map(|references| {
                            let col_name = col_info.name.clone();
                            ForeignKeyRef {
                                referenced_schema: default_schema.clone(),
                                referenced_table: references.referenced_table.clone(),
                                referenced_columns: vec![references.referenced_column.clone()],
                                columns: vec![col_name],
                                on_delete: references.on_delete.map(convert_fk_action),
                                on_update: references.on_update.map(convert_fk_action),
                            }
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Add all foreign keys to the table
            for fk in foreign_keys {
                table = table.add_foreign_key(fk);
            }

            // Get owned schema, add table, then put back
            let schema_name = default_schema.clone();
            let mut schema = catalog.schemas.remove(&schema_name).ok_or_else(|| {
                crate::CatalogError::SchemaNotFound(schema_name.clone())
            })?;

            if schema.has_table(&table.name) {
                return Err(crate::CatalogError::DuplicateTable {
                    schema: schema_name.clone(),
                    table: table.name.clone(),
                });
            }

            schema = schema.add_table(table)?;

            // Put schema back
            catalog.schemas.insert(schema_name, schema);
        }

        catalog.check_invariants()?;
        Ok(catalog)
    }

    /// Convert storage foreign key action to catalog foreign key action
    fn convert_fk_action(action: StorageFkAction) -> ForeignKeyAction {
        match action {
            StorageFkAction::Cascade => ForeignKeyAction::Cascade,
            StorageFkAction::SetNull => ForeignKeyAction::SetNull,
            StorageFkAction::Restrict => ForeignKeyAction::Restrict,
        }
    }

    fn create_test_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();
        let users_info = TableInfo {
            name: "users".to_string(),
            columns: vec![
                StorageColumn {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                StorageColumn {
                    name: "email".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                StorageColumn {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        };
        let orders_info = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                StorageColumn {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                StorageColumn {
                    name: "user_id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: Some(ForeignKeyConstraint {
                        referenced_table: "users".to_string(),
                        referenced_column: "id".to_string(),
                        on_delete: Some(StorageFkAction::Cascade),
                        on_update: None,
                    }),
                },
            ],
        };
        storage.create_table(&users_info).unwrap();
        storage.create_table(&orders_info).unwrap();
        storage
    }

    #[test]
    fn test_rebuild_empty_storage() {
        let storage = MemoryStorage::new();
        let catalog = rebuild_from_storage(&storage).unwrap();
        assert_eq!(catalog.schema_names(), vec!["public"]);
        assert_eq!(catalog.default_schema().unwrap().table_names().len(), 0);
    }

    #[test]
    fn test_rebuild_with_tables() {
        let storage = create_test_storage();
        let catalog = rebuild_from_storage(&storage).unwrap();
        let schema = catalog.default_schema().unwrap();
        assert_eq!(schema.table_names(), vec!["users", "orders"]);
        let users = schema.get_table("users").unwrap();
        assert_eq!(users.columns.len(), 3);
        assert!(users.columns.iter().any(|c| c.name == "id"));
        assert!(users.columns.iter().any(|c| c.name == "email"));
        assert!(users.columns.iter().any(|c| c.name == "name"));
    }

    #[test]
    fn test_rebuild_foreign_keys() {
        let storage = create_test_storage();
        let catalog = rebuild_from_storage(&storage).unwrap();
        let orders = catalog.default_schema().unwrap().get_table("orders").unwrap();
        assert_eq!(orders.columns.len(), 2);
    }

    #[test]
    fn test_rebuild_unknown_data_type() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![StorageColumn {
                name: "col".to_string(),
                data_type: "UNKNOWN_TYPE".to_string(),
                nullable: true,
                is_unique: false,
                references: None,
            }],
        };
        storage.create_table(&info).unwrap();
        let result = rebuild_from_storage(&storage);
        assert!(result.is_err());
    }

    #[test]
    fn test_rebuild_preserves_nullable() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![
                StorageColumn {
                    name: "nullable_col".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
                StorageColumn {
                    name: "not_null_col".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                },
            ],
        };
        storage.create_table(&info).unwrap();
        let catalog = rebuild_from_storage(&storage).unwrap();
        let table = catalog.default_schema().unwrap().get_table("test").unwrap();
        let nullable_col = table.columns.iter().find(|c| c.name == "nullable_col").unwrap();
        assert!(nullable_col.nullable);
        let not_null_col = table.columns.iter().find(|c| c.name == "not_null_col").unwrap();
        assert!(!not_null_col.nullable);
    }

    #[test]
    fn test_rebuild_preserves_unique() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![
                StorageColumn {
                    name: "unique_col".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: true,
                    references: None,
                },
                StorageColumn {
                    name: "regular_col".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        };
        storage.create_table(&info).unwrap();
        let catalog = rebuild_from_storage(&storage).unwrap();
        let table = catalog.default_schema().unwrap().get_table("test").unwrap();
        let unique_col = table.columns.iter().find(|c| c.name == "unique_col").unwrap();
        assert!(unique_col.is_unique);
        let regular_col = table.columns.iter().find(|c| c.name == "regular_col").unwrap();
        assert!(!regular_col.is_unique);
    }
}
