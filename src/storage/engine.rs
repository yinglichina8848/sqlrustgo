//! Storage Engine trait - abstraction for storage backends
//! Supports multiple storage implementations (File, Memory, etc.)

use crate::executor::TableInfo;
use crate::types::{SqlResult, Value};

/// Record type - a single row of values
pub type Record = Vec<Value>;

/// StorageEngine trait - abstraction for table storage
/// Enables multiple storage backends (FileStorage, MemoryStorage, etc.)
pub trait StorageEngine: Send + Sync {
    /// Scan all rows from a table
    fn scan(&self, table: &str) -> SqlResult<Vec<Record>>;

    /// Insert rows into a table
    fn insert(&self, table: &str, records: Vec<Record>) -> SqlResult<()>;

    /// Delete rows matching a filter
    fn delete(&self, table: &str, _filters: &[Value]) -> SqlResult<usize>;

    /// Update rows matching a filter
    fn update(
        &self,
        table: &str,
        _filters: &[Value],
        _updates: &[(usize, Value)],
    ) -> SqlResult<usize>;

    /// Create a new table
    fn create_table(&self, info: &TableInfo) -> SqlResult<()>;

    /// Drop a table
    fn drop_table(&self, table: &str) -> SqlResult<()>;

    /// Get table metadata
    fn get_table_info(&self, table: &str) -> SqlResult<TableInfo>;

    /// Check if table exists
    fn has_table(&self, table: &str) -> bool;

    /// List all tables
    fn list_tables(&self) -> Vec<String>;

    /// Create an index on a table
    fn create_index(
        &self,
        table: &str,
        column: &str,
    ) -> SqlResult<()>;

    /// Drop an index from a table
    fn drop_index(&self, table: &str, column: &str) -> SqlResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that StorageEngine trait is defined correctly
    #[test]
    fn test_storage_engine_trait_exists() {
        // Verify the trait is defined and has the expected methods
        fn _check_trait(_engine: &dyn StorageEngine) {}
    }
}
