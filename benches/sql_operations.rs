use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use std::sync::Arc;

fn bench_parse_select(c: &mut Criterion) {
    c.bench_function("parse_simple_select", |b| {
        b.iter(|| parse("SELECT * FROM users WHERE id = 1").unwrap());
    });
}

fn bench_execute_select(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
        .unwrap();
    for i in 0..10 {
        engine
            .execute(parse(&format!("INSERT INTO users VALUES ({i})")).unwrap())
            .unwrap();
    }

    c.bench_function("execute_select_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM users").unwrap())
                .unwrap()
        });
    });
}

fn bench_execute_insert(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(parse("CREATE TABLE bench (id INTEGER)").unwrap())
        .unwrap();

    c.bench_function("execute_insert_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            engine
                .execute(parse(&format!("INSERT INTO bench VALUES ({counter})")).unwrap())
                .unwrap()
        });
    });
}

fn bench_execute_insert_batch(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(parse("CREATE TABLE bench_batch (id INTEGER)").unwrap())
        .unwrap();

    c.bench_function("execute_insert_batch_100", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let values: Vec<String> = (0..100).map(|i| format!("{}", counter * 100 + i)).collect();
            let sql = format!("INSERT INTO bench_batch VALUES {}", values.join(", "));
            engine.execute(parse(&sql).unwrap()).unwrap()
        });
    });
}

fn bench_execute_aggregate(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new(Arc::new(MemoryStorage::new()));
    engine
        .execute(parse("CREATE TABLE orders (amount INTEGER)").unwrap())
        .unwrap();
    for i in 1..=10 {
        engine
            .execute(parse(&format!("INSERT INTO orders VALUES ({})", i * 10)).unwrap())
            .unwrap();
    }

    c.bench_function("execute_count", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT COUNT(*) FROM orders").unwrap())
                .unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_parse_select,
    bench_execute_select,
    bench_execute_insert,
    bench_execute_insert_batch,
    bench_execute_aggregate
);
criterion_main!(benches);
