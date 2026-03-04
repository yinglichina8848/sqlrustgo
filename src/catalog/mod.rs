//! Catalog module - metadata management
//! Provides Catalog trait for table and schema management

use crate::executor::TableInfo;
use crate::types::error::SqlError;
use crate::types::Value;
use std::collections::HashMap;

/// Catalog result type
pub type CatalogResult<T> = Result<T, SqlError>;

/// Table metadata with additional information
#[derive(Debug, Clone)]
pub struct TableMeta {
    /// Table name
    pub name: String,
    /// Table schema
    pub schema: String,
    /// Column definitions
    pub columns: Vec<ColumnMeta>,
    /// Table engine type
    pub engine: String,
    /// Row count (if known)
    pub row_count: Option<u64>,
    /// Created at timestamp
    pub created_at: u64,
    /// Updated at timestamp
    pub updated_at: u64,
}

/// Column metadata
#[derive(Debug, Clone)]
pub struct ColumnMeta {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: String,
    /// Is nullable
    pub nullable: bool,
    /// Is primary key
    pub is_primary_key: bool,
    /// Default value
    pub default_value: Option<Value>,
    /// Column ordinal position
    pub ordinal: usize,
}

/// Catalog trait - manages database metadata
/// Provides interface for table and schema operations
pub trait Catalog: Send + Sync {
    /// Get table metadata by name
    fn get_table(&self, name: &str) -> Option<TableMeta>;

    /// List all table names
    fn list_tables(&self) -> Vec<String>;

    /// Check if table exists
    fn has_table(&self, name: &str) -> bool;

    /// Add a new table to catalog
    fn add_table(&mut self, meta: TableMeta) -> CatalogResult<()>;

    /// Drop a table from catalog
    fn drop_table(&mut self, name: &str) -> CatalogResult<()>;

    /// Update table metadata
    fn update_table(&mut self, meta: TableMeta) -> CatalogResult<()>;

    /// Get table info for execution
    fn get_table_info(&self, name: &str) -> Option<TableInfo>;
}

/// Simple in-memory catalog implementation
pub struct SimpleCatalog {
    /// Tables metadata
    tables: HashMap<String, TableMeta>,
    /// Default schema name
    default_schema: String,
}

impl SimpleCatalog {
    /// Create a new empty catalog
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            default_schema: "default".to_string(),
        }
    }

    /// Create catalog with existing tables
    pub fn with_tables(tables: HashMap<String, TableMeta>) -> Self {
        Self {
            tables,
            default_schema: "default".to_string(),
        }
    }

    /// Get default schema name
    pub fn default_schema(&self) -> &str {
        &self.default_schema
    }
}

impl Default for SimpleCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl Catalog for SimpleCatalog {
    fn get_table(&self, name: &str) -> Option<TableMeta> {
        self.tables.get(name).cloned()
    }

    fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    fn has_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    fn add_table(&mut self, meta: TableMeta) -> CatalogResult<()> {
        if self.tables.contains_key(&meta.name) {
            return Err(SqlError::ExecutionError(format!(
                "Table {} already exists",
                meta.name
            )));
        }
        self.tables.insert(meta.name.clone(), meta);
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> CatalogResult<()> {
        self.tables
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| SqlError::TableNotFound(name.to_string()))
    }

    fn update_table(&mut self, meta: TableMeta) -> CatalogResult<()> {
        if !self.tables.contains_key(&meta.name) {
            return Err(SqlError::TableNotFound(meta.name));
        }
        self.tables.insert(meta.name.clone(), meta);
        Ok(())
    }

    fn get_table_info(&self, name: &str) -> Option<TableInfo> {
        self.tables.get(name).map(|meta| TableInfo {
            name: meta.name.clone(),
            columns: meta
                .columns
                .iter()
                .map(|c| crate::parser::ColumnDefinition {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                })
                .collect(),
        })
    }
}

impl From<TableInfo> for TableMeta {
    fn from(info: TableInfo) -> Self {
        let columns = info
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| ColumnMeta {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.nullable,
                is_primary_key: false,
                default_value: None,
                ordinal: i,
            })
            .collect();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        TableMeta {
            name: info.name,
            schema: "default".to_string(),
            columns,
            engine: "default".to_string(),
            row_count: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_catalog() {
        let mut catalog = SimpleCatalog::new();

        // Add table
        let meta = TableMeta {
            name: "users".to_string(),
            schema: "default".to_string(),
            columns: vec![ColumnMeta {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: true,
                default_value: None,
                ordinal: 0,
            }],
            engine: "default".to_string(),
            row_count: None,
            created_at: 0,
            updated_at: 0,
        };

        catalog.add_table(meta.clone()).unwrap();
        assert!(catalog.has_table("users"));
        assert_eq!(catalog.list_tables(), vec!["users"]);

        // Get table
        let retrieved = catalog.get_table("users").unwrap();
        assert_eq!(retrieved.name, "users");

        // Drop table
        catalog.drop_table("users").unwrap();
        assert!(!catalog.has_table("users"));
    }

    #[test]
    fn test_duplicate_table() {
        let mut catalog = SimpleCatalog::new();

        let meta = TableMeta {
            name: "users".to_string(),
            schema: "default".to_string(),
            columns: vec![],
            engine: "default".to_string(),
            row_count: None,
            created_at: 0,
            updated_at: 0,
        };

        catalog.add_table(meta.clone()).unwrap();
        let result = catalog.add_table(meta);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_info_conversion() {
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![
                crate::parser::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                crate::parser::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        };

        let meta: TableMeta = info.into();
        assert_eq!(meta.name, "test");
        assert_eq!(meta.columns.len(), 2);
        assert_eq!(meta.columns[0].name, "id");
    }
}
