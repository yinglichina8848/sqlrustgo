//! File-based table storage
//! Persists table data to JSON files

use crate::executor::{TableData, TableInfo};
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
}

impl FileStorage {
    /// Create a new FileStorage with the given data directory
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&data_dir)?;

        let mut storage = Self {
            data_dir,
            tables: HashMap::new(),
        };

        // Load existing tables
        storage.load_all_tables()?;

        Ok(storage)
    }

    /// Get the path for a table file
    fn table_path(&self, table_name: &str) -> PathBuf {
        self.data_dir.join(format!("{}.json", table_name))
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
