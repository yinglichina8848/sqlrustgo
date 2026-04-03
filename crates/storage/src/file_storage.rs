//! File-based table storage
//! Persists table data to JSON files

use crate::bplus_tree::BPlusTree;
use crate::engine::{
    ColumnDefinition, ColumnStats, Record, StorageEngine, TableData, TableInfo, TableStats,
    TriggerInfo,
};
use crate::wal::{WalManager, WalWriter};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::RwLock;

/// File-based storage manager
#[allow(dead_code)]
pub struct FileStorage {
    /// Base directory for database files
    data_dir: PathBuf,
    /// In-memory cache of tables
    tables: HashMap<String, TableData>,
    /// B+ Tree indexes protected by RwLock for concurrent access
    indexes: RwLock<HashMap<(String, String), BPlusTree>>,
    /// Insert buffer for batch optimization (INSERT 性能优化)
    insert_buffer: HashMap<String, Vec<Record>>,
    /// Buffer threshold - flush when reaching this count
    buffer_threshold: usize,
    /// Enable buffer for batch inserts
    enable_buffer: bool,
    /// WAL writer for durability
    wal_writer: Option<WalWriter>,
    /// WAL manager for recovery
    wal_manager: Option<WalManager>,
    /// Auto-increment counters: table_name -> (column_index -> next_value)
    auto_increment_counters: RwLock<HashMap<String, HashMap<usize, i64>>>,
    /// Triggers: trigger_name -> TriggerInfo
    triggers: RwLock<HashMap<String, TriggerInfo>>,
    /// Table triggers: table_name -> Vec<trigger_name>
    table_triggers: RwLock<HashMap<String, Vec<String>>>,
}

impl FileStorage {
    /// Create a new FileStorage with the given data directory
    pub fn new(data_dir: PathBuf) -> std::io::Result<Self> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&data_dir)?;

        let mut storage = Self {
            data_dir,
            tables: HashMap::new(),
            indexes: RwLock::new(HashMap::new()),
            insert_buffer: HashMap::new(),
            buffer_threshold: 10,
            enable_buffer: true,
            wal_writer: None,
            wal_manager: None,
            auto_increment_counters: RwLock::new(HashMap::new()),
            triggers: RwLock::new(HashMap::new()),
            table_triggers: RwLock::new(HashMap::new()),
        };

        // Load existing tables
        storage.load_all_tables()?;

        // Load existing indexes
        storage.load_all_indexes()?;

        Ok(storage)
    }

    /// 从 WAL 恢复数据 (简化版 - 待完善)
    #[allow(dead_code)]
    fn recover_from_wal(&mut self, _manager: &WalManager) -> SqlResult<()> {
        // TODO: 实现 WAL 恢复
        Ok(())
    }

    /// 启用/禁用批量缓冲模式
    pub fn set_batch_mode(&mut self, enable: bool) {
        self.enable_buffer = enable;
    }

    /// 刷新所有缓冲（事务提交时调用）
    pub fn flush_all_buffers(&mut self) -> SqlResult<()> {
        let tables: Vec<String> = self.insert_buffer.keys().cloned().collect();
        for table in tables {
            self.do_flush_buffer(&table);
        }
        Ok(())
    }

    /// 直接写入（无缓冲）- 内部方法
    fn do_insert_direct(&mut self, table: &str, records: Vec<Record>) {
        if let Some(ref mut data) = self.tables.get_mut(table) {
            data.rows.extend(records);
            let table_data = data.clone();
            let _ = self.save_table(table, &table_data);
        }
    }

    /// 刷新缓冲到磁盘 - 内部方法
    fn do_flush_buffer(&mut self, table: &str) {
        if let Some(records) = self.insert_buffer.remove(table) {
            if !records.is_empty() {
                if let Some(ref mut data) = self.tables.get_mut(table) {
                    data.rows.extend(records);
                    let table_data = data.clone();
                    let _ = self.save_table(table, &table_data);
                }
            }
        }
    }

    /// Get the path for a table file
    fn table_path(&self, table_name: &str) -> PathBuf {
        self.data_dir.join(format!("{}.json", table_name))
    }

    /// Get the path for an index file
    fn index_path(&self, table_name: &str, column_name: &str) -> PathBuf {
        self.data_dir
            .join(format!("{}_idx_{}.json", table_name, column_name))
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
                            if let Ok(mut indexes) = self.indexes.write() {
                                indexes.insert(
                                    (table_name.to_string(), column_name.to_string()),
                                    index,
                                );
                            }
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
    fn save_index(
        &self,
        table_name: &str,
        column_name: &str,
        index: &BPlusTree,
    ) -> std::io::Result<()> {
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
        self.indexes
            .read()
            .map(|indexes| indexes.contains_key(&(table_name.to_string(), column_name.to_string())))
            .unwrap_or(false)
    }

    /// Get an index for a table column (read-only)
    pub fn get_index(&self, table_name: &str, column_name: &str) -> Option<BPlusTree> {
        self.indexes.read().ok().and_then(|indexes| {
            indexes
                .get(&(table_name.to_string(), column_name.to_string()))
                .cloned()
        })
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
                if let Some(key) = value.to_index_key() {
                    index.insert(key, row_id as u32);
                }
            }
        }

        // Save to disk
        self.save_index(table_name, column_name, &index)?;

        // Store in memory
        if let Ok(mut indexes) = self.indexes.write() {
            indexes.insert((table_name.to_string(), column_name.to_string()), index);
        }

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

        // Check if index exists and get a clone of it for saving
        let index_clone = {
            let has_index = self
                .indexes
                .read()
                .map(|indexes| indexes.contains_key(&key_exists))
                .unwrap_or(false);

            if !has_index {
                return Ok(());
            }

            // Get write lock, insert, and save
            if let Ok(mut indexes) = self.indexes.write() {
                if let Some(index) = indexes.get_mut(&key_exists) {
                    index.insert(key, row_id);
                    // Clone the index for saving
                    Some(index.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Save to disk outside the lock
        if let Some(index) = index_clone {
            self.save_index(table_name, column_name, &index)?;
        }

        Ok(())
    }

    /// Search using index - returns row IDs matching the key
    pub fn search_index(&self, table_name: &str, column_name: &str, key: i64) -> Option<u32> {
        self.indexes.read().ok().and_then(|indexes| {
            indexes
                .get(&(table_name.to_string(), column_name.to_string()))
                .and_then(|index| index.search(key))
        })
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
            .read()
            .ok()
            .and_then(|indexes| {
                indexes
                    .get(&(table_name.to_string(), column_name.to_string()))
                    .map(|index| index.range_query(start, end))
            })
            .unwrap_or_default()
    }

    /// Drop an index
    pub fn drop_index(&mut self, table_name: &str, column_name: &str) -> std::io::Result<()> {
        let key = (table_name.to_string(), column_name.to_string());

        if let Ok(mut indexes) = self.indexes.write() {
            indexes.remove(&key);
        }

        let path = self.index_path(table_name, column_name);
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    /// Flush all indexes to disk
    pub fn flush_indexes(&self) -> std::io::Result<()> {
        if let Ok(indexes) = self.indexes.read() {
            for ((table_name, column_name), index) in indexes.iter() {
                self.save_index(table_name, column_name, index)?;
            }
        }
        Ok(())
    }
}

/// Stored table data (for serialization)
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredTableData {
    name: String,
    columns: Vec<ColumnDefinition>,
    rows: Vec<Vec<Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ColumnDefinition;
    use std::fs::remove_dir_all;

    #[test]
    fn test_file_storage() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_file_storage");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert table with data
            let table_data = TableData {
                info: TableInfo {
                    name: "idx_test".to_string(),
                    columns: vec![
                        ColumnDefinition {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        ColumnDefinition {
                            name: "value".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                    ],
                },
                rows: vec![
                    vec![Value::Integer(1), Value::Integer(100)],
                    vec![Value::Integer(2), Value::Integer(200)],
                ],
            };
            storage
                .insert_table("idx_test".to_string(), table_data)
                .unwrap();

            // Create index on id column (column_index = 0)
            storage.create_index("idx_test", "id", 0).unwrap();

            // Test has_index
            assert!(storage.has_index("idx_test", "id"));
            assert!(!storage.has_index("idx_test", "nonexistent"));

            // Test search_index
            let row_id = storage.search_index("idx_test", "id", 1);
            assert!(row_id.is_some());

            // Test range_index
            let range_results = storage.range_index("idx_test", "id", 1, 3);
            assert!(!range_results.is_empty());

            // Test insert_with_index
            storage.insert_with_index("idx_test", "id", 3, 2).unwrap();

            // Test drop_index
            storage.drop_index("idx_test", "id").unwrap();
            assert!(!storage.has_index("idx_test", "id"));

            // Test flush_indexes
            storage.flush_indexes().unwrap();
        }

        // Load from disk
        {
            let storage = FileStorage::new(temp_dir.clone()).unwrap();
            let table = storage.get_table("idx_test").unwrap();
            assert_eq!(table.info.name, "idx_test");
            assert_eq!(table.rows.len(), 2);
        }

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_contains_and_drop() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_contains");
        let _ = remove_dir_all(&temp_dir);

        let table_data = TableData {
            info: TableInfo {
                name: "test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();
        storage
            .insert_table("test".to_string(), table_data)
            .unwrap();

        // Test contains_table
        assert!(storage.contains_table("test"));
        assert!(!storage.contains_table("nonexistent"));

        // Test table_names
        let names = storage.table_names();
        assert!(names.contains(&"test".to_string()));

        // Test drop_table
        storage.drop_table("test").unwrap();
        assert!(!storage.contains_table("test"));

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_persist() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_persist");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Create table without saving
        let table_data = TableData {
            info: TableInfo {
                name: "persist_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("persist_test".to_string(), table_data)
            .unwrap();

        // Test persist_table
        storage.persist_table("persist_test").unwrap();

        // Test flush
        storage.flush().unwrap();

        // Verify table still exists after reload
        let storage2 = FileStorage::new(temp_dir.clone()).unwrap();
        assert!(storage2.contains_table("persist_test"));

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_get_mut() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_get_mut");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table_data = TableData {
            info: TableInfo {
                name: "mutable".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("mutable".to_string(), table_data)
            .unwrap();

        // Test get_table_mut
        {
            let table = storage.get_table_mut("mutable").unwrap();
            table.rows.push(vec![Value::Integer(1)]);
        }

        let table = storage.get_table("mutable").unwrap();
        assert_eq!(table.rows.len(), 1);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_index() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_index");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Insert table with data
        let table_data = TableData {
            info: TableInfo {
                name: "idx_test".to_string(),
                columns: vec![
                    ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                    ColumnDefinition {
                        name: "value".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                ],
            },
            rows: vec![
                vec![Value::Integer(1), Value::Integer(100)],
                vec![Value::Integer(2), Value::Integer(200)],
            ],
        };
        storage
            .insert_table("idx_test".to_string(), table_data)
            .unwrap();

        // Create index on id column (column_index = 0)
        eprintln!("TEST: about to call create_index");
        storage.create_index("idx_test", "id", 0).unwrap();
        eprintln!("TEST: create_index returned");

        // Test has_index
        eprintln!("TEST: about to call has_index");
        assert!(storage.has_index("idx_test", "id"));
        eprintln!("TEST: has_index passed");
        assert!(!storage.has_index("idx_test", "nonexistent"));
        eprintln!("TEST: has_index nonexistent passed");

        // Test search_index
        eprintln!("TEST: about to call search_index");
        let row_id = storage.search_index("idx_test", "id", 1);
        eprintln!("TEST: search_index returned: {:?}", row_id);
        assert!(row_id.is_some());
        eprintln!("TEST: search_index passed");

        // Test range_index
        eprintln!("TEST: about to call range_index");
        let range_results = storage.range_index("idx_test", "id", 1, 3);
        eprintln!("TEST: range_index returned: {:?}", range_results.len());
        assert!(!range_results.is_empty());
        eprintln!("TEST: range_index passed");

        // Test insert_with_index
        eprintln!("TEST: about to call insert_with_index");
        storage.insert_with_index("idx_test", "id", 3, 2).unwrap();
        eprintln!("TEST: insert_with_index returned");

        // Test drop_index
        eprintln!("TEST: about to call drop_index");
        storage.drop_index("idx_test", "id").unwrap();
        eprintln!("TEST: drop_index returned");
        assert!(!storage.has_index("idx_test", "id"));
        eprintln!("TEST: drop_index assertion passed");

        // Test flush_indexes
        eprintln!("TEST: about to call flush_indexes");
        storage.flush_indexes().unwrap();
        eprintln!("TEST: flush_indexes returned");

        let _ = remove_dir_all(&temp_dir);
        eprintln!("TEST: test complete");
    }

    #[test]
    fn test_file_storage_index_search() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_idx_search");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Create table and index
        let table_data = TableData {
            info: TableInfo {
                name: "search_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("search_test".to_string(), table_data)
            .unwrap();
        storage.create_index("search_test", "id", 0).unwrap();

        // Insert with index
        storage
            .insert_with_index("search_test", "id", 10, 0)
            .unwrap();
        storage
            .insert_with_index("search_test", "id", 20, 1)
            .unwrap();

        // Search
        let result = storage.search_index("search_test", "id", 10);
        assert_eq!(result, Some(0));

        // Range query
        let range = storage.range_index("search_test", "id", 5, 15);
        assert!(!range.is_empty());

        let _ = remove_dir_all(&temp_dir);
    }

    // ==================== Additional Coverage Tests ====================

    #[test]
    fn test_file_storage_get_index() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_get_index");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table_data = TableData {
            info: TableInfo {
                name: "get_idx_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("get_idx_test".to_string(), table_data)
            .unwrap();
        storage.create_index("get_idx_test", "id", 0).unwrap();

        // Test get_index
        let index = storage.get_index("get_idx_test", "id");
        assert!(index.is_some());

        // Test get_index for non-existent
        let index_none = storage.get_index("get_idx_test", "nonexistent");
        assert!(index_none.is_none());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_index_no_matching_rows() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_no_match");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Create table with non-integer columns (will skip indexing)
        let table_data = TableData {
            info: TableInfo {
                name: "text_table".to_string(),
                columns: vec![ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![vec![Value::Text("Alice".to_string())]],
        };
        storage
            .insert_table("text_table".to_string(), table_data)
            .unwrap();

        // Create index - this will work but won't have any entries
        storage.create_index("text_table", "name", 0).unwrap();

        // search_index should return None for TEXT column (no Integer keys)
        let result = storage.search_index("text_table", "name", 1);
        assert_eq!(result, None);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_empty_tables() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_empty");
        let _ = remove_dir_all(&temp_dir);

        let storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Test empty storage
        assert_eq!(storage.table_names().len(), 0);
        assert!(!storage.contains_table("anything"));
        assert!(storage.get_table("anything").is_none());

        // Test flush on empty storage
        storage.flush().unwrap();
        storage.flush_indexes().unwrap();

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_persist_nonexistent() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_persist_none");
        let _ = remove_dir_all(&temp_dir);

        let storage = FileStorage::new(temp_dir.clone()).unwrap();

        // persist_table on non-existent table should return Ok
        let result = storage.persist_table("nonexistent");
        assert!(result.is_ok());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_range_index_no_results() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_range_empty");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table_data = TableData {
            info: TableInfo {
                name: "range_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("range_test".to_string(), table_data)
            .unwrap();
        storage.create_index("range_test", "id", 0).unwrap();

        // Add some data
        storage.insert_with_index("range_test", "id", 5, 0).unwrap();

        // Range with no matching results
        let range = storage.range_index("range_test", "id", 100, 200);
        assert!(range.is_empty());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_insert_with_index_no_index() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_no_idx");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let table_data = TableData {
            info: TableInfo {
                name: "no_idx_test".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            },
            rows: vec![],
        };
        storage
            .insert_table("no_idx_test".to_string(), table_data)
            .unwrap();

        // Insert with index when no index exists - should be ok (no-op)
        let result = storage.insert_with_index("no_idx_test", "id", 1, 0);
        assert!(result.is_ok());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_has_index_no_table() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_has_idx_no");
        let _ = remove_dir_all(&temp_dir);

        let storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Check index on non-existent table - should return false
        let result = storage.has_index("nonexistent", "id");
        assert!(!result);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_drop_index_no_table() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_drop_no");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Try to drop index from non-existent table - should return Ok (no-op)
        let result = storage.drop_index("nonexistent", "id");
        assert!(result.is_ok());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_range_index_no_table() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_range_no");
        let _ = remove_dir_all(&temp_dir);

        let storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Range query on non-existent table - should return empty
        let result = storage.range_index("nonexistent", "id", 0, 100);
        assert_eq!(result, Vec::<u32>::new());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_scan() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_scan");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        // Create table first
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };
        storage.create_table(&info).unwrap();

        // Insert data
        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();

        // Scan
        let records = storage.scan("users").unwrap();
        assert_eq!(records.len(), 1);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_scan_empty() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_scan_empty");
        let _ = remove_dir_all(&temp_dir);

        let storage = FileStorage::new(temp_dir.clone()).unwrap();

        let records = storage.scan("nonexistent").unwrap();
        assert!(records.is_empty());

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_list_tables() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_list");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
        };
        storage.create_table(&info).unwrap();

        let tables = storage.list_tables();
        assert!(tables.contains(&"users".to_string()));

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_delete() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_delete");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
        };
        storage.create_table(&info).unwrap();
        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();

        let count = storage.delete("users", &[]).unwrap();
        assert_eq!(count, 1);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_delete_empty() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_del_emp");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let count = storage.delete("nonexistent", &[]).unwrap();
        assert_eq!(count, 0);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_update() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_update");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
        };
        storage.create_table(&info).unwrap();
        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();

        let count = storage
            .update("users", &[], &[(0, Value::Integer(2))])
            .unwrap();
        assert_eq!(count, 1);

        let _ = remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_storage_update_empty() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_upd_emp");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

        let count = storage.update("nonexistent", &[], &[]).unwrap();
        assert_eq!(count, 0);

        let _ = remove_dir_all(&temp_dir);
    }
}

impl StorageEngine for FileStorage {
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>> {
        Ok(self
            .get_table(table)
            .map(|data| data.rows.clone())
            .unwrap_or_default())
    }

    fn get_row(&self, table: &str, row_index: usize) -> SqlResult<Option<Record>> {
        Ok(self
            .get_table(table)
            .and_then(|data| data.rows.get(row_index).cloned()))
    }

    fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
        if records.is_empty() {
            return Ok(());
        }

        // 混合模式优化：批量 >= 10 条用缓冲，单条直接写
        if self.enable_buffer && records.len() >= self.buffer_threshold {
            // 批量模式：先缓冲，达到阈值后批量写入
            self.insert_buffer
                .entry(table.to_string())
                .or_default()
                .extend(records);

            // 达到阈值，批量持久化
            if self.insert_buffer.get(table).map(|b| b.len()).unwrap_or(0) >= self.buffer_threshold
            {
                self.do_flush_buffer(table);
            }
        } else {
            // 单条模式：直接写入（低延迟优先）
            self.do_insert_direct(table, records);
        }
        Ok(())
    }

    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize> {
        if let Some(ref mut data) = self.tables.get_mut(table) {
            let count = data.rows.len();
            data.rows.clear();
            let table_data = data.clone();
            self.save_table(table, &table_data)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        Ok(self.get_table(table).map(|d| d.rows.len()).unwrap_or(0))
    }

    fn create_table(&mut self, info: &TableInfo) -> SqlResult<()> {
        let table_data = TableData {
            info: info.clone(),
            rows: Vec::new(),
        };
        self.insert_table(info.name.clone(), table_data)
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        Ok(())
    }

    fn drop_table(&mut self, table: &str) -> SqlResult<()> {
        self.drop_table(table)
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        Ok(())
    }

    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo> {
        self.get_table(table)
            .map(|t| t.info.clone())
            .ok_or_else(|| SqlError::TableNotFound {
                table: table.to_string(),
            })
    }

    fn has_table(&self, table: &str) -> bool {
        self.tables.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    fn create_table_index(
        &mut self,
        table_name: &str,
        column_name: &str,
        column_index: usize,
    ) -> SqlResult<()> {
        let table = self
            .tables
            .get(table_name)
            .ok_or_else(|| SqlError::TableNotFound {
                table: table_name.to_string(),
            })?;

        let mut index = BPlusTree::new();
        for (row_id, row) in table.rows.iter().enumerate() {
            if let Some(value) = row.get(column_index) {
                if let Some(key) = value.to_index_key() {
                    index.insert(key, row_id as u32);
                }
            }
        }

        self.save_index(table_name, column_name, &index)
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;

        if let Ok(mut indexes) = self.indexes.write() {
            indexes.insert((table_name.to_string(), column_name.to_string()), index);
        }

        Ok(())
    }

    fn drop_table_index(&mut self, table_name: &str, column_name: &str) -> SqlResult<()> {
        if let Ok(mut indexes) = self.indexes.write() {
            indexes.remove(&(table_name.to_string(), column_name.to_string()));
        }

        let path = self.index_path(table_name, column_name);
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        }

        Ok(())
    }

    fn search_index(&self, table: &str, column: &str, key: i64) -> Vec<u32> {
        // FileStorage doesn't support indexes yet
        Vec::new()
    }

    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32> {
        self.range_index(table, column, start, end)
    }

    fn create_view(&mut self, _info: crate::engine::ViewInfo) -> SqlResult<()> {
        Ok(())
    }

    fn get_view(&self, _name: &str) -> Option<crate::engine::ViewInfo> {
        None
    }

    fn list_views(&self) -> Vec<String> {
        vec![]
    }

    fn has_view(&self, _name: &str) -> bool {
        false
    }

    fn create_trigger(&mut self, info: TriggerInfo) -> SqlResult<()> {
        let name = info.name.clone();
        let table_name = info.table_name.clone();
        self.triggers.write().unwrap().insert(name.clone(), info);
        self.table_triggers
            .write()
            .unwrap()
            .entry(table_name)
            .or_default()
            .push(name);
        Ok(())
    }

    fn drop_trigger(&mut self, name: &str) -> SqlResult<()> {
        let mut triggers = self.triggers.write().unwrap();
        if let Some(info) = triggers.remove(name) {
            if let Some(table_triggers) = self
                .table_triggers
                .write()
                .unwrap()
                .get_mut(&info.table_name)
            {
                table_triggers.retain(|n| n != name);
            }
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Trigger {} not found",
                name
            )))
        }
    }

    fn get_trigger(&self, name: &str) -> Option<TriggerInfo> {
        self.triggers.read().unwrap().get(name).cloned()
    }

    fn list_triggers(&self, table: &str) -> Vec<TriggerInfo> {
        self.table_triggers
            .read()
            .unwrap()
            .get(table)
            .map(|names| {
                let triggers = self.triggers.read().unwrap();
                names
                    .iter()
                    .filter_map(|n| triggers.get(n).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn analyze_table(&self, table: &str) -> SqlResult<TableStats> {
        let table_data = self
            .tables
            .get(table)
            .ok_or_else(|| SqlError::TableNotFound {
                table: table.to_string(),
            })?;

        let records = &table_data.rows;
        let table_info = &table_data.info;

        let mut column_stats = Vec::new();

        for col in &table_info.columns {
            let mut null_count = 0u64;
            let mut distinct_values = std::collections::HashSet::new();

            if let Some(idx) = table_info.columns.iter().position(|c| c.name == col.name) {
                for record in records {
                    if let Some(val) = record.get(idx) {
                        match val {
                            Value::Null => null_count += 1,
                            _ => {
                                distinct_values.insert(val.to_string());
                            }
                        }
                    }
                }
            }

            column_stats.push(ColumnStats {
                column_name: col.name.clone(),
                distinct_count: distinct_values.len() as u64,
                null_count,
                min_value: None,
                max_value: None,
            });
        }

        Ok(TableStats {
            table_name: table.to_string(),
            row_count: records.len() as u64,
            column_stats,
        })
    }

    fn get_next_auto_increment(&mut self, table: &str, column_index: usize) -> SqlResult<i64> {
        let mut counters = self
            .auto_increment_counters
            .write()
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        let table_counters = counters
            .entry(table.to_string())
            .or_insert_with(HashMap::new);
        let next = *table_counters.entry(column_index).or_insert(0);
        table_counters.insert(column_index, next + 1);
        Ok(next + 1)
    }

    fn get_auto_increment_counter(&self, table: &str, column_index: usize) -> SqlResult<i64> {
        let counters = self
            .auto_increment_counters
            .read()
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        let table_counters = counters.get(table).ok_or_else(|| SqlError::TableNotFound {
            table: table.to_string(),
        })?;
        Ok(*table_counters.get(&column_index).unwrap_or(&0))
    }
}

#[cfg(test)]
mod storage_engine_tests {
    use super::*;
    use crate::engine::ColumnDefinition;
    use std::fs::remove_dir_all;

    // === Tests for StorageEngine trait implementation ===

    #[test]
    fn test_storage_engine_trait_get_table_info() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_get_info");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert a table
            let table_data = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    }],
                },
                rows: vec![],
            };
            storage
                .insert_table("users".to_string(), table_data)
                .unwrap();

            // Get table info through trait
            let info = storage.get_table_info("users").unwrap();
            assert_eq!(info.name, "users");
            assert_eq!(info.columns.len(), 1);
        }
    }

    #[test]
    fn test_storage_engine_trait_has_table() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_has_table");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert a table
            let table_data = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![],
                },
                rows: vec![],
            };
            storage
                .insert_table("users".to_string(), table_data)
                .unwrap();

            // Test has_table
            assert!(storage.has_table("users"));
            assert!(!storage.has_table("nonexistent"));
        }
    }

    #[test]
    fn test_storage_engine_trait_list_tables() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_list");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert multiple tables
            let table1 = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![],
                },
                rows: vec![],
            };
            let table2 = TableData {
                info: TableInfo {
                    name: "orders".to_string(),
                    columns: vec![],
                },
                rows: vec![],
            };
            storage.insert_table("users".to_string(), table1).unwrap();
            storage.insert_table("orders".to_string(), table2).unwrap();

            // Test list_tables
            let tables = storage.list_tables();
            assert!(tables.contains(&"users".to_string()));
            assert!(tables.contains(&"orders".to_string()));
        }
    }

    #[test]
    fn test_storage_engine_trait_create_table_index() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_create_idx");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert a table with data
            let table_data = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    }],
                },
                rows: vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
            };
            storage
                .insert_table("users".to_string(), table_data)
                .unwrap();

            // Create index through trait
            storage.create_table_index("users", "id", 0).unwrap();

            // Verify index exists
            assert!(storage.has_index("users", "id"));
        }
    }

    #[test]
    fn test_storage_engine_trait_drop_table_index() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_test_drop_idx");
        let _ = remove_dir_all(&temp_dir);

        {
            let mut storage = FileStorage::new(temp_dir.clone()).unwrap();

            // Insert a table and create index
            let table_data = TableData {
                info: TableInfo {
                    name: "users".to_string(),
                    columns: vec![ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    }],
                },
                rows: vec![vec![Value::Integer(1)]],
            };
            storage
                .insert_table("users".to_string(), table_data)
                .unwrap();
            storage.create_index("users", "id", 0).unwrap();

            // Drop index through trait
            storage.drop_table_index("users", "id").unwrap();

            // Verify index is dropped
            assert!(!storage.has_index("users", "id"));
        }
    }
}
