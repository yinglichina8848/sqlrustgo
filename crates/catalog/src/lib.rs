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
        catalog.schemas.insert(
            "public".to_string(),
            Schema::new("public"),
        );
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

    /// Check if a schema exists
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    /// Get a table from the default schema
    pub fn get_table(&self, table_name: &str) -> Option<Arc<Table>> {
        self.default_schema()
            .and_then(|s| s.get_table(table_name))
    }

    /// Get a table from a specific schema
    pub fn get_table_in_schema(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> Option<Arc<Table>> {
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
                            referenced: format!(
                                "{}.{}",
                                fk.referenced_schema, fk.referenced_table
                            ),
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
                            referenced: format!(
                                "{}.{}",
                                fk.referenced_schema, fk.referenced_table
                            ),
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
