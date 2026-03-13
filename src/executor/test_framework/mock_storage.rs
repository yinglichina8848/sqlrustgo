//! Mock Storage for Testing
//!
//! Provides an in-memory storage implementation for testing the executor
//! without relying on file system operations.

use crate::executor::{TableData, TableInfo};
use crate::parser::ColumnDefinition;
use crate::types::Value;
use std::collections::HashMap;

/// Mock storage for testing - in-memory, no file I/O
#[derive(Debug, Clone)]
pub struct MockStorage {
    tables: HashMap<String, TableData>,
    indexes: HashMap<(String, String), HashMap<i64, Vec<u32>>>,
}

impl MockStorage {
    /// Create a new empty mock storage
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    /// Create mock storage with pre-populated data
    pub fn with_data(tables: Vec<(&str, TableData)>) -> Self {
        let mut storage = Self::new();
        for (name, data) in tables {
            storage.tables.insert(name.to_string(), data);
        }
        storage
    }

    /// Get a table by name (immutable)
    pub fn get_table(&self, name: &str) -> Option<&TableData> {
        self.tables.get(name)
    }

    /// Get a table by name (mutable)
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableData> {
        self.tables.get_mut(name)
    }

    /// Insert a new table
    pub fn insert_table(&mut self, name: String, table_data: TableData) -> std::io::Result<()> {
        self.tables.insert(name, table_data);
        Ok(())
    }

    /// Drop (delete) a table
    pub fn drop_table(&mut self, name: &str) -> std::io::Result<()> {
        self.tables.remove(name);
        Ok(())
    }

    /// Check if a table exists
    pub fn contains_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Persist table (no-op for mock)
    pub fn persist_table(&self, _name: &str) -> std::io::Result<()> {
        Ok(())
    }

    /// Check if an index exists
    pub fn has_index(&self, table_name: &str, column_name: &str) -> bool {
        self.indexes.contains_key(&(table_name.to_string(), column_name.to_string()))
    }

    /// Create an index on a table column
    pub fn create_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        column_index: usize,
    ) -> std::io::Result<()> {
        let table = self
            .tables
            .get(table_name)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Table not found"))?;

        let mut index: HashMap<i64, Vec<u32>> = HashMap::new();
        
        for (row_id, row) in table.rows.iter().enumerate() {
            if let Some(Value::Integer(key)) = row.get(column_index) {
                index.entry(*key).or_insert_with(Vec::new).push(row_id as u32);
            }
        }

        self.indexes.insert(
            (table_name.to_string(), column_name.to_string()),
            index,
        );

        Ok(())
    }

    /// Insert with index
    pub fn insert_with_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        key: i64,
        row_id: u32,
    ) -> std::io::Result<()> {
        let key_exists = (table_name.to_string(), column_name.to_string());

        if self.indexes.contains_key(&key_exists) {
            if let Some(index) = self.indexes.get_mut(&key_exists) {
                index.entry(key).or_insert_with(Vec::new).push(row_id);
            }
        }

        Ok(())
    }

    /// Search using index
    pub fn search_index(&self, table_name: &str, column_name: &str, key: i64) -> Option<u32> {
        self.indexes
            .get(&(table_name.to_string(), column_name.to_string()))
            .and_then(|index| index.get(&key))
            .and_then(|rows| rows.first().copied())
    }

    /// Range query using index
    pub fn range_index(
        &self,
        table_name: &str,
        column_name: &str,
        start: i64,
        end: i64,
    ) -> Vec<u32> {
        self.indexes
            .get(&(table_name.to_string(), column_name.to_string()))
            .map(|index| {
                index
                    .iter()
                    .filter(|(k, _)| **k >= start && **k <= end)
                    .flat_map(|(_, rows)| rows.iter())
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add a row to a table (for test data setup)
    pub fn add_row(&mut self, table_name: &str, row: Vec<Value>) -> std::io::Result<()> {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.rows.push(row);
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Table not found"))
        }
    }

    /// Clear all tables (for test cleanup)
    pub fn clear(&mut self) {
        self.tables.clear();
        self.indexes.clear();
    }

    /// Get the number of tables
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    /// Get row count for a table
    pub fn row_count(&self, table_name: &str) -> usize {
        self.tables
            .get(table_name)
            .map(|t| t.rows.len())
            .unwrap_or(0)
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MockStorage {
    /// Helper to create a simple users table for testing
    pub fn create_users_table() -> Self {
        let columns = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "email".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "age".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
        ];

        let table_data = TableData {
            info: TableInfo {
                name: "users".to_string(),
                columns,
            },
            rows: Vec::new(),
        };

        let mut storage = Self::new();
        storage.tables.insert("users".to_string(), table_data);
        storage
    }

    /// Helper to create a simple products table for testing
    pub fn create_products_table() -> Self {
        let columns = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "price".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "stock".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
        ];

        let table_data = TableData {
            info: TableInfo {
                name: "products".to_string(),
                columns,
            },
            rows: Vec::new(),
        };

        let mut storage = Self::new();
        storage.tables.insert("products".to_string(), table_data);
        storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_create() {
        let storage = MockStorage::new();
        assert!(storage.table_names().is_empty());
    }

    #[test]
    fn test_mock_storage_insert_table() {
        let mut storage = MockStorage::new();
        
        let columns = vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        }];
        
        let table_data = TableData {
            info: TableInfo {
                name: "test".to_string(),
                columns,
            },
            rows: vec![],
        };
        
        storage.insert_table("test".to_string(), table_data).unwrap();
        assert!(storage.contains_table("test"));
    }

    #[test]
    fn test_mock_storage_add_row() {
        let mut storage = MockStorage::create_users_table();
        
        storage.add_row("users", vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Text("alice@example.com".to_string()),
            Value::Integer(30),
        ]).unwrap();
        
        assert_eq!(storage.row_count("users"), 1);
    }

    #[test]
    fn test_mock_storage_index() {
        let mut storage = MockStorage::create_users_table();
        
        // Add some rows
        storage.add_row("users", vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Null,
            Value::Null,
        ]).unwrap();
        
        storage.add_row("users", vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Null,
            Value::Null,
        ]).unwrap();
        
        // Create index on id column
        storage.create_index("users", "id", 0).unwrap();
        
        assert!(storage.has_index("users", "id"));
        
        // Search using index
        let row_id = storage.search_index("users", "id", 1);
        assert_eq!(row_id, Some(0));
    }
}
