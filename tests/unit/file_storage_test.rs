// File Storage Tests
use sqlrustgo_storage::bplus_tree::index::BPlusTree;
use sqlrustgo_storage::file_storage::{FileStorage, TableData};
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn test_file_storage_new() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf());
    assert!(storage.is_ok());
}

#[test]
fn test_file_storage_get_table() {
    let dir = create_temp_dir();
    let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let result = storage.get_table("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_file_storage_contains_table() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    assert!(!storage.contains_table("nonexistent"));
}

#[test]
fn test_file_storage_table_names() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let names = storage.table_names();
    assert!(names.is_empty());
}

#[test]
fn test_file_storage_has_index() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    assert!(!storage.has_index("table", "column"));
}

#[test]
fn test_file_storage_get_index() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let index = storage.get_index("table", "column");
    assert!(index.is_none());
}

#[test]
fn test_file_storage_drop_index() {
    let dir = create_temp_dir();
    let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let result = storage.drop_index("nonexistent", "column");
    assert!(result.is_ok());
}

#[test]
fn test_file_storage_flush_indexes() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let result = storage.flush_indexes();
    assert!(result.is_ok());
}

#[test]
fn test_file_storage_search_index() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let result = storage.search_index("table", "column", 1);
    assert!(result.is_none());
}

#[test]
fn test_file_storage_range_index() {
    let dir = create_temp_dir();
    let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();

    let result = storage.range_index("table", "column", 1..=10);
    assert!(result.is_empty());
}
