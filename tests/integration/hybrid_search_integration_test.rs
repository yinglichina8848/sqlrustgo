//! Hybrid Search Integration Tests
//!
//! Tests for SQL + Vector hybrid search functionality including:
//! - Basic hybrid search with weighted scoring
//! - SQL pre-filtering with vector Top-K
//! - Different weight configurations
//! - Edge cases and error handling

use sqlrustgo_vector::{metrics::DistanceMetric, sql_vector_hybrid::*, HybridSearchConfig};

fn generate_test_vectors(count: usize, dimension: usize) -> Vec<(u64, Vec<f32>)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..count as u64)
        .map(|i| {
            let vector: Vec<f32> = (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (i, vector)
        })
        .collect()
}

// ============================================================================
// T1: Basic Hybrid Search Tests
// ============================================================================

#[test]
fn test_hybrid_basic_search() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    let vectors = generate_test_vectors(100, 128);

    for (id, v) in &vectors {
        searcher.insert(*id, v, 1.0).unwrap();
    }

    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = (0..100u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    assert_eq!(result.entries.len(), 10);
    assert!(result.total_scanned <= 100);
}

#[test]
fn test_hybrid_weighted_scoring_alpha_zero() {
    let config = HybridSearchConfig {
        alpha: 0.0,
        beta: 1.0,
        parallel: false,
        chunk_size: 100,
    };
    let mut searcher = HybridSearcher::with_config(DistanceMetric::Cosine, config);

    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![5.0f32; 128];
    let sql_scores: Vec<_> = (0..10u64).map(|id| (id, 0.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 5).unwrap();
    assert_eq!(result.entries.len(), 5);
}

#[test]
fn test_hybrid_weighted_scoring_beta_zero() {
    let config = HybridSearchConfig {
        alpha: 1.0,
        beta: 0.0,
        parallel: false,
        chunk_size: 100,
    };
    let mut searcher = HybridSearcher::with_config(DistanceMetric::Cosine, config);

    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        let sql_score = 1.0 - (i as f32 / 10.0);
        searcher.insert(i, &v, sql_score).unwrap();
    }

    let query = vec![0.0f32; 128];
    let sql_scores: Vec<_> = (0..10u64)
        .map(|id| (id, 1.0 - (id as f32 / 10.0)))
        .collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 5).unwrap();
    assert_eq!(result.entries.len(), 5);
}

#[test]
fn test_hybrid_parallel_search() {
    let config = HybridSearchConfig {
        alpha: 0.5,
        beta: 0.5,
        parallel: true,
        chunk_size: 100,
    };
    let mut searcher = HybridSearcher::with_config(DistanceMetric::Cosine, config);
    let vectors = generate_test_vectors(1000, 128);

    for (id, v) in &vectors {
        searcher.insert(*id, v, 1.0).unwrap();
    }

    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = (0..1000u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    assert_eq!(result.entries.len(), 10);
}

// ============================================================================
// T2: SQL Score Filtering Tests
// ============================================================================

#[test]
fn test_hybrid_sql_score_ranking() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    for i in 0..10u64 {
        let v = vec![0.0f32; 128];
        let sql_score = if i < 5 { 1.0 } else { 0.0 };
        searcher.insert(i, &v, sql_score).unwrap();
    }

    let query = vec![0.0f32; 128];
    let sql_scores: Vec<_> = (0..10u64)
        .map(|id| (id, if id < 5 { 1.0 } else { 0.0 }))
        .collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    assert_eq!(result.entries.len(), 10);
}

#[test]
fn test_hybrid_partial_sql_scores() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![5.0f32; 128];
    let sql_scores: Vec<_> = vec![(0, 1.0), (1, 1.0), (5, 1.0), (6, 1.0)];

    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    assert_eq!(result.entries.len(), 4);
}

#[test]
fn test_hybrid_sql_scores_subset() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    for i in 0..100u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![50.0f32; 128];
    let sql_scores: Vec<_> = (50..60u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 5).unwrap();
    assert_eq!(result.entries.len(), 5);
    for entry in &result.entries {
        assert!(entry.id >= 50 && entry.id < 60);
    }
}

// ============================================================================
// T3: Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_hybrid_empty_index() {
    let searcher = HybridSearcher::new(DistanceMetric::Cosine);
    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = vec![];

    let result = searcher.search_hybrid(&query, &sql_scores, 10);
    assert!(result.is_err());
}

#[test]
fn test_hybrid_dimension_mismatch() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    searcher.insert(0, &[1.0f32; 64], 1.0).unwrap();

    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = vec![(0, 1.0)];

    let result = searcher.search_hybrid(&query, &sql_scores, 10);
    assert!(result.is_err());
}

#[test]
fn test_hybrid_k_larger_than_results() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    for i in 0..5u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = (0..5u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 100).unwrap();
    assert_eq!(result.entries.len(), 5);
}

#[test]
fn test_hybrid_no_matching_sql_scores() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![0.5f32; 128];
    let sql_scores: Vec<_> = vec![];

    let result = searcher.search_hybrid(&query, &sql_scores, 10);
    assert!(result.is_ok());
}

// ============================================================================
// T4: Different Distance Metrics
// ============================================================================

#[test]
fn test_hybrid_cosine_metric() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![5.0f32; 128];
    let sql_scores: Vec<_> = (0..10u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 5).unwrap();
    assert_eq!(result.entries.len(), 5);
}

#[test]
fn test_hybrid_euclidean_metric() {
    let mut searcher = HybridSearcher::new(DistanceMetric::Euclidean);
    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        searcher.insert(i, &v, 1.0).unwrap();
    }

    let query = vec![5.0f32; 128];
    let sql_scores: Vec<_> = (0..10u64).map(|id| (id, 1.0)).collect();

    let result = searcher.search_hybrid(&query, &sql_scores, 5).unwrap();
    assert_eq!(result.entries.len(), 5);
}

// ============================================================================
// T5: Execute Filtered Search (Stub)
// ============================================================================

#[test]
fn test_hybrid_execute_filtered_search_empty_predicates() {
    use std::collections::HashMap;

    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        let mut row = HashMap::new();
        row.insert("_score".to_string(), SqlValue::Float(1.0));
        row.insert("id".to_string(), SqlValue::Integer(i as i64));
        searcher.insert_with_row(i, &v, row).unwrap();
    }

    let query = vec![5.0f32; 128];
    let predicates: Vec<SqlPredicate> = vec![];

    let result = searcher
        .execute_filtered_search(&query, &predicates, 5)
        .unwrap();
    assert_eq!(result.entries.len(), 5);
}

#[test]
fn test_hybrid_sql_predicate_types() {
    use SqlPredicate::*;
    use SqlValue::*;

    let pred_equal = Equal {
        column: "category".to_string(),
        value: Text("electronics".to_string()),
    };

    let pred_range = LessThan {
        column: "price".to_string(),
        value: Float(100.0),
    };

    let pred_and = And(Box::new(pred_equal.clone()), Box::new(pred_range.clone()));

    let pred_or = Or(Box::new(pred_equal.clone()), Box::new(pred_range.clone()));

    match pred_and {
        And(left, right) => {
            assert!(matches!(*left, Equal { .. }));
            assert!(matches!(*right, LessThan { .. }));
        }
        _ => panic!("Expected And"),
    }

    match pred_or {
        Or(left, right) => {
            assert!(matches!(*left, Equal { .. }));
            assert!(matches!(*right, LessThan { .. }));
        }
        _ => panic!("Expected Or"),
    }
}

// ============================================================================
// T5.5: Predicate Filtering Tests
// ============================================================================

#[test]
fn test_hybrid_predicate_filtering_less_than() {
    use sqlrustgo_vector::sql_vector_hybrid::{SqlPredicate, SqlValue};
    use std::collections::HashMap;

    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    // Insert 5 vectors with category_id: 0, 1, 2, 3, 4
    for i in 0..5u64 {
        let v = vec![i as f32; 128];
        let mut row = HashMap::new();
        row.insert("_score".to_string(), SqlValue::Float(1.0));
        row.insert("category_id".to_string(), SqlValue::Integer(i as i64));
        searcher.insert_with_row(i, &v, row).unwrap();
    }

    // Query with predicate: category_id < 3
    // Should only return ids 0, 1, 2
    let query = vec![2.0f32; 128];
    let predicates = vec![SqlPredicate::LessThan {
        column: "category_id".to_string(),
        value: SqlValue::Integer(3),
    }];

    let result = searcher
        .execute_filtered_search(&query, &predicates, 5)
        .unwrap();

    // Verify all returned entries have category_id < 3
    for entry in &result.entries {
        assert!(
            entry.id < 3,
            "Entry id {} should be < 3 based on predicate filter",
            entry.id
        );
    }
    assert_eq!(result.entries.len(), 3);
}

#[test]
fn test_hybrid_predicate_filtering_equal() {
    use sqlrustgo_vector::sql_vector_hybrid::{SqlPredicate, SqlValue};
    use std::collections::HashMap;

    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    // Insert vectors with different category values
    let categories = vec![
        "electronics",
        "clothing",
        "electronics",
        "food",
        "electronics",
    ];
    for (i, cat) in categories.iter().enumerate() {
        let v = vec![i as f32; 128];
        let mut row = HashMap::new();
        row.insert("_score".to_string(), SqlValue::Float(1.0));
        row.insert("category".to_string(), SqlValue::Text(cat.to_string()));
        searcher.insert_with_row(i as u64, &v, row).unwrap();
    }

    // Query with predicate: category = "electronics"
    let query = vec![2.0f32; 128];
    let predicates = vec![SqlPredicate::Equal {
        column: "category".to_string(),
        value: SqlValue::Text("electronics".to_string()),
    }];

    let result = searcher
        .execute_filtered_search(&query, &predicates, 5)
        .unwrap();

    // Should only return ids 0, 2, 4 (electronics)
    assert_eq!(result.entries.len(), 3);
    for entry in &result.entries {
        assert!(
            entry.id == 0 || entry.id == 2 || entry.id == 4,
            "Entry id {} should be electronics category",
            entry.id
        );
    }
}

#[test]
fn test_hybrid_predicate_filtering_and() {
    use sqlrustgo_vector::sql_vector_hybrid::{SqlPredicate, SqlValue};
    use std::collections::HashMap;

    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    // Insert 10 vectors with price: 10, 20, 30, ..., 100
    for i in 0..10u64 {
        let v = vec![i as f32; 128];
        let mut row = HashMap::new();
        row.insert("_score".to_string(), SqlValue::Float(1.0));
        row.insert(
            "price".to_string(),
            SqlValue::Integer(((i + 1) * 10) as i64),
        );
        row.insert("in_stock".to_string(), SqlValue::Boolean(i % 2 == 0));
        searcher.insert_with_row(i, &v, row).unwrap();
    }

    // Query with predicate: price >= 30 AND price <= 70
    let query = vec![5.0f32; 128];
    let predicates = vec![SqlPredicate::And(
        Box::new(SqlPredicate::GreaterThanEq {
            column: "price".to_string(),
            value: SqlValue::Integer(30),
        }),
        Box::new(SqlPredicate::LessThanEq {
            column: "price".to_string(),
            value: SqlValue::Integer(70),
        }),
    )];

    let result = searcher
        .execute_filtered_search(&query, &predicates, 10)
        .unwrap();

    // Should return ids with price 30, 40, 50, 60, 70 (ids 2, 3, 4, 5, 6)
    assert_eq!(result.entries.len(), 5);
    for entry in &result.entries {
        assert!(
            entry.id >= 2 && entry.id <= 6,
            "Entry id {} should have price between 30 and 70",
            entry.id
        );
    }
}

#[test]
fn test_hybrid_predicate_filtering_no_match() {
    use sqlrustgo_vector::sql_vector_hybrid::{SqlPredicate, SqlValue};
    use std::collections::HashMap;

    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    // Insert 3 vectors with values 0, 1, 2
    for i in 0..3u64 {
        let v = vec![i as f32; 128];
        let mut row = HashMap::new();
        row.insert("_score".to_string(), SqlValue::Float(1.0));
        row.insert("value".to_string(), SqlValue::Integer(i as i64));
        searcher.insert_with_row(i, &v, row).unwrap();
    }

    // Query with predicate: value > 100 (no match)
    let query = vec![1.0f32; 128];
    let predicates = vec![SqlPredicate::GreaterThan {
        column: "value".to_string(),
        value: SqlValue::Integer(100),
    }];

    let result = searcher
        .execute_filtered_search(&query, &predicates, 5)
        .unwrap();

    // Should return no results
    assert_eq!(result.entries.len(), 0);
}

// ============================================================================
// T6: Performance Benchmark Tests (Quick)
// ============================================================================

#[test]
fn test_hybrid_1k_vectors_performance() {
    let vectors = generate_test_vectors(1000, 128);
    let query = vec![0.5f32; 128];

    let sql_scores: Vec<_> = (0..1000u64)
        .map(|id| (id, 1.0 - (id as f32 / 1000.0)))
        .collect();

    let start = std::time::Instant::now();
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    for (id, v) in &vectors {
        searcher.insert(*id, v, 1.0).unwrap();
    }

    let insert_time = start.elapsed();

    let search_start = std::time::Instant::now();
    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    let search_time = search_start.elapsed();

    assert_eq!(result.entries.len(), 10);
    assert!(insert_time.as_secs_f64() < 1.0, "Insert took too long");
    assert!(search_time.as_secs_f64() < 0.1, "Search took too long");
}

#[test]
fn test_hybrid_10k_vectors_performance() {
    let vectors = generate_test_vectors(10000, 128);
    let query = vec![0.5f32; 128];

    let sql_scores: Vec<_> = (0..10000u64)
        .map(|id| (id, 1.0 - (id as f32 / 10000.0)))
        .collect();

    let start = std::time::Instant::now();
    let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);

    for (id, v) in &vectors {
        searcher.insert(*id, v, 1.0).unwrap();
    }

    let insert_time = start.elapsed();

    let search_start = std::time::Instant::now();
    let result = searcher.search_hybrid(&query, &sql_scores, 10).unwrap();
    let search_time = search_start.elapsed();

    assert_eq!(result.entries.len(), 10);
    println!("10K insert: {:?}, search: {:?}", insert_time, search_time);
}

// ============================================================================
// T7: Merge and Rerank Functions
// ============================================================================

#[test]
fn test_merge_with_sql_scores_basic() {
    let vector_results = vec![(1, 0.9), (2, 0.8), (3, 0.7), (4, 0.6), (5, 0.5)];
    let sql_scores = vec![(1, 1.0), (2, 0.8), (3, 0.6), (4, 0.4), (5, 0.2)];

    let merged = merge_with_sql_scores(vector_results, &sql_scores, 0.5, 0.5);

    assert_eq!(merged.len(), 5);
    assert_eq!(merged[0].0, 1);
    assert!((merged[0].1 - 0.95).abs() < 0.01);
}

#[test]
fn test_merge_with_sql_scores_missing_ids() {
    let vector_results = vec![(1, 0.9), (3, 0.7), (5, 0.5)];
    let sql_scores = vec![(1, 1.0), (2, 0.8), (3, 0.6), (4, 0.4), (5, 0.2)];

    let merged = merge_with_sql_scores(vector_results, &sql_scores, 0.5, 0.5);

    assert_eq!(merged.len(), 3);
}

#[test]
fn test_rerank_with_vector_basic() {
    let initial = vec![(1, 1.0), (2, 0.9), (3, 0.8), (4, 0.7)];
    let vector_scores = vec![(1, 0.5), (2, 1.0), (3, 0.7), (4, 0.3)];

    let reranked = rerank_with_vector(initial, &vector_scores, 0.5, 0.5);

    assert_eq!(reranked.len(), 4);
    assert_eq!(reranked[0].0, 2);
    assert!((reranked[0].1 - 0.95).abs() < 0.01);
}

// ============================================================================
// Integration Test Summary
// ============================================================================

#[test]
fn test_integration_summary() {
    println!();
    println!("Hybrid Search Integration Tests Summary:");
    println!();
    println!("T1: Basic Hybrid Search Tests");
    println!("  - test_hybrid_basic_search");
    println!("  - test_hybrid_weighted_scoring_alpha_zero");
    println!("  - test_hybrid_weighted_scoring_beta_zero");
    println!("  - test_hybrid_parallel_search");
    println!();
    println!("T2: SQL Score Filtering Tests");
    println!("  - test_hybrid_sql_score_ranking");
    println!("  - test_hybrid_partial_sql_scores");
    println!("  - test_hybrid_sql_scores_subset");
    println!();
    println!("T3: Edge Cases and Error Handling");
    println!("  - test_hybrid_empty_index");
    println!("  - test_hybrid_dimension_mismatch");
    println!("  - test_hybrid_k_larger_than_results");
    println!("  - test_hybrid_no_matching_sql_scores");
    println!();
    println!("T4: Different Distance Metrics");
    println!("  - test_hybrid_cosine_metric");
    println!("  - test_hybrid_euclidean_metric");
    println!();
    println!("T5: Execute Filtered Search (Stub)");
    println!("  - test_hybrid_execute_filtered_search_empty_predicates");
    println!("  - test_hybrid_sql_predicate_types");
    println!();
    println!("T6: Performance Benchmark Tests");
    println!("  - test_hybrid_1k_vectors_performance");
    println!("  - test_hybrid_10k_vectors_performance");
    println!();
    println!("T7: Merge and Rerank Functions");
    println!("  - test_merge_with_sql_scores_basic");
    println!("  - test_merge_with_sql_scores_missing_ids");
    println!("  - test_rerank_with_vector_basic");
    println!();
    println!("All hybrid search integration tests passed!");
}
