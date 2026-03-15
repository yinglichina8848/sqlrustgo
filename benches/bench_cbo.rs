//! v1.4.0 CBO Performance Benchmarks
//!
//! Benchmarks for comparing CBO (Cost-Based Optimization) performance
//! Target: Verify CBO optimization effectiveness
//!
//! Run: cargo bench --bench bench_cbo

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::Arc;

fn setup_table(engine: &mut ExecutionEngine, table_name: &str, rows: usize) {
    engine
        .execute(
            parse(&format!(
                "CREATE TABLE {} (id INTEGER, value INTEGER, name TEXT, category TEXT)",
                table_name
            ))
            .unwrap(),
        )
        .unwrap();

    for i in 0..rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO {} VALUES ({}, {}, 'name_{}', 'cat_{}')",
                    table_name,
                    i,
                    i % 100,
                    i,
                    i % 10
                ))
                .unwrap(),
            )
            .unwrap();
    }
}

fn setup_join_tables(engine: &mut ExecutionEngine, t1_rows: usize, t2_rows: usize) {
    engine
        .execute(parse("CREATE TABLE t1 (id INTEGER, value INTEGER, category TEXT)").unwrap())
        .unwrap();
    engine
        .execute(parse("CREATE TABLE t2 (id INTEGER, name TEXT, category TEXT)").unwrap())
        .unwrap();

    for i in 0..t1_rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO t1 VALUES ({}, {}, 'cat_{}')",
                    i,
                    i % 50,
                    i % 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    for i in 0..t2_rows {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO t2 VALUES ({}, 'name_{}', 'cat_{}')",
                    i,
                    i,
                    i % 10
                ))
                .unwrap(),
            )
            .unwrap();
    }
}

// ============================================================================
// CBO: Join Method Selection
// ============================================================================

fn bench_cbo_join_method(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbo_join");

    // Small inner table - hash join should be preferred
    for size in [1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, size, 100);

        group.bench_with_input(
            BenchmarkId::new("small_inner", size),
            &size,
            |b, _| {
                b.iter(|| {
                    engine
                        .execute(
                            parse("SELECT t1.id, t1.value, t2.name FROM t1 JOIN t2 ON t1.category = t2.category")
                                .unwrap(),
                        )
                        .unwrap()
                });
            },
        );
    }

    // Large inner table - nested loop might be preferred
    for size in [100, 1000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_join_tables(&mut engine, size, size * 10);

        group.bench_with_input(
            BenchmarkId::new("large_inner", size),
            &size,
            |b, _| {
                b.iter(|| {
                    engine
                        .execute(
                            parse("SELECT t1.id, t1.value, t2.name FROM t1 JOIN t2 ON t1.category = t2.category")
                                .unwrap(),
                        )
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// CBO: Index Scan vs Full Scan
// ============================================================================

fn bench_cbo_index_vs_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbo_scan");

    // High selectivity (small result) - index scan preferred
    for size in [1000, 10000, 100000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("high_selectivity", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE id = 50").unwrap())
                    .unwrap()
            });
        });
    }

    // Low selectivity (large result) - full scan preferred
    for size in [1000, 10000, 100000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("low_selectivity", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE value > 10").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// CBO: Aggregation Method
// ============================================================================

fn bench_cbo_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbo_aggregate");

    // Small group count - hash aggregate
    for size in [1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("few_groups", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT category, COUNT(*) FROM t1 GROUP BY category").unwrap())
                    .unwrap()
            });
        });
    }

    // Many groups - sort aggregate
    for size in [1000, 10000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        group.bench_with_input(BenchmarkId::new("many_groups", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT id, SUM(value) FROM t1 GROUP BY id").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

// ============================================================================
// CBO: Complex Query Planning
// ============================================================================

fn bench_cbo_complex_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("cbo_complex");

    // Multi-join query
    for size in [100, 1000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);
        setup_table(&mut engine, "t2", size);

        group.bench_with_input(
            BenchmarkId::new("multi_join", size),
            &size,
            |b, _| {
                b.iter(|| {
                    engine
                        .execute(
                            parse("SELECT t1.id, t1.value, t2.name FROM t1 JOIN t2 ON t1.category = t2.category WHERE t1.value > 50")
                                .unwrap(),
                        )
                        .unwrap()
                });
            },
        );
    }

    // Subquery optimization
    for size in [100, 1000] {
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

// ============================================================================
// Vectorization Performance
// ============================================================================

fn bench_vectorization_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("vectorization");

    for size in [1000, 10000, 100000] {
        let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
        setup_table(&mut engine, "t1", size);

        // Batch processing test
        group.bench_with_input(BenchmarkId::new("batch_scan", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT id, value FROM t1").unwrap())
                    .unwrap()
            });
        });

        // SIMD-like operations test
        group.bench_with_input(BenchmarkId::new("simd_filter", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM t1 WHERE value > 50").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    // CBO Join (4 benchmarks)
    bench_cbo_join_method,
    // CBO Scan (6 benchmarks)
    bench_cbo_index_vs_scan,
    // CBO Aggregate (4 benchmarks)
    bench_cbo_aggregation,
    // CBO Complex (4 benchmarks)
    bench_cbo_complex_query,
    // Vectorization (6 benchmarks)
    bench_vectorization_batch,
);
criterion_main!(benches);
