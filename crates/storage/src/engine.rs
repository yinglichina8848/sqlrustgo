//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::collections::{HashMap, HashSet};

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

/// Trigger timing: BEFORE or AFTER
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerTiming {
    Before,
    After,
}

/// Trigger event: INSERT, UPDATE, or DELETE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerInfo {
    pub name: String,
    pub table_name: String,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub body: String,
}

/// Partition type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartitionType {
    Range,
    List,
    Hash,
}

/// Partition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub partition_type: PartitionType,
    pub column: String,
    pub boundaries: Vec<Value>,
}

impl PartitionInfo {
    pub fn new_range(column: &str, boundaries: Vec<Value>) -> Self {
        Self {
            partition_type: PartitionType::Range,
            column: column.to_string(),
            boundaries,
        }
    }

    pub fn new_list(column: &str, values: Vec<Value>) -> Self {
        Self {
            partition_type: PartitionType::List,
            column: column.to_string(),
            boundaries: values,
        }
    }

    pub fn new_hash(column: &str, num_partitions: u32) -> Self {
        Self {
            partition_type: PartitionType::Hash,
            column: column.to_string(),
            boundaries: vec![Value::Integer(num_partitions as i64)],
        }
    }

    pub fn get_partition_index(&self, value: &Value) -> Option<usize> {
        match self.partition_type {
            PartitionType::Range => self.get_range_partition(value),
            PartitionType::List => self.get_list_partition(value),
            PartitionType::Hash => self.get_hash_partition(value),
        }
    }

    fn get_range_partition(&self, value: &Value) -> Option<usize> {
        if let Value::Integer(n) = value {
            for (i, boundary) in self.boundaries.iter().enumerate() {
                if let Value::Integer(b) = boundary {
                    if n < b {
                        return Some(i);
                    }
                }
            }
            Some(self.boundaries.len())
        } else {
            None
        }
    }

    fn get_list_partition(&self, value: &Value) -> Option<usize> {
        for (i, boundary) in self.boundaries.iter().enumerate() {
            if value == boundary {
                return Some(i);
            }
        }
        None
    }

    fn get_hash_partition(&self, value: &Value) -> Option<usize> {
        if let Value::Integer(n) = value {
            let num_partitions = self.boundaries.first()?.as_integer()? as u64;
            let hash = n.unsigned_abs() % num_partitions;
            Some(hash as usize)
        } else if let Value::Text(s) = value {
            let num_partitions = self.boundaries.first()?.as_integer()? as u32;
            let hash = calculate_hash(s.as_bytes()) % num_partitions;
            Some(hash as usize)
        } else {
            None
        }
    }
}

fn calculate_hash(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32).wrapping_mul(31))
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
    #[serde(skip)]
    pub partition_info: Option<PartitionInfo>,
}

/// Column definition for table schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub primary_key: bool,
}

impl ColumnDefinition {
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
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

    /// Create a trigger on a table
    fn create_trigger(&mut self, info: TriggerInfo) -> SqlResult<()>;

    /// Drop a trigger by name
    fn drop_trigger(&mut self, name: &str) -> SqlResult<()>;

    /// Get a trigger by name
    fn get_trigger(&self, name: &str) -> Option<TriggerInfo>;

    /// List all triggers for a table
    fn list_triggers(&self, table: &str) -> Vec<TriggerInfo>;

    /// List all indexes for a table, returns Vec of (column_name, index_name)
    fn list_indexes(&self, table: &str) -> Vec<(String, String)>;

    /// Check if a view exists
    fn has_view(&self, name: &str) -> bool;
}

/// In-memory storage implementation for testing and caching
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
    triggers: HashMap<String, TriggerInfo>,
    views: HashSet<String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
            triggers: HashMap::new(),
            views: HashSet::new(),
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
            .ok_or_else(|| SqlError::ExecutionError(format!("Table not found: {}", table)))
    }

    fn has_table(&self, table: &str) -> bool {
        self.table_infos.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.table_infos.keys().cloned().collect()
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
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Cannot add column: table {} not found",
                table
            )))
        }
    }

    fn rename_table(&mut self, table: &str, new_name: &str) -> SqlResult<()> {
        let info = self.table_infos.remove(table);
        let records = self.tables.remove(table);
        if let (Some(info), Some(records)) = (info, records) {
            let mut new_info = info;
            new_info.name = new_name.to_string();
            self.table_infos.insert(new_name.to_string(), new_info);
            self.tables.insert(new_name.to_string(), records);
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Cannot rename table: table {} not found",
                table
            )))
        }
    }

    fn create_trigger(&mut self, info: TriggerInfo) -> SqlResult<()> {
        self.triggers.insert(info.name.clone(), info);
        Ok(())
    }

    fn drop_trigger(&mut self, name: &str) -> SqlResult<()> {
        self.triggers
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| SqlError::ExecutionError(format!("Trigger not found: {}", name)))
    }

    fn get_trigger(&self, name: &str) -> Option<TriggerInfo> {
        self.triggers.get(name).cloned()
    }

    fn list_triggers(&self, table: &str) -> Vec<TriggerInfo> {
        self.triggers
            .values()
            .filter(|t| t.table_name == table)
            .cloned()
            .collect()
    }

    fn has_view(&self, name: &str) -> bool {
        self.views.contains(name)
    }

    fn list_indexes(&self, _table: &str) -> Vec<(String, String)> {
        Vec::new()
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
    fn test_memory_storage_create_and_drop() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info).unwrap();
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
    fn test_storage_engine_create_and_drop_table() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };

        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
        assert_eq!(storage.list_tables(), vec!["users"]);

        storage.drop_table("users").unwrap();
        assert!(!storage.has_table("users"));
    }

    #[test]
    fn test_storage_engine_get_table_info() {
        let mut storage = MemoryStorage::new();
        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: true,
            }],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };

        storage.create_table(&info).unwrap();
        let retrieved = storage.get_table_info("users").unwrap();
        assert_eq!(retrieved.name, "users");
        assert_eq!(retrieved.columns.len(), 1);
    }

    #[test]
    fn test_storage_engine_insert_records() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert("users".to_string(), vec![]);

        storage
            .insert("users", vec![vec![Value::Integer(1)]])
            .unwrap();
        let records = storage.scan("users").unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_storage_engine_delete_all() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
        );

        let deleted = storage.delete("users", &[]).unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn test_storage_engine_update_values() {
        let mut storage = MemoryStorage::new();
        storage.tables.insert(
            "users".to_string(),
            vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
        );

        let updated = storage
            .update("users", &[], &[(1, Value::Text("Bob".to_string()))][..])
            .unwrap();
        assert_eq!(updated, 1);
    }

    #[test]
    fn test_storage_engine_table_operations() {
        let mut storage = MemoryStorage::new();
        let info1 = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };
        let info2 = TableInfo {
            name: "orders".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info1).unwrap();
        storage.create_table(&info2).unwrap();

        let tables = storage.list_tables();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"orders".to_string()));
    }

    #[test]
    fn test_storage_engine_has_table_check() {
        let mut storage = MemoryStorage::new();
        assert!(!storage.has_table("users"));

        let info = TableInfo {
            name: "users".to_string(),
            columns: vec![],
            foreign_keys: vec![],
            unique_constraints: vec![],
            partition_info: None,
        };
        storage.create_table(&info).unwrap();
        assert!(storage.has_table("users"));
    }

    #[test]
    fn test_storage_engine_table_not_found() {
        let storage = MemoryStorage::new();
        let result = storage.get_table_info("nonexistent");
        assert!(result.is_err());
    }
}
