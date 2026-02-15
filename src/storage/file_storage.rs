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
    fn test_file_storage_crud() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_crud");
        let _ = remove_dir_all(&temp_dir);

        // Create storage
        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // CREATE: Insert first table
        let table1 = TableData {
            info: TableInfo {
                name: "test_table".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                }],
            },
            rows: vec![vec![Value::Integer(1)]],
        };
        storage.insert_table("test_table".to_string(), table1).unwrap();

        // READ: Verify table exists
        assert!(storage.contains_table("test_table"));
        let retrieved = storage.get_table("test_table").unwrap();
        assert_eq!(retrieved.info.name, "test_table");
        assert_eq!(retrieved.rows.len(), 1);

        // UPDATE: Add more rows
        {
            let table = storage.get_table_mut("test_table").unwrap();
            table.rows.push(vec![Value::Integer(2)]);
        }
        storage.persist_table("test_table").unwrap();

        // Verify update persisted
        let updated = storage.get_table("test_table").unwrap();
        assert_eq!(updated.rows.len(), 2);

        // DELETE: Drop table
        storage.drop_table("test_table").unwrap();
        assert!(!storage.contains_table("test_table"));
        assert!(storage.get_table("test_table").is_none());

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_multiple_tables() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_multi");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Create multiple tables
        let table_a = TableData {
            info: TableInfo {
                name: "table_a".to_string(),
                columns: vec![ColumnDefinition {
                    name: "col1".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                }],
            },
            rows: vec![vec![Value::Text("hello".to_string())]],
        };
        storage.insert_table("table_a".to_string(), table_a).unwrap();

        let table_b = TableData {
            info: TableInfo {
                name: "table_b".to_string(),
                columns: vec![ColumnDefinition {
                    name: "col2".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: true,
                }],
            },
            rows: vec![
                vec![Value::Integer(100)],
                vec![Value::Integer(200)],
            ],
        };
        storage.insert_table("table_b".to_string(), table_b).unwrap();

        // Verify all tables exist
        let names = storage.table_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"table_a".to_string()));
        assert!(names.contains(&"table_b".to_string()));

        // Verify individual tables
        assert!(storage.contains_table("table_a"));
        assert!(storage.contains_table("table_b"));
        assert_eq!(storage.get_table("table_b").unwrap().rows.len(), 2);

        // Drop one table, verify other still exists
        storage.drop_table("table_a").unwrap();
        assert!(!storage.contains_table("table_a"));
        assert!(storage.contains_table("table_b"));

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_persistence() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_persist");
        let _ = remove_dir_all(&temp_dir);

        // First session: create and populate table
        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();
            let table = TableData {
                info: TableInfo {
                    name: "persistent_table".to_string(),
                    columns: vec![
                        ColumnDefinition {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                        },
                        ColumnDefinition {
                            name: "value".to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: true,
                        },
                    ],
                },
                rows: vec![
                    vec![Value::Integer(1), Value::Text("first".to_string())],
                    vec![Value::Integer(2), Value::Text("second".to_string())],
                    vec![Value::Integer(3), Value::Text("third".to_string())],
                ],
            };
            storage.insert_table("persistent_table".to_string(), table).unwrap();
        }

        // Second session: verify data persisted
        {
            let storage = FileStorage::new(temp_dir.clone()).unwrap();
            assert!(storage.contains_table("persistent_table"));

            let table = storage.get_table("persistent_table").unwrap();
            assert_eq!(table.info.columns.len(), 2);
            assert_eq!(table.rows.len(), 3);
            assert_eq!(table.rows[0], vec![Value::Integer(1), Value::Text("first".to_string())]);
            assert_eq!(table.rows[1], vec![Value::Integer(2), Value::Text("second".to_string())]);
            assert_eq!(table.rows[2], vec![Value::Integer(3), Value::Text("third".to_string())]);
        }

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_drop_nonexistent() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_drop_nonexistent");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Dropping non-existent table should not error
        let result = storage.drop_table("nonexistent");
        assert!(result.is_ok());
        assert!(!storage.contains_table("nonexistent"));

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_flush() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_flush");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table = TableData {
            info: TableInfo {
                name: "flush_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "x".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                }],
            },
            rows: vec![],
        };
        storage.insert_table("flush_test".to_string(), table).unwrap();

        // Add rows
        {
            let table = storage.get_table_mut("flush_test").unwrap();
            table.rows.push(vec![Value::Integer(42)]);
        }

        // Flush should persist changes
        storage.flush().unwrap();

        // Verify flush worked by creating new storage instance
        let storage2 = FileStorage::new(temp_dir.clone()).unwrap();
        let table = storage2.get_table("flush_test").unwrap();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0], vec![Value::Integer(42)]);

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_get_mut() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_get_mut");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table = TableData {
            info: TableInfo {
                name: "mut_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "val".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                }],
            },
            rows: vec![vec![Value::Integer(10)]],
        };
        storage.insert_table("mut_test".to_string(), table).unwrap();

        // Modify via get_mut
        {
            let mut_table = storage.get_table_mut("mut_test").unwrap();
            mut_table.rows[0] = vec![Value::Integer(99)];
        }
        storage.persist_table("mut_test").unwrap();

        // Verify
        let table = storage.get_table("mut_test").unwrap();
        assert_eq!(table.rows[0], vec![Value::Integer(99)]);

        // Cleanup
        let _ = remove_dir_all(&temp_dir);
    }
}
