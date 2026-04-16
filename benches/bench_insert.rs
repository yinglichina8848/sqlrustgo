//! Insert Benchmark - benches/bench_insert.rs
//!
//! INSERT performance benchmark for SQLRustGo using StorageEngine

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;

/// Generate test data rows
fn generate_rows(count: usize) -> Vec<Vec<Value>> {
    (0..count)
        .map(|i| vec![Value::Integer(i as i64)])
        .collect()
}

/// Create test table info
fn create_table_info() -> TableInfo {
    TableInfo {
        name: "bench".to_string(),
        columns: vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        }],
    }
}

/// Benchmark INSERT with different data sizes using MemoryStorage
fn bench_insert_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for size in [1_000, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut storage = MemoryStorage::new();
                storage.create_table(&create_table_info()).unwrap();
                let rows = generate_rows(size);
                storage.insert("bench", rows).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark batch INSERT performance
fn bench_insert_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch");

    for batch_size in [10, 100, 1000] {
        let total_rows = 10_000;
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut storage = MemoryStorage::new();
                    storage.create_table(&create_table_info()).unwrap();

                    let mut batch = Vec::with_capacity(batch_size);
                    for i in 0..total_rows {
                        batch.push(vec![Value::Integer(i as i64)]);
                        if batch.len() >= batch_size {
                            storage.insert("bench", std::mem::take(&mut batch)).unwrap();
                            batch = Vec::with_capacity(batch_size);
                        }
                    }
                    if !batch.is_empty() {
                        storage.insert("bench", batch).unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark INSERT with multiple columns
fn bench_insert_multi_column(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_multi_column");

    let table_info = TableInfo {
        name: "bench".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
            },
            ColumnDefinition {
                name: "value".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
            },
        ],
    };

    for size in [1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut storage = MemoryStorage::new();
                storage.create_table(&table_info).unwrap();

                let rows: Vec<Vec<Value>> = (0..size)
                    .map(|i| {
                        vec![
                            Value::Integer(i as i64),
                            Value::Text(format!("user_{}", i)),
                            Value::Integer((i * 10) as i64),
                        ]
                    })
                    .collect();
                storage.insert("bench", rows).unwrap();
            });
        });
    }

    group.finish();
}

/// Measure throughput (rows/s) for 100k insert
fn bench_insert_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_throughput");

    group.bench_function("100k_rows", |b| {
        b.iter(|| {
            let mut storage = MemoryStorage::new();
            storage.create_table(&create_table_info()).unwrap();
            let rows = generate_rows(100_000);
            storage.insert("bench", rows).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_sizes,
    bench_insert_batch,
    bench_insert_multi_column,
    bench_insert_throughput
);
criterion_main!(benches);
