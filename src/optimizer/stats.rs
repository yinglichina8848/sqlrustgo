//! Statistics Provider Module
//!
//! # What (是什么)
//! 统计信息提供者模块，为 CBO (基于成本的优化) 提供表级统计信息
//!
//! # Why (为什么)
//! 优化器需要表级统计信息来估算查询成本，从而做出最优的执行计划决策
//!
//! # How (如何实现)
//! - StatisticsProvider trait: 统计信息提供者接口
//! - TableStats: 表级统计信息结构
//! - ColumnStats: 列级统计信息结构

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use serde::{Deserialize, Serialize};
use crate::types::Value;

/// Statistics provider error types
#[derive(Error, Debug)]
pub enum StatsError {
    #[error("Statistics not found for table: {0}")]
    TableNotFound(String),

    #[error("Invalid statistics: {0}")]
    InvalidStats(String),

    #[error("Update statistics failed: {0}")]
    UpdateFailed(String),
}

/// Result type for statistics operations
pub type StatsResult<T> = Result<T, StatsError>;

/// Column statistics for a single column
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnStats {
    /// Column name
    pub column_name: String,
    /// Number of distinct values (ndv)
    pub distinct_count: u64,
    /// Number of null values
    pub null_count: u64,
    /// Minimum value
    pub min_value: Option<Value>,
    /// Maximum value
    pub max_value: Option<Value>,
    /// Average value (for numeric types)
    pub avg_value: Option<f64>,
}

impl ColumnStats {
    pub fn new(column_name: impl Into<String>) -> Self {
        Self {
            column_name: column_name.into(),
            distinct_count: 0,
            null_count: 0,
            min_value: None,
            max_value: None,
            avg_value: None,
        }
    }

    pub fn with_distinct_count(mut self, count: u64) -> Self {
        self.distinct_count = count;
        self
    }

    pub fn with_null_count(mut self, count: u64) -> Self {
        self.null_count = count;
        self
    }

    pub fn with_range(mut self, min: Option<Value>, max: Option<Value>) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    pub fn with_average(mut self, avg: f64) -> Self {
        self.avg_value = Some(avg);
        self
    }

    /// Calculate selectivity for an equality predicate
    pub fn eq_selectivity(&self) -> f64 {
        if self.distinct_count == 0 {
            return 1.0;
        }
        1.0 / self.distinct_count as f64
    }

    /// Calculate selectivity for a range predicate
    pub fn range_selectivity(&self, _min: &Value, _max: &Value) -> f64 {
        // Simplified selectivity estimation
        // In a real implementation, this would use histogram data
        0.33
    }
}

/// Table-level statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableStats {
    /// Table name
    pub table_name: String,
    /// Total number of rows
    pub row_count: u64,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Column statistics
    pub column_stats: HashMap<String, ColumnStats>,
    /// Last updated timestamp (Unix epoch)
    pub last_updated: u64,
}

impl TableStats {
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            row_count: 0,
            size_bytes: 0,
            column_stats: HashMap::new(),
            last_updated: 0,
        }
    }

    pub fn with_row_count(mut self, count: u64) -> Self {
        self.row_count = count;
        self
    }

    pub fn with_size_bytes(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    pub fn add_column_stats(mut self, stats: ColumnStats) -> Self {
        self.column_stats.insert(stats.column_name.clone(), stats);
        self
    }

    pub fn with_last_updated(mut self, timestamp: u64) -> Self {
        self.last_updated = timestamp;
        self
    }

    /// Get column statistics by name
    pub fn column(&self, column: &str) -> Option<&ColumnStats> {
        self.column_stats.get(column)
    }

    /// Estimate selectivity for a predicate on a column
    pub fn estimate_selectivity(&self, column: &str) -> f64 {
        self.column_stats
            .get(column)
            .map(|c| c.eq_selectivity())
            .unwrap_or(0.1) // Default selectivity when no stats
    }
}

/// StatisticsProvider trait - interface for providing table statistics
///
/// # What
/// 统计信息提供者接口，为优化器提供表级和列级统计信息
///
/// # Why
/// CBO (基于成本的优化器) 需要统计信息来估算不同执行计划的成本
///
/// # How
/// - table_stats 方法获取指定表的统计信息
/// - update_stats 方法更新表的统计信息
pub trait StatisticsProvider: Send + Sync {
    /// Get statistics for a specific table
    fn table_stats(&self, table: &str) -> Option<TableStats>;

    /// Update statistics for a specific table
    fn update_stats(&self, table: &str, stats: TableStats) -> StatsResult<()>;

    /// Get row count estimate for a table
    fn estimated_rows(&self, table: &str) -> u64 {
        self.table_stats(table).map(|s| s.row_count).unwrap_or(0)
    }

    /// Get selectivity estimate for a column
    fn selectivity(&self, table: &str, column: &str) -> f64 {
        self.table_stats(table)
            .map(|s| s.estimate_selectivity(column))
            .unwrap_or(0.1)
    }
}

/// In-memory statistics provider implementation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InMemoryStatisticsProvider {
    stats: HashMap<String, TableStats>,
}

impl InMemoryStatisticsProvider {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    pub fn add_stats(&mut self, table_stats: TableStats) {
        self.stats
            .insert(table_stats.table_name.clone(), table_stats);
    }

    pub fn remove_stats(&mut self, table: &str) {
        self.stats.remove(table);
    }

    pub fn has_stats(&self, table: &str) -> bool {
        self.stats.contains_key(table)
    }
}

impl StatisticsProvider for InMemoryStatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        self.stats.get(table).cloned()
    }

    fn update_stats(&self, table: &str, _stats: TableStats) -> StatsResult<()> {
        if !self.stats.contains_key(table) {
            return Err(StatsError::TableNotFound(table.to_string()));
        }
        // In-memory provider doesn't persist, so we just validate
        Ok(())
    }
}

/// File-based statistics provider implementation
/// Persists statistics to disk in JSON format
#[derive(Debug, Default)]
pub struct FileStatisticsProvider {
    /// Base directory for storing statistics files
    stats_dir: String,
    /// In-memory cache for quick access
    cache: InMemoryStatisticsProvider,
}

impl FileStatisticsProvider {
    /// Create a new FileStatisticsProvider with the given stats directory
    pub fn new(stats_dir: impl Into<String>) -> Self {
        let stats_dir = stats_dir.into();
        // Ensure directory exists
        let _ = fs::create_dir_all(&stats_dir);
        Self {
            stats_dir,
            cache: InMemoryStatisticsProvider::default(),
        }
    }

    /// Load all statistics from disk into memory cache
    pub fn load(&mut self) -> StatsResult<()> {
        let path = Path::new(&self.stats_dir);
        if !path.exists() {
            return Ok(());
        }

        // Try to load from stats.json
        let stats_file = path.join("stats.json");
        if stats_file.exists() {
            let content = fs::read_to_string(&stats_file)
                .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;
            let loaded: InMemoryStatisticsProvider = serde_json::from_str(&content)
                .map_err(|e| StatsError::InvalidStats(e.to_string()))?;

            // Merge loaded stats into cache
            for (_table, stats) in loaded.stats {
                self.cache.add_stats(stats);
            }
        }
        Ok(())
    }

    /// Save all statistics to disk
    pub fn save(&self) -> StatsResult<()> {
        let path = Path::new(&self.stats_dir);
        let _ = fs::create_dir_all(path);

        let stats_file = path.join("stats.json");
        let content = serde_json::to_string_pretty(&self.cache)
            .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;

        fs::write(&stats_file, content)
            .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;

        Ok(())
    }

    /// Add or update statistics for a table
    pub fn add_stats(&mut self, table_stats: TableStats) {
        self.cache.add_stats(table_stats);
    }

    /// Remove statistics for a table
    pub fn remove_stats(&mut self, table: &str) {
        self.cache.remove_stats(table);
    }

    /// Check if statistics exist for a table
    pub fn has_stats(&self, table: &str) -> bool {
        self.cache.has_stats(table)
    }

    /// Get the stats directory path
    pub fn stats_dir(&self) -> &str {
        &self.stats_dir
    }
}

impl StatisticsProvider for FileStatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        self.cache.table_stats(table)
    }

    fn update_stats(&self, table: &str, _stats: TableStats) -> StatsResult<()> {
        if !self.cache.has_stats(table) {
            return Err(StatsError::TableNotFound(table.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_stats() {
        let stats = ColumnStats::new("id")
            .with_distinct_count(1000)
            .with_null_count(10)
            .with_range(Some(Value::Integer(1)), Some(Value::Integer(1000)))
            .with_average(500.5);

        assert_eq!(stats.column_name, "id");
        assert_eq!(stats.distinct_count, 1000);
        assert_eq!(stats.null_count, 10);
        assert!(stats.avg_value.is_some());
    }

    #[test]
    fn test_column_selectivity() {
        let stats = ColumnStats::new("id").with_distinct_count(100);
        let selectivity = stats.eq_selectivity();
        assert_eq!(selectivity, 0.01);
    }

    #[test]
    fn test_table_stats() {
        let col_stats = ColumnStats::new("id")
            .with_distinct_count(1000)
            .with_null_count(5);

        let table_stats = TableStats::new("users")
            .with_row_count(10000)
            .with_size_bytes(1_000_000)
            .add_column_stats(col_stats)
            .with_last_updated(1234567890);

        assert_eq!(table_stats.table_name, "users");
        assert_eq!(table_stats.row_count, 10000);
        assert!(table_stats.column("id").is_some());
    }

    #[test]
    fn test_table_selectivity() {
        let col_stats = ColumnStats::new("status").with_distinct_count(5);
        let table_stats = TableStats::new("orders")
            .with_row_count(1000)
            .add_column_stats(col_stats);

        let selectivity = table_stats.estimate_selectivity("status");
        assert!((selectivity - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_in_memory_provider() {
        let mut provider = InMemoryStatisticsProvider::new();

        let stats = TableStats::new("users")
            .with_row_count(5000)
            .with_size_bytes(500_000);

        provider.add_stats(stats);

        let retrieved = provider.table_stats("users");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().row_count, 5000);
    }

    #[test]
    fn test_estimated_rows() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(TableStats::new("orders").with_row_count(10000));

        assert_eq!(provider.estimated_rows("orders"), 10000);
        assert_eq!(provider.estimated_rows("unknown"), 0);
    }

    #[test]
    fn test_selectivity() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(
            TableStats::new("users")
                .add_column_stats(ColumnStats::new("age").with_distinct_count(50)),
        );

        let selectivity = provider.selectivity("users", "age");
        assert!((selectivity - 0.02).abs() < 0.001);
    }

    #[test]
    fn test_file_statistics_provider() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let stats_dir = temp_dir.path().join("stats");

        let mut provider = FileStatisticsProvider::new(stats_dir.to_string_lossy().to_string());

        // Add some stats
        let stats = TableStats::new("users")
            .with_row_count(5000)
            .with_size_bytes(500_000)
            .add_column_stats(ColumnStats::new("id").with_distinct_count(5000));
        provider.add_stats(stats);

        // Save to disk
        provider.save().unwrap();

        // Create new provider and load
        let mut provider2 = FileStatisticsProvider::new(stats_dir.to_string_lossy().to_string());
        provider2.load().unwrap();

        // Verify loaded stats
        let loaded = provider2.table_stats("users");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().row_count, 5000);
    }

    #[test]
    fn test_file_provider_serialization() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let stats_dir = temp_dir.path().join("stats2");

        let stats = TableStats::new("orders")
            .with_row_count(10000)
            .with_size_bytes(1_000_000)
            .add_column_stats(
                ColumnStats::new("amount")
                    .with_distinct_count(100)
                    .with_null_count(5)
                    .with_range(Some(Value::Integer(1)), Some(Value::Integer(10000)))
                    .with_average(500.5),
            );

        {
            let mut provider = FileStatisticsProvider::new(stats_dir.to_string_lossy().to_string());
            provider.add_stats(stats.clone());
            provider.save().unwrap();
        }

        // Load and verify
        let mut provider = FileStatisticsProvider::new(stats_dir.to_string_lossy().to_string());
        provider.load().unwrap();

        let loaded = provider.table_stats("orders").unwrap();
        assert_eq!(loaded.row_count, 10000);
        assert_eq!(loaded.column_stats.len(), 1);
        let col_stats = loaded.column("amount").unwrap();
        assert_eq!(col_stats.distinct_count, 100);
        assert_eq!(col_stats.null_count, 5);
    }
}
