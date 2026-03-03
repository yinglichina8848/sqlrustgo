//! Lexer Benchmark Tests
//!
//! Benchmarks for SQL tokenization performance.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::lexer::tokenize;

// Simple SQL queries for benchmarking
const SQL_SIMPLE: &str = "SELECT id FROM users WHERE id = 1";
const SQL_MEDIUM: &str = "SELECT id, name, email, age FROM users WHERE status = 'active'";
const SQL_COMPLEX: &str = "SELECT u.id, u.name, o.amount, o.status FROM users u JOIN orders o ON u.id = o.user_id WHERE u.age > 18 AND o.amount > 100";
const SQL_INSERT: &str = "INSERT INTO users (id, name, email, age, status) VALUES (1, 'Alice', 'alice@example.com', 25, 'active')";
const SQL_UPDATE: &str = "UPDATE users SET name = 'Bob', email = 'bob@example.com' WHERE id = 1";
const SQL_DELETE: &str = "DELETE FROM users WHERE id = 1 AND status = 'inactive'";
const SQL_CREATE: &str =
    "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, age INTEGER)";
const SQL_DROP: &str = "DROP TABLE users";
const SQL_AGGREGATE: &str = "SELECT COUNT(*), SUM(amount), AVG(price), MIN(id), MAX(name) FROM orders WHERE status = 'active'";
const SQL_MULTI_LINE: &str = "
    SELECT
        id,
        name,
        email,
        age,
        status,
        created_at,
        updated_at
    FROM
        users
    WHERE
        status = 'active'
        AND age >= 18
        AND age <= 65
    ORDER BY
        created_at DESC
    LIMIT 100
    OFFSET 50
";

fn bench_lexer_simple(c: &mut Criterion) {
    c.bench_function("lexer_simple_select", |b| {
        b.iter(|| tokenize(SQL_SIMPLE));
    });
}

fn bench_lexer_medium(c: &mut Criterion) {
    c.bench_function("lexer_medium_select", |b| {
        b.iter(|| tokenize(SQL_MEDIUM));
    });
}

fn bench_lexer_complex(c: &mut Criterion) {
    c.bench_function("lexer_complex_join", |b| {
        b.iter(|| tokenize(SQL_COMPLEX));
    });
}

fn bench_lexer_insert(c: &mut Criterion) {
    c.bench_function("lexer_insert", |b| {
        b.iter(|| tokenize(SQL_INSERT));
    });
}

fn bench_lexer_update(c: &mut Criterion) {
    c.bench_function("lexer_update", |b| {
        b.iter(|| tokenize(SQL_UPDATE));
    });
}

fn bench_lexer_delete(c: &mut Criterion) {
    c.bench_function("lexer_delete", |b| {
        b.iter(|| tokenize(SQL_DELETE));
    });
}

fn bench_lexer_create_table(c: &mut Criterion) {
    c.bench_function("lexer_create_table", |b| {
        b.iter(|| tokenize(SQL_CREATE));
    });
}

fn bench_lexer_drop_table(c: &mut Criterion) {
    c.bench_function("lexer_drop_table", |b| {
        b.iter(|| tokenize(SQL_DROP));
    });
}

fn bench_lexer_aggregate(c: &mut Criterion) {
    c.bench_function("lexer_aggregate", |b| {
        b.iter(|| tokenize(SQL_AGGREGATE));
    });
}

fn bench_lexer_multi_line(c: &mut Criterion) {
    c.bench_function("lexer_multi_line", |b| {
        b.iter(|| tokenize(SQL_MULTI_LINE));
    });
}

// Edge cases
fn bench_lexer_empty(c: &mut Criterion) {
    c.bench_function("lexer_empty", |b| {
        b.iter(|| tokenize(""));
    });
}

fn bench_lexer_whitespace_only(c: &mut Criterion) {
    c.bench_function("lexer_whitespace_only", |b| {
        b.iter(|| tokenize("   \t\n   "));
    });
}

fn bench_lexer_single_keyword(c: &mut Criterion) {
    c.bench_function("lexer_single_keyword", |b| {
        b.iter(|| tokenize("SELECT"));
    });
}

// Batch benchmark - tokenize many simple statements
fn bench_lexer_batch_simple(c: &mut Criterion) {
    c.bench_function("lexer_batch_100_simple", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = tokenize(SQL_SIMPLE);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_lexer_simple,
    bench_lexer_medium,
    bench_lexer_complex,
    bench_lexer_insert,
    bench_lexer_update,
    bench_lexer_delete,
    bench_lexer_create_table,
    bench_lexer_drop_table,
    bench_lexer_aggregate,
    bench_lexer_multi_line,
    bench_lexer_empty,
    bench_lexer_whitespace_only,
    bench_lexer_single_keyword,
    bench_lexer_batch_simple
);
criterion_main!(benches);
