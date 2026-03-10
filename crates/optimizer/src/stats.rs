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
//! - StatsCollector: 统计信息收集器

use std::collections::HashMap;
use thiserror::Error;

use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::Value;

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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
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

    /// Add multiple column stats at once
    pub fn with_column_stats(mut self, stats: HashMap<String, ColumnStats>) -> Self {
        self.column_stats = stats;
        self
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

    /// Get column statistics for a specific column
    fn column_stats(&self, table: &str, column: &str) -> Option<ColumnStats> {
        self.table_stats(table)?.column_stats.get(column).cloned()
    }

    /// Check if statistics exist for a table
    fn has_stats(&self, table: &str) -> bool {
        self.table_stats(table).is_some()
    }
}

/// In-memory statistics provider implementation
#[derive(Debug, Default)]
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

/// StatsCollector trait - for collecting statistics from tables
///
/// # What
/// 统计信息收集器接口，从表中收集统计信息
///
/// # Why
/// CBO 需要实时的表统计信息来估算查询成本
///
/// # How
/// - collect_table_stats 方法收集表的完整统计信息
/// - collect_row_count 方法只收集行数
/// - collect_column_stats 方法收集列级统计信息
pub trait StatsCollector: Send + Sync {
    /// Collect statistics for a table
    fn collect_table_stats(
        &self,
        storage: &dyn StorageEngine,
        table: &str,
    ) -> StatsResult<TableStats>;

    /// Collect row count for a table
    fn collect_row_count(&self, storage: &dyn StorageEngine, table: &str) -> StatsResult<u64>;

    /// Collect column statistics
    fn collect_column_stats(
        &self,
        storage: &dyn StorageEngine,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> StatsResult<ColumnStats>;
}

/// Default statistics collector implementation
#[derive(Debug, Clone, Default)]
pub struct DefaultStatsCollector;

impl DefaultStatsCollector {
    pub fn new() -> Self {
        Self
    }
}

impl StatsCollector for DefaultStatsCollector {
    fn collect_table_stats(
        &self,
        storage: &dyn StorageEngine,
        table: &str,
    ) -> StatsResult<TableStats> {
        let records = storage
            .scan(table)
            .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;
        let row_count = records.len() as u64;

        let mut column_stats = HashMap::new();

        if let Ok(table_info) = storage.get_table_info(table) {
            for (idx, col_meta) in table_info.columns.iter().enumerate() {
                let col_stats = self.collect_column_stats(storage, table, &col_meta.name, idx)?;
                column_stats.insert(col_meta.name.clone(), col_stats);
            }
        }

        Ok(TableStats::new(table)
            .with_row_count(row_count)
            .with_last_updated(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )
            .with_column_stats(column_stats))
    }

    fn collect_row_count(&self, storage: &dyn StorageEngine, table: &str) -> StatsResult<u64> {
        let records = storage
            .scan(table)
            .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;
        Ok(records.len() as u64)
    }

    fn collect_column_stats(
        &self,
        storage: &dyn StorageEngine,
        table: &str,
        column: &str,
        column_index: usize,
    ) -> StatsResult<ColumnStats> {
        let records = storage
            .scan(table)
            .map_err(|e| StatsError::UpdateFailed(e.to_string()))?;

        let mut null_count: u64 = 0;
        let mut distinct_values: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut min_value: Option<Value> = None;
        let mut max_value: Option<Value> = None;
        let mut sum: f64 = 0.0;
        let mut numeric_count: u64 = 0;

        for record in &records {
            if let Some(value) = record.get(column_index) {
                // Count nulls
                if matches!(value, Value::Null) {
                    null_count += 1;
                    continue;
                }

                // Track distinct values
                distinct_values.insert(value.to_string());

                // Track min/max
                match value {
                    Value::Integer(i) => {
                        let v = *i as f64;
                        sum += v;
                        numeric_count += 1;
                        match min_value {
                            None => min_value = Some(value.clone()),
                            Some(Value::Integer(min_i)) if *i < min_i => {
                                min_value = Some(value.clone())
                            }
                            _ => {}
                        }
                        match max_value {
                            None => max_value = Some(value.clone()),
                            Some(Value::Integer(max_i)) if *i > max_i => {
                                max_value = Some(value.clone())
                            }
                            _ => {}
                        }
                    }
                    Value::Float(f) => {
                        sum += *f;
                        numeric_count += 1;
                        match min_value {
                            None => min_value = Some(value.clone()),
                            Some(Value::Float(min_f)) if *f < min_f => {
                                min_value = Some(value.clone())
                            }
                            _ => {}
                        }
                        match max_value {
                            None => max_value = Some(value.clone()),
                            Some(Value::Float(max_f)) if *f > max_f => {
                                max_value = Some(value.clone())
                            }
                            _ => {}
                        }
                    }
                    Value::Text(_) | Value::Blob(_) => {
                        // For non-numeric types, just track min/max lexicographically
                        match &min_value {
                            None => min_value = Some(value.clone()),
                            Some(Value::Text(_)) | Some(Value::Blob(_)) => {
                                if value.to_string() < min_value.as_ref().unwrap().to_string() {
                                    min_value = Some(value.clone());
                                }
                            }
                            _ => {}
                        }
                        match &max_value {
                            None => max_value = Some(value.clone()),
                            Some(Value::Text(_)) | Some(Value::Blob(_)) => {
                                if value.to_string() > max_value.as_ref().unwrap().to_string() {
                                    max_value = Some(value.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        let avg_value = if numeric_count > 0 {
            Some(sum / numeric_count as f64)
        } else {
            None
        };

        Ok(ColumnStats::new(column)
            .with_distinct_count(distinct_values.len() as u64)
            .with_null_count(null_count)
            .with_range(min_value, max_value)
            .with_average(avg_value.unwrap_or(0.0)))
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

    // ============ Additional Tests ============

    #[test]
    fn test_column_stats_default() {
        let stats = ColumnStats::new("id");
        assert_eq!(stats.column_name, "id");
        assert_eq!(stats.distinct_count, 0);
        assert_eq!(stats.null_count, 0);
    }

    #[test]
    fn test_column_stats_range() {
        let stats = ColumnStats::new("price")
            .with_range(Some(Value::Float(0.0)), Some(Value::Float(999.99)));

        assert_eq!(stats.min_value, Some(Value::Float(0.0)));
        assert_eq!(stats.max_value, Some(Value::Float(999.99)));
    }

    #[test]
    fn test_column_stats_eq_selectivity() {
        let stats = ColumnStats::new("age").with_distinct_count(100);
        let selectivity = stats.eq_selectivity();
        assert!(selectivity > 0.0 && selectivity < 1.0);
    }

    #[test]
    fn test_table_stats_default() {
        let stats = TableStats::new("test");
        assert_eq!(stats.table_name, "test");
        assert_eq!(stats.row_count, 0);
        assert!(stats.column_stats.is_empty());
    }

    #[test]
    fn test_table_stats_column_not_found() {
        let stats = TableStats::new("users");
        assert!(stats.column("nonexistent").is_none());
    }

    #[test]
    fn test_stats_result() {
        let ok_result: StatsResult<i32> = Ok(42);
        assert!(ok_result.is_ok());

        let err_result: StatsResult<i32> = Err(StatsError::TableNotFound("test".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_provider_update_stats() {
        let mut provider = InMemoryStatisticsProvider::new();

        // First add the stats
        let stats = TableStats::new("users").with_row_count(100);
        provider.add_stats(stats);

        // Now try to update
        let updated_stats = TableStats::new("users").with_row_count(200);
        let result = provider.update_stats("users", updated_stats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stats_error_display() {
        let err = StatsError::TableNotFound("users".to_string());
        assert!(err.to_string().contains("users"));

        let err = StatsError::InvalidStats("invalid".to_string());
        assert!(err.to_string().contains("invalid"));

        let err = StatsError::UpdateFailed("failed".to_string());
        assert!(err.to_string().contains("failed"));
    }

    #[test]
    fn test_column_stats_range_selectivity() {
        let stats =
            ColumnStats::new("id").with_range(Some(Value::Integer(1)), Some(Value::Integer(100)));
        let selectivity = stats.range_selectivity(&Value::Integer(1), &Value::Integer(50));
        assert!(selectivity > 0.0);
    }

    #[test]
    fn test_column_stats_eq_selectivity_zero_distinct() {
        let stats = ColumnStats::new("id").with_distinct_count(0);
        let selectivity = stats.eq_selectivity();
        assert_eq!(selectivity, 1.0);
    }

    #[test]
    fn test_table_stats_with_column_stats() {
        let mut col_stats = HashMap::new();
        col_stats.insert(
            "id".to_string(),
            ColumnStats::new("id").with_distinct_count(100),
        );

        let stats = TableStats::new("users").with_column_stats(col_stats);
        assert!(stats.column("id").is_some());
    }

    #[test]
    fn test_in_memory_provider_remove_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(TableStats::new("users").with_row_count(100));

        assert!(provider.has_stats("users"));

        provider.remove_stats("users");

        assert!(!provider.has_stats("users"));
    }

    #[test]
    fn test_in_memory_provider_update_nonexistent() {
        let provider = InMemoryStatisticsProvider::new();
        let result = provider.update_stats("users", TableStats::new("users"));
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_column_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(
            TableStats::new("users")
                .add_column_stats(ColumnStats::new("id").with_distinct_count(100)),
        );

        let col_stats = provider.column_stats("users", "id");
        assert!(col_stats.is_some());
        assert_eq!(col_stats.unwrap().distinct_count, 100);
    }

    #[test]
    fn test_provider_column_stats_not_found() {
        let provider = InMemoryStatisticsProvider::new();
        let col_stats = provider.column_stats("users", "id");
        assert!(col_stats.is_none());
    }

    #[test]
    fn test_default_stats_collector_new() {
        let collector = DefaultStatsCollector::new();
        let _ = collector;
    }

    #[test]
    fn test_in_memory_statistics_provider_new() {
        let provider = InMemoryStatisticsProvider::new();
        assert!(!provider.has_stats("users"));
    }

    #[test]
    fn test_stats_error_debug() {
        let err = StatsError::TableNotFound("users".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("TableNotFound"));
    }

    #[test]
    fn test_stats_error_not_found() {
        let err = StatsError::TableNotFound("orders".to_string());
        assert!(err.to_string().contains("orders"));
    }

    #[test]
    fn test_stats_error_invalid() {
        let err = StatsError::InvalidStats("invalid stats".to_string());
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_stats_error_update_failed() {
        let err = StatsError::UpdateFailed("disk full".to_string());
        assert!(err.to_string().contains("disk"));
    }

    #[test]
    fn test_table_stats_with_size_bytes() {
        let stats = TableStats::new("users").with_size_bytes(1_000_000);
        assert_eq!(stats.size_bytes, 1_000_000);
    }

    #[test]
    fn test_table_stats_estimate_selectivity_default() {
        let stats = TableStats::new("users");
        let selectivity = stats.estimate_selectivity("unknown_column");
        assert_eq!(selectivity, 0.1);
    }

    #[test]
    fn test_in_memory_provider_has_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        assert!(!provider.has_stats("users"));

        provider.add_stats(TableStats::new("users"));
        assert!(provider.has_stats("users"));
    }

    #[test]
    fn test_statistics_provider_trait_object() {
        fn _check_provider(_p: &dyn StatisticsProvider) {}
        let provider = InMemoryStatisticsProvider::new();
        _check_provider(&provider);
    }

    #[test]
    fn test_stats_collector_trait_object() {
        fn _check_collector(_c: &dyn StatsCollector) {}
        let collector = DefaultStatsCollector::new();
        _check_collector(&collector);
    }

    #[test]
    fn test_stats_result_ok() {
        let result: StatsResult<u64> = Ok(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_stats_result_err() {
        let result: StatsResult<u64> = Err(StatsError::TableNotFound("users".to_string()));
        assert!(result.is_err());
    }
}
