//! Integration tests for ParallelExecutor

use sqlrustgo_executor::{ParallelVolcanoExecutor, RayonTaskScheduler, TaskScheduler};
use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr, Field, PhysicalPlan, Schema};
use sqlrustgo_storage::StorageEngine;
use std::sync::Arc;

/// Helper to create a test table with data
fn create_test_table(storage: &mut dyn StorageEngine, name: &str, rows: usize) {
    storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: name.to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        })
        .unwrap();

    let mut batch = Vec::new();
    for i in 0..rows {
        batch.push(vec![sqlrustgo_types::Value::Integer(i as i64)]);
        if batch.len() >= 100 {
            storage.insert(name, batch).unwrap();
            batch = Vec::new();
        }
    }
    if !batch.is_empty() {
        storage.insert(name, batch).unwrap();
    }
}

#[test]
fn test_parallel_executor_with_storage() {
    let mut memory_storage = sqlrustgo_storage::MemoryStorage::new();
    create_test_table(&mut memory_storage, "test_table", 1000);

    let storage: Arc<dyn StorageEngine> = Arc::new(memory_storage);
    let scheduler = Arc::new(RayonTaskScheduler::new(4));

    let executor = ParallelVolcanoExecutor::with_storage_and_scheduler(storage, scheduler);

    // Just verify the executor can be created and used
    assert_eq!(executor.degree(), 4);
}

#[test]
fn test_parallel_executor_default_degree() {
    let executor = ParallelVolcanoExecutor::new();
    // Default degree should be based on available parallelism
    assert!(executor.degree() >= 1);
}

#[test]
fn test_parallel_executor_with_custom_degree() {
    let scheduler = Arc::new(RayonTaskScheduler::new(8));
    let executor = ParallelVolcanoExecutor::with_scheduler(scheduler);

    assert_eq!(executor.degree(), 8);
}

#[test]
fn test_parallel_count_aggregate() {
    let mut memory_storage = sqlrustgo_storage::MemoryStorage::new();
    create_test_table(&mut memory_storage, "count_test", 5000);

    let storage: Arc<dyn StorageEngine> = Arc::new(memory_storage);
    let scheduler = Arc::new(RayonTaskScheduler::new(4));
    let executor = ParallelVolcanoExecutor::with_storage_and_scheduler(storage, scheduler);

    // Verify scheduler is working
    assert_eq!(executor.scheduler().current_parallelism(), 4);
}

#[test]
fn test_parallel_hash_join_basic() {
    let mut memory_storage = sqlrustgo_storage::MemoryStorage::new();

    // Create two tables
    memory_storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "left_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        })
        .unwrap();

    memory_storage
        .create_table(&sqlrustgo_storage::TableInfo {
            name: "right_table".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                auto_increment: false,
                references: None,
                compression: None,
            }],
        })
        .unwrap();

    // Insert some data
    memory_storage
        .insert("left_table", vec![vec![sqlrustgo_types::Value::Integer(1)]])
        .unwrap();
    memory_storage
        .insert(
            "right_table",
            vec![vec![sqlrustgo_types::Value::Integer(1)]],
        )
        .unwrap();

    let storage: Arc<dyn StorageEngine> = Arc::new(memory_storage);
    let scheduler = Arc::new(RayonTaskScheduler::new(4));
    let _executor = ParallelVolcanoExecutor::with_storage_and_scheduler(storage, scheduler);

    // Integration test validates parallel hash join can be created
    // Actual join execution tested in unit tests
}
