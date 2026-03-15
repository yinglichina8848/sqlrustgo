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

fn setup_join_tables(engine: &mut ExecutionEngine, left_rows: usize, right_rows: usize) {
    engine
        .execute(parse("CREATE TABLE left_table (id INTEGER, value INTEGER)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE right_table (id INTEGER, amount INTEGER)").unwrap())
        .unwrap();

    for i in 0..left_rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO left_table VALUES ({}, {})",
                    i,
                    i * 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    for i in 0..right_rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO right_table VALUES ({}, {})",
                    i,
                    i * 100
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
    let mut group = c.benchmark_group("tablescan");

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
    let mut group = c.benchmark_group("filter");

    for size in [100, 1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("equality", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id = 500").unwrap())
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

        group.bench_with_input(BenchmarkId::new("range", size), &size, |b, _| {
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

        group.bench_with_input(BenchmarkId::new("and_condition", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id > 10 AND value < 50").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// HashJoin Benchmarks (6 benchmarks)
// ============================================================================

fn bench_hashjoin_inner(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_inner");

    for (left, right) in [(100, 100), (1000, 1000), (10000, 10000)] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, left, right);

        group.bench_with_input(BenchmarkId::new("inner_join", left), &(left, right), |b, _| {
            b.iter(|| {
                engine.execute(parse(
                    "SELECT l.id, l.value, r.amount FROM left_table l INNER JOIN right_table r ON l.id = r.id"
                ).unwrap()).unwrap()
            });
        });
    }

    group.finish();
}

fn bench_hashjoin_left(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_left");

    for (left, right) in [(100, 100), (1000, 1000), (10000, 10000)] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, left, right);

        group.bench_with_input(BenchmarkId::new("left_join", left), &(left, right), |b, _| {
            b.iter(|| {
                engine.execute(parse(
                    "SELECT l.id, l.value, r.amount FROM left_table l LEFT JOIN right_table r ON l.id = r.id"
                ).unwrap()).unwrap()
            });
        });
    }

    group.finish();
}

fn bench_hashjoin_cross(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_cross");

    for (left, right) in [(10, 10), (100, 100), (1000, 1000)] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, left, right);

        group.bench_with_input(
            BenchmarkId::new("cross_join", left),
            &(left, right),
            |b, _| {
                b.iter(|| {
                    engine
                        .execute(
                            parse("SELECT l.id, r.id FROM left_table l CROSS JOIN right_table r")
                                .unwrap(),
                        )
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    // TableScan (6)
    bench_tablescan_full,
    bench_tablescan_projection,
    // Filter (6)
    bench_filter_equality,
    bench_filter_range,
    bench_filter_and,
    // HashJoin (6)
    bench_hashjoin_inner,
    bench_hashjoin_left,
    bench_hashjoin_cross,
);
criterion_main!(benches);
