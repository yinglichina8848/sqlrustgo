// Optimizer Cost Model Tests
//
// Tests for the cost model that estimates query execution costs.
// These tests verify that cost calculations follow expected relationships
// and produce reasonable values for various operations.

use sqlrustgo_optimizer::cost::SimpleCostModel;
use sqlrustgo_optimizer::stats::{ColumnStats, TableStats};

#[test]
fn test_cost_model_creation() {
    let model = SimpleCostModel::new(2.0, 20.0, 0.002);
    assert_eq!(model.cpu_cost_per_row, 2.0);
    assert_eq!(model.io_cost_per_page, 20.0);
}

#[test]
fn test_cost_model_default_values() {
    let model = SimpleCostModel::default_model();
    // Default: cpu=1.0, io=10.0, network=0.001
    assert_eq!(model.cpu_cost_per_row, 1.0);
    assert_eq!(model.io_cost_per_page, 10.0);
}

// ============================================================================
// Sequential Scan Cost Tests
// ============================================================================

#[test]
fn test_seq_scan_cost_scales_with_rows() {
    let model = SimpleCostModel::default_model();
    let cost_100 = model.seq_scan_cost(100, 10);
    let cost_200 = model.seq_scan_cost(200, 10);

    // Double the rows should double the CPU cost
    assert!(cost_200 > cost_100);
    assert!(cost_200 >= cost_100 * 1.5); // At least 1.5x due to I/O scaling
}

#[test]
fn test_seq_scan_cost_scales_with_pages() {
    let model = SimpleCostModel::default_model();
    let cost_10_pages = model.seq_scan_cost(1000, 10);
    let cost_20_pages = model.seq_scan_cost(1000, 20);

    // Double the pages should increase cost
    assert!(cost_20_pages > cost_10_pages);
}

#[test]
fn test_seq_scan_cost_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // cost = rows * cpu_cost + pages * io_cost
    // cost = 1000 * 1.0 + 10 * 10.0 = 1000 + 100 = 1100
    let cost = model.seq_scan_cost(1000, 10);
    assert!((cost - 1100.0).abs() < 0.001);
}

#[test]
fn test_seq_scan_cost_zero_rows() {
    let model = SimpleCostModel::default_model();
    let cost = model.seq_scan_cost(0, 0);
    assert_eq!(cost, 0.0);
}

// ============================================================================
// Index Scan Cost Tests
// ============================================================================

#[test]
fn test_index_scan_cost_less_than_seq_scan_small_result() {
    let model = SimpleCostModel::default_model();
    // Index scan with few result rows should be cheaper than seq scan
    let index_cost = model.index_scan_cost(10, 2, 100); // 10 rows, 2 index pages, 100 data pages
    let seq_cost = model.seq_scan_cost(1000, 100); // Full table scan

    // When index returns few rows, index scan should be cheaper
    assert!(index_cost < seq_cost);
}

#[test]
fn test_index_scan_cost_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // index_cost = index_pages * io + data_pages * io + rows * cpu
    // = 2 * 10 + 50 * 10 + 100 * 1 = 20 + 500 + 100 = 620
    let cost = model.index_scan_cost(100, 2, 50);
    assert!((cost - 620.0).abs() < 0.001);
}

#[test]
fn test_index_scan_cost_scales_with_result_rows() {
    let model = SimpleCostModel::default_model();
    let cost_10_rows = model.index_scan_cost(10, 2, 100);
    let cost_100_rows = model.index_scan_cost(100, 2, 100);

    // More result rows = higher CPU cost
    assert!(cost_100_rows > cost_10_rows);
}

// ============================================================================
// Join Cost Tests
// ============================================================================

#[test]
fn test_nested_loop_join_cost_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // nested_loop: left_rows * right_rows * cpu_cost
    let cost = model.join_cost(100, 200, "nested_loop");
    assert!((cost - 20000.0).abs() < 0.001); // 100 * 200 * 1.0
}

#[test]
fn test_hash_join_cost_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // hash_join: (left + right) * cpu_cost
    let cost = model.join_cost(100, 200, "hash_join");
    assert!((cost - 300.0).abs() < 0.001); // (100 + 200) * 1.0
}

#[test]
fn test_sort_merge_join_cost_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // sort_merge: 2 * (left + right) * cpu_cost
    let cost = model.join_cost(100, 200, "sort_merge");
    assert!((cost - 600.0).abs() < 0.001); // 2 * (100 + 200) * 1.0
}

#[test]
fn test_nested_loop_worse_than_hash_for_large_tables() {
    let model = SimpleCostModel::default_model();
    // Nested loop is O(n*m), hash join is O(n+m)
    let nl_cost = model.join_cost(10_000, 10_000, "nested_loop");
    let hash_cost = model.join_cost(10_000, 10_000, "hash_join");

    // For large tables, nested loop should be much more expensive
    assert!(nl_cost > hash_cost);
    assert!(nl_cost > 1000.0 * hash_cost); // At least 1000x more expensive
}

#[test]
fn test_join_cost_with_unknown_method_uses_default() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // Unknown method falls back to (left + right) * cpu
    let cost = model.join_cost(100, 200, "unknown_method");
    assert!((cost - 300.0).abs() < 0.001);
}

// ============================================================================
// Aggregation Cost Tests
// ============================================================================

#[test]
fn test_agg_cost_formula_no_group_by() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // agg_cost = row_count * cpu_cost (when group_by_cols == 0)
    let cost = model.agg_cost(1000, 0);
    assert!((cost - 1000.0).abs() < 0.001);
}

#[test]
fn test_agg_cost_formula_with_group_by() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // agg_cost = rows * cpu + rows * cpu * log2(cols)
    // = 1000 * 1 + 1000 * 1 * log2(4) = 1000 + 2000 = 3000
    let cost = model.agg_cost(1000, 4);
    let expected = 1000.0 + 1000.0 * (4.0_f64).log2();
    assert!((cost - expected).abs() < 0.001);
}

#[test]
fn test_agg_cost_scales_with_rows() {
    let model = SimpleCostModel::default_model();
    let cost_1000 = model.agg_cost(1000, 2);
    let cost_2000 = model.agg_cost(2000, 2);

    // Double rows should increase cost
    assert!(cost_2000 > cost_1000);
}

#[test]
fn test_agg_cost_increases_with_more_group_columns() {
    let model = SimpleCostModel::default_model();
    let cost_1_col = model.agg_cost(1000, 1);
    let cost_4_cols = model.agg_cost(1000, 4);

    // More group columns = more sorting needed = higher cost
    assert!(cost_4_cols > cost_1_col);
}

// ============================================================================
// Sort Cost Tests
// ============================================================================

#[test]
fn test_sort_cost_in_memory_formula() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // In-memory sort: rows * cpu * log2(rows)
    let cost = model.sort_cost(1000, 100);
    let expected = 1000.0 * 1.0 * (1000.0_f64).log2();
    assert!((cost - expected).abs() < 1.0); // Allow some rounding tolerance
}

#[test]
fn test_sort_cost_external_vs_in_memory() {
    let model = SimpleCostModel::default_model();
    // In-memory threshold is 1,000,000 rows
    let in_memory_cost = model.sort_cost(1000, 100);
    let external_cost = model.sort_cost(2_000_000, 100);

    // External sort should be significantly more expensive
    assert!(external_cost > in_memory_cost * 10.0);
}

#[test]
fn test_sort_cost_in_memory_scales_with_rows() {
    let model = SimpleCostModel::default_model();
    let cost_1000 = model.sort_cost(1000, 100);
    let cost_2000 = model.sort_cost(2000, 100);

    // More rows = higher sort cost
    assert!(cost_2000 > cost_1000);
}

// ============================================================================
// Statistics Integration Tests
// ============================================================================

#[test]
fn test_estimate_with_stats_uses_row_and_page_count() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // size_bytes = pages * 4096 (page size)
    let stats = TableStats::new("users")
        .with_row_count(1000)
        .with_size_bytes(50 * 4096);

    let cost = model.estimate_with_stats(&stats);
    // seq_scan_cost = rows * cpu + pages * io = 1000 * 1 + 50 * 10 = 1500
    assert!((cost - 1500.0).abs() < 0.001);
}

#[test]
fn test_estimate_index_scan_with_stats() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    let stats = TableStats::new("users")
        .with_row_count(10000)
        .with_size_bytes(100 * 4096) // 100 pages
        .add_column_stats(ColumnStats::new("id").with_distinct_count(10000));

    let cost = model.estimate_index_scan_with_stats(&stats, "id");
    // With 10000 distinct values, selectivity = 1/10000 = 0.0001
    // estimated_rows = 10000 * 0.0001 = 1
    // index_pages = max(1, 1/100) = 1
    // cost = 1*10 + 100*10 + 1*1 = 10 + 1000 + 1 = 1011
    assert!(cost > 0.0);
}

#[test]
fn test_estimate_join_with_stats() {
    let model = SimpleCostModel::new(1.0, 10.0, 0.001);
    let left_stats = TableStats::new("users")
        .with_row_count(1000)
        .with_size_bytes(50 * 4096); // 50 pages
    let right_stats = TableStats::new("orders")
        .with_row_count(5000)
        .with_size_bytes(200 * 4096); // 200 pages

    let cost = model.estimate_join_with_stats(&left_stats, &right_stats, "hash_join");
    // hash_join: (left + right) * cpu = (1000 + 5000) * 1 = 6000
    assert!((cost - 6000.0).abs() < 0.001);
}

// ============================================================================
// Cost Model Relationships Tests
// ============================================================================

#[test]
fn test_index_scan_appropriate_for_high_selectivity() {
    let model = SimpleCostModel::default_model();
    // When we need most rows, seq scan might be better
    let seq_cost = model.seq_scan_cost(1000, 10);
    let index_cost = model.index_scan_cost(500, 2, 10); // 50% selectivity

    // For 50% selectivity, index might not help much
    // Just verify both are calculated
    assert!(seq_cost > 0.0);
    assert!(index_cost > 0.0);
}

#[test]
fn test_cost_model_consistency() {
    // Same inputs should always produce same outputs
    let model = SimpleCostModel::default_model();
    let cost1 = model.seq_scan_cost(1000, 10);
    let cost2 = model.seq_scan_cost(1000, 10);
    assert!((cost1 - cost2).abs() < 0.001);
}

#[test]
fn test_zero_input_costs() {
    let model = SimpleCostModel::default_model();

    assert_eq!(model.seq_scan_cost(0, 0), 0.0);
    assert_eq!(model.index_scan_cost(0, 0, 0), 0.0);
    assert_eq!(model.join_cost(0, 0, "nested_loop"), 0.0);
    assert_eq!(model.agg_cost(0, 0), 0.0);
}
