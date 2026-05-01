//! QPS Benchmark Tests
//!
//! P0 tests for QPS benchmarking per ISSUE #847
//! Measures queries per second for various workload types
//!
//! These tests are designed to run with --ignored flag:
//!   cargo test --test qps_benchmark_test -- --ignored

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

const BENCHMARK_ITERATIONS: usize = 10000;
const CONCURRENT_THREADS: usize = 8;

fn create_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new(storage)
}

fn setup_tables(engine: &mut MemoryExecutionEngine) {
    let _ = engine.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT, age INTEGER)");
    let _ = engine
        .execute("CREATE TABLE IF NOT EXISTS orders (id INTEGER, user_id INTEGER, amount INTEGER)");
    let _ = engine
        .execute("CREATE TABLE IF NOT EXISTS products (id INTEGER, name TEXT, price INTEGER)");
}

fn cleanup_tables(engine: &mut MemoryExecutionEngine) {
    let _ = engine.execute("DROP TABLE IF EXISTS users");
    let _ = engine.execute("DROP TABLE IF EXISTS orders");
    let _ = engine.execute("DROP TABLE IF EXISTS products");
}

fn insert_test_data(engine: &mut MemoryExecutionEngine) {
    for i in 0..1000 {
        let _ = engine.execute(&format!(
            "INSERT INTO users VALUES ({}, 'user_{}', {})",
            i,
            i,
            20 + (i % 50)
        ));
    }
    for i in 0..5000 {
        let _ = engine.execute(&format!(
            "INSERT INTO orders VALUES ({}, {}, {})",
            i,
            i % 1000,
            10 + (i % 100)
        ));
    }
    for i in 0..100 {
        let _ = engine.execute(&format!(
            "INSERT INTO products VALUES ({}, 'product_{}', {})",
            i,
            i,
            100 + (i % 500)
        ));
    }
}

// ============================================================================
// QPS Benchmark Tests
// ============================================================================

/// Benchmark: Simple SELECT QPS
#[test]
#[ignore]
fn test_qps_simple_select() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!("SELECT * FROM users WHERE id = {}", i % 1000));
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "Simple SELECT QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: INSERT QPS
#[test]
#[ignore]
fn test_qps_insert() {
    let mut engine = create_engine();
    setup_tables(&mut engine);

    let start = Instant::now();

    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!(
            "INSERT INTO users VALUES ({}, 'bench_{}', {})",
            10000 + i,
            i,
            30
        ));
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "INSERT QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: UPDATE QPS
#[test]
#[ignore]
fn test_qps_update() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!(
            "UPDATE users SET age = {} WHERE id = {}",
            40 + (i % 30),
            i % 1000
        ));
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "UPDATE QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: DELETE QPS
#[test]
#[ignore]
fn test_qps_delete() {
    let mut engine = create_engine();
    setup_tables(&mut engine);

    // Insert data first
    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!(
            "INSERT INTO users VALUES ({}, 'bench_{}', {})",
            i, i, 30
        ));
    }

    let start = Instant::now();

    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!("DELETE FROM users WHERE id = {}", i));
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "DELETE QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: JOIN QPS
#[test]
#[ignore]
fn test_qps_join() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for _ in 0..(BENCHMARK_ITERATIONS / 10) {
        let _ = engine.execute(
            "SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.user_id LIMIT 100",
        );
    }

    let elapsed = start.elapsed();
    let query_count = BENCHMARK_ITERATIONS / 10;
    let qps = query_count as f64 / elapsed.as_secs_f64();

    println!(
        "JOIN QPS: {} queries in {:?} ({:.2} qps)",
        query_count, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: Aggregation QPS
#[test]
#[ignore]
fn test_qps_aggregation() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for _ in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute("SELECT COUNT(*), AVG(age) FROM users");
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "Aggregation QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: Concurrent SELECT QPS
#[test]
#[ignore]
fn test_qps_concurrent_select() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    {
        let mut engine = MemoryExecutionEngine::new(storage.clone());
        setup_tables(&mut engine);
        for i in 0..1000 {
            let _ = engine.execute(&format!(
                "INSERT INTO users VALUES ({}, 'user_{}', {})",
                i,
                i,
                20 + (i % 50)
            ));
        }
    }

    let start = Instant::now();

    let handles: Vec<thread::JoinHandle<()>> = (0..CONCURRENT_THREADS)
        .map(|_| {
            let storage = storage.clone();
            thread::spawn(move || {
                let mut engine = MemoryExecutionEngine::new(storage);
                for i in 0..(BENCHMARK_ITERATIONS / CONCURRENT_THREADS) {
                    let _ = engine.execute(&format!("SELECT * FROM users WHERE id = {}", i % 1000));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_queries = BENCHMARK_ITERATIONS;
    let qps = total_queries as f64 / elapsed.as_secs_f64();

    println!(
        "Concurrent SELECT QPS: {} queries in {:?} ({:.2} qps) - {} threads",
        total_queries, elapsed, qps, CONCURRENT_THREADS
    );
}

/// Benchmark: Concurrent mixed workload QPS
#[test]
#[ignore]
fn test_qps_concurrent_mixed() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));

    {
        let mut engine = MemoryExecutionEngine::new(storage.clone());
        setup_tables(&mut engine);
        insert_test_data(&mut engine);
    }

    let start = Instant::now();

    let handles: Vec<thread::JoinHandle<()>> = (0..CONCURRENT_THREADS)
        .map(|thread_id| {
            let storage = storage.clone();
            thread::spawn(move || {
                let mut engine = MemoryExecutionEngine::new(storage);
                for i in 0..(BENCHMARK_ITERATIONS / CONCURRENT_THREADS) {
                    let query_type = (thread_id + i) % 4;
                    match query_type {
                        0 => {
                            let _ = engine
                                .execute(&format!("SELECT * FROM users WHERE id = {}", i % 1000));
                        }
                        1 => {
                            let _ = engine.execute(&format!(
                                "INSERT INTO orders VALUES ({}, {}, {})",
                                thread_id * 10000 + i,
                                i % 1000,
                                100
                            ));
                        }
                        2 => {
                            let _ = engine.execute("SELECT COUNT(*) FROM users");
                        }
                        _ => {
                            let _ = engine.execute(&format!(
                                "UPDATE users SET age = {} WHERE id = {}",
                                30 + (i % 30),
                                i % 1000
                            ));
                        }
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();
    let total_queries = BENCHMARK_ITERATIONS;
    let qps = total_queries as f64 / elapsed.as_secs_f64();

    println!(
        "Concurrent mixed QPS: {} queries in {:?} ({:.2} qps) - {} threads",
        total_queries, elapsed, qps, CONCURRENT_THREADS
    );
}

/// Benchmark: Complex WHERE clause QPS
#[test]
#[ignore]
fn test_qps_complex_where() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for _ in 0..BENCHMARK_ITERATIONS {
        let _ = engine
            .execute("SELECT * FROM users WHERE age > 30 AND age < 60 AND name LIKE '%user_5%'");
    }

    let elapsed = start.elapsed();
    let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

    println!(
        "Complex WHERE QPS: {} queries in {:?} ({:.2} qps)",
        BENCHMARK_ITERATIONS, elapsed, qps
    );

    cleanup_tables(&mut engine);
}

/// Benchmark: Order BY QPS
#[test]
#[ignore]
fn test_qps_order_by() {
    let mut engine = create_engine();
    setup_tables(&mut engine);
    insert_test_data(&mut engine);

    let start = Instant::now();

    for _ in 0..(BENCHMARK_ITERATIONS / 10) {
        let _ = engine.execute("SELECT * FROM users ORDER BY age DESC LIMIT 100");
    }

    let elapsed = start.elapsed();
    let query_count = BENCHMARK_ITERATIONS / 10;
    let qps = query_count as f64 / elapsed.as_secs_f64();

    println!(
        "ORDER BY QPS: {} queries in {:?} ({:.2} qps)",
        query_count, elapsed, qps
    );

    cleanup_tables(&mut engine);
}
