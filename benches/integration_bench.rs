use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, ExecutionEngine};

fn bench_end_to_end_select(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();

    engine
        .execute(
            parse("CREATE TABLE e2e_users (id INTEGER, name TEXT, age INTEGER, email TEXT)")
                .unwrap(),
        )
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO e2e_users VALUES ({}, 'user{}', {}, 'user{}@example.com')",
                    i,
                    i,
                    i % 100,
                    i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    let mut group = c.benchmark_group("e2e_select");

    group.bench_function("select_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM e2e_users").unwrap())
                .unwrap()
        });
    });

    group.bench_function("select_where", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM e2e_users WHERE age > 50").unwrap())
                .unwrap()
        });
    });

    group.bench_function("select_projection", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT name, email FROM e2e_users").unwrap())
                .unwrap()
        });
    });

    group.bench_function("select_order_by", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM e2e_users ORDER BY age DESC").unwrap())
                .unwrap()
        });
    });

    group.bench_function("select_limit", |b| {
        b.iter(|| {
            engine
                .execute(parse("SELECT * FROM e2e_users LIMIT 10").unwrap())
                .unwrap()
        });
    });

    group.finish();
}

fn bench_end_to_end_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_insert");

    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut engine = ExecutionEngine::new();
                engine
                    .execute(parse("CREATE TABLE e2e_insert (id INTEGER, value TEXT)").unwrap())
                    .unwrap();
                for i in 0..size {
                    engine
                        .execute(
                            parse(&format!(
                                "INSERT INTO e2e_insert VALUES ({}, 'value{}')",
                                i, i
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

fn bench_end_to_end_update(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(parse("CREATE TABLE e2e_update (id INTEGER, value INTEGER)").unwrap())
        .unwrap();

    for i in 0..1000 {
        engine
            .execute(parse(&format!("INSERT INTO e2e_update VALUES ({}, {})", i, i)).unwrap())
            .unwrap();
    }

    c.bench_function("e2e_update_single", |b| {
        b.iter(|| {
            engine
                .execute(parse("UPDATE e2e_update SET value = 999 WHERE id = 500").unwrap())
                .unwrap()
        });
    });

    c.bench_function("e2e_update_all", |b| {
        b.iter(|| {
            engine
                .execute(parse("UPDATE e2e_update SET value = value + 1").unwrap())
                .unwrap()
        });
    });
}

fn bench_end_to_end_transaction(c: &mut Criterion) {
    c.bench_function("e2e_transaction", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            engine
                .execute(parse("CREATE TABLE e2e_tx (id INTEGER, value TEXT)").unwrap())
                .unwrap();

            for i in 0..10 {
                engine
                    .execute(
                        parse(&format!("INSERT INTO e2e_tx VALUES ({}, 'tx{}')", i, i)).unwrap(),
                    )
                    .unwrap();
            }

            engine
                .execute(parse("SELECT * FROM e2e_tx").unwrap())
                .unwrap();
            engine
                .execute(parse("UPDATE e2e_tx SET value = 'updated' WHERE id < 5").unwrap())
                .unwrap();
            engine
                .execute(parse("SELECT COUNT(*) FROM e2e_tx").unwrap())
                .unwrap();
        });
    });
}

fn bench_end_to_end_join(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();
    engine
        .execute(
            parse("CREATE TABLE e2e_orders (id INTEGER, user_id INTEGER, amount INTEGER)").unwrap(),
        )
        .unwrap();
    engine
        .execute(parse("CREATE TABLE e2e_customers (id INTEGER, name TEXT)").unwrap())
        .unwrap();

    for i in 0..500 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO e2e_orders VALUES ({}, {}, {})",
                    i,
                    i % 100,
                    i * 10
                ))
                .unwrap(),
            )
            .unwrap();
    }

    for i in 0..100 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO e2e_customers VALUES ({}, 'customer{}')",
                    i, i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    c.bench_function("e2e_join", |b| {
        b.iter(|| {
            let _ = engine.execute(parse("SELECT c.name, o.amount FROM e2e_customers c JOIN e2e_orders o ON c.id = o.user_id").unwrap());
        });
    });
}

fn bench_end_to_end_complex(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();

    engine.execute(parse("CREATE TABLE events (id INTEGER, user_id INTEGER, event_type TEXT, value INTEGER, created_at INTEGER)").unwrap()).unwrap();

    for i in 0..1000 {
        engine
            .execute(
                parse(&format!(
                    "INSERT INTO events VALUES ({}, {}, 'event_type_{}', {}, {})",
                    i,
                    i % 100,
                    i % 5,
                    i * 10,
                    i
                ))
                .unwrap(),
            )
            .unwrap();
    }

    c.bench_function("e2e_complex_query", |b| {
        b.iter(|| {
            engine.execute(
                parse("SELECT user_id, event_type, SUM(value) as total FROM events WHERE created_at > 100 AND event_type IN ('event_type_0', 'event_type_2') GROUP BY user_id, event_type HAVING total > 1000 ORDER BY total DESC LIMIT 50")
                    .unwrap()
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
