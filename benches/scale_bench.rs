// Benchmark tests for large scale data performance
// Target: 100K+ rows processed efficiently

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::Arc;

fn setup_engine_with_rows(count: usize) -> ExecutionEngine {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));

    // Create table
    engine
        .execute(
            parse("CREATE TABLE bench_data (id INTEGER, name TEXT, age INTEGER, value INTEGER)")
                .unwrap(),
        )
        .unwrap();

    // Insert rows using batch insert
    let batch_size = 1000;
    for batch in 0..(count / batch_size) {
        let mut values = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let id = batch * batch_size + i;
            values.push(format!(
                "({}, 'user_{}', {}, {})",
                id,
                id,
                id % 100,
                id * 10
            ));
        }

        let sql = format!("INSERT INTO bench_data VALUES {}", values.join(", "));
        engine.execute(parse(&sql).unwrap()).unwrap();
    }

    engine
}

// 10K rows tests
fn bench_10k_select_all(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(10000);

    c.bench_function("10k_select_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM bench_data").unwrap())
                .unwrap()
        });
    });
}

fn bench_10k_select_where(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(10000);

    c.bench_function("10k_select_where", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM bench_data WHERE age > 50").unwrap())
                .unwrap()
        });
    });
}

// 100K rows tests
fn bench_100k_select_all(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(100000);

    c.bench_function("100k_select_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM bench_data").unwrap())
                .unwrap()
        });
    });
}

fn bench_100k_select_where(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(100000);

    c.bench_function("100k_select_where", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM bench_data WHERE age > 50").unwrap())
                .unwrap()
        });
    });
}

fn bench_100k_limit(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(100000);

    c.bench_function("100k_limit", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM bench_data LIMIT 100").unwrap())
                .unwrap()
        });
    });
}

fn bench_100k_projection(c: &mut Criterion) {
    let mut engine = setup_engine_with_rows(100000);

    c.bench_function("100k_projection", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT id, name, value FROM bench_data").unwrap())
                .unwrap()
        });
    });
}

// Insert performance test
fn bench_insert_10k(c: &mut Criterion) {
    c.bench_function("insert_10k", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
            engine
                .execute(parse("CREATE TABLE insert_test (id INTEGER, value TEXT)").unwrap())
                .unwrap();

            for i in 0..10000 {
                engine
                    .execute(
                        parse(&format!(
                            "INSERT INTO insert_test VALUES ({}, 'value{}')",
                            i, i
                        ))
                        .unwrap(),
                    )
                    .unwrap();
            }
        });
    });
}

criterion_group!(
    benches,
    bench_10k_select_all,
    bench_10k_select_where,
    bench_100k_select_all,
    bench_100k_select_where,
    bench_100k_limit,
    bench_100k_projection,
    bench_insert_10k
);
criterion_main!(benches);
