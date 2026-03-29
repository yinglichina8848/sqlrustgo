//! Columnar Storage Integration Tests
//!
//! These tests verify end-to-end functionality of columnar storage:
//! - TableStore operations
//! - Serialization/deserialization
//! - Column projection pushdown
//! - Statistics tracking

use sqlrustgo_storage::columnar::TableStore;
use sqlrustgo_storage::columnar::{
    ColumnChunk, ColumnSegment, ColumnStats, ColumnStatsDisk, CompressionType,
};
use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
use sqlrustgo_types::Value;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_table_info() -> TableInfo {
    TableInfo {
        name: "test_table".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: true,
                references: None,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "value".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "active".to_string(),
                data_type: "BOOLEAN".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
        ],
    }
}

// ============================================================================
// ColumnChunk Integration Tests
// ============================================================================

#[test]
fn test_column_chunk_with_mixed_types() {
    let mut chunk = ColumnChunk::new();

    chunk.push(Value::Integer(42));
    chunk.push(Value::Text("hello".to_string()));
    chunk.push(Value::Float(3.14));
    chunk.push(Value::Boolean(true));
    chunk.push_null();
    chunk.push(Value::Integer(100));

    assert_eq!(chunk.len(), 6);
    assert!(!chunk.is_null(0));
    assert!(!chunk.is_null(1));
    assert!(chunk.is_null(4));

    assert_eq!(chunk.get(0), Some(&Value::Integer(42)));
    assert_eq!(chunk.get(1), Some(&Value::Text("hello".to_string())));
    assert_eq!(chunk.get(2), Some(&Value::Float(3.14)));
    assert_eq!(chunk.get(3), Some(&Value::Boolean(true)));
    assert_eq!(chunk.get(4), Some(&Value::Null));
    assert_eq!(chunk.get(5), Some(&Value::Integer(100)));

    println!("✓ ColumnChunk with mixed types");
}

#[test]
fn test_column_chunk_iter_skips_nulls() {
    let mut chunk = ColumnChunk::new();

    chunk.push(Value::Integer(1));
    chunk.push_null();
    chunk.push(Value::Integer(2));
    chunk.push_null();
    chunk.push(Value::Integer(3));

    let values: Vec<_> = chunk.iter().collect();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], &Value::Integer(1));
    assert_eq!(values[1], &Value::Integer(2));
    assert_eq!(values[2], &Value::Integer(3));

    println!("✓ ColumnChunk iter skips nulls");
}

#[test]
fn test_column_chunk_set_null_after_insert() {
    let mut chunk = ColumnChunk::new();

    chunk.push(Value::Integer(10));
    chunk.push(Value::Integer(20));
    chunk.push(Value::Integer(30));

    assert!(!chunk.is_null(0));
    assert!(!chunk.is_null(1));
    assert!(!chunk.is_null(2));

    chunk.set_null(1);

    assert!(!chunk.is_null(0));
    assert!(chunk.is_null(1));
    assert!(!chunk.is_null(2));

    println!("✓ ColumnChunk set_null after insert");
}

// ============================================================================
// TableStore Integration Tests
// ============================================================================

#[test]
fn test_table_store_insert_and_get() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    store
        .insert_row(&[
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Float(3.14),
            Value::Boolean(true),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Float(2.71),
            Value::Boolean(false),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Integer(3),
            Value::Null,
            Value::Float(1.41),
            Value::Boolean(true),
        ])
        .unwrap();

    assert_eq!(store.row_count(), 3);

    let row0 = store.get_row(0).unwrap();
    assert_eq!(row0[0], Value::Integer(1));
    assert_eq!(row0[1], Value::Text("Alice".to_string()));
    assert_eq!(row0[2], Value::Float(3.14));
    assert_eq!(row0[3], Value::Boolean(true));

    let row2 = store.get_row(2).unwrap();
    assert_eq!(row2[0], Value::Integer(3));
    assert_eq!(row2[1], Value::Null);

    println!("✓ TableStore insert and get");
}

#[test]
fn test_table_store_scan_columns_projection() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    // Insert 100 rows
    for i in 0..100 {
        store
            .insert_row(&[
                Value::Integer(i as i64),
                Value::Text(format!("name_{}", i)),
                Value::Float(i as f64 * 1.5),
                Value::Boolean(i % 2 == 0),
            ])
            .unwrap();
    }

    assert_eq!(store.row_count(), 100);

    // Scan only id column (index 0)
    let single_column = store.scan_columns(&[0]);
    assert_eq!(single_column.len(), 100);
    assert_eq!(single_column[0].len(), 1);
    assert_eq!(single_column[0][0], Value::Integer(0));
    assert_eq!(single_column[99][0], Value::Integer(99));

    // Scan id and active columns (indices 0 and 3)
    let two_columns = store.scan_columns(&[0, 3]);
    assert_eq!(two_columns.len(), 100);
    assert_eq!(two_columns[0].len(), 2);
    assert_eq!(two_columns[0][0], Value::Integer(0));
    assert_eq!(two_columns[0][1], Value::Boolean(true));

    // Scan all columns
    let all_columns = store.scan_columns(&[0, 1, 2, 3]);
    assert_eq!(all_columns.len(), 100);
    assert_eq!(all_columns[0].len(), 4);

    println!("✓ TableStore scan columns projection");
}

#[test]
fn test_table_store_column_stats() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    // Insert integer values: 10, 20, 30, null, 40, 50
    store
        .insert_row(&[
            Value::Integer(10),
            Value::Text("a".to_string()),
            Value::Float(1.0),
            Value::Boolean(true),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Integer(20),
            Value::Text("b".to_string()),
            Value::Float(2.0),
            Value::Boolean(false),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Integer(30),
            Value::Text("c".to_string()),
            Value::Float(3.0),
            Value::Boolean(true),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Null,
            Value::Text("d".to_string()),
            Value::Float(4.0),
            Value::Boolean(false),
        ])
        .unwrap();

    store
        .insert_row(&[
            Value::Integer(40),
            Value::Text("e".to_string()),
            Value::Float(5.0),
            Value::Boolean(true),
        ])
        .unwrap();

    // Check stats for id column (index 0)
    let id_stats = store.get_column_stats(0).unwrap();
    assert_eq!(id_stats.null_count, 1);
    assert_eq!(id_stats.min_value, Some(Value::Integer(10)));
    assert_eq!(id_stats.max_value, Some(Value::Integer(40)));

    println!(
        "✓ TableStore column stats: null_count={}, min={:?}, max={:?}",
        id_stats.null_count, id_stats.min_value, id_stats.max_value
    );
}

// ============================================================================
// Serialization Integration Tests
// ============================================================================

#[test]
fn test_column_segment_roundtrip_no_compression() {
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("segment_no_compress.bin");

    // Create chunk with mixed values
    let mut chunk = ColumnChunk::new();
    for i in 0..100 {
        if i % 10 == 0 {
            chunk.push_null();
        } else {
            chunk.push(Value::Integer(i as i64));
        }
    }

    // Write segment
    let mut segment = ColumnSegment::with_compression(0, CompressionType::None);
    let stats = ColumnStatsDisk::from(chunk.stats());
    segment.set_stats(stats);
    segment.set_num_values(chunk.len() as u64);
    segment
        .write_to_file(&segment_path, chunk.values(), chunk.null_bitmap())
        .unwrap();

    // Read segment
    let mut read_segment = ColumnSegment::new(0);
    let (values, bitmap) = read_segment.read_from_file(&segment_path).unwrap();

    assert_eq!(values.len(), 100);
    assert!(bitmap.is_some());

    // Verify data integrity
    for i in 0..100 {
        if i % 10 == 0 {
            assert!(bitmap.as_ref().unwrap().is_null(i));
            assert_eq!(values[i], Value::Null);
        } else {
            assert!(!bitmap.as_ref().unwrap().is_null(i));
            assert_eq!(values[i], Value::Integer(i as i64));
        }
    }

    println!("✓ ColumnSegment roundtrip (no compression)");
}

#[test]
fn test_column_segment_roundtrip_zstd_compression() {
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("segment_zstd.bin");

    // Create chunk with text values (compressible)
    let mut chunk = ColumnChunk::new();
    for i in 0..1000 {
        if i % 7 == 0 {
            chunk.push_null();
        } else {
            chunk.push(Value::Text(format!("string_{}", i % 100)));
        }
    }

    // Write segment
    let mut segment = ColumnSegment::with_compression(0, CompressionType::Zstd);
    let stats = ColumnStatsDisk::from(chunk.stats());
    segment.set_stats(stats);
    segment.set_num_values(chunk.len() as u64);
    segment
        .write_to_file(&segment_path, chunk.values(), chunk.null_bitmap())
        .unwrap();

    // Read segment
    let mut read_segment = ColumnSegment::new(0);
    let (values, bitmap) = read_segment.read_from_file(&segment_path).unwrap();

    assert_eq!(values.len(), 1000);
    assert!(bitmap.is_some());

    // Verify data integrity
    for i in 0..1000 {
        if i % 7 == 0 {
            assert!(bitmap.as_ref().unwrap().is_null(i));
        } else {
            assert!(!bitmap.as_ref().unwrap().is_null(i));
            assert_eq!(values[i], Value::Text(format!("string_{}", i % 100)));
        }
    }

    println!("✓ ColumnSegment roundtrip (Zstd compression)");
}

#[test]
fn test_table_store_serialize_deserialize() {
    let temp_dir = TempDir::new().unwrap();
    let table_path = temp_dir.path().join("table_store");

    // Create table store with data
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    for i in 0..50 {
        store
            .insert_row(&[
                Value::Integer(i as i64),
                Value::Text(format!("user_{}", i)),
                Value::Float(i as f64 * 0.5),
                Value::Boolean(i % 2 == 0),
            ])
            .unwrap();
    }

    assert_eq!(store.row_count(), 50);

    // Serialize
    store.serialize(&table_path).unwrap();

    // Deserialize
    let restored = TableStore::deserialize(&table_path).unwrap();

    assert_eq!(restored.row_count(), 50);

    // Verify first and last rows
    let first_row = restored.get_row(0).unwrap();
    assert_eq!(first_row[0], Value::Integer(0));
    assert_eq!(first_row[1], Value::Text("user_0".to_string()));

    let last_row = restored.get_row(49).unwrap();
    assert_eq!(last_row[0], Value::Integer(49));
    assert_eq!(last_row[1], Value::Text("user_49".to_string()));

    println!("✓ TableStore serialize/deserialize");
}

// ============================================================================
// Large Scale Integration Tests
// ============================================================================

#[test]
fn test_table_store_large_scale_insert() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    let row_count = 10_000;

    for i in 0..row_count {
        store
            .insert_row(&[
                Value::Integer(i as i64),
                Value::Text(format!("record_{}", i)),
                Value::Float(i as f64 * 0.01),
                Value::Boolean(i % 100 == 0), // Every 100th is active=true
            ])
            .unwrap();
    }

    assert_eq!(store.row_count(), row_count);

    // Verify some specific rows
    assert_eq!(store.get_row(0).unwrap()[0], Value::Integer(0));
    assert_eq!(store.get_row(99).unwrap()[0], Value::Integer(99));
    assert_eq!(store.get_row(9999).unwrap()[0], Value::Integer(9999));

    // Check stats
    let id_stats = store.get_column_stats(0).unwrap();
    assert_eq!(id_stats.min_value, Some(Value::Integer(0)));
    assert_eq!(id_stats.max_value, Some(Value::Integer(9999)));

    println!("✓ TableStore large scale insert: {} rows", row_count);
}

#[test]
fn test_table_store_null_handling_large_scale() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    // Insert with every 10th value being null for name column
    for i in 0..1000 {
        let name = if i % 10 == 0 {
            Value::Null
        } else {
            Value::Text(format!("name_{}", i))
        };

        store
            .insert_row(&[
                Value::Integer(i as i64),
                name,
                Value::Float(i as f64),
                Value::Boolean(true),
            ])
            .unwrap();
    }

    assert_eq!(store.row_count(), 1000);

    // Scan and count nulls
    let names = store.scan_columns(&[1]);
    let null_count = names.iter().filter(|v| v[0] == Value::Null).count();
    assert_eq!(null_count, 100); // Every 10th = 1000/10 = 100

    println!(
        "✓ TableStore null handling large scale: {} nulls out of 1000",
        null_count
    );
}

#[test]
fn test_column_projection_efficiency() {
    let info = create_test_table_info();
    let mut store = TableStore::new(info);

    // Insert 1000 rows
    for i in 0..1000 {
        store
            .insert_row(&[
                Value::Integer(i as i64),
                Value::Text(format!("data_{}", i)),
                Value::Float(i as f64),
                Value::Boolean(i % 2 == 0),
            ])
            .unwrap();
    }

    // Full scan
    let start = std::time::Instant::now();
    let _full = store.scan_columns(&[0, 1, 2, 3]);
    let full_time = start.elapsed();

    // Single column scan (projection pushdown)
    let start = std::time::Instant::now();
    let _single = store.scan_columns(&[0]);
    let single_time = start.elapsed();

    // Two column scan
    let start = std::time::Instant::now();
    let _two = store.scan_columns(&[0, 3]);
    let two_time = start.elapsed();

    println!("Projection benchmark (1000 rows):");
    println!("  Full scan (4 cols): {:?}", full_time);
    println!("  Single column:      {:?}", single_time);
    println!("  Two columns:       {:?}", two_time);

    // Single column should be faster than full scan
    assert!(
        single_time <= full_time,
        "Single column should be <= full scan time"
    );

    println!("✓ Column projection efficiency verified");
}
