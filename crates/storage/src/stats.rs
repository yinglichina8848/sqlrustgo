//! Statistics module for table and column statistics
//!
//! Provides basic statistics collection and storage for query optimization

use std::collections::HashMap;

/// Table statistics
#[derive(Debug, Clone)]
pub struct TableStats {
    /// Table name
    pub table_name: String,
    /// Number of rows
    pub row_count: u64,
    /// Number of data pages
    pub page_count: u32,
    /// Table size in bytes
    pub table_size: u64,
    /// Last updated timestamp
    pub last_updated: u64,
    /// Column statistics
    pub column_stats: HashMap<String, ColumnStats>,
}

impl TableStats {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name,
            row_count: 0,
            page_count: 0,
            table_size: 0,
            last_updated: 0,
            column_stats: HashMap::new(),
        }
    }

    /// Update row count
    pub fn set_row_count(&mut self, count: u64) {
        self.row_count = count;
        self.last_updated = now();
    }

    /// Update page count
    pub fn set_page_count(&mut self, count: u32) {
        self.page_count = count;
        self.last_updated = now();
    }

    /// Update table size
    pub fn set_table_size(&mut self, size: u64) {
        self.table_size = size;
        self.last_updated = now();
    }

    /// Add column statistics
    pub fn add_column_stats(&mut self, column_name: String, stats: ColumnStats) {
        self.column_stats.insert(column_name, stats);
        self.last_updated = now();
    }
}

/// Column statistics
#[derive(Debug, Clone)]
pub struct ColumnStats {
    /// Column name
    pub column_name: String,
    /// Number of null values
    pub null_count: u64,
    /// Number of distinct values
    pub distinct_count: u64,
    /// Minimum value (as bytes for comparison)
    pub min_value: Option<Vec<u8>>,
    /// Maximum value (as bytes for comparison)
    pub max_value: Option<Vec<u8>>,
    /// Average value size
    pub avg_size: f64,
}

impl ColumnStats {
    pub fn new(column_name: String) -> Self {
        Self {
            column_name,
            null_count: 0,
            distinct_count: 0,
            min_value: None,
            max_value: None,
            avg_size: 0.0,
        }
    }

    /// Update with a new value
    pub fn update(&mut self, value: Option<&[u8]>, size: usize) {
        match value {
            Some(v) => {
                // Update min/max
                if let Some(ref min) = self.min_value {
                    if v < min.as_slice() {
                        self.min_value = Some(v.to_vec());
                    }
                } else {
                    self.min_value = Some(v.to_vec());
                }

                if let Some(ref max) = self.max_value {
                    if v > max.as_slice() {
                        self.max_value = Some(v.to_vec());
                    }
                } else {
                    self.max_value = Some(v.to_vec());
                }

                // Update average size
                let total_count = self.distinct_count + self.null_count;
                if total_count > 0 {
                    self.avg_size = (self.avg_size * total_count as f64 + size as f64)
                        / (total_count + 1) as f64;
                } else {
                    self.avg_size = size as f64;
                }

                self.distinct_count += 1;
            }
            None => {
                self.null_count += 1;
            }
        }
    }
}

/// Statistics manager
pub struct StatsManager {
    /// Table statistics
    table_stats: HashMap<String, TableStats>,
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            table_stats: HashMap::new(),
        }
    }

    /// Get or create table statistics
    pub fn get_or_create_table(&mut self, table_name: &str) -> &mut TableStats {
        self.table_stats
            .entry(table_name.to_string())
            .or_insert_with(|| TableStats::new(table_name.to_string()))
    }

    /// Get table statistics
    pub fn get_table_stats(&self, table_name: &str) -> Option<&TableStats> {
        self.table_stats.get(table_name)
    }

    /// Update table row count
    pub fn update_row_count(&mut self, table_name: &str, count: u64) {
        if let Some(stats) = self.table_stats.get_mut(table_name) {
            stats.set_row_count(count);
        }
    }

    /// Update table page count
    pub fn update_page_count(&mut self, table_name: &str, count: u32) {
        if let Some(stats) = self.table_stats.get_mut(table_name) {
            stats.set_page_count(count);
        }
    }

    /// Update table size
    pub fn update_table_size(&mut self, table_name: &str, size: u64) {
        if let Some(stats) = self.table_stats.get_mut(table_name) {
            stats.set_table_size(size);
        }
    }

    /// Update column statistics
    pub fn update_column_stats(
        &mut self,
        table_name: &str,
        column_name: &str,
        value: Option<&[u8]>,
        size: usize,
    ) {
        let table_stats = self.get_or_create_table(table_name);
        let col_stats = table_stats
            .column_stats
            .entry(column_name.to_string())
            .or_insert_with(|| ColumnStats::new(column_name.to_string()));
        col_stats.update(value, size);
    }

    /// List all tables with statistics
    pub fn list_tables(&self) -> Vec<String> {
        self.table_stats.keys().cloned().collect()
    }

    /// Clear all statistics
    pub fn clear(&mut self) {
        self.table_stats.clear();
    }

    /// Remove table statistics
    pub fn remove_table(&mut self, table_name: &str) {
        self.table_stats.remove(table_name);
    }
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_stats_creation() {
        let stats = TableStats::new("test".to_string());
        assert_eq!(stats.table_name, "test");
        assert_eq!(stats.row_count, 0);
    }

    #[test]
    fn test_column_stats_update() {
        let mut stats = ColumnStats::new("id".to_string());
        stats.update(Some(&[10, 0, 0, 0]), 4);
        assert_eq!(stats.distinct_count, 1);
        assert_eq!(stats.null_count, 0);
    }

    #[test]
    fn test_column_stats_null() {
        let mut stats = ColumnStats::new("name".to_string());
        stats.update(None, 0);
        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.distinct_count, 0);
    }

    #[test]
    fn test_stats_manager() {
        let mut manager = StatsManager::new();

        // Get or create
        let table_stats = manager.get_or_create_table("users");
        assert_eq!(table_stats.table_name, "users");

        // Update row count
        manager.update_row_count("users", 1000);
        let stats = manager.get_table_stats("users").unwrap();
        assert_eq!(stats.row_count, 1000);
    }

    #[test]
    fn test_stats_manager_column() {
        let mut manager = StatsManager::new();

        manager.update_column_stats("users", "id", Some(&[1, 0, 0, 0]), 4);
        manager.update_column_stats("users", "id", Some(&[2, 0, 0, 0]), 4);

        let stats = manager.get_table_stats("users").unwrap();
        let col_stats = stats.column_stats.get("id").unwrap();
        assert_eq!(col_stats.distinct_count, 2);
    }

    #[test]
    fn test_stats_manager_list() {
        let mut manager = StatsManager::new();
        manager.get_or_create_table("users");
        manager.get_or_create_table("orders");

        let tables = manager.list_tables();
        assert_eq!(tables.len(), 2);
    }

    #[test]
    fn test_stats_manager_remove() {
        let mut manager = StatsManager::new();
        manager.get_or_create_table("users");
        manager.remove_table("users");

        assert!(manager.get_table_stats("users").is_none());
    }
}
