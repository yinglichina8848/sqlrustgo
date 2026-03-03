//! Executor Benchmark Tests
//!
//! Benchmarks for SQL execution performance.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::{executor::ExecutionEngine, parser::parse};

// Test SQL statements
const SQL_CREATE: &str = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, age INTEGER)";
const SQL_INSERT: &str = "INSERT INTO users (id, name, email, age) VALUES (1, 'Alice', 'alice@example.com', 25)";
const SQL_SELECT_ALL: &str = "SELECT * FROM users";
const SQL_SELECT_WHERE: &str = "SELECT * FROM users WHERE id = 1";
const SQL_UPDATE: &str = "UPDATE users SET name = 'Bob' WHERE id = 1";
const SQL_DELETE: &str = "DELETE FROM users WHERE id = 1";

/// Setup a test table with sample data
fn setup_engine() -> ExecutionEngine {
    let mut engine = ExecutionEngine::new();

    // Create table
    engine.execute(parse(SQL_CREATE).unwrap()).unwrap();

    // Insert sample data (100 rows)
    for i in 1..=100 {
        let sql = format!(
            "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
            i, i, i, (i % 50) + 18
        );
        let _ = engine.execute(parse(&sql).unwrap());
    }

    engine
}

fn bench_executor_create_table(c: &mut Criterion) {
    c.bench_function("executor_create_table", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            engine.execute(parse(SQL_CREATE).unwrap())
        });
    });
}

fn bench_executor_insert(c: &mut Criterion) {
    c.bench_function("executor_insert_single", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            let _ = engine.execute(parse(SQL_CREATE).unwrap());
            engine.execute(parse(SQL_INSERT).unwrap())
        });
    });
}

fn bench_executor_select_all(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_all_100", |b| {
        b.iter(|| {
            engine.execute(parse(SQL_SELECT_ALL).unwrap())
        });
    });
}

fn bench_executor_select_where(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_where", |b| {
        b.iter(|| {
            engine.execute(parse(SQL_SELECT_WHERE).unwrap())
        });
    });
}

fn bench_executor_update(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_update", |b| {
        b.iter(|| {
            engine.execute(parse(SQL_UPDATE).unwrap())
        });
    });
}

fn bench_executor_delete(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_delete", |b| {
        b.iter(|| {
            engine.execute(parse(SQL_DELETE).unwrap())
        });
    });
}

// Batch operations
fn bench_executor_batch_insert(c: &mut Criterion) {
    c.bench_function("executor_batch_insert_100", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            let _ = engine.execute(parse(SQL_CREATE).unwrap());
            for i in 1..=100 {
                let sql = format!(
                    "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', 25)",
                    i, i, i
                );
                let _ = engine.execute(parse(&sql).unwrap());
            }
        });
    });
}

// Query with different result sizes
fn bench_executor_select_limit_10(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_limit_10", |b| {
        b.iter(|| {
            engine.execute(parse("SELECT * FROM users LIMIT 10").unwrap())
        });
    });
}

fn bench_executor_select_limit_50(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_limit_50", |b| {
        b.iter(|| {
            engine.execute(parse("SELECT * FROM users LIMIT 50").unwrap())
        });
    });
}

// Complex WHERE clauses
fn bench_executor_select_complex_where(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_complex_where", |b| {
        b.iter(|| {
            engine.execute(parse("SELECT * FROM users WHERE age > 30 AND age < 60").unwrap())
        });
    });
}

// Projection (selecting specific columns)
fn bench_executor_select_projection(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_projection", |b| {
        b.iter(|| {
            engine.execute(parse("SELECT name, email FROM users").unwrap())
        });
    });
}

// Empty table
fn bench_executor_select_empty(c: &mut Criterion) {
    let mut engine = ExecutionEngine::new();
    let _ = engine.execute(parse(SQL_CREATE).unwrap());
    c.bench_function("executor_select_empty", |b| {
        b.iter(|| {
            engine.execute(parse(SQL_SELECT_ALL).unwrap())
        });
    });
}

// Not found scenario
fn bench_executor_select_not_found(c: &mut Criterion) {
    let mut engine = setup_engine();
    c.bench_function("executor_select_not_found", |b| {
        b.iter(|| {
            engine.execute(parse("SELECT * FROM users WHERE id = 9999").unwrap())
        });
    });
}

criterion_group!(
    benches,
    bench_executor_create_table,
    bench_executor_insert,
    bench_executor_select_all,
    bench_executor_select_where,
    bench_executor_update,
    bench_executor_delete,
    bench_executor_batch_insert,
    bench_executor_select_limit_10,
    bench_executor_select_limit_50,
    bench_executor_select_complex_where,
    bench_executor_select_projection,
    bench_executor_select_empty,
    bench_executor_select_not_found
);
criterion_main!(benches);
