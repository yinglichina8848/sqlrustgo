//! Parser Benchmark Tests
//!
//! Benchmarks for SQL parsing performance.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::parse;

// Simple SQL queries for benchmarking
const SQL_SELECT_SIMPLE: &str = "SELECT id FROM users WHERE id = 1";
const SQL_SELECT_ALL: &str = "SELECT * FROM users";
const SQL_SELECT_MULTI_COL: &str = "SELECT id, name, email, age FROM users";
const SQL_SELECT_WHERE_AND: &str = "SELECT * FROM users WHERE age > 18 AND status = 'active'";
const SQL_SELECT_WHERE_OR: &str = "SELECT * FROM users WHERE age < 18 OR status = 'banned'";
const SQL_SELECT_WHERE_COMPLEX: &str =
    "SELECT * FROM users WHERE (age >= 18 AND age <= 65) AND status IN ('active', 'pending')";
const SQL_SELECT_JOIN: &str =
    "SELECT u.id, u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id";
const SQL_SELECT_ORDER_BY: &str = "SELECT * FROM users ORDER BY created_at DESC";
const SQL_SELECT_LIMIT: &str = "SELECT * FROM users LIMIT 10";
const SQL_SELECT_LIMIT_OFFSET: &str = "SELECT * FROM users LIMIT 10 OFFSET 20";

const SQL_INSERT_SIMPLE: &str = "INSERT INTO users VALUES (1)";
const SQL_INSERT_MULTI_COL: &str = "INSERT INTO users VALUES (1, 'Alice')";
const SQL_INSERT_WITH_COLS: &str =
    "INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')";
const SQL_INSERT_MULTI_ROW: &str =
    "INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie')";

const SQL_UPDATE_SIMPLE: &str = "UPDATE users SET name = 'Bob'";
const SQL_UPDATE_WITH_WHERE: &str = "UPDATE users SET name = 'Bob' WHERE id = 1";
const SQL_UPDATE_MULTI_SET: &str =
    "UPDATE users SET name = 'Bob', email = 'bob@example.com', age = 30";

const SQL_DELETE_SIMPLE: &str = "DELETE FROM users";
const SQL_DELETE_WITH_WHERE: &str = "DELETE FROM users WHERE id = 1";

const SQL_CREATE_SIMPLE: &str = "CREATE TABLE users";
const SQL_CREATE_WITH_COLS: &str = "CREATE TABLE users (id INTEGER, name TEXT)";
const SQL_CREATE_MULTI_COLS: &str = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT, age INTEGER DEFAULT 0)";
const SQL_CREATE_ALL_TYPES: &str = "CREATE TABLE test (id INTEGER, name TEXT, price DECIMAL, active BOOLEAN, created_at TIMESTAMP)";

const SQL_DROP_TABLE: &str = "DROP TABLE users";

const SQL_AGGREGATE_COUNT: &str = "SELECT COUNT(*) FROM users";
const SQL_AGGREGATE_SUM: &str = "SELECT SUM(amount) FROM orders";
const SQL_AGGREGATE_AVG: &str = "SELECT AVG(price) FROM products";
const SQL_AGGREGATE_MIN_MAX: &str = "SELECT MIN(age), MAX(age) FROM users";
const SQL_AGGREGATE_MULTI: &str = "SELECT COUNT(*), SUM(amount), AVG(price) FROM orders";
const SQL_AGGREGATE_WITH_WHERE: &str = "SELECT COUNT(*) FROM users WHERE status = 'active'";

fn bench_parser_select_simple(c: &mut Criterion) {
    c.bench_function("parser_select_simple", |b| {
        b.iter(|| parse(SQL_SELECT_SIMPLE));
    });
}

fn bench_parser_select_all(c: &mut Criterion) {
    c.bench_function("parser_select_all", |b| {
        b.iter(|| parse(SQL_SELECT_ALL));
    });
}

fn bench_parser_select_multi_col(c: &mut Criterion) {
    c.bench_function("parser_select_multi_col", |b| {
        b.iter(|| parse(SQL_SELECT_MULTI_COL));
    });
}

fn bench_parser_select_where_and(c: &mut Criterion) {
    c.bench_function("parser_select_where_and", |b| {
        b.iter(|| parse(SQL_SELECT_WHERE_AND));
    });
}

fn bench_parser_select_where_or(c: &mut Criterion) {
    c.bench_function("parser_select_where_or", |b| {
        b.iter(|| parse(SQL_SELECT_WHERE_OR));
    });
}

fn bench_parser_select_where_complex(c: &mut Criterion) {
    c.bench_function("parser_select_where_complex", |b| {
        b.iter(|| parse(SQL_SELECT_WHERE_COMPLEX));
    });
}

fn bench_parser_select_join(c: &mut Criterion) {
    c.bench_function("parser_select_join", |b| {
        b.iter(|| parse(SQL_SELECT_JOIN));
    });
}

fn bench_parser_select_order_by(c: &mut Criterion) {
    c.bench_function("parser_select_order_by", |b| {
        b.iter(|| parse(SQL_SELECT_ORDER_BY));
    });
}

fn bench_parser_select_limit(c: &mut Criterion) {
    c.bench_function("parser_select_limit", |b| {
        b.iter(|| parse(SQL_SELECT_LIMIT));
    });
}

fn bench_parser_select_limit_offset(c: &mut Criterion) {
    c.bench_function("parser_select_limit_offset", |b| {
        b.iter(|| parse(SQL_SELECT_LIMIT_OFFSET));
    });
}

fn bench_parser_insert_simple(c: &mut Criterion) {
    c.bench_function("parser_insert_simple", |b| {
        b.iter(|| parse(SQL_INSERT_SIMPLE));
    });
}

fn bench_parser_insert_multi_col(c: &mut Criterion) {
    c.bench_function("parser_insert_multi_col", |b| {
        b.iter(|| parse(SQL_INSERT_MULTI_COL));
    });
}

fn bench_parser_insert_with_cols(c: &mut Criterion) {
    c.bench_function("parser_insert_with_cols", |b| {
        b.iter(|| parse(SQL_INSERT_WITH_COLS));
    });
}

fn bench_parser_insert_multi_row(c: &mut Criterion) {
    c.bench_function("parser_insert_multi_row", |b| {
        b.iter(|| parse(SQL_INSERT_MULTI_ROW));
    });
}

fn bench_parser_update_simple(c: &mut Criterion) {
    c.bench_function("parser_update_simple", |b| {
        b.iter(|| parse(SQL_UPDATE_SIMPLE));
    });
}

fn bench_parser_update_with_where(c: &mut Criterion) {
    c.bench_function("parser_update_with_where", |b| {
        b.iter(|| parse(SQL_UPDATE_WITH_WHERE));
    });
}

fn bench_parser_update_multi_set(c: &mut Criterion) {
    c.bench_function("parser_update_multi_set", |b| {
        b.iter(|| parse(SQL_UPDATE_MULTI_SET));
    });
}

fn bench_parser_delete_simple(c: &mut Criterion) {
    c.bench_function("parser_delete_simple", |b| {
        b.iter(|| parse(SQL_DELETE_SIMPLE));
    });
}

fn bench_parser_delete_with_where(c: &mut Criterion) {
    c.bench_function("parser_delete_with_where", |b| {
        b.iter(|| parse(SQL_DELETE_WITH_WHERE));
    });
}

fn bench_parser_create_simple(c: &mut Criterion) {
    c.bench_function("parser_create_simple", |b| {
        b.iter(|| parse(SQL_CREATE_SIMPLE));
    });
}

fn bench_parser_create_with_cols(c: &mut Criterion) {
    c.bench_function("parser_create_with_cols", |b| {
        b.iter(|| parse(SQL_CREATE_WITH_COLS));
    });
}

fn bench_parser_create_multi_cols(c: &mut Criterion) {
    c.bench_function("parser_create_multi_cols", |b| {
        b.iter(|| parse(SQL_CREATE_MULTI_COLS));
    });
}

fn bench_parser_create_all_types(c: &mut Criterion) {
    c.bench_function("parser_create_all_types", |b| {
        b.iter(|| parse(SQL_CREATE_ALL_TYPES));
    });
}

fn bench_parser_drop_table(c: &mut Criterion) {
    c.bench_function("parser_drop_table", |b| {
        b.iter(|| parse(SQL_DROP_TABLE));
    });
}

fn bench_parser_aggregate_count(c: &mut Criterion) {
    c.bench_function("parser_aggregate_count", |b| {
        b.iter(|| parse(SQL_AGGREGATE_COUNT));
    });
}

fn bench_parser_aggregate_sum(c: &mut Criterion) {
    c.bench_function("parser_aggregate_sum", |b| {
        b.iter(|| parse(SQL_AGGREGATE_SUM));
    });
}

fn bench_parser_aggregate_avg(c: &mut Criterion) {
    c.bench_function("parser_aggregate_avg", |b| {
        b.iter(|| parse(SQL_AGGREGATE_AVG));
    });
}

fn bench_parser_aggregate_min_max(c: &mut Criterion) {
    c.bench_function("parser_aggregate_min_max", |b| {
        b.iter(|| parse(SQL_AGGREGATE_MIN_MAX));
    });
}

fn bench_parser_aggregate_multi(c: &mut Criterion) {
    c.bench_function("parser_aggregate_multi", |b| {
        b.iter(|| parse(SQL_AGGREGATE_MULTI));
    });
}

fn bench_parser_aggregate_with_where(c: &mut Criterion) {
    c.bench_function("parser_aggregate_with_where", |b| {
        b.iter(|| parse(SQL_AGGREGATE_WITH_WHERE));
    });
}

// Error cases
fn bench_parser_empty(c: &mut Criterion) {
    c.bench_function("parser_empty", |b| {
        b.iter(|| parse(""));
    });
}

fn bench_parser_whitespace_only(c: &mut Criterion) {
    c.bench_function("parser_whitespace_only", |b| {
        b.iter(|| parse("   \t\n   "));
    });
}

// Batch benchmark - parse many simple statements
fn bench_parser_batch_select(c: &mut Criterion) {
    c.bench_function("parser_batch_100_select", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = parse(SQL_SELECT_SIMPLE);
            }
        });
    });
}

criterion_group!(
    benches,
    // SELECT benchmarks
    bench_parser_select_simple,
    bench_parser_select_all,
    bench_parser_select_multi_col,
    bench_parser_select_where_and,
    bench_parser_select_where_or,
    bench_parser_select_where_complex,
    bench_parser_select_join,
    bench_parser_select_order_by,
    bench_parser_select_limit,
    bench_parser_select_limit_offset,
    // INSERT benchmarks
    bench_parser_insert_simple,
    bench_parser_insert_multi_col,
    bench_parser_insert_with_cols,
    bench_parser_insert_multi_row,
    // UPDATE benchmarks
    bench_parser_update_simple,
    bench_parser_update_with_where,
    bench_parser_update_multi_set,
    // DELETE benchmarks
    bench_parser_delete_simple,
    bench_parser_delete_with_where,
    // CREATE TABLE benchmarks
    bench_parser_create_simple,
    bench_parser_create_with_cols,
    bench_parser_create_multi_cols,
    bench_parser_create_all_types,
    // DROP TABLE benchmark
    bench_parser_drop_table,
    // Aggregate benchmarks
    bench_parser_aggregate_count,
    bench_parser_aggregate_sum,
    bench_parser_aggregate_avg,
    bench_parser_aggregate_min_max,
    bench_parser_aggregate_multi,
    bench_parser_aggregate_with_where,
    // Error case benchmarks
    bench_parser_empty,
    bench_parser_whitespace_only,
    // Batch benchmark
    bench_parser_batch_select
);
criterion_main!(benches);
