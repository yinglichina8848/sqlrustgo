// Optimizer Stats Tests
use sqlrustgo_optimizer::stats::{
    ColumnStats, DefaultStatsCollector, InMemoryStatisticsProvider, StatsError, TableStats,
};

#[test]
fn test_column_stats_new() {
    let stats = ColumnStats::new("test_col".to_string());
    assert_eq!(stats.column_name, "test_col");
}

#[test]
fn test_column_stats_with_values() {
    let stats = ColumnStats::new("test_col".to_string()).with_distinct_count(50);
    assert_eq!(stats.distinct_count, 50);
}

#[test]
fn test_table_stats_new() {
    let stats = TableStats::new("test_table".to_string());
    assert_eq!(stats.table_name, "test_table");
    assert_eq!(stats.row_count, 0);
}

#[test]
fn test_table_stats_row_count() {
    let stats = TableStats::new("test_table".to_string()).with_row_count(1000);
    assert_eq!(stats.row_count, 1000);
}

#[test]
fn test_stats_error_display() {
    let err = StatsError::TableNotFound("test".to_string());
    assert!(format!("{}", err).contains("not found"));

    let err = StatsError::InvalidStats("test".to_string());
    assert!(format!("{}", err).contains("Invalid"));

    let err = StatsError::UpdateFailed("test".to_string());
    assert!(format!("{}", err).contains("Update"));
}

#[test]
fn test_in_memory_statistics_provider() {
    let provider = InMemoryStatisticsProvider::new();
    let _ = provider;
}

#[test]
fn test_default_stats_collector() {
    let collector = DefaultStatsCollector::new();
    let _ = collector;
}
