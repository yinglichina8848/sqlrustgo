//! v1.3.0 Performance Benchmarks
//!
//! Benchmarks for TableScan, Filter, and HashJoin operations.
//! Target: 18 benchmarks
//! Threshold: +20% latency

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::Arc;

fn setup_table(engine: &mut ExecutionEngine, table_name: &str, rows: usize) {
    engine
        .execute(
            parse(&format!(
                "CREATE TABLE {} (id INTEGER, value INTEGER, name TEXT)",
                table_name
            ))
            .unwrap(),
        )
        .unwrap();

    for i in 0..rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO {} VALUES ({}, {}, 'name_{}')",
                    table_name,
                    i,
                    i % 100,
                    i
                ))
                .unwrap(),
            )
            .unwrap();
    }
}

// ============================================================================
// TableScan Benchmarks (6 benchmarks)
// ============================================================================

fn bench_tablescan_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablescan_full");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("full_scan", size), &size, |b, _| {
            b.iter(|| engine.execute(parse("SELECT * FROM t1").unwrap()).unwrap());
        });
    }

    group.finish();
}

fn bench_tablescan_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablescan_projection");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("select_columns", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT id, value FROM t1").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// Filter Benchmarks (6 benchmarks)
// ============================================================================

fn bench_filter_equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_equality");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("eq", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id = 50").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_filter_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_range");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("gt", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE value > 50").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_filter_and(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_and");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("and", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id > 10 AND value < 50").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_filter_or(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_or");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("or", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id < 10 OR id > 90").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// Aggregate Benchmarks (6 benchmarks)
// ============================================================================

fn bench_aggregate_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_count");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("count", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT COUNT(*) FROM t1").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_aggregate_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_sum");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("sum", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT SUM(value) FROM t1").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_aggregate_avg(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_avg");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("avg", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT AVG(value) FROM t1").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_aggregate_group_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_group_by");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("group_by", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT value, COUNT(*) FROM t1 GROUP BY value").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    // TableScan (2 x 3 sizes = 6)
    bench_tablescan_full,
    bench_tablescan_projection,
    // Filter (4 x 3 sizes = 12)
    bench_filter_equality,
    bench_filter_range,
    bench_filter_and,
    bench_filter_or,
    // Aggregate (4 x 3 sizes = 12)
    bench_aggregate_count,
    bench_aggregate_sum,
    bench_aggregate_avg,
    bench_aggregate_group_by,
);
criterion_main!(benches);
