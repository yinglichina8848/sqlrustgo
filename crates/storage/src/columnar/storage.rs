//! Columnar Storage Engine Implementation
//!
//! A column-oriented storage engine implementing the StorageEngine trait.

use crate::columnar::chunk::{Bitmap, ColumnChunk, ColumnStats};
use crate::columnar::segment::{ColumnSegment, ColumnStatsDisk, CompressionType};
use crate::engine::{StorageEngine, TableInfo, TableStats, TriggerInfo, ViewInfo};
use crate::wal::{WalManager, WalWriter};
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Columnar storage error types
#[derive(Error, Debug)]
pub enum ColumnarError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type ColumnarResult<T> = Result<T, ColumnarError>;

/// A column-oriented storage for a single table
#[derive(Debug, Clone)]
pub struct TableStore {
    /// Table metadata
    info: TableInfo,
    /// Column chunks - each column has its own ColumnChunk
    columns: HashMap<usize, ColumnChunk>,
    /// Column names to indices mapping
    column_indices: HashMap<String, usize>,
    /// Number of rows
    row_count: usize,
}

impl TableStore {
    /// Create a new table store
    pub fn new(info: TableInfo) -> Self {
        let column_indices: HashMap<String, usize> = info
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();

        Self {
            info,
            columns: HashMap::new(),
            column_indices,
            row_count: 0,
        }
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Insert a record (as a row, will be split into columns)
    pub fn insert_row(&mut self, record: &[Value]) -> ColumnarResult<()> {
        if record.len() != self.info.columns.len() {
            return Err(ColumnarError::Storage(format!(
                "Expected {} columns, got {}",
                self.info.columns.len(),
                record.len()
            )));
        }

        for (col_idx, value) in record.iter().enumerate() {
            let chunk = self.columns.entry(col_idx).or_insert_with(ColumnChunk::new);
            if matches!(value, Value::Null) {
                chunk.push_null();
            } else {
                chunk.push(value.clone());
            }
        }

        self.row_count += 1;
        Ok(())
    }

    /// Get a row by index
    pub fn get_row(&self, row_idx: usize) -> Option<Vec<Value>> {
        if row_idx >= self.row_count {
            return None;
        }

        let mut row = Vec::with_capacity(self.info.columns.len());
        for col_idx in 0..self.info.columns.len() {
            if let Some(chunk) = self.columns.get(&col_idx) {
                row.push(chunk.get(row_idx).cloned().unwrap_or(Value::Null));
            } else {
                row.push(Value::Null);
            }
        }
        Some(row)
    }

    /// Get specific columns from rows
    pub fn scan_columns(&self, column_indices: &[usize]) -> Vec<Vec<Value>> {
        let mut result = Vec::with_capacity(self.row_count);

        for row_idx in 0..self.row_count {
            let mut row = Vec::with_capacity(column_indices.len());
            for &col_idx in column_indices {
                if let Some(chunk) = self.columns.get(&col_idx) {
                    row.push(chunk.get(row_idx).cloned().unwrap_or(Value::Null));
                } else {
                    row.push(Value::Null);
                }
            }
            result.push(row);
        }

        result
    }

    /// Get statistics for a column
    pub fn get_column_stats(&self, col_idx: usize) -> Option<&ColumnStats> {
        self.columns.get(&col_idx).map(|c| c.stats())
    }

    /// Serialize to disk
    pub fn serialize(&self, path: &PathBuf) -> ColumnarResult<()> {
        // Create directory if it doesn't exist
        fs::create_dir_all(path)?;

        // Write each column as a segment
        for (col_idx, chunk) in &self.columns {
            let segment_path = path.join(format!("column_{}.bin", col_idx));
            let mut segment =
                ColumnSegment::with_compression(*col_idx as u32, CompressionType::Zstd);

            let stats = ColumnStatsDisk::from(chunk.stats());
            segment.stats = stats;
            segment.num_values = chunk.len() as u64;

            segment
                .write_to_file(&segment_path, chunk.values(), chunk.null_bitmap())
                .map_err(|e| ColumnarError::Storage(e.to_string()))?;
        }

        // Write metadata
        let metadata_path = path.join("metadata.json");
        let metadata = TableMetadata {
            info: self.info.clone(),
            row_count: self.row_count,
            column_count: self.columns.len(),
        };
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ColumnarError::Serialization(e.to_string()))?;
        fs::write(&metadata_path, metadata_json)?;

        Ok(())
    }

    /// Deserialize from disk
    pub fn deserialize(path: &PathBuf) -> ColumnarResult<Self> {
        // Read metadata
        let metadata_path = path.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path)?;
        let metadata: TableMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| ColumnarError::Serialization(e.to_string()))?;

        let mut columns: HashMap<usize, ColumnChunk> = HashMap::new();

        // Read each column segment
        for col_idx in 0..metadata.column_count {
            let segment_path = path.join(format!("column_{}.bin", col_idx));
            if segment_path.exists() {
                let mut segment = ColumnSegment::new(col_idx as u32);
                let (values, null_bitmap) = segment
                    .read_from_file(&segment_path)
                    .map_err(|e| ColumnarError::Storage(e.to_string()))?;

                let mut chunk = ColumnChunk::with_capacity(values.len());
                for (i, value) in values.iter().enumerate() {
                    if let Some(ref bitmap) = null_bitmap {
                        if bitmap.is_null(i) {
                            chunk.push_null();
                        } else {
                            chunk.push(value.clone());
                        }
                    } else {
                        chunk.push(value.clone());
                    }
                }

                columns.insert(col_idx, chunk);
            }
        }

        let column_indices: HashMap<String, usize> = metadata
            .info
            .columns
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();

        Ok(Self {
            info: metadata.info,
            columns,
            column_indices,
            row_count: metadata.row_count,
        })
    }
}

/// Metadata for a table store
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TableMetadata {
    info: TableInfo,
    row_count: usize,
    column_count: usize,
}

/// ColumnarStorage - A column-oriented storage engine
pub struct ColumnarStorage {
    /// Base path for storage
    base_path: PathBuf,
    /// Tables stored in memory (for now, can be persisted)
    tables: HashMap<String, TableStore>,
    /// WAL for durability (optional) - not included in Debug
    #[allow(dead_code)]
    wal_manager: Option<WalManager>,
}

impl Debug for ColumnarStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnarStorage")
            .field("base_path", &self.base_path)
            .field("tables", &self.tables)
            .finish()
    }
}

impl ColumnarStorage {
    /// Create a new columnar storage with memory only
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::new(),
            tables: HashMap::new(),
            wal_manager: None,
        }
    }

    /// Create a new columnar storage with persistence
    pub fn with_persistence(base_path: PathBuf) -> ColumnarResult<Self> {
        if !base_path.exists() {
            fs::create_dir_all(&base_path)?;
        }

        Ok(Self {
            base_path,
            tables: HashMap::new(),
            wal_manager: None,
        })
    }

    /// Create a new columnar storage with WAL
    pub fn with_wal(base_path: PathBuf, wal_manager: WalManager) -> ColumnarResult<Self> {
        let mut storage = Self::with_persistence(base_path)?;
        storage.wal_manager = Some(wal_manager);
        Ok(storage)
    }

    /// Get table store path
    fn get_table_path(&self, table: &str) -> PathBuf {
        self.base_path.join(format!("columnar_{}", table))
    }

    /// Check if a table is loaded in memory
    fn is_table_loaded(&self, table: &str) -> bool {
        self.tables.contains_key(table)
    }

    /// Load a table from disk into memory
    fn load_table(&mut self, table: &str) -> ColumnarResult<()> {
        if self.is_table_loaded(table) {
            return Ok(());
        }

        let path = self.get_table_path(table);
        if path.exists() {
            let store = TableStore::deserialize(&path)?;
            self.tables.insert(table.to_string(), store);
        }

        Ok(())
    }
}

impl Default for ColumnarStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for ColumnarStorage {
    fn scan(&self, table: &str) -> crate::engine::SqlResult<Vec<Vec<Value>>> {
        let store = self.tables.get(table).ok_or_else(|| {
            crate::engine::SqlError::ExecutionError(format!("Table not found: {}", table))
        })?;

        let mut records = Vec::with_capacity(store.row_count());
        for i in 0..store.row_count() {
            if let Some(row) = store.get_row(i) {
                records.push(row);
            }
        }
        Ok(records)
    }

    fn insert(&mut self, table: &str, records: Vec<Vec<Value>>) -> crate::engine::SqlResult<()> {
        // Load table if not in memory
        if !self.is_table_loaded(table) {
            self.load_table(table)
                .map_err(|e| crate::engine::SqlError::ExecutionError(e.to_string()))?;
        }

        let store = self.tables.get_mut(table).ok_or_else(|| {
            crate::engine::SqlError::ExecutionError(format!("Table not found: {}", table))
        })?;

        for record in records {
            store
                .insert_row(&record)
                .map_err(|e| crate::engine::SqlError::ExecutionError(e.to_string()))?;
        }

        // Persist if we have a base path
        if !self.base_path.as_os_str().is_empty() {
            // Drop mutable borrow of store before calling get_table_path
            drop(store);
            let path = self.get_table_path(table);
            if let Some(store) = self.tables.get_mut(table) {
                store
                    .serialize(&path)
                    .map_err(|e| crate::engine::SqlError::ExecutionError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn delete(&mut self, table: &str, _filters: &[Value]) -> crate::engine::SqlResult<usize> {
        // For now, not implemented - would require creating new ColumnChunks without deleted rows
        Err(crate::engine::SqlError::ExecutionError(
            "DELETE not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn update(
        &mut self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> crate::engine::SqlResult<usize> {
        Err(crate::engine::SqlError::ExecutionError(
            "UPDATE not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn create_table(&mut self, info: &TableInfo) -> crate::engine::SqlResult<()> {
        let table_name = info.name.clone();
        let store = TableStore::new(info.clone());
        self.tables.insert(table_name, store);

        // Create on disk if we have a base path
        if !self.base_path.as_os_str().is_empty() {
            let path = self.get_table_path(&info.name);
            fs::create_dir_all(&path)
                .map_err(|e| crate::engine::SqlError::ExecutionError(e.to_string()))?;
        }

        Ok(())
    }

    fn drop_table(&mut self, table: &str) -> crate::engine::SqlResult<()> {
        self.tables.remove(table).ok_or_else(|| {
            crate::engine::SqlError::ExecutionError(format!("Table not found: {}", table))
        })?;

        // Remove from disk
        if !self.base_path.as_os_str().is_empty() {
            let path = self.get_table_path(table);
            if path.exists() {
                fs::remove_dir_all(&path)
                    .map_err(|e| crate::engine::SqlError::ExecutionError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn get_table_info(&self, table: &str) -> crate::engine::SqlResult<TableInfo> {
        self.tables
            .get(table)
            .map(|s| s.info.clone())
            .ok_or_else(|| {
                crate::engine::SqlError::ExecutionError(format!("Table not found: {}", table))
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
        _table: &str,
        _column: &str,
        _column_index: usize,
    ) -> crate::engine::SqlResult<()> {
        // Index creation not yet implemented for columnar storage
        Err(crate::engine::SqlError::ExecutionError(
            "Index creation not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn drop_table_index(&mut self, _table: &str, _column: &str) -> crate::engine::SqlResult<()> {
        Err(crate::engine::SqlError::ExecutionError(
            "Index dropping not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn search_index(&self, _table: &str, _column: &str, _key: i64) -> Option<u32> {
        None
    }

    fn range_index(&self, _table: &str, _column: &str, _start: i64, _end: i64) -> Vec<u32> {
        Vec::new()
    }

    fn create_view(&mut self, _info: ViewInfo) -> crate::engine::SqlResult<()> {
        Err(crate::engine::SqlError::ExecutionError(
            "Views not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn get_view(&self, _name: &str) -> Option<ViewInfo> {
        None
    }

    fn list_views(&self) -> Vec<String> {
        Vec::new()
    }

    fn has_view(&self, _name: &str) -> bool {
        false
    }

    fn create_trigger(&mut self, _info: TriggerInfo) -> crate::engine::SqlResult<()> {
        Err(crate::engine::SqlError::ExecutionError(
            "Triggers not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn drop_trigger(&mut self, _name: &str) -> crate::engine::SqlResult<()> {
        Err(crate::engine::SqlError::ExecutionError(
            "Trigger dropping not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn get_trigger(&self, _name: &str) -> Option<TriggerInfo> {
        None
    }

    fn list_triggers(&self, _table: &str) -> Vec<TriggerInfo> {
        Vec::new()
    }

    fn analyze_table(&self, _table: &str) -> crate::engine::SqlResult<TableStats> {
        Err(crate::engine::SqlError::ExecutionError(
            "Table analysis not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn get_next_auto_increment(
        &mut self,
        _table: &str,
        _column_index: usize,
    ) -> crate::engine::SqlResult<i64> {
        Err(crate::engine::SqlError::ExecutionError(
            "Auto-increment not yet implemented for ColumnarStorage".to_string(),
        ))
    }

    fn get_auto_increment_counter(
        &self,
        _table: &str,
        _column_index: usize,
    ) -> crate::engine::SqlResult<i64> {
        Err(crate::engine::SqlError::ExecutionError(
            "Auto-increment not yet implemented for ColumnarStorage".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{ColumnDefinition, TableInfo};
    use std::sync::Arc;

    fn create_test_table_info() -> TableInfo {
        TableInfo {
            name: "test_table".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: true,
                    references: None,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "value".to_string(),
                    data_type: "FLOAT".to_string(),
                    nullable: true,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                },
            ],
        }
    }

    #[test]
    fn test_columnar_storage_new() {
        let storage = ColumnarStorage::new();
        assert!(storage.list_tables().is_empty());
    }

    #[test]
    fn test_create_and_drop_table() {
        let mut storage = ColumnarStorage::new();
        let info = create_test_table_info();

        storage.create_table(&info).unwrap();
        assert!(storage.has_table("test_table"));
        assert_eq!(storage.list_tables(), vec!["test_table"]);

        storage.drop_table("test_table").unwrap();
        assert!(!storage.has_table("test_table"));
    }

    #[test]
    fn test_insert_and_scan() {
        let mut storage = ColumnarStorage::new();
        let info = create_test_table_info();
        storage.create_table(&info).unwrap();

        let records = vec![
            vec![
                Value::Integer(1),
                Value::Text("Alice".to_string()),
                Value::Float(3.14),
            ],
            vec![
                Value::Integer(2),
                Value::Text("Bob".to_string()),
                Value::Float(2.71),
            ],
            vec![Value::Integer(3), Value::Null, Value::Float(1.41)],
        ];

        storage.insert("test_table", records).unwrap();

        let scanned = storage.scan("test_table").unwrap();
        assert_eq!(scanned.len(), 3);
        assert_eq!(scanned[0][0], Value::Integer(1));
        assert_eq!(scanned[1][1], Value::Text("Bob".to_string()));
        assert_eq!(scanned[2][1], Value::Null);
    }

    #[test]
    fn test_table_not_found() {
        let storage = ColumnarStorage::new();
        let result = storage.scan("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_columnar_storage_default() {
        let storage: ColumnarStorage = Default::default();
        assert!(storage.list_tables().is_empty());
    }

    #[test]
    fn test_table_store_row_count() {
        let info = create_test_table_info();
        let mut store = TableStore::new(info);

        store
            .insert_row(&[
                Value::Integer(1),
                Value::Text("A".to_string()),
                Value::Float(1.0),
            ])
            .unwrap();
        store
            .insert_row(&[
                Value::Integer(2),
                Value::Text("B".to_string()),
                Value::Float(2.0),
            ])
            .unwrap();

        assert_eq!(store.row_count(), 2);
    }

    #[test]
    fn test_table_store_get_row() {
        let info = create_test_table_info();
        let mut store = TableStore::new(info);

        store
            .insert_row(&[
                Value::Integer(1),
                Value::Text("Alice".to_string()),
                Value::Float(3.14),
            ])
            .unwrap();
        store
            .insert_row(&[
                Value::Integer(2),
                Value::Text("Bob".to_string()),
                Value::Null,
            ])
            .unwrap();

        let row0 = store.get_row(0).unwrap();
        assert_eq!(row0[0], Value::Integer(1));
        assert_eq!(row0[1], Value::Text("Alice".to_string()));

        let row1 = store.get_row(1).unwrap();
        assert_eq!(row1[0], Value::Integer(2));
        assert_eq!(row1[2], Value::Null);

        assert!(store.get_row(2).is_none());
    }
}
