use sqlrustgo_optimizer::cost::SimpleCostModel;

fn default_model() -> SimpleCostModel {
    SimpleCostModel::default_model()
}

fn custom_model(cpu: f64, io: f64, network: f64) -> SimpleCostModel {
    SimpleCostModel::new(cpu, io, network)
}

#[test]
fn test_index_scan_cheaper_than_seq_scan_for_selective_query() {
    let model = default_model();

    let seq_cost = model.seq_scan_cost(1_000_000, 10_000);
    let index_cost = model.index_scan_cost(100, 5, 10);

    assert!(
        index_cost < seq_cost * 0.1,
        "Index scan ({}) should be < 10% of seq scan ({})",
        index_cost,
        seq_cost
    );
}

#[test]
fn test_seq_scan_cheaper_for_large_result_sets() {
    let model = default_model();

    let seq_cost = model.seq_scan_cost(1_000_000, 10_000);
    let index_cost = model.index_scan_cost(500_000, 5, 5_000);

    assert!(
        seq_cost < index_cost * 2.0,
        "Seq scan ({}) should be competitive with index ({})",
        seq_cost,
        index_cost
    );
}

#[test]
fn test_hash_join_cheaper_than_nested_loop() {
    let model = default_model();

    let left_rows = 100_000u64;
    let right_rows = 100_000u64;

    let nested_loop_cost = model.join_cost(left_rows, right_rows, "nested_loop");
    let hash_join_cost = model.join_cost(left_rows, right_rows, "hash_join");

    assert!(
        hash_join_cost < nested_loop_cost,
        "Hash join ({}) should be cheaper than nested loop ({})",
        hash_join_cost,
        nested_loop_cost
    );

    let expected_ratio = (left_rows + right_rows) as f64 / (left_rows * right_rows) as f64;
    let actual_ratio = hash_join_cost / nested_loop_cost;

    assert!(
        (actual_ratio - expected_ratio).abs() < 0.001,
        "Hash join ratio {} should match expected {}",
        actual_ratio,
        expected_ratio
    );
}

#[test]
fn test_sort_merge_join_scales_linearly() {
    let model = default_model();

    let base_rows = 10_000u64;
    let base_cost = model.join_cost(base_rows, base_rows, "sort_merge");

    let double_rows = 20_000u64;
    let double_cost = model.join_cost(double_rows, double_rows, "sort_merge");

    let ratio = double_cost / base_cost;

    assert!(
        ratio > 1.8 && ratio < 2.5,
        "Sort merge cost ratio {} should be ~2x for doubled rows",
        ratio
    );
}

#[test]
fn test_agg_cost_increases_with_group_by() {
    let model = default_model();

    let row_count = 100_000u64;

    let cost_no_group = model.agg_cost(row_count, 0);
    let cost_one_group = model.agg_cost(row_count, 1);
    let cost_two_groups = model.agg_cost(row_count, 2);
    let cost_four_groups = model.agg_cost(row_count, 4);

    assert!(
        cost_two_groups > cost_one_group,
        "Agg with 2 group by ({}) should exceed cost with 1 ({})",
        cost_two_groups,
        cost_one_group
    );

    assert!(
        cost_four_groups > cost_two_groups,
        "Agg with 4 group by ({}) should exceed cost with 2 ({})",
        cost_four_groups,
        cost_two_groups
    );

    assert!(
        cost_one_group >= cost_no_group,
        "Agg with 1 group by ({}) should be >= cost without ({})",
        cost_one_group,
        cost_no_group
    );
}

#[test]
fn test_sort_cost_external_vs_internal() {
    let model = default_model();
    let avg_row_size = 100u32;

    let in_memory_cost = model.sort_cost(10_000, avg_row_size);
    let external_cost = model.sort_cost(1_000_000, avg_row_size);

    assert!(
        external_cost > in_memory_cost * 3.0,
        "External sort ({}) should be > 3x in-memory sort ({})",
        external_cost,
        in_memory_cost
    );
}

#[test]
fn test_custom_cost_model_parameters() {
    let io_heavy = custom_model(1.0, 100.0, 0.001);
    let seq_cost = io_heavy.seq_scan_cost(1_000_000, 10_000);

    assert!(
        seq_cost > 1_000_000.0,
        "I/O heavy seq scan should have high cost"
    );

    let cpu_heavy = custom_model(100.0, 1.0, 0.001);
    let cpu_seq_cost = cpu_heavy.seq_scan_cost(1_000_000, 10_000);

    assert!(
        cpu_seq_cost > 50_000_000.0,
        "CPU heavy seq scan should have very high cost"
    );
}

#[test]
fn test_hash_join_asymmetry() {
    let model = default_model();

    let small_left_cost = model.join_cost(1_000, 1_000_000, "hash_join");
    let small_right_cost = model.join_cost(1_000_000, 1_000, "hash_join");

    let ratio = small_left_cost / small_right_cost;

    assert!(
        (ratio - 1.0).abs() < 0.1,
        "Hash join costs should be similar: ratio={}",
        ratio
    );
}

#[test]
fn test_all_costs_are_positive() {
    let model = default_model();

    assert!(model.seq_scan_cost(100, 10) > 0.0);
    assert!(model.index_scan_cost(100, 5, 10) > 0.0);
    assert!(model.join_cost(100, 100, "nested_loop") > 0.0);
    assert!(model.join_cost(100, 100, "hash_join") > 0.0);
    assert!(model.join_cost(100, 100, "sort_merge") > 0.0);
    assert!(model.agg_cost(100, 2) > 0.0);
    assert!(model.sort_cost(100, 100) > 0.0);
}

#[test]
fn test_seq_scan_cost_scales_linearly() {
    let model = default_model();

    let base_cost = model.seq_scan_cost(1_000_000, 10);
    let double_cost = model.seq_scan_cost(2_000_000, 10);

    let ratio = double_cost / base_cost;

    assert!(
        (ratio - 2.0).abs() < 0.01,
        "Seq scan cost should scale linearly with row count: ratio={}",
        ratio
    );
}

#[test]
fn test_seq_scan_cost_scales_with_pages() {
    let model = default_model();

    let base_cost = model.seq_scan_cost(10, 100);
    let double_pages_cost = model.seq_scan_cost(10, 200);

    let ratio = double_pages_cost / base_cost;

    assert!(
        (ratio - 2.0).abs() < 0.01,
        "Seq scan cost should scale linearly with pages: ratio={}",
        ratio
    );
}
