//! v1.4.0 Performance Benchmarks
//!
//! Benchmarks for v1.4.0 performance testing vs v1.3.0
//! Target: 30% performance improvement
//!
//! Run: cargo bench --bench bench_v140

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
        .execute(parse("CREATE TABLE t1 (id INTEGER, value INTEGER)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE t2 (id INTEGER, name TEXT)").unwrap())
        .unwrap();

    for i in 0..left_rows {
        engine
            .execute(parse(&format!("INSERT INTO t1 VALUES ({}, {})", i, i % 50)).unwrap())
            .unwrap();
    }

    for i in 0..right_rows {
        engine
            .execute(parse(&format!("INSERT INTO t2 VALUES ({}, 'name_{}')", i, i)).unwrap())
            .unwrap();
    }
}

// ============================================================================
// TableScan Benchmarks
// ============================================================================

fn bench_tablescan_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("tablescan_full");

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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
// Filter Benchmarks
// ============================================================================

fn bench_filter_equality(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_equality");

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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

// ============================================================================
// HashJoin Benchmarks
// ============================================================================

fn bench_hashjoin_inner(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_inner");

    for size in [100, 1_000, 10_000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, size, size);

        group.bench_with_input(BenchmarkId::new("inner_join", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(
                        parse("SELECT t1.id, t1.value, t2.name FROM t1 JOIN t2 ON t1.id = t2.id")
                            .unwrap(),
                    )
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_hashjoin_left(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_left");

    for size in [100, 1_000, 10_000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, size, size);

        group.bench_with_input(BenchmarkId::new("left_join", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(
                        parse(
                            "SELECT t1.id, t1.value, t2.name FROM t1 LEFT JOIN t2 ON t1.id = t2.id",
                        )
                        .unwrap(),
                    )
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_hashjoin_cross(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashjoin_cross");

    for size in [10, 100, 1000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, size, size);

        group.bench_with_input(BenchmarkId::new("cross_join", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT t1.id, t2.id FROM t1, t2").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// Aggregate Benchmarks
// ============================================================================

fn bench_aggregate_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_count");

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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

    for size in [100, 1_000, 10_000, 100_000] {
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

// ============================================================================
// Complex Query Benchmarks
// ============================================================================

fn bench_complex_query_1(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_query");

    for size in [100, 1_000, 10_000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("filter_aggregate", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(
                        parse("SELECT value, COUNT(*) FROM t1 WHERE id > 10 GROUP BY value")
                            .unwrap(),
                    )
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_complex_query_2(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_query");

    for size in [100, 1_000, 10_000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("subquery", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(
                        parse("SELECT * FROM t1 WHERE id IN (SELECT id FROM t1 WHERE value > 50)")
                            .unwrap(),
                    )
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    // TableScan (2 x 4 sizes = 8)
    bench_tablescan_full,
    bench_tablescan_projection,
    // Filter (3 x 4 sizes = 12)
    bench_filter_equality,
    bench_filter_range,
    bench_filter_and,
    // HashJoin (3 x 3 sizes = 9)
    bench_hashjoin_inner,
    bench_hashjoin_left,
    bench_hashjoin_cross,
    // Aggregate (4 x 4 sizes = 16)
    bench_aggregate_count,
    bench_aggregate_sum,
    bench_aggregate_avg,
    bench_aggregate_group_by,
    // Complex (2 x 3 sizes = 6)
    bench_complex_query_1,
    bench_complex_query_2,
);
criterion_main!(benches);
