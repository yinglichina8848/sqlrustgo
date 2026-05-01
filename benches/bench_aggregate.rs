use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::ExecutionEngine;
use sqlrustgo::MemoryStorage;

fn setup_engine(size: usize) -> ExecutionEngine<MemoryStorage> {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER)")
        .unwrap();

    for i in 0..size {
        engine
            .execute(&format!("INSERT INTO orders VALUES ({}, {})", i, i * 10))
            .unwrap();
    }
    engine
}

fn bench_aggregate_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_count");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("count_star", size), &size, |b, _| {
            b.iter(|| engine.execute("SELECT COUNT(*) FROM orders").unwrap());
        });
    }

    group.finish();
}

fn bench_aggregate_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_sum");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("sum_amount", size), &size, |b, _| {
            b.iter(|| engine.execute("SELECT SUM(amount) FROM orders").unwrap());
        });
    }

    group.finish();
}

fn bench_aggregate_avg(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_avg");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("avg_amount", size), &size, |b, _| {
            b.iter(|| engine.execute("SELECT AVG(amount) FROM orders").unwrap());
        });
    }

    group.finish();
}

fn bench_aggregate_min_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_min_max");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("min_amount", size), &size, |b, _| {
            b.iter(|| engine.execute("SELECT MIN(amount) FROM orders").unwrap());
        });

        group.bench_with_input(BenchmarkId::new("max_amount", size), &size, |b, _| {
            b.iter(|| engine.execute("SELECT MAX(amount) FROM orders").unwrap());
        });
    }

    group.finish();
}

fn bench_aggregate_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_multiple");

    for size in [100, 1000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("count_sum_avg", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute("SELECT COUNT(*), SUM(amount), AVG(amount) FROM orders")
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_aggregate_count,
    bench_aggregate_sum,
    bench_aggregate_avg,
    bench_aggregate_min_max,
    bench_aggregate_multiple
);
criterion_main!(benches);
