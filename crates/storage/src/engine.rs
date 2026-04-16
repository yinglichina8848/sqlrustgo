//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;

/// Referential action for foreign key constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForeignKeyAction {
    Cascade,
    SetNull,
    Restrict,
    NoAction,
}

/// Foreign key constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyConstraint {
    pub name: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueConstraint {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKeyConstraint>,
    #[serde(default)]
    pub unique_constraints: Vec<UniqueConstraint>,
}

/// Column definition for table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
}

impl Default for ColumnDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            data_type: String::new(),
            nullable: false,
            primary_key: false,
        }
    }
}

/// Table data - combines metadata and rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub info: TableInfo,
    pub rows: Vec<Record>,
}

/// Record type - a single row of values
pub type Record = Vec<Value>;

/// StorageEngine trait - abstraction for table storage
/// Enables multiple storage backends (FileStorage, MemoryStorage, etc.)
pub trait StorageEngine: Send + Sync {
    /// Scan all rows from a table
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>>;

    /// Insert rows into a table
    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()>;

    /// Delete rows matching a filter
    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize>;

    /// Update rows matching a filter
    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize>;

    /// Create a new table
    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()>;

    /// Drop a table
    fn drop_table(&mut self, table: &str) -> SqlResult<()>;

    /// Get table metadata
    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo>;

    /// Check if table exists
    fn has_table(&self, table: &str) -> bool;

    /// List all tables
    fn list_tables(&self) -> Vec<String>;

    /// Create an index on a table
    fn create_index(&mut self, table: &str, column: &str, column_index: usize) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_index(&mut self, table: &str, column: &str) -> SqlResult<()>;

    /// Add a column to an existing table
    fn add_column(&mut self, table: &str, column: ColumnDefinition) -> SqlResult<()>;

    /// Rename a table
    fn rename_table(&mut self, table: &str, new_name: &str) -> SqlResult<()>;
}

/// In-memory storage implementation for testing and caching
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MemoryStorage {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        Ok(self.tables.get(table).cloned().unwrap_or_default())
    }

    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        self.tables
            .entry(table.to_string())
            .or_default()
            .extend(records);
        Ok(())
    }

    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize> {
        let mut count = 0;
        if let Some(records) = self.tables.get_mut(table) {
            count = records.len();
            records.clear();
        }
        Ok(count)
    }

    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        Ok(self.tables.get(table).map(|r| r.len()).unwrap_or(0))
    }

    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()> {
        self.table_infos.insert(info.name.clone(), info.clone());
        self.tables.entry(info.name.clone()).or_default();
        Ok(())
    }

    fn drop_table(&mut self, table: &str) -> SqlResult<()> {
        self.tables.remove(table);
        self.table_infos.remove(table);
        Ok(())
    }

    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo> {
        self.table_infos
            .get(table)
            .cloned()
            .ok_or_else(|| sqlrustgo_types::SqlError::TableNotFound(table.to_string()))
    }

    fn has_table(&self, table: &str) -> bool {
        self.tables.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    fn create_index(&mut self, _table: &str, _column: &str, _column_index: usize) -> SqlResult<()> {
        Ok(())
    }

    fn drop_index(&mut self, _table: &str, _column: &str) -> SqlResult<()> {
        Ok(())
    }

    fn add_column(&mut self, table: &str, column: ColumnDefinition) -> SqlResult<()> {
        if let Some(info) = self.table_infos.get_mut(table) {
            info.columns.push(column);
        }
        Ok(())
    }

    fn rename_table(&mut self, table: &str, new_name: &str) -> SqlResult<()> {
        if let Some(info) = self.table_infos.remove(table) {
            let mut new_info = info;
            new_info.name = new_name.to_string();
            self.table_infos.insert(new_name.to_string(), new_info);
            if let Some(records) = self.tables.remove(table) {
                self.tables.insert(new_name.to_string(), records);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that StorageEngine trait is defined correctly
    #[test]
    fn test_storage_engine_trait_exists() {
        fn _check_trait(_engine: &dyn StorageEngine) {}
    }

    #[test]
    fn test_memory_storage_new() {
        let storage = MemoryStorage::new();
        assert!(storage.list_tables().is_empty());
    }

    #[test]
    fn test_memory_storage_has_table() {
        let storage = MemoryStorage::new();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_list_tables() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        let tables = storage.list_tables();
        assert!(tables.contains(&"users".to_string()));
    }

    #[test]
    fn test_memory_storage_scan_empty() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        let result = storage.scan("users").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_memory_storage_insert_and_scan() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );
        let result = storage.scan("users").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_storage_engine_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MemoryStorage>();
    }
}
