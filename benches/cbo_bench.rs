// Quick CBO index scan benchmark
use sqlrustgo::{ExecutionEngine, MemoryStorage, Value};
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);

    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine
        .execute("CREATE INDEX idx_users_id ON users (id)")
        .unwrap();

    for i in 0..10000 {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i))
            .unwrap();
    }

    // Warmup
    for _ in 0..100 {
        let _ = engine.execute("SELECT * FROM users WHERE id = 42");
    }

    // Benchmark: Point SELECT with WHERE id = N (should use index)
    let iterations = 50000;
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine.execute(&format!("SELECT * FROM users WHERE id = {}", i % 10000));
    }
    let elapsed = start.elapsed();
    let qps = iterations as f64 / elapsed.as_secs_f64();
    println!(
        "CBO + Index Point SELECT: {} queries in {:?} ({:.2} qps)",
        iterations, elapsed, qps
    );

    // Also test with CBO disabled
    drop(engine);
    let storage2 = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine2 = ExecutionEngine::with_cbo(storage2, false);

    engine2
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine2
        .execute("CREATE INDEX idx_users_id ON users (id)")
        .unwrap();

    for i in 0..10000 {
        engine2
            .execute(&format!("INSERT INTO users VALUES ({}, 'user{}')", i, i))
            .unwrap();
    }

    // Warmup
    for _ in 0..100 {
        let _ = engine2.execute("SELECT * FROM users WHERE id = 42");
    }

    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine2.execute(&format!("SELECT * FROM users WHERE id = {}", i % 10000));
    }
    let elapsed = start.elapsed();
    let qps = iterations as f64 / elapsed.as_secs_f64();
    println!(
        "No-CBO (full scan) Point SELECT: {} queries in {:?} ({:.2} qps)",
        iterations, elapsed, qps
    );
}
