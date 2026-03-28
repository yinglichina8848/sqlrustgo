//! Columnar Storage Benchmark
//!
//! Benchmarks for column-oriented storage operations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_storage::columnar::{
    ColumnChunk, ColumnSegment, ColumnStats, CompressionType, TableStore,
};
use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
use sqlrustgo_types::Value;
use tempfile::TempDir;

fn create_test_table_info(columns: usize) -> TableInfo {
    let cols: Vec<ColumnDefinition> = (0..columns)
        .map(|i| ColumnDefinition {
            name: format!("col_{}", i),
            data_type: "INTEGER".to_string(),
            nullable: true,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
        })
        .collect();

    TableInfo {
        name: "bench_table".to_string(),
        columns: cols,
    }
}

// ============================================================================
// ColumnChunk Benchmarks
// ============================================================================

fn bench_column_chunk_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("column_chunk_insert");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut chunk = ColumnChunk::with_capacity(size);
                for i in 0..size {
                    chunk.push(Value::Integer(i as i64));
                }
            });
        });
    }

    group.finish();
}

fn bench_column_chunk_insert_with_nulls(c: &mut Criterion) {
    let mut group = c.benchmark_group("column_chunk_insert_with_nulls");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut chunk = ColumnChunk::with_capacity(size);
                for i in 0..size {
                    if i % 10 == 0 {
                        chunk.push_null();
                    } else {
                        chunk.push(Value::Integer(i as i64));
                    }
                }
            });
        });
    }

    group.finish();
}

fn bench_column_chunk_get(c: &mut Criterion) {
    let mut chunk = ColumnChunk::with_capacity(10000);
    for i in 0..10000 {
        chunk.push(Value::Integer(i as i64));
    }

    let mut group = c.benchmark_group("column_chunk_get");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let _ = chunk.get(i);
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// TableStore Benchmarks
// ============================================================================

fn bench_table_store_insert_row(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_store_insert_row");

    for size in [100, 1000] {
        let info = create_test_table_info(10); // 10 columns
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut store = TableStore::new(info.clone());
                for row_idx in 0..size {
                    let record: Vec<Value> = (0..10)
                        .map(|col_idx| Value::Integer((row_idx * 10 + col_idx) as i64))
                        .collect();
                    store.insert_row(&record).unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_table_store_scan(c: &mut Criterion) {
    let info = create_test_table_info(10);
    let mut store = TableStore::new(info);

    // Insert 1000 rows
    for row_idx in 0..1000 {
        let record: Vec<Value> = (0..10)
            .map(|col_idx| Value::Integer((row_idx * 10 + col_idx) as i64))
            .collect();
        store.insert_row(&record).unwrap();
    }

    let mut group = c.benchmark_group("table_store_scan");

    group.bench_function("scan_all_columns", |b| {
        b.iter(|| {
            let mut count = 0;
            for i in 0..store.row_count() {
                if let Some(row) = store.get_row(i) {
                    count += row.len();
                }
            }
        });
    });

    group.finish();
}

fn bench_table_store_scan_columns(c: &mut Criterion) {
    let info = create_test_table_info(10);
    let mut store = TableStore::new(info);

    // Insert 1000 rows
    for row_idx in 0..1000 {
        let record: Vec<Value> = (0..10)
            .map(|col_idx| Value::Integer((row_idx * 10 + col_idx) as i64))
            .collect();
        store.insert_row(&record).unwrap();
    }

    let mut group = c.benchmark_group("table_store_scan_columns");

    // Scan single column
    group.bench_function("scan_single_column", |b| {
        b.iter(|| {
            let result = store.scan_columns(&[5]); // Only column 5
            assert_eq!(result.len(), 1000);
        });
    });

    // Scan 3 columns (projection pushdown)
    group.bench_function("scan_three_columns", |b| {
        b.iter(|| {
            let result = store.scan_columns(&[0, 5, 9]);
            assert_eq!(result.len(), 1000);
            assert_eq!(result[0].len(), 3);
        });
    });

    group.finish();
}

// ============================================================================
// Serialization Benchmarks
// ============================================================================

fn bench_segment_serialize(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("segment.bin");

    // Create segment with 10000 values
    let mut chunk = ColumnChunk::with_capacity(10000);
    for i in 0..10000 {
        if i % 100 == 0 {
            chunk.push_null();
        } else {
            chunk.push(Value::Integer(i as i64));
        }
    }

    let mut group = c.benchmark_group("segment_serialize");

    for compression in [CompressionType::None, CompressionType::Zstd] {
        group.bench_function(format!("{:?}", compression), |b| {
            b.iter(|| {
                let mut segment = ColumnSegment::with_compression(0, compression);
                let stats = ColumnStatsDisk::from(chunk.stats());
                segment.set_stats(stats);
                segment.set_num_values(chunk.len() as u64);
                segment
                    .write_to_file(&segment_path, chunk.values(), chunk.null_bitmap())
                    .unwrap();
            });
        });
    }

    group.finish();
}

fn bench_segment_deserialize(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let segment_path = temp_dir.path().join("segment.bin");

    // Create and serialize segment once
    let mut chunk = ColumnChunk::with_capacity(10000);
    for i in 0..10000 {
        if i % 100 == 0 {
            chunk.push_null();
        } else {
            chunk.push(Value::Integer(i as i64));
        }
    }

    let mut segment = ColumnSegment::with_compression(0, CompressionType::Zstd);
    let stats = ColumnStatsDisk::from(chunk.stats());
    segment.set_stats(stats);
    segment.set_num_values(chunk.len() as u64);
    segment
        .write_to_file(&segment_path, chunk.values(), chunk.null_bitmap())
        .unwrap();

    let mut group = c.benchmark_group("segment_deserialize");

    group.bench_function("zstd", |b| {
        b.iter(|| {
            let mut read_segment = ColumnSegment::new(0);
            let (values, bitmap) = read_segment.read_from_file(&segment_path).unwrap();
            assert_eq!(values.len(), 10000);
            assert!(bitmap.is_some());
        });
    });

    group.finish();
}

// ============================================================================
// Statistics Benchmarks
// ============================================================================

fn bench_column_stats_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("column_stats_update");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut stats = ColumnStats::new();
                for i in 0..size {
                    stats.update(&Value::Integer(i as i64), false);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_column_chunk_insert,
    bench_column_chunk_insert_with_nulls,
    bench_column_chunk_get,
    bench_table_store_insert_row,
    bench_table_store_scan,
    bench_table_store_scan_columns,
    bench_segment_serialize,
    bench_segment_deserialize,
    bench_column_stats_update
);
criterion_main!(benches);
