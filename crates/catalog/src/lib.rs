//! SQLRustGo Catalog Module
//!
//! This module provides the catalog system for managing schemas, tables, columns,
//! and constraints in the database.

mod column;
mod data_type;
mod error;
mod index;
mod rebuild;
mod schema;
mod table;

pub use column::ColumnDefinition;
pub use data_type::DataType;
pub use error::{CatalogError, CatalogResult};
pub use index::{IndexInfo, IndexType};
pub use schema::Schema;
pub use table::{ForeignKeyAction, ForeignKeyRef, Table, TableRef};

use serde::{Deserialize, Serialize};
use sqlrustgo_storage::{ForeignKeyAction as StorageFkAction, StorageEngine};
use std::collections::HashMap;
use std::sync::Arc;

/// Catalog containing all schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// Default schema name
    default_schema: String,
    /// Schemas in the catalog (name -> Schema)
    schemas: HashMap<String, Schema>,
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

impl Catalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        let mut catalog = Self {
            default_schema: "public".to_string(),
            schemas: HashMap::new(),
        };
        // Create the default public schema
        catalog
            .schemas
            .insert("public".to_string(), Schema::new("public"));
        catalog
    }

    /// Create a catalog with a custom default schema name
    pub fn with_default_schema(default_schema: impl Into<String>) -> Self {
        let mut catalog = Self {
            default_schema: default_schema.into(),
            schemas: HashMap::new(),
        };
        catalog.schemas.insert(
            catalog.default_schema.clone(),
            Schema::new(&catalog.default_schema),
        );
        catalog
    }

    /// Get the default schema name
    pub fn default_schema_name(&self) -> &str {
        &self.default_schema
    }

    /// Get the default schema
    pub fn default_schema(&self) -> Option<&Schema> {
        self.schemas.get(&self.default_schema)
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }

    /// Get a mutable schema by name
    pub fn get_schema_mut(&mut self, name: &str) -> Option<&mut Schema> {
        self.schemas.get_mut(name)
    }

    /// Add a schema to the catalog
    pub fn add_schema(&mut self, schema: Schema) -> CatalogResult<()> {
        if self.schemas.contains_key(&schema.name) {
            return Err(CatalogError::DuplicateSchema(schema.name));
        }
        self.schemas.insert(schema.name.clone(), schema);
        Ok(())
    }

    /// Get all schema names
    pub fn schema_names(&self) -> Vec<&str> {
        self.schemas.keys().map(|s| s.as_str()).collect()
    }

    /// Get all schemas
    pub fn all_schemas(&self) -> Vec<&Schema> {
        self.schemas.values().collect()
    }

    /// Check if a schema exists
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    /// Get a table from the default schema
    pub fn get_table(&self, table_name: &str) -> Option<Arc<Table>> {
        self.default_schema().and_then(|s| s.get_table(table_name))
    }

    /// Get a table from a specific schema
    pub fn get_table_in_schema(&self, schema_name: &str, table_name: &str) -> Option<Arc<Table>> {
        self.schemas
            .get(schema_name)
            .and_then(|s| s.get_table(table_name))
    }

    /// Check all catalog invariants
    ///
    /// This validates:
    /// - All schemas exist
    /// - All tables have valid definitions
    /// - All column references are valid
    /// - All foreign key references are valid
    pub fn check_invariants(&self) -> CatalogResult<()> {
        // Validate all schemas
        for schema in self.schemas.values() {
            schema.validate()?;
        }

        // Validate foreign key references between schemas
        for schema in self.schemas.values() {
            for table in schema.tables() {
                for fk in &table.foreign_keys {
                    // Check that referenced schema exists
                    if !self.schemas.contains_key(&fk.referenced_schema) {
                        return Err(CatalogError::ForeignKeyViolation {
                            schema: schema.name.clone(),
                            table: table.name.clone(),
                            column: fk.columns.join(", "),
                            referenced: format!("{}.{}", fk.referenced_schema, fk.referenced_table),
                            reason: format!(
                                "Referenced schema '{}' does not exist",
                                fk.referenced_schema
                            ),
                        });
                    }

                    // Check that referenced table exists in the schema
                    let ref_schema = self.schemas.get(&fk.referenced_schema).unwrap();
                    if !ref_schema.has_table(&fk.referenced_table) {
                        return Err(CatalogError::ForeignKeyViolation {
                            schema: schema.name.clone(),
                            table: table.name.clone(),
                            column: fk.columns.join(", "),
                            referenced: format!("{}.{}", fk.referenced_schema, fk.referenced_table),
                            reason: format!(
                                "Referenced table '{}.{}' does not exist",
                                fk.referenced_schema, fk.referenced_table
                            ),
                        });
                    }

                    // Check that referenced columns exist
                    let ref_table = ref_schema.get_table(&fk.referenced_table).unwrap();
                    for col_name in &fk.referenced_columns {
                        if ref_table.get_column(col_name).is_none() {
                            return Err(CatalogError::ForeignKeyViolation {
                                schema: schema.name.clone(),
                                table: table.name.clone(),
                                column: fk.columns.join(", "),
                                referenced: format!(
                                    "{}.{}",
                                    fk.referenced_schema, fk.referenced_table
                                ),
                                reason: format!(
                                    "Referenced column '{}' does not exist in '{}.{}'",
                                    col_name, fk.referenced_schema, fk.referenced_table
                                ),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Rebuild catalog from a storage engine
    ///
    /// This reconstructs catalog metadata from an existing storage engine,
    /// including schemas, tables, columns, and foreign key constraints.
    pub fn rebuild(storage: &dyn StorageEngine) -> CatalogResult<Self> {
        let mut catalog = Self::new();
        let default_schema = catalog.default_schema_name().to_string();

        for table_name in storage.list_tables() {
            let info = storage.get_table_info(&table_name).map_err(|e| {
                CatalogError::InvariantViolation(format!(
                    "Failed to get table info for '{}': {}",
                    table_name, e
                ))
            })?;

            let columns: Vec<ColumnDefinition> = info
                .columns
                .into_iter()
                .map(|col| {
                    let data_type = DataType::parse_sql_name(&col.data_type).ok_or_else(|| {
                        CatalogError::InvariantViolation(format!(
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
            let foreign_keys: Vec<ForeignKeyRef> =
                if let Ok(info) = storage.get_table_info(&table.name) {
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
            let mut schema = catalog
                .schemas
                .remove(&schema_name)
                .ok_or_else(|| CatalogError::SchemaNotFound(schema_name.clone()))?;

            if schema.has_table(&table.name) {
                return Err(CatalogError::DuplicateTable {
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
}

/// Convert storage foreign key action to catalog foreign key action
fn convert_fk_action(action: StorageFkAction) -> ForeignKeyAction {
    match action {
        StorageFkAction::Cascade => ForeignKeyAction::Cascade,
        StorageFkAction::SetNull => ForeignKeyAction::SetNull,
        StorageFkAction::Restrict => ForeignKeyAction::Restrict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = Catalog::new();
        assert_eq!(catalog.default_schema_name(), "public");
        assert!(catalog.has_schema("public"));
        assert_eq!(catalog.schema_names(), vec!["public"]);
    }

    #[test]
    fn test_catalog_with_custom_default_schema() {
        let catalog = Catalog::with_default_schema("custom");
        assert_eq!(catalog.default_schema_name(), "custom");
        assert!(catalog.has_schema("custom"));
    }

    #[test]
    fn test_add_schema() {
        let mut catalog = Catalog::new();
        let result = catalog.add_schema(Schema::new("test"));
        assert!(result.is_ok());
        assert!(catalog.has_schema("test"));
    }

    #[test]
    fn test_duplicate_schema() {
        let mut catalog = Catalog::new();
        let result = catalog.add_schema(Schema::new("public"));
        assert!(matches!(result, Err(CatalogError::DuplicateSchema(_))));
    }

    #[test]
    fn test_get_schema() {
        let catalog = Catalog::new();
        assert!(catalog.get_schema("public").is_some());
        assert!(catalog.get_schema("nonexistent").is_none());
    }

    #[test]
    fn test_check_invariants_empty_catalog() {
        let catalog = Catalog::new();
        assert!(catalog.check_invariants().is_ok());
    }
}
