// Optimizer Cost Model Tests
use sqlrustgo_optimizer::cost::SimpleCostModel;
use sqlrustgo_optimizer::stats::TableStats;

#[test]
fn test_cost_model_new() {
    let _model = SimpleCostModel::new(1.0, 10.0, 0.001);
    // Just ensure it can be created
    assert!(true);
}

#[test]
fn test_cost_model_default() {
    let _model = SimpleCostModel::default_model();
    // Just ensure it can be created
    assert!(true);
}

#[test]
fn test_seq_scan_cost() {
    let model = SimpleCostModel::default_model();
    let cost = model.seq_scan_cost(1000, 10);
    assert!(cost > 0.0);
}

#[test]
fn test_index_scan_cost() {
    let model = SimpleCostModel::default_model();
    let cost = model.index_scan_cost(100, 5, 10);
    assert!(cost > 0.0);
}

#[test]
fn test_join_cost() {
    let model = SimpleCostModel::default_model();
    let cost = model.join_cost(1000, 500, "nested_loop");
    assert!(cost > 0.0);

    let cost_hash = model.join_cost(1000, 500, "hash_join");
    assert!(cost_hash > 0.0);

    let cost_merge = model.join_cost(1000, 500, "merge_join");
    assert!(cost_merge > 0.0);
}

#[test]
fn test_agg_cost() {
    let model = SimpleCostModel::default_model();
    let cost = model.agg_cost(1000, 2);
    assert!(cost > 0.0);
}

#[test]
fn test_sort_cost() {
    let model = SimpleCostModel::default_model();
    let cost = model.sort_cost(1000, 100);
    assert!(cost > 0.0);
}

#[test]
fn test_estimate_with_stats() {
    let model = SimpleCostModel::default_model();
    let stats = TableStats::new("test".to_string());
    let cost = model.estimate_with_stats(&stats);
    assert!(cost > 0.0);
}
