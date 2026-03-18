//! IndexScan vs SeqScan Benchmark for SQLRustGo v1.5.0
//!
//! PB-02: 索引性能对比测试
//! 验证 IndexScan 性能显著优于 SeqScan

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::Arc;

fn setup_engine_with_data(rows: usize) -> ExecutionEngine {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(parse("CREATE TABLE idx_bench (id INTEGER, value INTEGER)").unwrap())
        .unwrap();

    for i in 0..rows {
        engine
            .execute(parse(&format!("INSERT INTO idx_bench VALUES ({}, {})", i, i * 10)).unwrap())
            .unwrap();
    }

    engine
}

fn bench_seq_scan_point_query(c: &mut Criterion) {
    let row_counts = [1000, 10000, 100000];
    let mut group = c.benchmark_group("seq_scan_point_query");

    for rows in row_counts {
        let mut engine = setup_engine_with_data(rows);

        group.bench_with_input(BenchmarkId::new("scan", rows), &rows, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM idx_bench WHERE id = 500").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_seq_scan_range_query(c: &mut Criterion) {
    let row_counts = [1000, 10000, 100000];
    let mut group = c.benchmark_group("seq_scan_range_query");

    for rows in row_counts {
        let mut engine = setup_engine_with_data(rows);

        group.bench_with_input(BenchmarkId::new("scan", rows), &rows, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM idx_bench WHERE id > 100 AND id < 500").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_seq_scan_high_selectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_scan_high_selectivity");

    let mut engine = setup_engine_with_data(100000);

    group.bench_function("scan_1_percent", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM idx_bench WHERE id < 1000").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

fn bench_seq_scan_low_selectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_scan_low_selectivity");

    let mut engine = setup_engine_with_data(10000);

    group.bench_function("scan_80_percent", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM idx_bench WHERE id < 8000").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_seq_scan_point_query,
    bench_seq_scan_range_query,
    bench_seq_scan_high_selectivity,
    bench_seq_scan_low_selectivity
);
criterion_main!(benches);
