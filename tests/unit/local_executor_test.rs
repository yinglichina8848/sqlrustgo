// Local Executor Tests
use sqlrustgo_executor::query_cache_config::QueryCacheConfig;
use sqlrustgo_executor::{Executor, ExecutorResult, LocalExecutor};
use sqlrustgo_storage::engine::StorageEngine;
use sqlrustgo_storage::file_storage::FileStorage;
use sqlrustgo_types::Value;
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn test_local_executor_new() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let executor = LocalExecutor::new(&storage);
    assert_eq!(executor.name(), "local");
    assert!(executor.is_ready());
}

#[test]
fn test_local_executor_with_cache_config() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let config = QueryCacheConfig::default();
    let executor = LocalExecutor::with_cache_config(&storage, config);
    assert_eq!(executor.name(), "local");
}

#[test]
fn test_local_executor_clear_cache() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let executor = LocalExecutor::new(&storage);
    executor.clear_cache();
}

#[test]
fn test_local_executor_invalidate_table() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let executor = LocalExecutor::new(&storage);
    executor.invalidate_table("test_table");
}

#[test]
fn test_executor_result_new() {
    let rows = vec![vec![Value::Integer(1), Value::Text("hello".to_string())]];
    let result = ExecutorResult::new(rows.clone(), 1);
    assert_eq!(result.rows, rows);
    assert_eq!(result.affected_rows, 1);
}

#[test]
fn test_executor_result_empty() {
    let result = ExecutorResult::empty();
    assert!(result.rows.is_empty());
    assert_eq!(result.affected_rows, 0);
}

#[test]
fn test_executor_result_debug() {
    let result = ExecutorResult::empty();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("ExecutorResult"));
}
