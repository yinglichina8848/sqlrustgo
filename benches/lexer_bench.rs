//! Lexer Benchmark Suite
//!
//! Performance benchmarks for SQL tokenization.
//!
//! ## Run benchmarks
//! ```bash
//! cargo bench --bench lexer_bench
//! ```
//!
//! ## Performance Targets
//! - Simple SELECT: < 1μs
//! - Complex query: < 5μs

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::tokenize;

/// Benchmark: Simple SELECT statement tokenization
fn bench_lexer_simple_select(c: &mut Criterion) {
    let sql = "SELECT id FROM users WHERE id = 1";

    c.bench_function("lexer_simple_select", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: SELECT with multiple columns
fn bench_lexer_select_columns(c: &mut Criterion) {
    let sql = "SELECT id, name, email, age, created_at FROM users";

    c.bench_function("lexer_select_multiple_columns", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: INSERT statement
fn bench_lexer_insert(c: &mut Criterion) {
    let sql = "INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')";

    c.bench_function("lexer_insert", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: UPDATE statement
fn bench_lexer_update(c: &mut Criterion) {
    let sql = "UPDATE users SET name = 'Jane', email = 'jane@example.com' WHERE id = 1";

    c.bench_function("lexer_update", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: DELETE statement
fn bench_lexer_delete(c: &mut Criterion) {
    let sql = "DELETE FROM users WHERE id = 1";

    c.bench_function("lexer_delete", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: CREATE TABLE statement
fn bench_lexer_create_table(c: &mut Criterion) {
    let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE)";

    c.bench_function("lexer_create_table", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Complex WHERE clause with multiple AND conditions
fn bench_lexer_complex_where(c: &mut Criterion) {
    let sql = "SELECT * FROM users WHERE id > 10 AND name LIKE '%John%' AND age >= 18 AND active = true";

    c.bench_function("lexer_complex_where", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: JOIN query
fn bench_lexer_join(c: &mut Criterion) {
    let sql = "SELECT u.id, u.name, o.amount FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE o.amount > 100";

    c.bench_function("lexer_join", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Subquery
fn bench_lexer_subquery(c: &mut Criterion) {
    let sql = "SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE amount > 100)";

    c.bench_function("lexer_subquery", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Aggregate with GROUP BY and HAVING
fn bench_lexer_aggregate(c: &mut Criterion) {
    let sql = "SELECT department, COUNT(*) as count, AVG(salary) as avg_salary FROM employees GROUP BY department HAVING COUNT(*) > 10";

    c.bench_function("lexer_aggregate", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Very long SQL statement
fn bench_lexer_long_query(c: &mut Criterion) {
    let sql = "SELECT u.id, u.name, u.email, u.age, u.created_at, u.updated_at, o.id as order_id, o.amount, o.status, p.id as product_id, p.name as product_name, p.price FROM users u INNER JOIN orders o ON u.id = o.user_id INNER JOIN order_items oi ON o.id = oi.order_id INNER JOIN products p ON oi.product_id = p.id WHERE u.age > 18 AND o.amount > 50 AND p.price < 1000 ORDER BY o.amount DESC LIMIT 100";

    c.bench_function("lexer_long_query", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Empty input (edge case)
fn bench_lexer_empty(c: &mut Criterion) {
    let sql = "";

    c.bench_function("lexer_empty", |b| {
        b.iter(|| tokenize(sql));
    });
}

/// Benchmark: Single keyword (edge case)
fn bench_lexer_single_keyword(c: &mut Criterion) {
    let sql = "SELECT";

    c.bench_function("lexer_single_keyword", |b| {
        b.iter(|| tokenize(sql));
    });
}

criterion_group!(
    lexer_benches,
    bench_lexer_simple_select,
    bench_lexer_select_columns,
    bench_lexer_insert,
    bench_lexer_update,
    bench_lexer_delete,
    bench_lexer_create_table,
    bench_lexer_complex_where,
    bench_lexer_join,
    bench_lexer_subquery,
    bench_lexer_aggregate,
    bench_lexer_long_query,
    bench_lexer_empty,
    bench_lexer_single_keyword,
);
criterion_main!(lexer_benches);
