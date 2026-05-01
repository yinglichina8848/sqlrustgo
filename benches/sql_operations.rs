use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::ExecutionEngine;

fn bench_parse_select(c: &mut Criterion) {
    c.bench_function("parse_simple_select", |b| {
        b.iter(|| "SELECT * FROM users WHERE id = 1");
    });
}

fn bench_execute_select(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine.execute("CREATE TABLE users (id INTEGER)").unwrap();
    for i in 0..10 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({i})"))
            .unwrap();
    }

    c.bench_function("execute_select_all", |b| {
        b.iter(|| engine.execute("SELECT * FROM users").unwrap());
    });
}

fn bench_execute_insert(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine.execute("CREATE TABLE bench (id INTEGER)").unwrap();

    c.bench_function("execute_insert_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            engine
                .execute(&format!("INSERT INTO bench VALUES ({counter})"))
                .unwrap()
        });
    });
}

fn bench_execute_aggregate(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE orders (amount INTEGER)")
        .unwrap();
    for i in 1..=10 {
        engine
            .execute(&format!("INSERT INTO orders VALUES ({})", i * 10))
            .unwrap();
    }

    c.bench_function("execute_count", |b| {
        b.iter(|| engine.execute("SELECT COUNT(*) FROM orders").unwrap());
    });
}

criterion_group!(
    benches,
    bench_parse_select,
    bench_execute_select,
    bench_execute_insert,
    bench_execute_aggregate
);
criterion_main!(benches);
