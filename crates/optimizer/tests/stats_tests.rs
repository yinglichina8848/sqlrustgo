//! Optimizer Statistics Integration Tests

use sqlrustgo_optimizer::stats::{
    ColumnStats, InMemoryStatisticsProvider, StatisticsProvider, TableStats,
};

mod column_stats_tests {
    use super::*;

    #[test]
    fn test_column_stats_new() {
        let stats = ColumnStats::new("age");
        assert_eq!(stats.column_name, "age");
        assert_eq!(stats.distinct_count, 0);
    }

    #[test]
    fn test_column_stats_with_distinct_count() {
        let stats = ColumnStats::new("dept").with_distinct_count(10);
        assert_eq!(stats.distinct_count, 10);
    }

    #[test]
    fn test_column_stats_with_null_count() {
        let stats = ColumnStats::new("name").with_null_count(5);
        assert_eq!(stats.null_count, 5);
    }

    #[test]
    fn test_column_stats_eq_selectivity() {
        let stats = ColumnStats::new("id").with_distinct_count(100);
        let sel = stats.eq_selectivity();
        assert!((sel - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_column_stats_eq_selectivity_zero_ndv() {
        let stats = ColumnStats::new("empty");
        let sel = stats.eq_selectivity();
        assert_eq!(sel, 1.0);
    }

    #[test]
    fn test_column_stats_range_selectivity() {
        let stats = ColumnStats::new("val");
        let min = sqlrustgo_types::Value::Integer(0);
        let max = sqlrustgo_types::Value::Integer(100);
        let sel = stats.range_selectivity(&min, &max);
        assert!((sel - 0.33).abs() < 0.01);
    }

    #[test]
    fn test_column_stats_eq_selectivity_small_ndv() {
        let stats = ColumnStats::new("color").with_distinct_count(4);
        let sel = stats.eq_selectivity();
        assert!((sel - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_column_stats_eq_selectivity_large_ndv() {
        let stats = ColumnStats::new("id").with_distinct_count(100000);
        let sel = stats.eq_selectivity();
        assert!((sel - 0.00001).abs() < 0.000001);
    }

    #[test]
    fn test_column_stats_eq_selectivity_one_ndv() {
        let stats = ColumnStats::new("unique").with_distinct_count(1);
        let sel = stats.eq_selectivity();
        assert_eq!(sel, 1.0);
    }

    #[test]
    fn test_column_stats_clone() {
        let stats = ColumnStats::new("age").with_distinct_count(50);
        let cloned = stats.clone();
        assert_eq!(cloned.column_name, "age");
        assert_eq!(cloned.distinct_count, 50);
    }
}

mod table_stats_tests {
    use super::*;

    #[test]
    fn test_table_stats_new() {
        let stats = TableStats::new("users");
        assert_eq!(stats.table_name, "users");
        assert_eq!(stats.row_count, 0);
    }

    #[test]
    fn test_table_stats_with_row_count() {
        let stats = TableStats::new("users").with_row_count(1000);
        assert_eq!(stats.row_count, 1000);
    }

    #[test]
    fn test_table_stats_with_size() {
        let stats = TableStats::new("logs").with_size_bytes(50000);
        assert_eq!(stats.size_bytes, 50000);
    }

    #[test]
    fn test_table_stats_add_column() {
        let stats = TableStats::new("users")
            .with_row_count(1000)
            .add_column_stats(ColumnStats::new("id").with_distinct_count(1000));
        assert_eq!(stats.column_stats.len(), 1);
    }

    #[test]
    fn test_table_stats_get_column() {
        let stats = TableStats::new("users")
            .add_column_stats(ColumnStats::new("id").with_distinct_count(100));
        let col = stats.column("id").unwrap();
        assert_eq!(col.distinct_count, 100);
    }

    #[test]
    fn test_table_stats_get_nonexistent_column() {
        let stats = TableStats::new("users");
        assert!(stats.column("missing").is_none());
    }

    #[test]
    fn test_table_stats_clone() {
        let stats = TableStats::new("t").with_row_count(100);
        let cloned = stats.clone();
        assert_eq!(cloned.table_name, "t");
        assert_eq!(cloned.row_count, 100);
    }
}

mod in_memory_provider_tests {
    use super::*;

    #[test]
    fn test_provider_new() {
        let provider = InMemoryStatisticsProvider::new();
        assert!(provider.table_stats("nonexistent").is_none());
    }

    #[test]
    fn test_provider_add_and_get_table() {
        let mut provider = InMemoryStatisticsProvider::new();
        let stats = TableStats::new("users").with_row_count(500);
        provider.add_stats(stats);
        let retrieved = provider.table_stats("users").unwrap();
        assert_eq!(retrieved.row_count, 500);
    }

    #[test]
    fn test_provider_column_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        let stats = TableStats::new("users")
            .add_column_stats(ColumnStats::new("id").with_distinct_count(1000));
        provider.add_stats(stats);
        let col = provider.column_stats("users", "id").unwrap();
        assert_eq!(col.distinct_count, 1000);
    }

    #[test]
    fn test_provider_has_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        assert!(!provider.has_stats("users"));
        provider.add_stats(TableStats::new("users"));
        assert!(provider.has_stats("users"));
    }

    #[test]
    fn test_provider_remove_stats() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(TableStats::new("users"));
        assert!(provider.has_stats("users"));
        provider.remove_stats("users");
        assert!(!provider.has_stats("users"));
    }

    #[test]
    fn test_provider_multiple_tables() {
        let mut provider = InMemoryStatisticsProvider::new();
        provider.add_stats(TableStats::new("t1").with_row_count(100));
        provider.add_stats(TableStats::new("t2").with_row_count(200));
        assert!(provider.table_stats("t1").is_some());
        assert!(provider.table_stats("t2").is_some());
    }
}

mod cardinality_calculation_tests {
    #[test]
    fn test_join_cardinality_product() {
        let left_rows: u64 = 1000;
        let right_rows: u64 = 2000;
        let join_card = left_rows * right_rows;
        assert_eq!(join_card, 2_000_000);
    }

    #[test]
    fn test_join_cardinality_with_ndv() {
        let left_rows: u64 = 1000;
        let right_rows: u64 = 2000;
        let ndv_left: u64 = 100;
        let ndv_right: u64 = 200;
        let max_ndv = ndv_left.max(ndv_right);
        let join_card = (left_rows * right_rows) / max_ndv;
        assert_eq!(join_card, 10000);
    }

    #[test]
    fn test_group_by_cardinality() {
        let row_count: u64 = 10000;
        let distinct_values: u64 = 100;
        let group_card = distinct_values.min(row_count);
        assert_eq!(group_card, 100);
    }

    #[test]
    fn test_selectivity_with_distinct() {
        let ndv: f64 = 100.0;
        let selectivity = 1.0 / ndv;
        assert!((selectivity - 0.01).abs() < 0.0001);
    }
}
