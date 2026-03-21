// Local Executor Tests
use sqlrustgo_executor::LocalExecutor;
use sqlrustgo_executor::query_cache_config::QueryCacheConfig;
use sqlrustgo_storage::engine::StorageEngine;
use sqlrustgo_storage::file_storage::FileStorage;
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn test_local_executor_new() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let executor = LocalExecutor::new(&storage);
    // Just verify it can be created
    assert!(true);
}

#[test]
fn test_local_executor_with_cache_config() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
    let config = QueryCacheConfig::default();
    let executor = LocalExecutor::with_cache_config(&storage, config);
    assert!(true);
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
