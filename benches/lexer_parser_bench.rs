use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::{parse, tokenize};

fn bench_lexer_simple(c: &mut Criterion) {
    let sql = "SELECT id, name, age FROM users WHERE id = 1";

    c.bench_function("lexer_simple", |b| {
        b.iter(|| tokenize(sql));
    });
}

fn bench_lexer_complex(c: &mut Criterion) {
    let sql = "SELECT u.id, u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id WHERE u.age > 18 AND o.status = 'active' ORDER BY o.amount DESC LIMIT 100";

    c.bench_function("lexer_complex", |b| {
        b.iter(|| tokenize(sql));
    });
}

fn bench_lexer_long(c: &mut Criterion) {
    let mut sql = String::from("SELECT ");
    for i in 0..50 {
        sql.push_str(&format!("user_{}, ", i));
    }
    sql.push_str("created_at FROM users WHERE ");
    for i in 0..20 {
        sql.push_str(&format!("col{} = {} AND ", i, i));
    }
    sql.pop();
    sql.pop();
    sql.pop();

    c.bench_function("lexer_long", |b| {
        b.iter(|| tokenize(&sql));
    });
}

fn bench_parser_simple(c: &mut Criterion) {
    c.bench_function("parser_simple_select", |b| {
        b.iter(|| parse("SELECT * FROM users").unwrap());
    });
}

fn bench_parser_where(c: &mut Criterion) {
    c.bench_function("parser_where", |b| {
        b.iter(|| parse("SELECT * FROM users WHERE id = 1 AND name = 'test'").unwrap());
    });
}

fn bench_parser_join(c: &mut Criterion) {
    c.bench_function("parser_join", |b| {
        b.iter(|| parse("SELECT a.*, b.* FROM table1 a JOIN table2 b ON a.id = b.id").unwrap());
    });
}

fn bench_parser_aggregate(c: &mut Criterion) {
    c.bench_function("parser_aggregate", |b| {
        b.iter(|| parse("SELECT COUNT(*), SUM(amount), AVG(price), MIN(id), MAX(name) FROM orders GROUP BY category").unwrap());
    });
}

fn bench_parser_subquery(c: &mut Criterion) {
    c.bench_function("parser_subquery", |b| {
        b.iter(|| {
            parse("SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE amount > 100)")
                .unwrap()
        });
    });
}

fn bench_parser_insert(c: &mut Criterion) {
    c.bench_function("parser_insert", |b| {
        b.iter(|| parse("INSERT INTO users (id, name, age) VALUES (1, 'test', 25)").unwrap());
    });
}

fn bench_parser_update(c: &mut Criterion) {
    c.bench_function("parser_update", |b| {
        b.iter(|| parse("UPDATE users SET name = 'new_name', age = 30 WHERE id = 1").unwrap());
    });
}

fn bench_parser_delete(c: &mut Criterion) {
    c.bench_function("parser_delete", |b| {
        b.iter(|| parse("DELETE FROM users WHERE id = 1 AND status = 'inactive'").unwrap());
    });
}

fn bench_parser_create_table(c: &mut Criterion) {
    c.bench_function("parser_create_table", |b| {
        b.iter(|| parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER, email TEXT UNIQUE)").unwrap());
    });
}

criterion_group!(
    benches,
    bench_lexer_simple,
    bench_lexer_complex,
    bench_lexer_long,
    bench_parser_simple,
    bench_parser_where,
    bench_parser_join,
    bench_parser_aggregate,
    bench_parser_subquery,
    bench_parser_insert,
    bench_parser_update,
    bench_parser_delete,
    bench_parser_create_table
);
criterion_main!(benches);
