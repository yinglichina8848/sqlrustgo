//! In-memory storage engine implementation
//! Provides fast, ephemeral storage for testing and caching

use crate::executor::{TableData, TableInfo};
use crate::storage::engine::{StorageEngine, StorageResult};
use crate::types::error::SqlError;
use crate::types::Value;
use std::collections::HashMap;

/// In-memory storage engine
/// Useful for testing, caching, or temporary data
pub struct MemoryStorage {
    /// In-memory table storage
    tables: HashMap<String, TableData>,
    /// Indexes: (table_name, column_name) -> B+Tree
    indexes: HashMap<(String, String), crate::storage::BPlusTree>,
}

impl MemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    /// Get table data reference
    pub fn get_table_ref(&self, name: &str) -> Option<&TableData> {
        self.tables.get(name)
    }

    /// Get mutable table data
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableData> {
        self.tables.get_mut(name)
    }

    /// Check if table exists
    pub fn contains_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// List all table names
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Get row count
    pub fn row_count(&self, name: &str) -> Option<usize> {
        self.tables.get(name).map(|t| t.rows.len())
    }

    /// Create index on column
    pub fn create_index(&mut self, table_name: &str, column_name: &str) -> StorageResult<()> {
        let table = self
            .tables
            .get(table_name)
            .ok_or_else(|| SqlError::TableNotFound(table_name.to_string()))?;

        let col_idx = table
            .info
            .columns
            .iter()
            .position(|c| c.name == column_name)
            .ok_or_else(|| SqlError::ColumnNotFound(column_name.to_string()))?;

        let mut tree = crate::storage::BPlusTree::new();

        // Build index from existing data
        for (row_idx, row) in table.rows.iter().enumerate() {
            if let Some(&Value::Integer(key)) = row.get(col_idx) {
                tree.insert(key, row_idx as u32);
            }
        }

        self.indexes
            .insert((table_name.to_string(), column_name.to_string()), tree);

        Ok(())
    }

    /// Drop index
    pub fn drop_index(&mut self, table_name: &str, column_name: &str) -> StorageResult<()> {
        self.indexes
            .remove(&(table_name.to_string(), column_name.to_string()));
        Ok(())
    }

    /// Check if index exists
    pub fn has_index(&self, table_name: &str, column_name: &str) -> bool {
        self.indexes
            .contains_key(&(table_name.to_string(), column_name.to_string()))
    }

    /// Search index
    pub fn search_index(&self, table_name: &str, column_name: &str, key: i64) -> Option<u32> {
        self.indexes
            .get(&(table_name.to_string(), column_name.to_string()))
            .and_then(|tree| tree.search(key))
    }

    /// Range query using index
    pub fn range_index(
        &self,
        table_name: &str,
        column_name: &str,
        min: i64,
        max: i64,
    ) -> Vec<u32> {
        self.indexes
            .get(&(table_name.to_string(), column_name.to_string()))
            .map(|tree| tree.range_query(min, max))
            .unwrap_or_default()
    }

    /// Insert with index update
    pub fn insert_with_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        key: i64,
        row_idx: u32,
    ) -> StorageResult<()> {
        if let Some(tree) = self
            .indexes
            .get_mut(&(table_name.to_string(), column_name.to_string()))
        {
            tree.insert(key, row_idx);
        }
        Ok(())
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.tables.clear();
        self.indexes.clear();
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MemoryStorage {
    fn get_table(&self, name: &str) -> Option<TableData> {
        self.tables.get(name).cloned()
    }

    fn table_names(&self) -> Vec<String> {
        MemoryStorage::table_names(self)
    }

    fn create_table(&mut self, name: &str, info: TableInfo) -> StorageResult<()> {
        if self.tables.contains_key(name) {
            return Err(SqlError::ExecutionError(format!(
                "Table {} already exists",
                name
            )));
        }

        self.tables.insert(
            name.to_string(),
            TableData {
                info,
                rows: Vec::new(),
            },
        );
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> StorageResult<()> {
        self.tables
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| SqlError::TableNotFound(name.to_string()))
    }

    fn insert(&mut self, table: &str, row: Vec<Value>) -> StorageResult<()> {
        let table_data = self
            .tables
            .get_mut(table)
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))?;

        let row_idx = table_data.rows.len() as u32;

        // Update indexes if they exist
        for (col_idx, value) in row.iter().enumerate() {
            if let Value::Integer(key) = value {
                let col_name = table_data.info.columns.get(col_idx).map(|c| &c.name);
                if let Some(col_name) = col_name {
                    if let Some(tree) = self
                        .indexes
                        .get_mut(&(table.to_string(), col_name.to_string()))
                    {
                        tree.insert(*key, row_idx);
                    }
                }
            }
        }

        table_data.rows.push(row);
        Ok(())
    }

    fn scan(&self, table: &str) -> StorageResult<Vec<Vec<Value>>> {
        self.tables
            .get(table)
            .map(|t| t.rows.clone())
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))
    }

    fn update(
        &mut self,
        table: &str,
        updates: Vec<(String, Value)>,
        filter: Option<crate::storage::engine::RowFilter>,
    ) -> StorageResult<u32> {
        let table_data = self
            .tables
            .get_mut(table)
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))?;

        let columns: HashMap<String, usize> = table_data
            .info
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();

        let mut count = 0u32;
        for row in table_data.rows.iter_mut() {
            let should_update = match &filter {
                Some(f) => f(row, &table_data.info),
                None => true,
            };

            if should_update {
                for (col_name, value) in &updates {
                    if let Some(&idx) = columns.get(col_name) {
                        if idx < row.len() {
                            row[idx] = value.clone();
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    fn delete(
        &mut self,
        table: &str,
        filter: Option<crate::storage::engine::RowFilter>,
    ) -> StorageResult<u32> {
        let table_data = self
            .tables
            .get_mut(table)
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))?;

        let original_len = table_data.rows.len();
        table_data.rows.retain(|row| match &filter {
            Some(f) => !f(row, &table_data.info),
            None => false,
        });
        Ok((original_len - table_data.rows.len()) as u32)
    }

    fn get_table_info(&self, name: &str) -> Option<TableInfo> {
        self.tables.get(name).map(|t| t.info.clone())
    }

    fn has_table(&self, name: &str) -> bool {
        MemoryStorage::contains_table(self, name)
    }

    fn row_count(&self, table: &str) -> StorageResult<usize> {
        MemoryStorage::row_count(self, table).ok_or_else(|| SqlError::TableNotFound(table.to_string()))
    }

    fn create_index(&mut self, table: &str, column: &str) -> StorageResult<()> {
        MemoryStorage::create_index(self, table, column)
    }

    fn drop_index(&mut self, table: &str, column: &str) -> StorageResult<()> {
        MemoryStorage::drop_index(self, table, column)
    }

    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32> {
        MemoryStorage::search_index(self, table, column, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ColumnDefinition;

    #[test]
    fn test_memory_storage_basic() {
        let mut storage = MemoryStorage::new();

        // Create table
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        };
        storage.create_table("users", info).unwrap();

        // Insert
        storage
            .insert("users", vec![Value::Integer(1)])
            .unwrap();

        // Scan
        let rows = storage.scan("users").unwrap();
        assert_eq!(rows.len(), 1);

        // Row count
        assert_eq!(storage.row_count("users").unwrap(), 1);

        // Has table
        assert!(storage.has_table("users"));
        assert!(!storage.has_table("nonexistent"));
    }

    #[test]
    fn test_memory_storage_index() {
        let mut storage = MemoryStorage::new();

        // Create table with data
        let info = TableInfo {
            name: "test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        };
        storage.create_table("test", info).unwrap();
        storage.insert("test", vec![Value::Integer(10)]).unwrap();
        storage.insert("test", vec![Value::Integer(20)]).unwrap();

        // Create index
        storage.create_index("test", "id").unwrap();

        // Search
        let result = storage.search_index("test", "id", 10);
        assert_eq!(result, Some(0));

        // Range
        let range = storage.range_index("test", "id", 5, 15);
        assert!(!range.is_empty());

        // Drop index
        storage.drop_index("test", "id").unwrap();
    }

    #[test]
    fn test_memory_storage_update_delete() {
        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "items".to_string(),
            columns: vec![ColumnDefinition {
                name: "value".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            }],
        };
        storage.create_table("items", info).unwrap();

        storage.insert("items", vec![Value::Integer(10)]).unwrap();
        storage.insert("items", vec![Value::Integer(20)]).unwrap();

        // Update
        let count = storage
            .update("items", vec![("value".to_string(), Value::Integer(100))], None)
            .unwrap();
        assert_eq!(count, 2);

        // Delete
        let count = storage
            .delete("items", None)
            .unwrap();
        assert_eq!(count, 2);

        assert_eq!(storage.row_count("items").unwrap(), 0);
    }
}
