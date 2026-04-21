//! FileStorage insert buffering performance benchmarks
//!
//! Performance targets (Issue #1667):
//! - Single row INSERT: < 1ms (direct write)
//! - Batch INSERT (1000 rows): < 50ms (buffered)
//! - Buffer flush: < 10ms per 100 records

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_storage::{
    engine::{ColumnDefinition, StorageEngine, TableInfo},
    file_storage::FileStorage,
};
use std::fs::remove_dir_all;
use std::hint::black_box;
use std::path::PathBuf;

/// Generate test records for benchmarking
fn generate_records(count: usize) -> Vec<Vec<sqlrustgo_types::Value>> {
    (0..count)
        .map(|i| vec![sqlrustgo_types::Value::Integer(i as i64)])
        .collect()
}

/// Create a test table in temporary storage
fn setup_test_storage(
    temp_dir: &PathBuf,
    buffer_threshold: usize,
    enable_buffer: bool,
) -> FileStorage {
    let mut storage =
        FileStorage::new_with_buffer_config(temp_dir.clone(), buffer_threshold, enable_buffer)
            .unwrap();

    let table_info = TableInfo {
        name: "test_table".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: true,
        }],
        foreign_keys: vec![],
        unique_constraints: vec![],
    };
    storage.create_table(&table_info).unwrap();

    storage
}

/// Benchmark: Single row INSERT with buffering disabled (direct write)
fn bench_single_insert_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_single_no_buffer");

    for i in 0..100 {
        let temp_dir = std::env::temp_dir().join(format!("sqlrustgo_bench_direct_{}", i));
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 100, false);

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                let records = generate_records(1);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Single row INSERT with buffering enabled
fn bench_single_insert_buffered(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_single_buffered");

    for i in 0..100 {
        let temp_dir = std::env::temp_dir().join(format!("sqlrustgo_bench_buffered_{}", i));
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 100, true);

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                let records = generate_records(1);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Small batch INSERT (10 rows) - triggers buffered path
fn bench_batch_insert_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch_small_10");

    for size in [10].iter() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_bench_small_batch");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 100, true);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Medium batch INSERT (100 rows) - at threshold
fn bench_batch_insert_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch_medium_100");

    for size in [100].iter() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_bench_medium_batch");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 100, true);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Large batch INSERT (1000 rows) - exceeds threshold, triggers flush
fn bench_batch_insert_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch_large_1000");

    for size in [1000].iter() {
        let temp_dir = std::env::temp_dir().join("sqlrustgo_bench_large_batch");
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 100, true);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Compare buffered vs direct for various batch sizes
fn bench_buffer_vs_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_vs_direct");

    for size in [10, 50, 100, 500, 1000].iter() {
        // Direct write (buffering disabled)
        let temp_dir_direct = std::env::temp_dir().join(format!("sqlrustgo_bench_direct_{}", size));
        let _ = remove_dir_all(&temp_dir_direct);

        group.bench_with_input(BenchmarkId::new("direct", size), size, |b, &size| {
            let mut storage = setup_test_storage(&temp_dir_direct, 100, false);
            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir_direct);

        // Buffered write (buffering enabled)
        let temp_dir_buffered =
            std::env::temp_dir().join(format!("sqlrustgo_bench_buffered_{}", size));
        let _ = remove_dir_all(&temp_dir_buffered);

        group.bench_with_input(BenchmarkId::new("buffered", size), size, |b, &size| {
            let mut storage = setup_test_storage(&temp_dir_buffered, 100, true);
            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });
        });

        let _ = remove_dir_all(&temp_dir_buffered);
    }

    group.finish();
}

/// Benchmark: Buffer flush performance
fn bench_buffer_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_flush");

    for size in [100, 500, 1000].iter() {
        let temp_dir = std::env::temp_dir().join(format!("sqlrustgo_bench_flush_{}", size));
        let _ = remove_dir_all(&temp_dir);

        let mut storage = setup_test_storage(&temp_dir, 10000, true); // High threshold to prevent auto-flush

        // Pre-fill buffer
        let pre_records = generate_records(*size);
        let _ = storage.insert("test_table", pre_records);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                // Reset and pre-fill for each iteration
                let mut s = setup_test_storage(&temp_dir, 10000, true);
                let pre_records = generate_records(size);
                let _ = s.insert("test_table", pre_records);

                // Flush all buffers
                let _ = s.flush_all_buffers();
                black_box(size);
            });
        });

        let _ = remove_dir_all(&temp_dir);
    }

    group.finish();
}

/// Benchmark: Insert throughput (records per second)
fn bench_insert_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_throughput");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let temp_dir =
                std::env::temp_dir().join(format!("sqlrustgo_bench_throughput_{}", size));
            let _ = remove_dir_all(&temp_dir);

            let mut storage = setup_test_storage(&temp_dir, 100, true);

            b.iter(|| {
                let records = generate_records(size);
                let _ = storage.insert("test_table", records);
            });

            let _ = remove_dir_all(&temp_dir);
        });
    }

    group.finish();
}

/// Benchmark: Multiple table inserts with shared buffer
fn bench_multi_table_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_multi_table");

    let tables = ["table1", "table2", "table3", "table4", "table5"];
    let records_per_table = 50;

    let temp_dir = std::env::temp_dir().join("sqlrustgo_bench_multi_table");
    let _ = remove_dir_all(&temp_dir);

    group.bench_function("5_tables_50_records_each", |b| {
        b.iter(|| {
            let mut storage = setup_test_storage(&temp_dir, 100, true);

            for table_name in tables.iter() {
                let table_info = TableInfo {
                    name: table_name.to_string(),
                    columns: vec![ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        primary_key: true,
                    }],
                    foreign_keys: vec![],
                    unique_constraints: vec![],
                };
                let _ = storage.create_table(&table_info);
            }

            for table_name in tables.iter() {
                let records = generate_records(records_per_table);
                let _ = storage.insert(table_name, records);
            }

            let _ = storage.flush_all_buffers();
        });
    });

    let _ = remove_dir_all(&temp_dir);
    group.finish();
}

criterion_group!(
    benches,
    bench_single_insert_direct,
    bench_single_insert_buffered,
    bench_batch_insert_small,
    bench_batch_insert_medium,
    bench_batch_insert_large,
    bench_buffer_vs_direct,
    bench_buffer_flush,
    bench_insert_throughput,
    bench_multi_table_insert
);
criterion_main!(benches);
