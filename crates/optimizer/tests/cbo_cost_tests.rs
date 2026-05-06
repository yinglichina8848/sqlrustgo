//! Optimizer Cost Model Integration Tests

use sqlrustgo_optimizer::{
    cost::{CboOptimizer, SimpleCostModel},
    CostModel,
};

mod simple_cost_model_tests {
    use super::*;

    #[test]
    fn test_cost_model_construction() {
        let model = SimpleCostModel::new(1.0, 10.0, 0.001);
        let cost = model.estimate_cost(&42i32);
        assert_eq!(cost, 100.0);
    }

    #[test]
    fn test_cost_model_default() {
        let model = SimpleCostModel::default_model();
        let cost = model.estimate_cost(&42i32);
        assert_eq!(cost, 100.0);
    }

    #[test]
    fn test_seq_scan_cost_calculation() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(1000, 10);
        assert!((cost - 1100.0).abs() < 0.001);
    }

    #[test]
    fn test_seq_scan_cost_empty() {
        let model = SimpleCostModel::default_model();
        let cost = model.seq_scan_cost(0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_index_scan_cost_calculation() {
        let model = SimpleCostModel::default_model();
        let cost = model.index_scan_cost(100, 2, 5);
        assert!((cost - 170.0).abs() < 0.001);
    }

    #[test]
    fn test_index_scan_cost_empty() {
        let model = SimpleCostModel::default_model();
        let cost = model.index_scan_cost(0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_join_cost_nested_loop() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "nested_loop");
        assert!((cost - 2_000_000.0).abs() < 0.001);
    }

    #[test]
    fn test_join_cost_hash_join() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "hash_join");
        assert!((cost - 3000.0).abs() < 0.001);
    }

    #[test]
    fn test_join_cost_sort_merge() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "sort_merge");
        assert!((cost - 6000.0).abs() < 0.001);
    }

    #[test]
    fn test_join_cost_unknown_method() {
        let model = SimpleCostModel::default_model();
        let cost = model.join_cost(1000, 2000, "unknown");
        assert!((cost - 3000.0).abs() < 0.001);
    }

    #[test]
    fn test_agg_cost_with_group_by() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(10000, 2);
        assert!(cost > 19000.0);
    }

    #[test]
    fn test_agg_cost_no_group_by() {
        let model = SimpleCostModel::default_model();
        let cost = model.agg_cost(1000, 0);
        assert!((cost - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_sort_cost_in_memory() {
        let model = SimpleCostModel::default_model();
        let cost = model.sort_cost(1000, 100);
        assert!(cost > 9000.0);
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
        assert_eq!(cost, 100.0);
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
        let cost = cbo.estimate_scan_cost(1000, 10);
        assert!((cost - 1100.0).abs() < 0.001);
    }

    #[test]
    fn test_cbo_estimate_scan_cost_different_values() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_scan_cost(5000, 100);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_index_scan_cost() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_index_scan_cost(100, 2, 5);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_index_scan_cost_zero() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_index_scan_cost(0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost_nested_loop() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "nested_loop");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost_hash_join() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "hash_join");
        assert!(cost > 0.0);
    }

    #[test]
    fn test_cbo_estimate_join_cost_sort_merge() {
        let cbo = CboOptimizer::new();
        let cost = cbo.estimate_join_cost(1000, 2000, "sort_merge");
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
        assert!((sort_merge - 40_000.0).abs() < 0.001);
    }

    #[test]
    fn test_agg_cost_increases_with_groups() {
        let model = SimpleCostModel::default_model();
        let cost_1_col = model.agg_cost(10_000, 1);
        let cost_2_cols = model.agg_cost(10_000, 2);
        let cost_4_cols = model.agg_cost(10_000, 4);
        assert!(cost_4_cols > cost_2_cols);
        assert!(cost_2_cols > cost_1_col);
    }

    #[test]
    fn test_index_scan_cheaper_than_seq_scan_small_result() {
        let model = SimpleCostModel::default_model();
        let seq_cost = model.seq_scan_cost(1_000_000, 10_000);
        let index_cost = model.index_scan_cost(100, 5, 10);
        assert!(index_cost < seq_cost);
    }
}
