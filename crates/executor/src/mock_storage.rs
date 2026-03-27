//! Mock Storage Implementation for Testing
//!
//! This module provides an in-memory mock storage implementation
//! that can be used for testing executor operations without
//! requiring a real database connection.

use sqlrustgo_storage::engine::{
    ColumnDefinition, ColumnStats, StorageEngine, TableInfo, TableStats, TriggerInfo, ViewInfo,
};
use sqlrustgo_types::{SqlResult, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// MockStorage - In-memory storage for testing
///
/// This is a simple in-memory storage implementation that stores
/// data in a HashMap. It implements the StorageEngine trait
/// for use with the executor.
#[derive(Clone)]
pub struct MockStorage {
    tables: Arc<RwLock<HashMap<String, Vec<Vec<Value>>>>>,
    table_infos: Arc<RwLock<HashMap<String, TableInfo>>>,
    auto_increment_counters: Arc<RwLock<HashMap<String, HashMap<usize, i64>>>>,
}

impl MockStorage {
    /// Create a new empty MockStorage
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            table_infos: Arc::new(RwLock::new(HashMap::new())),
            auto_increment_counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a MockStorage with pre-populated data
    pub fn with_data(table_name: &str, data: Vec<Vec<Value>>) -> Self {
        let storage = Self::new();
        storage.put_table_data(table_name, data);
        storage
    }

    /// Put table data into storage
    pub fn put_table_data(&self, table_name: &str, data: Vec<Vec<Value>>) {
        let mut tables = self.tables.write().unwrap();
        tables.insert(table_name.to_string(), data);
    }

    /// Get table data from storage
    pub fn get_table_data(&self, table_name: &str) -> Option<Vec<Vec<Value>>> {
        let tables = self.tables.read().unwrap();
        tables.get(table_name).cloned()
    }

    /// Add a table schema
    pub fn add_schema(
        &self,
        table_name: &str,
        columns: Vec<(String, sqlrustgo_planner::DataType)>,
    ) {
        let column_defs: Vec<ColumnDefinition> = columns
            .iter()
            .map(|(name, data_type)| ColumnDefinition {
                name: name.clone(),
                data_type: format!("{:?}", data_type),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            })
            .collect();

        let table_info = TableInfo {
            name: table_name.to_string(),
            columns: column_defs,
        };

        let mut infos = self.table_infos.write().unwrap();
        infos.insert(table_name.to_string(), table_info);
    }

    /// Clear all data
    pub fn clear(&self) {
        let mut tables = self.tables.write().unwrap();
        tables.clear();
        let mut infos = self.table_infos.write().unwrap();
        infos.clear();
    }

    /// Get the number of tables
    pub fn table_count(&self) -> usize {
        let tables = self.tables.read().unwrap();
        tables.len()
    }

    /// Check if a table exists
    pub fn has_table(&self, table_name: &str) -> bool {
        let tables = self.tables.read().unwrap();
        tables.contains_key(table_name)
    }

    /// Get table row count
    pub fn row_count(&self, table_name: &str) -> usize {
        let tables = self.tables.read().unwrap();
        tables.get(table_name).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageEngine for MockStorage {
    fn scan(&self, table_name: &str) -> SqlResult<Vec<Vec<Value>>> {
        let tables = self.tables.read().unwrap();
        Ok(tables.get(table_name).cloned().unwrap_or_default())
    }

    fn insert(&mut self, table_name: &str, values: Vec<Vec<Value>>) -> SqlResult<()> {
        let mut tables = self.tables.write().unwrap();
        let table_data = tables.entry(table_name.to_string()).or_default();
        table_data.extend(values);
        Ok(())
    }

    fn create_table(&mut self, table_info: &TableInfo) -> SqlResult<()> {
        let mut tables = self.tables.write().unwrap();
        tables.entry(table_info.name.clone()).or_default();

        let mut infos = self.table_infos.write().unwrap();
        infos.insert(table_info.name.clone(), table_info.clone());

        Ok(())
    }

    fn drop_table(&mut self, table_name: &str) -> SqlResult<()> {
        let mut tables = self.tables.write().unwrap();
        tables.remove(table_name);

        let mut infos = self.table_infos.write().unwrap();
        infos.remove(table_name);

        Ok(())
    }

    fn has_table(&self, table_name: &str) -> bool {
        let tables = self.tables.read().unwrap();
        tables.contains_key(table_name)
    }

    fn get_table_info(&self, table_name: &str) -> SqlResult<TableInfo> {
        let infos = self.table_infos.read().unwrap();
        infos
            .get(table_name)
            .cloned()
            .ok_or_else(|| sqlrustgo_types::SqlError::TableNotFound {
                table: table_name.to_string(),
            })
    }

    fn list_tables(&self) -> Vec<String> {
        let tables = self.tables.read().unwrap();
        tables.keys().cloned().collect()
    }

    fn delete(&mut self, _table: &str, _filters: &[Value]) -> SqlResult<usize> {
        Ok(0)
    }

    fn update(
        &mut self,
        _table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize> {
        Ok(0)
    }

    fn create_table_index(
        &mut self,
        _table: &str,
        _column: &str,
        _column_index: usize,
    ) -> SqlResult<()> {
        Ok(())
    }

    fn drop_table_index(&mut self, _table: &str, _column: &str) -> SqlResult<()> {
        Ok(())
    }

    fn search_index(&self, _table: &str, _column: &str, _key: i64) -> Option<u32> {
        None
    }

    fn range_index(&self, _table: &str, _column: &str, _start: i64, _end: i64) -> Vec<u32> {
        Vec::new()
    }

    fn create_view(&mut self, _info: ViewInfo) -> SqlResult<()> {
        Ok(())
    }

    fn get_view(&self, _name: &str) -> Option<ViewInfo> {
        None
    }

    fn list_views(&self) -> Vec<String> {
        vec![]
    }

    fn has_view(&self, _name: &str) -> bool {
        false
    }

    fn create_trigger(&mut self, _info: TriggerInfo) -> SqlResult<()> {
        Ok(())
    }

    fn drop_trigger(&mut self, _name: &str) -> SqlResult<()> {
        Ok(())
    }

    fn get_trigger(&self, _name: &str) -> Option<TriggerInfo> {
        None
    }

    fn list_triggers(&self, _table: &str) -> Vec<TriggerInfo> {
        vec![]
    }

    fn analyze_table(&self, table: &str) -> SqlResult<TableStats> {
        let tables = self.tables.read().unwrap();
        let records =
            tables
                .get(table)
                .cloned()
                .ok_or_else(|| sqlrustgo_types::SqlError::TableNotFound {
                    table: table.to_string(),
                })?;

        let table_infos = self.table_infos.read().unwrap();
        let table_info = table_infos.get(table).cloned().ok_or_else(|| {
            sqlrustgo_types::SqlError::TableNotFound {
                table: table.to_string(),
            }
        })?;

        let mut column_stats = Vec::new();

        for col in &table_info.columns {
            let mut null_count = 0u64;
            let mut distinct_values = std::collections::HashSet::new();

            if let Some(idx) = table_info.columns.iter().position(|c| c.name == col.name) {
                for record in &records {
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
            .map_err(|e| sqlrustgo_types::SqlError::ExecutionError(e.to_string()))?;
        let table_counters = counters
            .entry(table.to_string())
            .or_insert_with(HashMap::new);
        let next = table_counters.entry(column_index).or_insert(0).clone();
        table_counters.insert(column_index, next + 1);
        Ok(next + 1)
    }

    fn get_auto_increment_counter(&self, table: &str, column_index: usize) -> SqlResult<i64> {
        let counters = self
            .auto_increment_counters
            .read()
            .map_err(|e| sqlrustgo_types::SqlError::ExecutionError(e.to_string()))?;
        let table_counters =
            counters
                .get(table)
                .ok_or_else(|| sqlrustgo_types::SqlError::TableNotFound {
                    table: table.to_string(),
                })?;
        Ok(*table_counters.get(&column_index).unwrap_or(&0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_new() {
        let storage = MockStorage::new();
        assert_eq!(storage.table_count(), 0);
    }

    #[test]
    fn test_mock_storage_with_data() {
        let data = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let storage = MockStorage::with_data("users", data);
        assert!(storage.has_table("users"));
        assert_eq!(storage.row_count("users"), 2);
    }

    #[test]
    fn test_mock_storage_put_get() {
        let storage = MockStorage::new();
        let data = vec![vec![Value::Integer(1)]];
        storage.put_table_data("test", data.clone());
        assert_eq!(storage.get_table_data("test"), Some(data));
    }

    #[test]
    fn test_mock_storage_clear() {
        let storage = MockStorage::new();
        storage.put_table_data("test", vec![vec![Value::Integer(1)]]);
        storage.clear();
        assert_eq!(storage.table_count(), 0);
    }

    #[test]
    fn test_mock_storage_scan() {
        let storage = MockStorage::with_data(
            "users",
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );

        let result = storage.scan("users").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_mock_storage_insert() {
        let mut storage = MockStorage::new();
        storage
            .create_table(&TableInfo {
                name: "users".to_string(),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();

        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();
        assert_eq!(storage.row_count("users"), 1);
    }

    #[test]
    fn test_mock_storage_clone() {
        let storage = MockStorage::new();
        storage.put_table_data("test", vec![vec![Value::Integer(1)]]);

        let cloned = storage.clone();
        assert!(cloned.has_table("test"));
    }

    #[test]
    fn test_mock_storage_drop_table() {
        let mut storage = MockStorage::new();
        storage.put_table_data("test", vec![vec![Value::Integer(1)]]);

        storage.drop_table("test").unwrap();

        assert!(!storage.has_table("test"));
    }

    #[test]
    fn test_mock_storage_get_table_info() {
        let storage = MockStorage::new();
        storage.add_schema(
            "users",
            vec![("id".to_string(), sqlrustgo_planner::DataType::Integer)],
        );

        let info = storage.get_table_info("users").unwrap();
        assert_eq!(info.name, "users");
    }

    #[test]
    fn test_mock_storage_get_table_info_not_found() {
        let storage = MockStorage::new();
        let result = storage.get_table_info("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_storage_list_tables() {
        let storage = MockStorage::new();
        storage.put_table_data("users", vec![]);
        storage.put_table_data("orders", vec![]);

        let tables = storage.list_tables();
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn test_mock_storage_delete() {
        let mut storage = MockStorage::new();
        storage.put_table_data("users", vec![vec![Value::Integer(1)]]);

        let deleted = storage.delete("users", &[]).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_mock_storage_update() {
        let mut storage = MockStorage::new();
        storage.put_table_data("users", vec![vec![Value::Integer(1)]]);

        let updated = storage
            .update("users", &[], &[(0, Value::Integer(2))])
            .unwrap();
        assert_eq!(updated, 0);
    }

    #[test]
    fn test_mock_storage_create_index() {
        let mut storage = MockStorage::new();
        let result = storage.create_table_index("users", "id", 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_storage_drop_index() {
        let mut storage = MockStorage::new();
        let result = storage.drop_table_index("users", "id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_storage_default() {
        let storage: MockStorage = Default::default();
        assert_eq!(storage.table_count(), 0);
    }

    #[test]
    fn test_mock_storage_scan_nonexistent() {
        let storage = MockStorage::new();
        let result = storage.scan("nonexistent").unwrap();
        assert!(result.is_empty());
    }
}
