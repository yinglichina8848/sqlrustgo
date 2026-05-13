//! Optimizer Cost Model Integration Tests

use sqlrustgo_optimizer::{
    cost::{CboOptimizer, SimpleCostModel},
    CostModel,
};

mod simple_cost_model_tests {
    use super::*;

    #[test]
    fn test_cost_model_construction() {
        let model = SimpleCostModel::default_model();
        let cost = model.estimate_cost(&42i32);
        // seq_scan_cost(1000, 10).total() = 10 * 0.1 + 1000 * 1.0 = 1001.0
        assert_eq!(cost, 1001.0);
    }

    #[test]
    fn test_cost_model_default() {
        let model = SimpleCostModel::default_model();
        let cost = model.estimate_cost(&42i32);
        assert_eq!(cost, 1001.0);
    }

    #[test]
    fn test_seq_scan_cost_calculation() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(1000, 10).total();
        // io = 10 * 0.1 = 1.0, cpu = 1000 * 1.0 = 1000.0, total = 1001.0
        assert!((cost - 1001.0).abs() < 0.001);
    }

    #[test]
    fn test_seq_scan_cost_empty() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(0, 0).total();
        assert!((cost - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_index_scan_cost_calculation() {
        let model = SimpleCostModel::default_model();
        // btree_height=3, index_pages=2, matching_rows=5, data_pages=10
        // index_search_cpu = 3 * 0.1 = 0.3
        // index_io = 2 * 1.0 = 2.0 (random)
        // data_io = 10 * 0.1 = 1.0 (seq)
        // cpu_cost = 5 * 1.0 + 0.3 = 5.3
        // total = 2.0 + 1.0 + 5.3 = 8.3
        let cost = model.index_scan_cost_f64(3, 2, 5, 10);
        assert!((cost - 8.3).abs() < 0.001);
    }

    #[test]
    fn test_index_scan_cost_empty() {
        let model = SimpleCostModel::default_model();
        let cost = model.index_scan_cost_f64(0, 0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_join_cost_nested_loop() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "nested_loop");
        // 1000 * 2000 * 1.0 = 2_000_000.0
        assert!((cost - 2_000_000.0).abs() < 0.001);
    }

    #[test]
    fn test_join_cost_hash_join() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "hash_join");
        // build_cpu = 1000 * 1.0 = 1000.0
        // probe_cpu = 2000 * 1.0 * 1.2 = 2400.0
        // memory_cost ≈ 0.0001
        // total ≈ 3400.0001
        assert!(cost > 3400.0 && cost < 3401.0);
    }

    #[test]
    fn test_join_cost_sort_merge() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "sort_merge");
        // left_sort = 1000 * log2(1000) * 1.0 ≈ 9956.3
        // right_sort = 2000 * log2(2000) * 1.0 ≈ 21971.5
        // merge = 3000 * 1.0 = 3000.0
        // total ≈ 34927.8
        assert!(cost > 34000.0 && cost < 36000.0);
    }

    #[test]
    fn test_join_cost_unknown_method() {
        let model = SimpleCostModel::default_model();
        // Unknown method falls back to nested_loop
        let cost = model.join_cost(1000, 2000, "unknown");
        assert!((cost - 2_000_000.0).abs() < 0.001);
    }

    #[test]
    fn test_agg_cost_with_group_by() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(10000, 2);
        // num_groups = 10000 / 100 = 100
        // hash_agg_cost(10000, 100) ≈ 10100.0
        assert!(cost > 10000.0);
    }

    #[test]
    fn test_agg_cost_no_group_by() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(1000, 0);
        // num_groups = 1000 / 100 = 10
        // hash_agg_cost(1000, 10) ≈ 1010.0
        assert!(cost > 1000.0);
    }

    #[test]
    fn test_sort_cost_in_memory() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(1000, 100);
        // sort_cost_detail returns Cost, sort_cost returns .total()
        assert!(cost > 0.0);
    }

    #[test]
    fn test_sort_cost_external() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(2_000_000, 100);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cost_model_trait_estimate() {
        let model: SimpleCostModel = SimpleCostModel::default_model();
        let cost = model.estimate_cost(&42i32);
        assert_eq!(cost, 1001.0);
    }
}

mod cbo_optimizer_tests {
    use super::*;

    #[test]
    fn test_cbo_optimizer_new() {
        let cbo = CboOptimizer::new();
        assert_eq!(cbo.default_row_count(), 1000);
        assert_eq!(cbo.default_page_count(), 10);
    }

    #[test]
    fn test_cbo_optimizer_with_custom_defaults() {
        let cbo = CboOptimizer::with_defaults(5000, 50);
        assert_eq!(cbo.default_row_count(), 5000);
        assert_eq!(cbo.default_page_count(), 50);
    }

    #[test]
    fn test_cbo_optimizer_cost_model_access() {
        let cbo = CboOptimizer::new();
        let cost = cbo.cost_model().estimate_cost(&());
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost(1000, 10).total();
        // Same as seq_scan_cost: io = 10 * 0.1 = 1.0, cpu = 1000 * 1.0 = 1000.0
        assert!((cost - 1001.0).abs() < 0.001);
    }

    #[test]
    fn test_cbo_estimate_scan_cost_different_values() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost(5000, 100).total();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_index_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_index_scan_cost(100, 2, 5, 10);
        // btree_height=100, index_pages=2, matching_rows=5, data_pages=10
        // index_search_cpu = 100 * 0.1 = 10.0
        // index_io = 2 * 1.0 = 2.0
        // data_io = 10 * 0.1 = 1.0
        // cpu_cost = 5 * 1.0 + 10.0 = 15.0
        // total = 2.0 + 1.0 + 15.0 = 18.0
        assert!(cost.total() > 0.0);
    }

    #[test]
    fn test_cbo_estimate_index_scan_cost_zero() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_index_scan_cost(0, 0, 0, 0);
        assert!((cost.total() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cbo_estimate_join_cost_nested_loop() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "nested_loop").total();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost_hash_join() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "hash_join").total();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost_sort_merge() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "sort_merge").total();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_cost_model_trait() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_cost(&());
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_with_large_defaults() {
        let cbo = CboOptimizer::with_defaults(1_000_000, 10_000);
        let cost = cbo.estimate_cost(&());
        assert!(cost > 0.0);
    }
}

mod cost_comparison_tests {
    use super::*;

    #[test]
    fn test_hash_join_cheaper_than_nested_loop_large() {
        let model = SimpleCostModel::default_model();
        let nested_loop = model.join_cost(100_000, 100_000, "nested_loop");
        let hash_join = model.join_cost(100_000, 100_000, "hash_join");
        assert!(hash_join < nested_loop);
    }

    #[test]
    fn test_sort_merge_join_cost() {
        let model = SimpleCostModel::default_model();
        let sort_merge = model.join_cost(10_000, 10_000, "sort_merge");
        // left_sort = 10000 * log2(10000) * 1.0 ≈ 132877.1
        // right_sort = 10000 * log2(10000) * 1.0 ≈ 132877.1
        // merge = 20000 * 1.0 = 20000.0
        // total ≈ 285754.2
        assert!(sort_merge > 280_000.0);
    }

    #[test]
    fn test_agg_cost_increases_with_row_count() {
        let model = SimpleCostModel::default_model();
        let cost_1k = model.agg_cost(1_000, 1);
        let cost_10k = model.agg_cost(10_000, 1);
        let cost_100k = model.agg_cost(100_000, 1);
        assert!(cost_10k > cost_1k);
        assert!(cost_100k > cost_10k);
    }

    #[test]
    fn test_index_scan_cheaper_than_seq_scan_small_result() {
        let model = SimpleCostModel::default_model();
        let seq_cost = model.seq_scan_cost(1_000_000, 10_000).total();
        // index_scan with btree_height=3, index_pages=5, matching_rows=10, data_pages=100
        let index_cost = model.index_scan_cost_f64(3, 5, 10, 100);
        assert!(index_cost < seq_cost);
    }
}
