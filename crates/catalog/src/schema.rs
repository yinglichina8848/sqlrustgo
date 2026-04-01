//! Schema definition for catalog

use crate::error::{CatalogError, CatalogResult};
use crate::table::{Table, TableRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Schema definition containing tables
///
/// Note: Tables are stored directly (not as Arc<Table>) to support serde
/// serialization. Arc<Table> wrapping happens when returning from methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema name
    pub name: String,
    /// Tables in this schema (name -> Table)
    tables: HashMap<String, Table>,
}

impl Schema {
    /// Create a new empty schema
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tables: HashMap::new(),
        }
    }

    /// Add a table to the schema
    pub fn add_table(mut self, table: Table) -> CatalogResult<Self> {
        if self.tables.contains_key(&table.name) {
            return Err(CatalogError::DuplicateTable {
                schema: self.name.clone(),
                table: table.name.clone(),
            });
        }
        self.tables.insert(table.name.clone(), table);
        Ok(self)
    }

    /// Get a table by name (wrapped in Arc for shared ownership)
    pub fn get_table(&self, name: &str) -> Option<TableRef> {
        self.tables.get(name).map(|t| Arc::new(t.clone()))
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<&str> {
        self.tables.keys().map(|s| s.as_str()).collect()
    }

    /// Get all tables (each wrapped in Arc)
    pub fn tables(&self) -> Vec<TableRef> {
        self.tables.values().map(|t| Arc::new(t.clone())).collect()
    }

    /// Check if a table exists
    pub fn has_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Remove a table from the schema
    pub fn remove_table(&mut self, name: &str) -> Option<TableRef> {
        self.tables.remove(name).map(Arc::new)
    }

    /// Get the number of tables
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    /// Validate the schema and all its tables
    pub fn validate(&self) -> CatalogResult<()> {
        for table in self.tables.values() {
            table.validate(&self.name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnDefinition;
    use crate::data_type::DataType;

    fn create_test_schema() -> Schema {
        Schema::new("test_schema")
            .add_table(
                Table::new(
                    "users",
                    vec![
                        ColumnDefinition::new("id", DataType::Integer),
                        ColumnDefinition::new("name", DataType::Text),
                    ],
                )
                .primary_key(vec!["id".to_string()])
                .unwrap(),
            )
            .unwrap()
    }

    #[test]
    fn test_schema_creation() {
        let schema = create_test_schema();
        assert_eq!(schema.name, "test_schema");
        assert_eq!(schema.table_count(), 1);
    }

    #[test]
    fn test_get_table() {
        let schema = create_test_schema();
        assert!(schema.get_table("users").is_some());
        assert!(schema.get_table("nonexistent").is_none());
    }

    #[test]
    fn test_duplicate_table() {
        let schema = create_test_schema();
        let result = schema.add_table(Table::new("users", vec![]));
        assert!(matches!(result, Err(CatalogError::DuplicateTable { .. })));
    }

    #[test]
    fn test_table_names() {
        let schema = create_test_schema();
        let names = schema.table_names();
        assert_eq!(names, vec!["users"]);
    }

    #[test]
    fn test_remove_table() {
        let mut schema = create_test_schema();
        let removed = schema.remove_table("users");
        assert!(removed.is_some());
        assert_eq!(schema.table_count(), 0);
    }
}
