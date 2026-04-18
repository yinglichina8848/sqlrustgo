// CBO (Cost-Based Optimizer) Integration Tests
//
// These tests verify the cost estimation and optimization functionality
// that was added for Issue #1597 (CBO 优化器启用与统计信息)

use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};

#[test]
fn test_cbo_enabled_by_default() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let engine = ExecutionEngine::new(storage);
    assert!(engine.is_cbo_enabled());
}

#[test]
fn test_cbo_can_be_disabled() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::with_cbo(storage, false);
    assert!(!engine.is_cbo_enabled());
    engine.set_cbo_enabled(true);
    assert!(engine.is_cbo_enabled());
}

#[test]
fn test_analyze_collects_table_stats() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice', 30)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob', 25)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (3, 'Charlie', 30)")
        .unwrap();

    let result = engine.execute("ANALYZE users").unwrap();
    assert_eq!(result.affected_rows, 1);
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(3));

    let stats = engine.get_table_stats();
    let stats_guard = stats.read().unwrap();
    let table_stats = stats_guard.table_stats.get("users").unwrap();
    assert_eq!(table_stats.row_count, 3);
}

#[test]
fn test_estimate_row_count() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (1, 'Alice')")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (2, 'Bob')")
        .unwrap();
    engine
        .execute("INSERT INTO users VALUES (3, 'Charlie')")
        .unwrap();

    // Before ANALYZE, should return default estimate
    assert_eq!(engine.estimate_row_count("users"), 1000);

    // After ANALYZE, should return actual count
    engine.execute("ANALYZE users").unwrap();
    assert_eq!(engine.estimate_row_count("users"), 3);
}

#[test]
fn test_estimate_selectivity() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
            .unwrap();
    }

    // Before ANALYZE, should return default selectivity
    let selectivity = engine.estimate_selectivity("users", "id");
    assert_eq!(selectivity, 0.1); // Default 10%

    // After ANALYZE with distinct_count, should return better estimate
    engine.execute("ANALYZE users").unwrap();
    let selectivity = engine.estimate_selectivity("users", "id");
    assert_eq!(selectivity, 0.01); // 1/100 distinct values
}

#[test]
fn test_estimate_index_benefit() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    for i in 0..1000 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
            .unwrap();
    }

    // High selectivity (1/1000) - index should be very beneficial
    let high_sel = engine.estimate_selectivity("users", "id");
    let benefit = engine.estimate_index_benefit("users", high_sel);
    assert!(benefit > 0.0); // Index should be beneficial

    // With ANALYZE, we get actual stats
    engine.execute("ANALYZE users").unwrap();
    let benefit_after_analyze = engine.estimate_index_benefit("users", high_sel);
    assert!(benefit_after_analyze > 0.0);
}

#[test]
fn test_should_use_index() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    for i in 0..10000 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
            .unwrap();
    }

    // With low selectivity (high cardinality), index is beneficial
    let use_index = engine.should_use_index("users", "id");
    assert!(use_index);

    // After ANALYZE, should still recommend index for high cardinality
    engine.execute("ANALYZE users").unwrap();
    let use_index_after = engine.should_use_index("users", "id");
    assert!(use_index_after);
}

#[test]
fn test_estimate_join_cost() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE orders (id INTEGER, user_id INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();

    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO orders VALUES ({}, {})", i, i % 10))
            .unwrap();
    }
    for i in 0..10 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'User{}')", i, i))
            .unwrap();
    }

    let hash_cost = engine.estimate_join_cost("orders", "users", "hash");
    let nl_cost = engine.estimate_join_cost("orders", "users", "nested_loop");
    let merge_cost = engine.estimate_join_cost("orders", "users", "merge");

    // All costs should be positive
    assert!(hash_cost > 0.0);
    assert!(nl_cost > 0.0);
    assert!(merge_cost > 0.0);
}

#[test]
fn test_optimize_join_order() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // Create tables of different sizes
    engine.execute("CREATE TABLE large (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE medium (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE small (id INTEGER)").unwrap();

    for i in 0..1000 {
        engine
            .execute(&format!("INSERT INTO large VALUES ({})", i))
            .unwrap();
    }
    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO medium VALUES ({})", i))
            .unwrap();
    }
    for i in 0..10 {
        engine
            .execute(&format!("INSERT INTO small VALUES ({})", i))
            .unwrap();
    }

    // Analyze to get accurate row counts
    engine.execute("ANALYZE large").unwrap();
    engine.execute("ANALYZE medium").unwrap();
    engine.execute("ANALYZE small").unwrap();

    let tables = vec!["large", "medium", "small"];
    let optimal = engine.optimize_join_order(&tables);

    // Smallest table should be first after ANALYZE
    assert_eq!(optimal[0], "small");
    // Should have all tables
    assert_eq!(optimal.len(), 3);
}

#[test]
fn test_optimize_join_order_respects_stats() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine.execute("CREATE TABLE t1 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t2 (id INTEGER)").unwrap();
    engine.execute("CREATE TABLE t3 (id INTEGER)").unwrap();

    for i in 0..500 {
        engine
            .execute(&format!("INSERT INTO t1 VALUES ({})", i))
            .unwrap();
    }
    for i in 0..50 {
        engine
            .execute(&format!("INSERT INTO t2 VALUES ({})", i))
            .unwrap();
    }
    for i in 0..5 {
        engine
            .execute(&format!("INSERT INTO t3 VALUES ({})", i))
            .unwrap();
    }

    // Analyze to get accurate stats
    engine.execute("ANALYZE t1").unwrap();
    engine.execute("ANALYZE t2").unwrap();
    engine.execute("ANALYZE t3").unwrap();

    let tables = vec!["t1", "t2", "t3"];
    let optimal = engine.optimize_join_order(&tables);

    // Smallest (t3 with 5 rows) should be first after ANALYZE
    assert_eq!(optimal[0], "t3");
}

#[test]
fn test_call_statement_returns_error() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // CALL statement should return an error (not fully implemented)
    let result = engine.execute("CALL my_proc(1, 2)");
    assert!(result.is_err());
}

#[test]
fn test_create_procedure_statement_returns_error() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    // CREATE PROCEDURE should return an error (not fully implemented)
    let result = engine.execute("CREATE PROCEDURE test_proc() BEGIN END");
    assert!(result.is_err());
}
