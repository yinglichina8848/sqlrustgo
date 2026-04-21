use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::ExecutionEngine;

fn bench_executor_select_where(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();

    // Use 1000 rows for benchmark
    for i in 0..1000 {
        engine
            .execute(&format!(
                "INSERT INTO users VALUES ({}, 'user{}', {})",
                i,
                i,
                i % 50
            ))
            .unwrap();
    }

    let mut group = c.benchmark_group("executor_select");

    group.bench_function("select_all_1k", |b| {
        b.iter(|| engine.execute("SELECT * FROM users").unwrap());
    });

    group.bench_function("select_where_id_1k", |b| {
        b.iter(|| {
            engine
                .execute("SELECT * FROM users WHERE id = 500")
                .unwrap()
        });
    });

    group.bench_function("select_where_age_1k", |b| {
        b.iter(|| {
            engine
                .execute("SELECT * FROM users WHERE age > 25")
                .unwrap()
        });
    });

    group.finish();
}

fn bench_executor_insert(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE bench_insert (id INTEGER, value TEXT)")
        .unwrap();

    let mut group = c.benchmark_group("executor_insert");

    for size in [1000, 10000, 100000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    engine
                        .execute(&format!(
                            "INSERT INTO bench_insert VALUES ({}, 'value{}')",
                            i, i
                        ))
                        .unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_executor_update(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE bench_update (id INTEGER, value INTEGER)")
        .unwrap();

    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO bench_update VALUES ({}, {})", i, i))
            .unwrap();
    }

    let mut group = c.benchmark_group("executor_update");

    group.bench_function("update_single", |b| {
        b.iter(|| {
            engine
                .execute("UPDATE bench_update SET value = 999 WHERE id = 50")
                .unwrap()
        });
    });

    group.bench_function("update_multiple", |b| {
        b.iter(|| {
            engine
                .execute("UPDATE bench_update SET value = value + 1")
                .unwrap()
        });
    });

    group.finish();
}

fn bench_executor_delete(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE bench_delete (id INTEGER)")
        .unwrap();

    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO bench_delete VALUES ({})", i))
            .unwrap();
    }

    let mut group = c.benchmark_group("executor_delete");

    group.bench_function("delete_single", |b| {
        b.iter(|| {
            engine
                .execute("DELETE FROM bench_delete WHERE id = 50")
                .unwrap()
        });
    });

    group.bench_function("delete_all", |b| {
        b.iter(|| engine.execute("DELETE FROM bench_delete").unwrap());
    });

    group.finish();
}

fn bench_executor_aggregate(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE orders (id INTEGER, amount INTEGER, category TEXT)")
        .unwrap();

    for i in 0..1000 {
        let amount = (i % 100) as i64;
        let category = format!("cat{}", i % 5);
        engine
            .execute(&format!(
                "INSERT INTO orders VALUES ({}, {}, '{}')",
                i, amount, category
            ))
            .unwrap();
    }

    let mut group = c.benchmark_group("executor_aggregate");

    group.bench_function("count_all", |b| {
        b.iter(|| engine.execute("SELECT COUNT(*) FROM orders").unwrap());
    });

    group.bench_function("sum_amount", |b| {
        b.iter(|| engine.execute("SELECT SUM(amount) FROM orders").unwrap());
    });

    group.bench_function("avg_amount", |b| {
        b.iter(|| engine.execute("SELECT AVG(amount) FROM orders").unwrap());
    });

    group.bench_function("group_by_category", |b| {
        b.iter(|| {
            engine
                .execute("SELECT category, COUNT(*) FROM orders GROUP BY category")
                .unwrap()
        });
    });

    group.finish();
}

fn bench_executor_join(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE TABLE orders (user_id INTEGER, amount INTEGER)")
        .unwrap();

    for i in 0..100 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i))
            .unwrap();
        for j in 0..5 {
            engine
                .execute(&format!("INSERT INTO orders VALUES ({}, {})", i, j * 10))
                .unwrap();
        }
    }

    let mut group = c.benchmark_group("executor_join");

    group.bench_function("inner_join", |b| {
        b.iter(|| {
            engine.execute(
                "SELECT users.name, orders.amount FROM users INNER JOIN orders ON users.id = orders.user_id"
            )
            .unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_executor_select_where,
    bench_executor_insert,
    bench_executor_update,
    bench_executor_delete,
    bench_executor_aggregate,
    bench_executor_join
);
criterion_main!(benches);
