//! Integration Benchmark Tests
//!
//! End-to-end benchmarks for complete query execution pipeline.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::{executor::ExecutionEngine, lexer::tokenize, parser::parse};

// Complete SQL statements for integration testing
const SQL_CREATE_TABLE: &str = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, age INTEGER)";
const SQL_INSERT_1: &str = "INSERT INTO users (id, name, email, age) VALUES (1, 'Alice', 'alice@example.com', 25)";
const SQL_SELECT_ALL: &str = "SELECT * FROM users";
const SQL_SELECT_WHERE: &str = "SELECT * FROM users WHERE id = 1";
const SQL_UPDATE: &str = "UPDATE users SET name = 'Bob' WHERE id = 1";
const SQL_DELETE: &str = "DELETE FROM users WHERE id = 1";

// ==================== Full Pipeline Benchmarks ====================

/// Complete pipeline: Lexer + Parser + Executor
fn bench_full_pipeline_create(c: &mut Criterion) {
    c.bench_function("integration_create_table", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            let tokens = tokenize(SQL_CREATE_TABLE);
            let stmt = parse(SQL_CREATE_TABLE).unwrap();
            engine.execute(stmt)
        });
    });
}

fn bench_full_pipeline_insert(c: &mut Criterion) {
    c.bench_function("integration_insert_single", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
            let stmt = parse(SQL_INSERT_1).unwrap();
            engine.execute(stmt)
        });
    });
}

fn bench_full_pipeline_select(c: &mut Criterion) {
    // Setup: Create table with 100 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=100 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    let mut engine = setup();
    c.bench_function("integration_select_all", |b| {
        b.iter(|| {
            let stmt = parse(SQL_SELECT_ALL).unwrap();
            engine.execute(stmt)
        });
    });
}

fn bench_full_pipeline_select_where(c: &mut Criterion) {
    // Setup: Create table with 100 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=100 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    let mut engine = setup();
    c.bench_function("integration_select_where", |b| {
        b.iter(|| {
            let stmt = parse(SQL_SELECT_WHERE).unwrap();
            engine.execute(stmt)
        });
    });
}

// ==================== Lexer + Parser Pipeline ====================

fn bench_lexer_parser_simple(c: &mut Criterion) {
    c.bench_function("integration_lexer_parser_simple", |b| {
        b.iter(|| {
            let tokens = tokenize(SQL_SELECT_ALL);
            let _ = parse(SQL_SELECT_ALL).unwrap();
        });
    });
}

fn bench_lexer_parser_complex(c: &mut Criterion) {
    // Simplified complex query that the parser can handle
    let complex_sql = "SELECT id, name, age FROM users WHERE age > 18 ORDER BY id DESC LIMIT 10";
    c.bench_function("integration_lexer_parser_complex", |b| {
        b.iter(|| {
            let tokens = tokenize(complex_sql);
            let _ = parse(complex_sql).unwrap();
        });
    });
}

// ==================== Batch Query Benchmarks ====================

fn bench_batch_10_queries(c: &mut Criterion) {
    // Setup: Create table with 100 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=100 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    let mut engine = setup();
    c.bench_function("integration_batch_10_selects", |b| {
        b.iter(|| {
            for i in 1..=10 {
                let sql = format!("SELECT * FROM users WHERE id = {}", i);
                let stmt = parse(&sql).unwrap();
                let _ = engine.execute(stmt);
            }
        });
    });
}

fn bench_batch_100_queries(c: &mut Criterion) {
    // Setup: Create table with 100 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=100 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    let mut engine = setup();
    c.bench_function("integration_batch_100_selects", |b| {
        b.iter(|| {
            for i in 1..=100 {
                let sql = format!("SELECT * FROM users WHERE id = {}", i);
                let stmt = parse(&sql).unwrap();
                let _ = engine.execute(stmt);
            }
        });
    });
}

// ==================== Mixed Operations ====================

fn bench_mixed_operations(c: &mut Criterion) {
    // Setup: Create table with 10 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=10 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    c.bench_function("integration_mixed_operations", |b| {
        b.iter(|| {
            let mut engine = setup();
            // Insert
            let stmt = parse("INSERT INTO users (id, name, email, age) VALUES (100, 'NewUser', 'new@example.com', 30)").unwrap();
            let _ = engine.execute(stmt);
            // Select
            let stmt = parse("SELECT * FROM users WHERE id = 100").unwrap();
            let _ = engine.execute(stmt);
            // Update
            let stmt = parse("UPDATE users SET name = 'UpdatedUser' WHERE id = 100").unwrap();
            let _ = engine.execute(stmt);
            // Select again
            let stmt = parse("SELECT * FROM users WHERE id = 100").unwrap();
            let _ = engine.execute(stmt);
            // Delete
            let stmt = parse("DELETE FROM users WHERE id = 100").unwrap();
            let _ = engine.execute(stmt);
        });
    });
}

// ==================== Stress Tests ====================

fn bench_stress_1000_rows_insert(c: &mut Criterion) {
    c.bench_function("integration_insert_1000_rows", |b| {
        b.iter(|| {
            let mut engine = ExecutionEngine::new();
            let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
            for i in 1..=1000 {
                let sql = format!(
                    "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                    i, i, i, (i % 50) + 18
                );
                let stmt = parse(&sql).unwrap();
                let _ = engine.execute(stmt);
            }
        });
    });
}

fn bench_stress_1000_rows_select(c: &mut Criterion) {
    // Setup: Create table with 1000 rows
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=1000 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    let mut engine = setup();
    c.bench_function("integration_select_1000_rows", |b| {
        b.iter(|| {
            let stmt = parse(SQL_SELECT_ALL).unwrap();
            engine.execute(stmt)
        });
    });
}

// ==================== Concurrency Simulation ====================

fn bench_concurrent_like_10_queries(c: &mut Criterion) {
    // Simulate concurrent queries by running them sequentially but measuring total time
    fn setup() -> ExecutionEngine {
        let mut engine = ExecutionEngine::new();
        let _ = engine.execute(parse(SQL_CREATE_TABLE).unwrap());
        for i in 1..=100 {
            let sql = format!(
                "INSERT INTO users (id, name, email, age) VALUES ({}, 'User{}', 'user{}@example.com', {})",
                i, i, i, (i % 50) + 18
            );
            let _ = engine.execute(parse(&sql).unwrap());
        }
        engine
    }

    c.bench_function("integration_concurrent_10_queries", |b| {
        b.iter(|| {
            let mut engine = setup();
            let queries = vec![
                "SELECT * FROM users WHERE age > 25",
                "SELECT * FROM users WHERE age < 30",
                "SELECT * FROM users WHERE id > 50",
                "SELECT * FROM users ORDER BY id DESC LIMIT 5",
                "SELECT DISTINCT age FROM users",
                "SELECT name FROM users WHERE id < 20",
                "SELECT email FROM users WHERE name = 'User50'",
                "SELECT * FROM users WHERE age >= 20 AND age <= 40",
                "SELECT id, name FROM users WHERE id BETWEEN 10 AND 30",
                "SELECT * FROM users WHERE id != 50",
            ];
            for q in queries {
                let stmt = parse(q).unwrap();
                let _ = engine.execute(stmt);
            }
        });
    });
}

criterion_group!(
    benches,
    // Full pipeline
    bench_full_pipeline_create,
    bench_full_pipeline_insert,
    bench_full_pipeline_select,
    bench_full_pipeline_select_where,
    // Lexer + Parser
    bench_lexer_parser_simple,
    bench_lexer_parser_complex,
    // Batch
    bench_batch_10_queries,
    bench_batch_100_queries,
    // Mixed operations
    bench_mixed_operations,
    // Stress
    bench_stress_1000_rows_insert,
    bench_stress_1000_rows_select,
    // Concurrency simulation
    bench_concurrent_like_10_queries
);
criterion_main!(benches);
