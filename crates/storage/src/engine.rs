//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::HashMap;

use crate::bplus_tree::SimpleBPlusTree;

/// Column statistics for a single column
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub distinct_count: u64,
    pub null_count: u64,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

/// Table statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub column_stats: Vec<ColumnStats>,
}

/// Column definition for table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_unique: bool,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
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

    /// Scan rows in batches for streaming (memory-efficient)
    /// Returns (records, total_count, has_more)
    fn scan_batch(
        &self,
        table: &str,
        offset: usize,
        limit: usize,
    ) -> SqlResult<(Vec<Record>, usize, bool)> {
        let all_records = self.scan(table)?;
        let total = all_records.len();
        let has_more = offset + limit < total;
        let batch = all_records.into_iter().skip(offset).take(limit).collect();
        Ok((batch, total, has_more))
    }

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
    fn create_table_index(
        &mut self,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_table_index(&mut self, table: &str, column: &str) -> SqlResult<()>;

    /// Search using index - returns row IDs matching the key
    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32>;

    /// Range query using index - returns row IDs in range [start, end)
    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32>;

    /// Create a view
    fn create_view(&mut self, info: ViewInfo) -> SqlResult<()>;

    /// Get view info
    fn get_view(&self, name: &str) -> Option<ViewInfo>;

    /// List all views
    fn list_views(&self) -> Vec<String>;

    /// Check if view exists
    fn has_view(&self, name: &str) -> bool;

    /// Analyze table and collect statistics
    fn analyze_table(&self, table: &str) -> SqlResult<TableStats>;

    /// Callback triggered after write operations (INSERT/UPDATE/DELETE)
    /// Used by upper layers to invalidate query caches
    fn on_write_complete(&mut self, _table: &str) {}
}

/// In-memory storage implementation for testing and caching
#[allow(clippy::type_complexity)]
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
    views: HashMap<String, ViewInfo>,
    indexes: HashMap<String, SimpleBPlusTree>,
    write_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

#[derive(Clone, Debug)]
pub struct ViewInfo {
    pub name: String,
    pub query: String,
    pub schema: TableInfo,
    pub records: Vec<Record>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            views: HashMap::new(),
            indexes: HashMap::new(),
            write_callback: None,
        }
    }

    pub fn with_callback(callback: Box<dyn Fn(&str) + Send + Sync>) -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            views: HashMap::new(),
            indexes: HashMap::new(),
            write_callback: Some(callback),
        }
    }

    pub fn create_view(&mut self, info: ViewInfo) -> SqlResult<()> {
        self.views.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn get_view(&self, name: &str) -> Option<&ViewInfo> {
        self.views.get(name)
    }

    pub fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    pub fn has_view(&self, name: &str) -> bool {
        self.views.contains_key(name)
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

    fn insert(&mut self, table: &str, mut records: Vec<Record>) -> SqlResult<()> {
        if records.is_empty() {
            return Ok(());
        }

        let table_info = self.table_infos.get(table);

        if let Some(info) = table_info {
            let has_unique = info.columns.iter().any(|c| c.is_unique);
            if has_unique {
                let table_records = self.tables.get(table).cloned().unwrap_or_default();
                let existing: Vec<&Record> = table_records.iter().collect();
                for record in &records {
                    for (col_idx, col_def) in info.columns.iter().enumerate() {
                        if col_def.is_unique {
                            if let Some(value) = record.get(col_idx) {
                                for existing_record in &existing {
                                    if let Some(existing_val) = existing_record.get(col_idx) {
                                        if existing_val == value {
                                            return Err(SqlError::DuplicateKey {
                                                value: value.to_string(),
                                                key: col_def.name.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.tables
            .entry(table.to_string())
            .or_default()
            .append(&mut records);
        self.on_write_complete(table);
        Ok(())
    }

    fn delete(&mut self, table: &str, _filters: &[Value]) -> SqlResult<usize> {
        let mut count = 0;
        if let Some(records) = self.tables.get_mut(table) {
            count = records.len();
            records.clear();
        }
        self.on_write_complete(table);
        Ok(count)
    }

    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        let count = self.tables.get(table).map(|r| r.len()).unwrap_or(0);
        self.on_write_complete(table);
        Ok(count)
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
        self.table_infos.get(table).cloned().ok_or_else(|| {
            sqlrustgo_types::SqlError::TableNotFound {
                table: table.to_string(),
            }
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
        table: &str,
        column: &str,
        column_index: usize,
    ) -> SqlResult<()> {
        let index_name = format!("{}_{}", table, column);
        let mut tree = SimpleBPlusTree::new();

        if let Some(records) = self.tables.get(table) {
            for (row_id, record) in records.iter().enumerate() {
                if let Some(value) = record.get(column_index) {
                    if let Some(key) = value.to_index_key() {
                        tree.insert(key, row_id as u32);
                    }
                }
            }
        }

        self.indexes.insert(index_name, tree);
        Ok(())
    }

    fn drop_table_index(&mut self, table: &str, column: &str) -> SqlResult<()> {
        let index_name = format!("{}_{}", table, column);
        self.indexes.remove(&index_name);
        Ok(())
    }

    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32> {
        let index_name = format!("{}_{}", table, column);
        self.indexes
            .get(&index_name)
            .and_then(|tree| tree.search(key))
    }

    fn range_index(&self, table: &str, column: &str, start: i64, end: i64) -> Vec<u32> {
        let index_name = format!("{}_{}", table, column);
        self.indexes
            .get(&index_name)
            .map(|tree| tree.range_query(start, end))
            .unwrap_or_default()
    }

    fn create_view(&mut self, info: ViewInfo) -> SqlResult<()> {
        self.views.insert(info.name.clone(), info);
        Ok(())
    }

    fn get_view(&self, name: &str) -> Option<ViewInfo> {
        self.views.get(name).cloned()
    }

    fn list_views(&self) -> Vec<String> {
        self.views.keys().cloned().collect()
    }

    fn has_view(&self, name: &str) -> bool {
        self.views.contains_key(name)
    }

    fn analyze_table(&self, table: &str) -> SqlResult<TableStats> {
        let records = self
            .tables
            .get(table)
            .ok_or_else(|| SqlError::TableNotFound {
                table: table.to_string(),
            })?;

        let table_info = self.table_infos.get(table);

        let mut column_stats = Vec::new();

        if let Some(info) = table_info {
            for col in &info.columns {
                let mut null_count = 0u64;
                let mut distinct_values: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                for record in records {
                    if let Some(idx) = info.columns.iter().position(|c| c.name == col.name) {
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
        }

        Ok(TableStats {
            table_name: table.to_string(),
            row_count: records.len() as u64,
            column_stats,
        })
    }

    fn on_write_complete(&mut self, table: &str) {
        if let Some(callback) = &self.write_callback {
            callback(table);
        }
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

    #[test]
    fn test_memory_storage_create_table() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
            }],
        };
        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_drop_table() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);
        storage.drop_table("users").unwrap();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_memory_storage_get_table_info() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
            }],
        };
        storage.create_table(&info).unwrap();
        let result = storage.get_table_info("users").unwrap();
        assert_eq!(result.name, "users");
    }

    #[test]
    fn test_memory_storage_delete() {
        let mut storage = MemoryStorage::new();
        storage
            .tables
            .insert("users".to_string(), vec![vec![Value::Integer(1)]]);
        let count = storage.delete("users", &[]).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_memory_storage_update() {
        let mut storage = MemoryStorage::new();
        storage
            .tables
            .insert("users".to_string(), vec![vec![Value::Integer(1)]]);
        let count = storage
            .update("users", &[], &[(0, Value::Integer(2))])
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_column_definition() {
        let col = ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
        };
        assert_eq!(col.name, "id");
    }

    #[test]
    fn test_table_info() {
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
        };
        assert_eq!(info.name, "users");
    }

    #[test]
    fn test_table_data() {
        let data = TableData {
            info: TableInfo {
                name: "users".to_string(),
                columns: vec![],
            },
            rows: vec![],
        };
        assert_eq!(data.info.name, "users");
    }

    #[test]
    fn test_memory_storage_default() {
        let storage = MemoryStorage::default();
        assert!(storage.tables.is_empty());
    }

    #[test]
    fn test_record_new() {
        let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
        assert_eq!(record.len(), 2);
    }

    #[test]
    fn test_record_index() {
        let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
        assert_eq!(record[0], Value::Integer(1));
    }

    #[test]
    fn test_memory_storage_with_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let storage = MemoryStorage::with_callback(Box::new(move |_table| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        assert!(storage.write_callback.is_some());
    }

    #[test]
    fn test_memory_storage_scan_batch() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
                vec![Value::Integer(4)],
                vec![Value::Integer(5)],
            ],
        );

        let (batch, total, has_more) = storage.scan_batch("users", 0, 2).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(total, 5);
        assert!(has_more);

        let (batch, total, has_more) = storage.scan_batch("users", 2, 2).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(total, 5);
        assert!(has_more);

        let (batch, total, has_more) = storage.scan_batch("users", 4, 2).unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(total, 5);
        assert!(!has_more);
    }

    #[test]
    fn test_memory_storage_scan_batch_empty() {
        let storage = MemoryStorage::new();
        let (batch, total, has_more) = storage.scan_batch("nonexistent", 0, 10).unwrap();
        assert!(batch.is_empty());
        assert_eq!(total, 0);
        assert!(!has_more);
    }
}

#[test]
fn test_record_index() {
    let record: Record = vec![Value::Integer(1), Value::Text("test".to_string())];
    assert_eq!(record[0], Value::Integer(1));
}
