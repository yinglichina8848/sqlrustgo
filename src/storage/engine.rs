//! Storage Engine trait definition
//! Provides abstraction for multiple storage backends

use crate::executor::{TableData, TableInfo};
use crate::types::error::SqlError;
use crate::types::Value;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, SqlError>;

/// Filter predicate type for row filtering
pub type RowFilter = Box<dyn Fn(&Vec<Value>, &TableInfo) -> bool + Send + Sync>;

/// StorageEngine trait - abstraction for storage backends
#[allow(clippy::type_complexity)]
pub trait StorageEngine: Send + Sync {
    /// Get table information
    fn get_table(&self, name: &str) -> Option<TableData>;

    /// List all table names
    fn table_names(&self) -> Vec<String>;

    /// Create a new table
    fn create_table(&mut self, name: &str, info: TableInfo) -> StorageResult<()>;

    /// Drop a table
    fn drop_table(&mut self, name: &str) -> StorageResult<()>;

    /// Insert a row into a table
    fn insert(&mut self, table: &str, row: Vec<Value>) -> StorageResult<()>;

    /// Scan all rows from a table
    fn scan(&self, table: &str) -> StorageResult<Vec<Vec<Value>>>;

    /// Update rows matching a filter
    fn update(
        &mut self,
        table: &str,
        updates: Vec<(String, Value)>,
        filter: Option<RowFilter>,
    ) -> StorageResult<u32>;

    /// Delete rows matching a filter
    fn delete(&mut self, table: &str, filter: Option<RowFilter>) -> StorageResult<u32>;

    /// Get table info (metadata only)
    fn get_table_info(&self, name: &str) -> Option<TableInfo>;

    /// Check if table exists
    fn has_table(&self, name: &str) -> bool;

    /// Get row count for a table
    fn row_count(&self, table: &str) -> StorageResult<usize>;

    /// Create index on a column
    fn create_index(&mut self, table: &str, column: &str) -> StorageResult<()>;

    /// Drop index on a column
    fn drop_index(&mut self, table: &str, column: &str) -> StorageResult<()>;

    /// Search using index
    fn search_index(&self, table: &str, column: &str, key: i64) -> Option<u32>;
}

/// Schema definition for tables
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

/// Column definition
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
}

/// Data types supported in schema
#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
    Blob,
    Null,
}

impl DataType {
    #[allow(clippy::should_implement_trait)]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INTEGER" | "INT" => Some(DataType::Integer),
            "FLOAT" | "REAL" | "DOUBLE" => Some(DataType::Float),
            "TEXT" | "VARCHAR" | "CHAR" => Some(DataType::Text),
            "BOOLEAN" | "BOOL" => Some(DataType::Boolean),
            "BLOB" | "BINARY" => Some(DataType::Blob),
            "NULL" => Some(DataType::Null),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_parse() {
        assert_eq!(DataType::parse("INTEGER"), Some(DataType::Integer));
        assert_eq!(DataType::parse("TEXT"), Some(DataType::Text));
        assert_eq!(DataType::parse("FLOAT"), Some(DataType::Float));
        assert_eq!(DataType::parse("UNKNOWN"), None);
    }
}
