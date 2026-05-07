// CBO Index Scan vs Full Scan comparison
use sqlrustgo::{ExecutionEngine, MemoryStorage, Value};
use std::sync::{Arc, RwLock};
use std::time::Instant;

fn main() {
    let iterations = 50000;

    // === Test 1: Without index (full scan) ===
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();
    for i in 0..100000 {
        engine
            .execute(&format!(
                "INSERT INTO users VALUES ({}, 'user{}', {})",
                i,
                i,
                20 + (i % 50)
            ))
            .unwrap();
    }
    // Warmup
    for _ in 0..1000 {
        let _ = engine.execute("SELECT * FROM users WHERE id = 50000");
    }

    // No-index benchmark
    let engine_no_idx = engine;
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine_no_idx
            .execute(&format!("SELECT * FROM users WHERE id = {}", i % 100000))
            .unwrap();
    }
    let elapsed = start.elapsed();
    let qps_no_idx = iterations as f64 / elapsed.as_secs_f64();

    // === Test 2: With index (CBO index scan) ===
    drop(engine_no_idx);
    let storage2 = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine2 = ExecutionEngine::new(storage2);
    engine2
        .execute("CREATE TABLE users (id INTEGER, name TEXT, age INTEGER)")
        .unwrap();
    engine2
        .execute("CREATE INDEX idx_id ON users (id)")
        .unwrap();
    for i in 0..100000 {
        engine2
            .execute(&format!(
                "INSERT INTO users VALUES ({}, 'user{}', {})",
                i,
                i,
                20 + (i % 50)
            ))
            .unwrap();
    }
    // Warmup
    for _ in 0..1000 {
        let _ = engine2.execute("SELECT * FROM users WHERE id = 50000");
    }

    // CBO + Index benchmark
    let engine_idx = engine2;
    let start = Instant::now();
    for i in 0..iterations {
        let _ = engine_idx
            .execute(&format!("SELECT * FROM users WHERE id = {}", i % 100000))
            .unwrap();
    }
    let elapsed = start.elapsed();
    let qps_idx = iterations as f64 / elapsed.as_secs_f64();

    println!("========================================");
    println!(
        "Point SELECT Performance (100k rows, {} iterations)",
        iterations
    );
    println!("========================================");
    println!("  No index (full scan): {:.0} qps", qps_no_idx);
    println!("  With index (CBO scan): {:.0} qps", qps_idx);
    println!("  Speedup: {:.2}x", qps_idx / qps_no_idx);
    println!("========================================");
}
