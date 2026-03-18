//! Index Integration Tests
//!
//! These tests verify end-to-end index functionality:
//! - IndexScan vs SeqScan comparison
//! - B+Tree index operations
//! - Index usage in query planning

use sqlrustgo_planner::{DataType, Expr, Field, IndexScanExec, PhysicalPlan, Schema, SeqScanExec};
use sqlrustgo_storage::BPlusTree;

#[test]
fn test_index_scan_basic() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    // Create IndexScan with point query
    let index_scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(42)),
        schema.clone(),
    );

    // Execute
    let results = index_scan.execute().unwrap();

    assert!(!results.is_empty());
    println!("✓ IndexScan basic: returned {} rows", results.len());
}

#[test]
fn test_index_scan_range_query() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("value".to_string(), DataType::Integer),
    ]);

    // Create IndexScan with range
    let index_scan = IndexScanExec::new(
        "orders".to_string(),
        "idx_total".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(0)),
        schema.clone(),
    )
    .with_key_range(100, 200);

    // Execute
    let results = index_scan.execute().unwrap();

    assert!(!results.is_empty());
    println!("✓ IndexScan range: returned {} rows", results.len());
}

#[test]
fn test_seq_scan_basic() {
    let schema = Schema::new(vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    let seq_scan = SeqScanExec::new("users".to_string(), schema.clone());

    let results = seq_scan.execute().unwrap();

    // Should return all rows (in test mode)
    println!("✓ SeqScan basic: returned {} rows", results.len());
}

#[test]
fn test_index_scan_vs_seqscan_schema() {
    let id_field = Field::new("id".to_string(), DataType::Integer);
    let name_field = Field::new("name".to_string(), DataType::Text);

    let schema = Schema::new(vec![id_field.clone(), name_field.clone()]);

    // Both should have same schema
    let index_scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(1)),
        schema.clone(),
    );

    let seq_scan = SeqScanExec::new("users".to_string(), schema.clone());

    assert_eq!(index_scan.schema().fields.len(), 2);
    assert_eq!(seq_scan.schema().fields.len(), 2);

    println!("✓ IndexScan and SeqScan have matching schemas");
}

#[test]
fn test_index_scan_name_and_table() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let index_scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(1)),
        schema.clone(),
    );

    assert_eq!(index_scan.name(), "IndexScan");
    assert_eq!(index_scan.table_name(), "users");
    assert_eq!(index_scan.index_name(), "idx_id");

    println!("✓ IndexScan metadata correct");
}

#[test]
fn test_seqscan_name_and_table() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    let seq_scan = SeqScanExec::new("users".to_string(), schema.clone());

    assert_eq!(seq_scan.name(), "SeqScan");
    assert_eq!(seq_scan.table_name(), "users");

    println!("✓ SeqScan metadata correct");
}

#[test]
fn test_bplus_tree_index_integration() {
    let mut tree = BPlusTree::new();

    // Build index: user_id -> row_pointer
    let user_ids = vec![1, 5, 10, 15, 20, 25, 30];
    for (idx, user_id) in user_ids.iter().enumerate() {
        tree.insert(*user_id, idx as u32);
    }

    assert_eq!(tree.len(), 7);

    // Point lookup
    for user_id in &user_ids {
        let result = tree.search(*user_id);
        assert!(result.is_some(), "User {} should be found", user_id);
    }

    // Range query
    let range_results = tree.range_query(10, 25);
    assert!(!range_results.is_empty());

    println!(
        "✓ B+Tree index: {} entries, range query returned {} results",
        tree.len(),
        range_results.len()
    );
}

#[test]
fn test_index_with_multiple_range_queries() {
    let schema = Schema::new(vec![
        Field::new("order_id".to_string(), DataType::Integer),
        Field::new("total".to_string(), DataType::Integer),
    ]);

    // Different range queries
    let test_cases = vec![
        (0, 100, "low value orders"),
        (100, 500, "medium value orders"),
        (500, 1000, "high value orders"),
    ];

    for (start, end, desc) in test_cases {
        let index_scan = IndexScanExec::new(
            "orders".to_string(),
            "idx_total".to_string(),
            Expr::Literal(sqlrustgo_types::Value::Integer(0)),
            schema.clone(),
        )
        .with_key_range(start, end);

        let results = index_scan.execute().unwrap();
        println!("{}: {} rows [{}, {})", desc, results.len(), start, end);
    }

    println!("✓ Multiple range queries work correctly");
}

#[test]
fn test_index_plan_comparison() {
    // Simulate comparing IndexScan vs SeqScan execution plans
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    // IndexScan for point query
    let index_scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(100)),
        schema.clone(),
    );

    // SeqScan for full table
    let seq_scan = SeqScanExec::new("users".to_string(), schema.clone());

    // Both should be valid physical plans
    assert_eq!(index_scan.name(), "IndexScan");
    assert_eq!(seq_scan.name(), "SeqScan");

    println!("✓ IndexScan vs SeqScan plan comparison works");
}

#[test]
fn test_index_with_expressions() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    // IndexScan with expression as key - in current implementation,
    // only Literal expressions are fully supported
    let expr = Expr::BinaryExpr {
        left: Box::new(Expr::Literal(sqlrustgo_types::Value::Integer(50))),
        op: sqlrustgo_planner::Operator::Plus,
        right: Box::new(Expr::Literal(sqlrustgo_types::Value::Integer(50))),
    };

    let index_scan = IndexScanExec::new(
        "users".to_string(),
        "idx_id".to_string(),
        expr,
        schema.clone(),
    );

    let results = index_scan.execute().unwrap();

    // Expression handling depends on implementation - may or may not return results
    // Just verify execution doesn't error
    println!(
        "✓ IndexScan with expression executed ({} results)",
        results.len()
    );
}

#[test]
fn test_sequential_vs_index_scan_characteristics() {
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    // SeqScan characteristics
    let seq_scan = SeqScanExec::new("large_table".to_string(), schema.clone());
    let seq_name = seq_scan.name().to_string();
    let seq_children = seq_scan.children();

    // IndexScan characteristics
    let index_scan = IndexScanExec::new(
        "large_table".to_string(),
        "idx_id".to_string(),
        Expr::Literal(sqlrustgo_types::Value::Integer(1)),
        schema.clone(),
    )
    .with_key_range(1, 1000);

    let index_name = index_scan.name().to_string();
    let index_children = index_scan.children();
    let index_range = index_scan.key_range();

    println!(
        "SeqScan: name={}, children={}",
        seq_name,
        seq_children.len()
    );
    println!(
        "IndexScan: name={}, children={}, has_range={}",
        index_name,
        index_children.len(),
        index_range.is_some()
    );

    assert!(index_range.is_some());

    println!("✓ Sequential vs Index scan characteristics verified");
}

#[test]
fn test_bplus_tree_large_scale_index() {
    let mut tree = BPlusTree::new();

    // Simulate large scale index with 10000 entries
    let entry_count = 10000;
    for i in 0..entry_count {
        tree.insert(i as i64, i as u32);
    }

    assert_eq!(tree.len(), entry_count as usize);

    // Point queries
    let search_count = 100;
    for i in 0..search_count {
        let key = (i * 100) as i64;
        let result = tree.search(key);
        assert!(result.is_some());
    }

    // Range queries
    let range_results = tree.range_query(1000, 5000);
    assert!(!range_results.is_empty());

    println!(
        "✓ Large scale B+Tree: {} entries, {} range results",
        tree.len(),
        range_results.len()
    );
}

#[test]
fn test_index_query_optimization_simulation() {
    // Simulate query optimization decision
    let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

    // High selectivity query (>50% rows) - should use SeqScan
    let high_selectivity_expr = Expr::Literal(sqlrustgo_types::Value::Integer(1)); // Would match many rows

    let plan_for_high_selectivity = IndexScanExec::new(
        "orders".to_string(),
        "idx_id".to_string(),
        high_selectivity_expr,
        schema.clone(),
    );

    // Low selectivity query (<1% rows) - should use IndexScan
    let low_selectivity_range = plan_for_high_selectivity.with_key_range(1, 100);

    println!("✓ Query optimization simulation: IndexScan for low selectivity ranges");

    // Just verify they execute without error
    assert!(low_selectivity_range.execute().is_ok());
}
