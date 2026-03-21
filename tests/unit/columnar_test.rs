//! Columnar Storage Integration Tests
//!
//! Tests for columnar storage and Parquet file format support.

use sqlrustgo_storage::{
    ColumnDefinition, ColumnarStorage, ColumnarTable, ParquetReader, ParquetWriter, TableInfo,
};
use sqlrustgo_types::Value;
use std::fs;
use tempfile::TempDir;

fn create_test_table_info() -> TableInfo {
    TableInfo {
        name: "test_users".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
            },
            ColumnDefinition {
                name: "email".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: true,
            },
            ColumnDefinition {
                name: "age".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                is_unique: false,
            },
        ],
    }
}

fn create_test_records() -> Vec<Vec<Value>> {
    vec![
        vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Text("alice@test.com".to_string()),
            Value::Integer(30),
        ],
        vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Text("bob@test.com".to_string()),
            Value::Integer(25),
        ],
        vec![
            Value::Integer(3),
            Value::Text("Charlie".to_string()),
            Value::Text("charlie@test.com".to_string()),
            Value::Integer(35),
        ],
        vec![
            Value::Integer(4),
            Value::Text("Diana".to_string()),
            Value::Text("diana@test.com".to_string()),
            Value::Null,
        ],
    ]
}

#[test]
fn test_columnar_table_creation() {
    let info = create_test_table_info();
    let table = ColumnarTable::new(info.clone());

    assert_eq!(table.num_rows(), 0);
    assert_eq!(table.num_columns(), 4);
    assert_eq!(table.info.name, "test_users");
}

#[test]
fn test_columnar_table_append_rows() {
    let info = create_test_table_info();
    let mut table = ColumnarTable::new(info);
    let records = create_test_records();

    for record in &records {
        table.append_row(record);
    }

    assert_eq!(table.num_rows(), 4);
    assert_eq!(table.num_columns(), 4);
}

#[test]
fn test_columnar_table_to_records() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    let result = table.to_records();

    assert_eq!(result.len(), 4);
    assert_eq!(result[0][0], Value::Integer(1));
    assert_eq!(result[0][1], Value::Text("Alice".to_string()));
}

#[test]
fn test_projection_pushdown() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    // Project only id and name (indices 0 and 1)
    let projected = table.project_to_records(&[0, 1]);

    assert_eq!(projected.len(), 4);
    assert_eq!(projected[0].len(), 2);
    assert_eq!(projected[0][0], Value::Integer(1));
    assert_eq!(projected[0][1], Value::Text("Alice".to_string()));
    assert!(matches!(projected[3][1], Value::Text(_))); // Diana has a name
}

#[test]
fn test_columnar_filter() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    // Filter where age > 28
    let filtered = table.filter(&|row| {
        if let Value::Integer(age) = row[3] {
            age > 28
        } else {
            false
        }
    });

    assert_eq!(filtered.len(), 2);
    assert_eq!(filtered[0][0], Value::Integer(1)); // Alice, age 30
    assert_eq!(filtered[1][0], Value::Integer(3)); // Charlie, age 35
}

#[test]
fn test_column_stats() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    // Stats for id column (should be integers)
    let stats = table.column_stats(0).unwrap();
    assert_eq!(stats.num_values, 4);
    assert_eq!(stats.null_count, 0);
    assert_eq!(stats.min_value, Some(Value::Integer(1)));
    assert_eq!(stats.max_value, Some(Value::Integer(4)));

    // Stats for age column (has null)
    let age_stats = table.column_stats(3).unwrap();
    assert_eq!(age_stats.num_values, 4);
    assert_eq!(age_stats.null_count, 1);
}

#[test]
fn test_columnar_storage_new() {
    let storage = ColumnarStorage::new();
    assert!(!storage.has_table("users"));
}

#[test]
fn test_columnar_storage_create_table() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    let result = storage.create_table(info.clone());
    assert!(result.is_ok());
    assert!(storage.has_table("test_users"));
}

#[test]
fn test_columnar_storage_insert() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    storage.create_table(info).unwrap();
    let records = create_test_records();
    storage.insert("test_users", records).unwrap();

    let scanned = storage.scan("test_users").unwrap();
    assert_eq!(scanned.len(), 4);
}

#[test]
fn test_columnar_storage_scan_with_projection() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    storage.create_table(info).unwrap();
    let records = create_test_records();
    storage.insert("test_users", records).unwrap();

    // Scan only id column
    let scanned = storage.scan_with_projection("test_users", &[0]).unwrap();
    assert_eq!(scanned.len(), 4);
    assert_eq!(scanned[0].len(), 1);
    assert_eq!(scanned[0][0], Value::Integer(1));
}

#[test]
fn test_columnar_storage_filter_project() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    storage.create_table(info).unwrap();
    let records = create_test_records();
    storage.insert("test_users", records).unwrap();

    // Filter where id > 2 and project only name and email
    let result = storage
        .filter_project("test_users", &[1, 2], &|row| {
            if let Value::Integer(id) = row[0] {
                id > 2
            } else {
                false
            }
        })
        .unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].len(), 2);
    assert_eq!(result[0][0], Value::Text("Charlie".to_string()));
    assert_eq!(result[0][1], Value::Text("charlie@test.com".to_string()));
}

#[test]
fn test_parquet_write_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.parquet");
    let path_str = path.to_str().unwrap();

    let info = create_test_table_info();
    let records = create_test_records();

    // Write to parquet
    let writer = ParquetWriter::new(path_str.to_string());
    let result = writer.write(&info, &records);
    assert!(result.is_ok());

    // Verify file exists
    assert!(path.exists());

    // Read from parquet
    let reader = ParquetReader::new(path_str.to_string());
    let read_records = reader.read().unwrap();

    assert_eq!(read_records.len(), 4);
    assert_eq!(read_records[0][0], Value::Integer(1));
}

#[test]
fn test_parquet_projection_pushdown() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.parquet");
    let path_str = path.to_str().unwrap();

    let info = create_test_table_info();
    let records = create_test_records();

    // Write to parquet
    let writer = ParquetWriter::new(path_str.to_string());
    writer.write(&info, &records).unwrap();

    // Read with projection (only id and name)
    let reader = ParquetReader::new(path_str.to_string());
    let read_records = reader.read_projected(&[0, 1]).unwrap();

    assert_eq!(read_records.len(), 4);
    assert_eq!(read_records[0].len(), 2);
    assert_eq!(read_records[0][0], Value::Integer(1));
    assert_eq!(read_records[0][1], Value::Text("Alice".to_string()));
}

#[test]
fn test_columnar_export_import_parquet() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("export_test.parquet");
    let path_str = path.to_str().unwrap();

    // Create and populate storage
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();
    storage.create_table(info).unwrap();
    let records = create_test_records();
    storage.insert("test_users", records).unwrap();

    // Export to parquet
    let result = storage.export_to_parquet("test_users", path_str);
    assert!(result.is_ok());
    assert!(path.exists());

    // Create new storage and import
    let mut new_storage = ColumnarStorage::new();
    new_storage
        .import_from_parquet("test_users", path_str)
        .unwrap();

    let scanned = new_storage.scan("test_users").unwrap();
    assert_eq!(scanned.len(), 4);
}

#[test]
fn test_parquet_write_projected() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("projected.parquet");
    let path_str = path.to_str().unwrap();

    let info = create_test_table_info();
    let records = create_test_records();

    // Write only projected columns (id and email)
    let writer = ParquetWriter::new(path_str.to_string());
    let result = writer.write_projected(&info, &records, &[0, 2]);
    assert!(result.is_ok());

    // Read back
    let reader = ParquetReader::new(path_str.to_string());
    let read_records = reader.read().unwrap();

    assert_eq!(read_records.len(), 4);
    assert_eq!(read_records[0].len(), 2);
    assert_eq!(read_records[0][0], Value::Integer(1));
}

#[test]
fn test_columnar_with_nulls() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    // Check that null is preserved
    let records = table.to_records();
    assert_eq!(records[3][3], Value::Null);

    // Check column stats
    let stats = table.column_stats(3).unwrap();
    assert_eq!(stats.null_count, 1);
}

#[test]
fn test_columnar_project_with_nulls() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    // Project columns that include nulls
    let projected = table.project_to_records(&[2, 3]);
    assert_eq!(projected.len(), 4);
    assert_eq!(projected[3][1], Value::Null); // Diana has no age
}

#[test]
fn test_empty_columnar_table() {
    let info = create_test_table_info();
    let table = ColumnarTable::new(info);

    assert_eq!(table.num_rows(), 0);
    assert_eq!(table.num_columns(), 4);

    let records = table.to_records();
    assert!(records.is_empty());
}

#[test]
fn test_storage_with_duplicate_insert() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    storage.create_table(info).unwrap();

    // Insert in two batches
    let batch1 = vec![create_test_records()[0].clone()];
    let batch2 = create_test_records()[1..].to_vec();

    storage.insert("test_users", batch1).unwrap();
    storage.insert("test_users", batch2).unwrap();

    let scanned = storage.scan("test_users").unwrap();
    assert_eq!(scanned.len(), 4);
}

#[test]
fn test_table_not_found() {
    let storage = ColumnarStorage::new();

    let result = storage.scan("nonexistent");
    assert!(result.is_err());

    let result = storage.scan_with_projection("nonexistent", &[0]);
    assert!(result.is_err());
}

#[test]
fn test_duplicate_table_creation() {
    let mut storage = ColumnarStorage::new();
    let info = create_test_table_info();

    storage.create_table(info.clone()).unwrap();
    let result = storage.create_table(info);

    assert!(result.is_err());
}

#[test]
fn test_columnar_column_values() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    let col = table.column(1).unwrap();
    assert_eq!(col.len(), 4);
    assert_eq!(col.get(0), Some(Value::Text("Alice".to_string())));
    assert_eq!(col.get(1), Some(Value::Text("Bob".to_string())));
}

#[test]
fn test_columnar_column_stats() {
    let info = create_test_table_info();
    let records = create_test_records();
    let table = ColumnarTable::from_records(info, &records);

    let stats = table.column_stats(3).unwrap(); // age column
    assert_eq!(stats.num_values, 4);
    assert_eq!(stats.null_count, 1);
    assert!(stats.is_available());
}

#[test]
fn test_columnar_column_stats_none() {
    let info = TableInfo {
        name: "empty".to_string(),
        columns: vec![ColumnDefinition {
            name: "col".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            is_unique: false,
        }],
    };
    let table = ColumnarTable::new(info);

    // No data, so no stats
    let stats = table.column_stats(0);
    assert!(stats.is_none());
}

#[test]
fn test_parquet_file_not_found() {
    let reader = ParquetReader::new("/nonexistent/path.parquet");
    let result = reader.read();
    assert!(result.is_err());
}

#[test]
fn test_columnar_storage_register_parquet() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.parquet");
    let path_str = path.to_str().unwrap();

    // Write a parquet file first
    let info = create_test_table_info();
    let records = create_test_records();
    let writer = ParquetWriter::new(path_str.to_string());
    writer.write(&info, &records).unwrap();

    // Register in storage
    let mut storage = ColumnarStorage::new();
    storage.register_parquet("test_users", path_str.to_string());

    assert!(storage.has_table("test_users"));

    // Read from parquet through storage
    let scanned = storage.scan("test_users").unwrap();
    assert_eq!(scanned.len(), 4);
}
