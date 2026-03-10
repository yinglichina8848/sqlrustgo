use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine};

fn setup_engine(size: usize) -> ExecutionEngine {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(parse("CREATE TABLE orders (id INTEGER, amount INTEGER)").unwrap())
        .unwrap();

    for i in 0..size {
        engine
            .execute(parse(&format!("INSERT INTO orders VALUES ({}, {})", i, i * 10)).unwrap())
            .unwrap();
    }
    engine
}

fn bench_aggregate_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_count");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("count_all", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM orders").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_aggregate_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("aggregate_sum");

    for size in [100, 1000, 10000] {
        let mut engine = setup_engine(size);

        group.bench_with_input(BenchmarkId::new("sum_all", size), &size, |b, _| {
            b.iter(|| {
                engine
                    .execute(parse("SELECT * FROM orders WHERE id > 0").unwrap())
                    .unwrap()
            });
        });
    }

    group.finish();
}

fn bench_aggregate_group_by(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(
            parse("CREATE TABLE transactions (id INTEGER, category TEXT, amount INTEGER)").unwrap(),
        )
        .unwrap();

    for i in 0..1000 {
        let category = format!("cat{}", i % 10);
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO transactions VALUES ({}, '{}', {})",
                    i, category, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("aggregate_group_by");

    group.bench_function("filter_category", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM transactions WHERE category = 'cat0'").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_aggregate_count,
    bench_aggregate_sum,
    bench_aggregate_group_by
);
criterion_main!(benches);
