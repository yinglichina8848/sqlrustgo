//! Scan Benchmark for SQLRustGo
//!
//! Benchmarks for full table scan operations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine};

fn setup_engine_with_data(rows: usize) -> ExecutionEngine {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(parse("CREATE TABLE scan_bench (id INTEGER, name TEXT, value INTEGER)").unwrap())
        .unwrap();

    for i in 0..rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO scan_bench VALUES ({}, 'name_{}', {})",
                    i,
                    i % 1000,
                    i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    engine
}

fn bench_scan_1k(c: &mut Criterion) {
    let mut engine = setup_engine_with_data(1_000);

    c.bench_function("scan_1k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM scan_bench").unwrap())
                .unwrap()
        });
    });
}

fn bench_scan_10k(c: &mut Criterion) {
    let mut engine = setup_engine_with_data(10_000);

    c.bench_function("scan_10k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM scan_bench").unwrap())
                .unwrap()
        });
    });
}

fn bench_scan_100k(c: &mut Criterion) {
    let mut engine = setup_engine_with_data(100_000);

    c.bench_function("scan_100k", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM scan_bench").unwrap())
                .unwrap()
        });
    });
}

fn bench_scan_with_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_with_filter");

    for size in [1_000, 10_000, 100_000] {
        let mut engine = setup_engine_with_data(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM scan_bench WHERE value > 500").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_scan_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_projection");

    for size in [1_000, 10_000, 100_000] {
        let mut engine = setup_engine_with_data(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT id, value FROM scan_bench").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_scan_1k,
    bench_scan_10k,
    bench_scan_100k,
    bench_scan_with_filter,
    bench_scan_projection
);
criterion_main!(benches);
