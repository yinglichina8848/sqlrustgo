//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Storage error type
#[derive(Error, Debug)]
pub enum SqlError {
    #[error("Table not found: {0}")]
    TableNotFound(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type SqlResult<T> = Result<T, SqlError>;

impl From<std::io::Error> for SqlError {
    fn from(e: std::io::Error) -> Self {
        SqlError::IoError(e.to_string())
    }
}

/// SQL Value enum - minimal version for storage crate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Boolean(_) => "BOOLEAN",
            Value::Integer(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::Text(_) => "TEXT",
            Value::Blob(_) => "BLOB",
        }
    }

    /// Convert value to index key (i64)
    pub fn to_index_key(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            Value::Text(s) => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                s.hash(&mut hasher);
                Some(hasher.finish() as i64)
            }
            _ => None,
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

impl TableInfo {
    pub fn new(name: impl Into<String>, columns: Vec<ColumnDefinition>) -> Self {
        Self {
            name: name.into(),
            columns,
        }
    }
}

/// Table data with rows
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
    fn create_index(&self, table: &str, column: &str, _column_index: usize) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_index(&self, table: &str, column: &str) -> SqlResult<()>;
}

/// In-memory storage implementation for testing and caching
pub struct MemoryStorage {
    tables: HashMap<String, Vec<Record>>,
    table_infos: HashMap<String, TableInfo>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_infos: HashMap::new(),
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
            .or_insert_with(Vec::new)
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
        self.tables
            .entry(info.name.clone())
            .or_insert_with(Vec::new);
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
            .ok_or_else(|| SqlError::TableNotFound(table.to_string()))
    }

    fn has_table(&self, table: &str) -> bool {
        self.tables.contains_key(table)
    }

    fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    fn create_index(&self, _table: &str, _column: &str, _column_index: usize) -> SqlResult<()> {
        Ok(())
    }

    fn drop_index(&self, _table: &str, _column: &str) -> SqlResult<()> {
        Ok(())
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
    fn test_value_type_name() {
        assert_eq!(Value::Null.type_name(), "NULL");
        assert_eq!(Value::Boolean(true).type_name(), "BOOLEAN");
        assert_eq!(Value::Integer(42).type_name(), "INTEGER");
    }

    #[test]
    fn test_table_info_new() {
        let cols = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
        ];
        let info = TableInfo::new("users", cols);
        assert_eq!(info.name, "users");
        assert_eq!(info.columns.len(), 1);
    }
}
