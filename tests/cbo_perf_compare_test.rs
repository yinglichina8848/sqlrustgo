// CBO Performance Comparison Test
use sqlrustgo::{ExecutionEngine, MemoryStorage};
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[test]
#[ignore]
fn test_cbo_index_scan_vs_full_scan_performance() {
    let iterations = 5000;
    let data_size = 100_000;

    // ===== Full scan (no index) =====
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new(storage);
    engine
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    for i in 0..data_size {
        engine
            .execute(&format!("INSERT INTO users VALUES ({}, 'u{}')", i, i))
            .unwrap();
    }
    // warmup
    for _ in 0..100 {
        let _ = engine.execute("SELECT * FROM users WHERE id = 50000");
    }

    let start = Instant::now();
    for i in 0..iterations {
        let r = engine
            .execute(&format!("SELECT * FROM users WHERE id = {}", i % data_size))
            .unwrap();
        assert_eq!(r.rows.len(), 1);
    }
    let full_elapsed = start.elapsed();
    let full_qps = iterations as f64 / full_elapsed.as_secs_f64();

    // ===== Index scan (CBO + index) =====
    let storage2 = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine2 = ExecutionEngine::new(storage2);
    engine2
        .execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .unwrap();
    engine2
        .execute("CREATE INDEX idx_id ON users (id)")
        .unwrap();
    for i in 0..data_size {
        engine2
            .execute(&format!("INSERT INTO users VALUES ({}, 'u{}')", i, i))
            .unwrap();
    }
    // warmup
    for _ in 0..100 {
        let _ = engine2.execute("SELECT * FROM users WHERE id = 50000");
    }

    let start = Instant::now();
    for i in 0..iterations {
        let r = engine2
            .execute(&format!("SELECT * FROM users WHERE id = {}", i % data_size))
            .unwrap();
        assert_eq!(r.rows.len(), 1);
    }
    let idx_elapsed = start.elapsed();
    let idx_qps = iterations as f64 / idx_elapsed.as_secs_f64();

    let speedup = idx_qps / full_qps;
    println!(
        "\n=== CBO Index Scan Performance (100k rows, {} iterations) ===",
        iterations
    );
    println!("  Full scan:  {:.0} qps ({:?})", full_qps, full_elapsed);
    println!("  Index scan: {:.0} qps ({:?})", idx_qps, idx_elapsed);
    println!("  Speedup:    {:.2}x", speedup);
    println!("========================================");
    // Index scan should be faster, but in MemoryStorage the difference
    // is modest since full scan is also fast (no I/O bottleneck)
}
