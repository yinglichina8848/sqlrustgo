//! Insert Benchmark for SQLRustGo
//!
//! Benchmarks for INSERT operations at various scales.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine};

fn setup_engine() -> ExecutionEngine {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(parse("CREATE TABLE insert_bench (id INTEGER, name TEXT, value INTEGER)").unwrap())
        .unwrap();
    engine
}

fn bench_insert_single_row(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_single");

    for size in [1, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut engine = setup_engine();
                for i in 0..size {
                    engine
                        .execute(
                            parse(&format!(
                                "INSERT INTO insert_bench VALUES ({}, 'name_{}', {})",
                                i,
                                i % 100,
                                i
                            ))
                            .unwrap(),
                        )
                        .unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_insert_1k(c: &mut Criterion) {
    c.bench_function("insert_1k", |b| {
        b.iter(|| {
            let mut engine = setup_engine();
            for i in 0..1_000 {
                engine
                    .execute(
                        parse(&format!(
                            "INSERT INTO insert_bench VALUES ({}, 'name_{}', {})",
                            i,
                            i % 1000,
                            i
                        ))
                        .unwrap(),
                    )
                    .unwrap();
            }
        });
    });
}

fn bench_insert_10k(c: &mut Criterion) {
    c.bench_function("insert_10k", |b| {
        b.iter(|| {
            let mut engine = setup_engine();
            for i in 0..10_000 {
                engine
                    .execute(
                        parse(&format!(
                            "INSERT INTO insert_bench VALUES ({}, 'name_{}', {})",
                            i,
                            i % 10000,
                            i
                        ))
                        .unwrap(),
                    )
                    .unwrap();
            }
        });
    });
}

fn bench_insert_100k(c: &mut Criterion) {
    c.bench_function("insert_100k", |b| {
        b.iter(|| {
            let mut engine = setup_engine();
            for i in 0..100_000 {
                engine
                    .execute(
                        parse(&format!(
                            "INSERT INTO insert_bench VALUES ({}, 'name_{}', {})",
                            i,
                            i % 100000,
                            i
                        ))
                        .unwrap(),
                    )
                    .unwrap();
            }
        });
    });
}

fn bench_insert_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_batch");

    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut engine = setup_engine();
                    for i in 0..batch_size {
                        engine
                            .execute(
                                parse(&format!(
                                    "INSERT INTO insert_bench VALUES ({}, 'name_{}', {})",
                                    i, i, i
                                ))
                                .unwrap(),
                            )
                            .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_insert_text_column(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_text");

    for text_len in [10, 100, 1000] {
        let text = "x".repeat(text_len);
        group.bench_with_input(BenchmarkId::from_parameter(text_len), &text_len, |b, _| {
            b.iter(|| {
                let mut engine = setup_engine();
                engine
                    .execute(
                        parse(&format!(
                            "INSERT INTO insert_bench VALUES (1, '{}', 1)",
                            text
                        ))
                        .unwrap(),
                    )
                    .unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_single_row,
    bench_insert_1k,
    bench_insert_10k,
    bench_insert_100k,
    bench_insert_batch,
    bench_insert_text_column
);
criterion_main!(benches);
