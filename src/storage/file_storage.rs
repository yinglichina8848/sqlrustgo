//! File-based table storage
//! Persists table data to JSON files

use crate::executor::{TableData, TableInfo};
use crate::storage::BPlusTree;
use crate::types::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

/// File-based storage manager
pub struct FileStorage {
    /// Base directory for database files
    data_dir: PathBuf,
    /// In-memory cache of tables
    tables: HashMap<String, TableData>,
    /// B+ Tree indexes: (table_name, column_name) -> BPlusTree
    indexes: HashMap<(String, String), BPlusTree>,
}

impl FileStorage {
    /// Create a new FileStorage with the given data directory
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&data_dir)?;

        let mut storage = Self {
            data_dir,
            tables: HashMap::new(),
            indexes: HashMap::new(),
        };

        // Load existing tables
        storage.load_all_tables()?;

        // Load existing indexes
        storage.load_all_indexes()?;

        Ok(storage)
    }

    /// Get the path for a table file
    fn table_path(&self, table_name: &str) -> PathBuf {
        self.data_dir.join(format!("{}.json", table_name))
    }

    /// Get the path for an index file
    fn index_path(&self, table_name: &str, column_name: &str) -> PathBuf {
        self.data_dir.join(format!("{}_idx_{}.json", table_name, column_name))
    }

    /// Load all tables from the data directory
    fn load_all_tables(&mut self) -> std::io::Result<()> {
        if !self.data_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(table_name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(table_data) = self.load_table(table_name) {
                        self.tables.insert(table_name.to_string(), table_data);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load all indexes from the data directory
    fn load_all_indexes(&mut self) -> std::io::Result<()> {
        if !self.data_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Look for index files: table_idx_column.json
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                if file_name.ends_with(".json") && file_name.contains("_idx_") {
                    // Parse table_idx_column.json
                    if let Some((table_name, column_name)) = file_name
                        .strip_suffix(".json")
                        .and_then(|s| s.split_once("_idx_"))
                    {
                        if let Ok(index) = self.load_index(table_name, column_name) {
                            self.indexes.insert(
                                (table_name.to_string(), column_name.to_string()),
                                index,
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single index from disk
    fn load_index(&self, table_name: &str, column_name: &str) -> std::io::Result<BPlusTree> {
        let path = self.index_path(table_name, column_name);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let index: BPlusTree = serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(index)
    }

    /// Save an index to disk
    fn save_index(&self, table_name: &str, column_name: &str, index: &BPlusTree) -> std::io::Result<()> {
        let path = self.index_path(table_name, column_name);
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        let json = serde_json::to_string_pretty(index)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        writer.write_all(json.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    /// Load a single table from disk
    fn load_table(&self, table_name: &str) -> std::io::Result<TableData> {
        let path = self.table_path(table_name);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let stored: StoredTableData = serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(TableData {
            info: TableInfo {
                name: stored.name,
                columns: stored.columns,
            },
            rows: stored.rows,
        })
    }

    /// Save a table to disk
    fn save_table(&self, table_name: &str, table_data: &TableData) -> std::io::Result<()> {
        let path = self.table_path(table_name);
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        let stored = StoredTableData {
            name: table_data.info.name.clone(),
            columns: table_data.info.columns.clone(),
            rows: table_data.rows.clone(),
        };

        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        writer.write_all(json.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    /// Get a table by name
    pub fn get_table(&self, name: &str) -> Option<&TableData> {
        self.tables.get(name)
    }

    /// Get a mutable table by name
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableData> {
        self.tables.get_mut(name)
    }

    /// Insert a new table
    pub fn insert_table(&mut self, name: String, table_data: TableData) -> std::io::Result<()> {
        self.tables.insert(name.clone(), table_data.clone());
        self.save_table(&name, &table_data)
    }

    /// Drop (delete) a table
    pub fn drop_table(&mut self, name: &str) -> std::io::Result<()> {
        self.tables.remove(name);

        let path = self.table_path(name);
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Force save all dirty tables to disk
    pub fn flush(&self) -> std::io::Result<()> {
        for (name, table_data) in &self.tables {
            self.save_table(name, table_data)?;
        }
        Ok(())
    }

    /// Check if a table exists
    pub fn contains_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Save a table to disk (call after modifications)
    pub fn persist_table(&self, name: &str) -> std::io::Result<()> {
        if let Some(table_data) = self.tables.get(name) {
            self.save_table(name, table_data)
        } else {
            Ok(())
        }
    }

    // ==================== Index Methods ====================

    /// Check if an index exists for a table column
    pub fn has_index(&self, table_name: &str, column_name: &str) -> bool {
        self.indexes.contains_key(&(table_name.to_string(), column_name.to_string()))
    }

    /// Get an index for a table column (read-only)
    pub fn get_index(&self, table_name: &str, column_name: &str) -> Option<&BPlusTree> {
        self.indexes.get(&(table_name.to_string(), column_name.to_string()))
    }

    /// Create or update an index for a table column from existing data
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

        // Build B+ Tree from existing rows
        let mut index = BPlusTree::new();
        for (row_id, row) in table.rows.iter().enumerate() {
            if let Some(value) = row.get(column_index) {
                if let Value::Integer(key) = value {
                    index.insert(*key, row_id as u32);
                }
            }
        }

        // Save to disk
        self.save_index(table_name, column_name, &index)?;

        // Store in memory
        self.indexes.insert(
            (table_name.to_string(), column_name.to_string()),
            index,
        );

        Ok(())
    }

    /// Insert a row and update index
    pub fn insert_with_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        key: i64,
        row_id: u32,
    ) -> std::io::Result<()> {
        let key_exists = (table_name.to_string(), column_name.to_string());

        // Clone the key for later use
        let has_index = self.indexes.contains_key(&key_exists);

        if has_index {
            // Get mutable reference, insert, then save
            if let Some(index) = self.indexes.get_mut(&key_exists) {
                index.insert(key, row_id);
            }
            // Now we can borrow immutably to save
            if let Some(index) = self.indexes.get(&key_exists) {
                self.save_index(table_name, column_name, index)?;
            }
        }

        Ok(())
    }

    /// Search using index - returns row IDs matching the key
    pub fn search_index(&self, table_name: &str, column_name: &str, key: i64) -> Option<u32> {
        self.indexes
            .get(&(table_name.to_string(), column_name.to_string()))
            .and_then(|index| index.search(key))
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
            .map(|index| index.range_query(start, end))
            .unwrap_or_default()
    }

    /// Drop an index
    pub fn drop_index(&mut self, table_name: &str, column_name: &str) -> std::io::Result<()> {
        let key = (table_name.to_string(), column_name.to_string());
        self.indexes.remove(&key);

        let path = self.index_path(table_name, column_name);
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Flush all indexes to disk
    pub fn flush_indexes(&self) -> std::io::Result<()> {
        for ((table_name, column_name), index) in &self.indexes {
            self.save_index(table_name, column_name, index)?;
        }
        Ok(())
    }
}

/// Stored table data (for serialization)
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredTableData {
    name: String,
    columns: Vec<crate::parser::ColumnDefinition>,
    rows: Vec<Vec<Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ColumnDefinition;
    use std::fs::remove_dir_all;

    #[test]
    fn test_file_storage() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_file_storage");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert a table
            let table_data = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![
                        ColumnDefinition {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                        },
                        ColumnDefinition {
                            name: "name".to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: true,
                        },
                    ],
                },
                rows: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                ],
            };

            storage.insert_table("users".to_string(), table_data).unwrap();
        }

        // Load from disk
        {
            let storage = FileStorage::new(temp_dir.clone()).unwrap();
            let table = storage.get_table("users").unwrap();
            assert_eq!(table.info.name, "users");
            assert_eq!(table.rows.len(), 1);
        }

        let _ = remove_dir_all(&temp_dir);
    }
}
