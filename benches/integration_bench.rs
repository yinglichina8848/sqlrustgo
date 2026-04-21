use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::ExecutionEngine;

fn bench_end_to_end_select(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();

    engine
        .execute("CREATE TABLE e2e_users (id INTEGER, name TEXT, age INTEGER, email TEXT)")
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(&format!(
                "INSERT INTO e2e_users VALUES ({}, 'user{}', {}, 'user{}@example.com')",
                i,
                i,
                i % 100,
                i
            ))
            .unwrap();
    }

    let mut group = c.benchmark_group("e2e_select");

    group.bench_function("select_all", |b| {
        b.iter(|| engine.execute("SELECT * FROM e2e_users").unwrap());
    });

    group.bench_function("select_where", |b| {
        b.iter(|| {
            engine
                .execute("SELECT * FROM e2e_users WHERE age > 50")
                .unwrap()
        });
    });

    group.bench_function("select_projection", |b| {
        b.iter(|| engine.execute("SELECT name, email FROM e2e_users").unwrap());
    });

    group.bench_function("select_order_by", |b| {
        b.iter(|| {
            engine
                .execute("SELECT * FROM e2e_users ORDER BY age DESC")
                .unwrap()
        });
    });

    group.bench_function("select_limit", |b| {
        b.iter(|| engine.execute("SELECT * FROM e2e_users LIMIT 10").unwrap());
    });

    group.finish();
}

fn bench_end_to_end_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_insert");

    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut engine = ExecutionEngine::with_memory();
                engine
                    .execute("CREATE TABLE e2e_insert (id INTEGER, value TEXT)")
                    .unwrap();
                for i in 0..size {
                    engine
                        .execute(&format!(
                            "INSERT INTO e2e_insert VALUES ({}, 'value{}')",
                            i, i
                        ))
                        .unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_end_to_end_update(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE e2e_update (id INTEGER, value INTEGER)")
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(&format!("INSERT INTO e2e_update VALUES ({}, {})", i, i))
            .unwrap();
    }

    c.bench_function("e2e_update_single", |b| {
        b.iter(|| {
            engine
                .execute("UPDATE e2e_update SET value = 999 WHERE id = 500")
                .unwrap()
        });
    });

    c.bench_function("e2e_update_all", |b| {
        b.iter(|| {
            engine
                .execute("UPDATE e2e_update SET value = value + 1")
                .unwrap()
        });
    });
}

fn bench_end_to_end_transaction(c: &mut Criterion) {
    c.bench_function("e2e_transaction", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::with_memory();
            engine
                .execute("CREATE TABLE e2e_tx (id INTEGER, value TEXT)")
                .unwrap();

            for i in 0..10 {
                engine
                    .execute(&format!("INSERT INTO e2e_tx VALUES ({}, 'tx{}')", i, i))
                    .unwrap();
            }

            engine.execute("SELECT * FROM e2e_tx").unwrap();
            engine
                .execute("UPDATE e2e_tx SET value = 'updated' WHERE id < 5")
                .unwrap();
            engine.execute("SELECT COUNT(*) FROM e2e_tx").unwrap();
        });
    });
}

fn bench_end_to_end_join(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();
    engine
        .execute("CREATE TABLE e2e_orders (id INTEGER, user_id INTEGER, amount INTEGER)")
        .unwrap();
    engine
        .execute("CREATE TABLE e2e_customers (id INTEGER, name TEXT)")
        .unwrap();

    for i in 0..500 {
        engine
            .execute(&format!(
                "INSERT INTO e2e_orders VALUES ({}, {}, {})",
                i,
                i % 100,
                i * 10
            ))
            .unwrap();
    }

    for i in 0..100 {
        engine
            .execute(&format!(
                "INSERT INTO e2e_customers VALUES ({}, 'customer{}')",
                i, i
            ))
            .unwrap();
    }

    c.bench_function("e2e_join", |b| {
        b.iter(|| {
            let _ = engine.execute("SELECT c.name, o.amount FROM e2e_customers c JOIN e2e_orders o ON c.id = o.user_id");
        });
    });
}

fn bench_end_to_end_complex(c: &mut Criterion) {
    let mut engine = ExecutionEngine::with_memory();

    engine.execute("CREATE TABLE events (id INTEGER, user_id INTEGER, event_type TEXT, value INTEGER, created_at INTEGER)").unwrap();

    for i in 0..1000 {
        engine
            .execute(&format!(
                "INSERT INTO events VALUES ({}, {}, 'event_type_{}', {}, {})",
                i,
                i % 100,
                i % 5,
                i * 10,
                i
            ))
            .unwrap();
    }

    c.bench_function("e2e_complex_query", |b| {
        b.iter(|| {
            engine.execute(
                "SELECT user_id, event_type, SUM(value) as total FROM events WHERE created_at > 100 AND event_type IN ('event_type_0', 'event_type_2') GROUP BY user_id, event_type HAVING total > 1000 ORDER BY total DESC LIMIT 50"
            ).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_end_to_end_select,
    bench_end_to_end_insert,
    bench_end_to_end_update,
    bench_end_to_end_transaction,
    bench_end_to_end_join,
    bench_end_to_end_complex
);
criterion_main!(benches);
