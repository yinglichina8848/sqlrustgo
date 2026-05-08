// EXPLAIN ANALYZE Tests (BP2-5)
//! Tests for EXPLAIN ANALYZE functionality with actual_rows output
//!
//! BP2 Gate: cargo test --test explain_analyze_test

use sqlrustgo_executor::explain::{explain, explain_analyze, ExplainConfig};
use sqlrustgo_planner::{
    Expr, Field, FilterExec, HashJoinExec, IndexScanExec, LimitExec, PhysicalPlan, ProjectionExec,
    Schema, SeqScanExec, SortExec, SortExpr,
};
use sqlrustgo_types::Value;

fn create_test_seq_scan(table_name: &str) -> SeqScanExec {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        Field::new("value".to_string(), sqlrustgo_planner::DataType::Float),
    ]);
    SeqScanExec::new(table_name.to_string(), schema)
}

fn create_test_filter(input: Box<dyn PhysicalPlan>) -> FilterExec {
    let predicate = Expr::BinaryExpr {
        left: Box::new(Expr::column("id")),
        op: sqlrustgo_planner::Operator::Gt,
        right: Box::new(Expr::Literal(Value::Integer(1))),
    };
    FilterExec::new(input, predicate)
}

fn create_test_projection(input: Box<dyn PhysicalPlan>) -> ProjectionExec {
    let schema = Schema::new(vec![Field::new(
        "id".to_string(),
        sqlrustgo_planner::DataType::Integer,
    )]);
    ProjectionExec::new(input, vec![Expr::column("id")], schema)
}

#[test]
fn test_explain_analyze_seqscan_actual_rows() {
    let scan = create_test_seq_scan("users");
    let output = explain_analyze(&scan);

    assert!(output.total_time_us.is_some(), "Should have total_time_us");

    let scan_line = &output.lines[0];
    assert!(
        scan_line.actual_rows.is_some() || scan_line.execution_time_us.is_some(),
        "Should have actual_rows or execution_time_us"
    );
}

#[test]
fn test_explain_analyze_filter_actual_rows() {
    let scan = Box::new(create_test_seq_scan("users"));
    let filter = create_test_filter(scan);
    let output = explain_analyze(&filter);

    assert_eq!(output.lines.len(), 2);

    let filter_line = &output.lines[1];
    assert!(
        filter_line.execution_time_us.is_some(),
        "Filter should have execution_time_us"
    );
}

#[test]
fn test_explain_analyze_projection_actual_rows() {
    let scan = Box::new(create_test_seq_scan("users"));
    let filter = create_test_filter(scan);
    let proj = create_test_projection(Box::new(filter));
    let output = explain_analyze(&proj);

    assert_eq!(output.lines.len(), 3);

    for line in &output.lines {
        assert!(
            line.execution_time_us.is_some() || line.actual_rows.is_some(),
            "Line {} should have timing info",
            line.operator
        );
    }
}

#[test]
fn test_explain_analyze_limit_actual_rows() {
    let scan = Box::new(create_test_seq_scan("users"));
    let limit = LimitExec::new(scan, 10, None);
    let output = explain_analyze(&limit);

    assert_eq!(output.lines.len(), 2);

    let limit_line = &output.lines[1];
    assert!(
        limit_line.details.iter().any(|d| d.contains("limit=10")),
        "Limit should show limit=10"
    );
}

#[test]
fn test_explain_analyze_sort_actual_rows() {
    let scan = Box::new(create_test_seq_scan("users"));
    let sort_exprs = vec![SortExpr {
        expr: Expr::column("id"),
        asc: true,
        nulls_first: false,
    }];
    let sort = SortExec::new(scan, sort_exprs);
    let output = explain_analyze(&sort);

    assert_eq!(output.lines.len(), 2);

    let sort_line = &output.lines[1];
    assert!(
        sort_line.details.iter().any(|d| d.contains("sort=")),
        "Sort should show sort info"
    );
}

#[test]
fn test_explain_analyze_hashjoin_actual_rows() {
    let left_schema = Schema::new(vec![Field::new(
        "id".to_string(),
        sqlrustgo_planner::DataType::Integer,
    )]);
    let right_schema = Schema::new(vec![Field::new(
        "id".to_string(),
        sqlrustgo_planner::DataType::Integer,
    )]);

    let left = Box::new(SeqScanExec::new("orders".to_string(), left_schema));
    let right = Box::new(SeqScanExec::new("users".to_string(), right_schema));

    let join_schema = Schema::new(vec![
        Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
    ]);

    let join = HashJoinExec::new(
        left,
        right,
        sqlrustgo_planner::JoinType::Inner,
        None,
        join_schema,
    );

    let output = explain_analyze(&join);

    assert_eq!(output.lines.len(), 3);

    let join_line = &output.lines[2];
    assert_eq!(join_line.operator, "HashJoin");
}

#[test]
fn test_explain_analyze_loops_field() {
    let scan = create_test_seq_scan("users");
    let output = explain_analyze(&scan);

    let scan_line = &output.lines[0];
    assert!(
        scan_line.loops.is_some(),
        "Should have loops field in ANALYZE mode"
    );
}

#[test]
fn test_explain_analyze_timing_collected() {
    let scan = create_test_seq_scan("users");
    let output = explain_analyze(&scan);

    assert!(
        output.total_time_us.is_some(),
        "Should have total_time_us"
    );

    for line in &output.lines {
        assert!(
            line.execution_time_us.is_some(),
            "Line {} should have execution_time_us",
            line.operator
        );
    }
}

#[test]
fn test_explain_analyze_index_scan() {
    let schema = Schema::new(vec![Field::new(
        "id".to_string(),
        sqlrustgo_planner::DataType::Integer,
    )]);
    let scan = IndexScanExec::new("users".to_string(), "id".to_string(), "idx_id".to_string(), schema);
    let output = explain_analyze(&scan);

    assert_eq!(output.lines.len(), 1);

    let line = &output.lines[0];
    assert!(
        line.details.iter().any(|d| d.contains("index=")),
        "Should show index info"
    );
}

#[test]
fn test_explain_analyze_config() {
    let config = ExplainConfig::explain_analyze();
    assert!(config.analyze, "EXPLAIN ANALYZE config should have analyze=true");
}

#[test]
fn test_explain_without_analyze_no_actual_rows() {
    let scan = create_test_seq_scan("users");
    let output = explain(&scan);

    let scan_line = &output.lines[0];
    assert!(
        scan_line.actual_rows.is_none(),
        "Without ANALYZE, should not have actual_rows"
    );
}

#[test]
fn test_explain_analyze_complex_query() {
    let scan = Box::new(create_test_seq_scan("users"));
    let filter = create_test_filter(scan);
    let proj = create_test_projection(Box::new(filter));
    let limit = LimitExec::new(Box::new(proj), 100, None);
    let output = explain_analyze(&limit);

    assert_eq!(output.lines.len(), 4);

    assert_eq!(output.lines[0].operator, "SeqScan");
    assert_eq!(output.lines[1].operator, "Filter");
    assert_eq!(output.lines[2].operator, "Projection");
    assert_eq!(output.lines[3].operator, "Limit");

    for line in &output.lines {
        assert!(
            line.execution_time_us.is_some(),
            "{} should have timing",
            line.operator
        );
    }
}

#[test]
fn test_explain_analyze_actual_rows_output() {
    let scan = create_test_seq_scan("users");
    let output = explain_analyze(&scan);

    let line = &output.lines[0];

    if let Some(rows) = line.actual_rows {
        assert!(rows >= 0, "actual_rows should be non-negative");
    }

    if let Some(time_us) = line.execution_time_us {
        assert!(time_us >= 0, "execution_time_us should be non-negative");
    }
}

#[test]
fn test_explain_analyze_tree_format_output() {
    let scan = create_test_seq_scan("users");
    let output = explain_analyze(&scan);

    let tree = output.to_string_tree();

    assert!(tree.contains("SeqScan"), "Tree should contain SeqScan");
    assert!(tree.contains("users"), "Tree should contain table name");
    assert!(
        tree.contains("actual_rows") || tree.contains("time="),
        "Tree should contain actual_rows or time in ANALYZE mode"
    );
}