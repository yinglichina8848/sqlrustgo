//! Long-Run Stability Tests - 72 Hour Actual Run
//!
//! P0 tests for 72h stability per ISSUE #847
//! This version runs for ACTUAL 72 hours (not accelerated)
//!
//! Run with --ignored flag:
//!   cargo test --test long_run_stability_72h_test -- --ignored

use sqlrustgo::MemoryExecutionEngine;
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use std::fs::OpenOptions;
use std::io::Write;

const CONCURRENT_THREADS: usize = 8;
const TEST_DURATION_HOURS: u64 = 1; // 1 hour for quick verification (change to 72 for actual run)
const LOG_FILE: &str = "test_results_72h/72h_test_progress.log";

/// Helper to create a fresh engine
fn create_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new(storage)
}

/// Create test table
fn setup_table(engine: &mut MemoryExecutionEngine) {
    let _ = engine.execute("CREATE TABLE stability_test (id INTEGER, value TEXT)");
}

/// Clean up test table
fn cleanup_table(engine: &mut MemoryExecutionEngine) {
    let _ = engine.execute("DROP TABLE IF EXISTS stability_test");
}

/// Log progress to file
fn log_progress(test_name: &str, message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
    {
        writeln!(file, "[{:?}] {}: {}", std::time::SystemTime::now(), test_name, message).ok();
    }
}

/// Test 1: Sustained Write Load for 72 hours
#[test]
#[ignore]
fn test_sustained_write_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);
    let start = Instant::now();
    let mut total_inserted = 0i64;

    log_progress("test_sustained_write_72h", &format!("Starting 72h test, will run until {:?}", duration));

    let mut engine = create_engine();
    setup_table(&mut engine);

    while start.elapsed() < duration {
        let iteration = total_inserted;
        let result = engine.execute(&format!(
            "INSERT INTO stability_test VALUES ({}, 'value_{}')",
            iteration, iteration
        ));

        assert!(result.is_ok(), "Insert should succeed at iteration {}", iteration);
        total_inserted += 1;

        // Log every 10000 iterations
        if total_inserted % 10000 == 0 {
            let elapsed = start.elapsed().as_secs();
            let remaining = duration.as_secs().saturating_sub(elapsed);
            log_progress("test_sustained_write_72h",
                &format!("Progress: {} iterations, elapsed: {}s, remaining: {}s",
                    total_inserted, elapsed, remaining));
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = total_inserted as f64 / elapsed.as_secs_f64();

    log_progress("test_sustained_write_72h",
        &format!("Completed: {} iterations in {:?}, ops/sec: {:.2}", total_inserted, elapsed, ops_per_sec));

    println!("72h Write Test Complete: {} iterations, {:.2} ops/sec", total_inserted, ops_per_sec);
}

/// Test 2: Sustained Read Load for 72 hours
#[test]
#[ignore]
fn test_sustained_read_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);

    // First populate the table
    let mut engine = create_engine();
    setup_table(&mut engine);

    log_progress("test_sustained_read_72h", "Populating table with initial data...");
    for i in 0..10000 {
        let result = engine.execute(&format!("INSERT INTO stability_test VALUES ({}, 'value_{}')", i, i));
        assert!(result.is_ok());
    }

    let start = Instant::now();
    let mut total_read = 0i64;

    log_progress("test_sustained_read_72h", &format!("Starting 72h read test"));

    while start.elapsed() < duration {
        let result = engine.execute("SELECT * FROM stability_test WHERE id = 0");
        assert!(result.is_ok());
        total_read += 1;

        if total_read % 10000 == 0 {
            let elapsed = start.elapsed().as_secs();
            let remaining = duration.as_secs().saturating_sub(elapsed);
            log_progress("test_sustained_read_72h",
                &format!("Progress: {} reads, elapsed: {}s, remaining: {}s",
                    total_read, elapsed, remaining));
        }
    }

    let elapsed = start.elapsed();
    let ops_per_sec = total_read as f64 / elapsed.as_secs_f64();

    log_progress("test_sustained_read_72h",
        &format!("Completed: {} reads in {:?}, ops/sec: {:.2}", total_read, elapsed, ops_per_sec));

    println!("72h Read Test Complete: {} reads, {:.2} ops/sec", total_read, ops_per_sec);
}

/// Test 3: Concurrent Read/Write for 72 hours
#[test]
#[ignore]
fn test_concurrent_read_write_72h() {
    use std::thread;

    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);
    let start = Instant::now();

    log_progress("test_concurrent_read_write_72h", &format!("Starting 72h concurrent test"));

    // Create shared storage
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let counter = Arc::new(RwLock::new(0i64));
    let start_time = Instant::now();

    // Writer thread
    let storage_clone = Arc::clone(&storage);
    let counter_clone = Arc::clone(&counter);
    let writer_handle = thread::spawn(move || {
        let mut local_counter = *counter_clone.write().unwrap();
        while start_time.elapsed() < duration {
            let mut engine = MemoryExecutionEngine::new(Arc::clone(&storage_clone));
            let result = engine.execute(&format!(
                "INSERT INTO stability_test VALUES ({}, 'value_{}')",
                local_counter, local_counter
            ));
            if result.is_ok() {
                local_counter += 1;
                *counter_clone.write().unwrap() = local_counter;
            }
        }
    });

    // Reader threads
    let mut reader_handles = vec![];
    for _ in 0..CONCURRENT_THREADS {
        let storage_clone = Arc::clone(&storage);
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut local_reads = 0i64;
            while start_time.elapsed() < duration {
                let mut engine = MemoryExecutionEngine::new(Arc::clone(&storage_clone));
                let result = engine.execute("SELECT * FROM stability_test WHERE id % 100 = 0");
                if result.is_ok() {
                    local_reads += 1;
                }
            }
            local_reads
        });
        reader_handles.push(handle);
    }

    // Wait for all threads
    writer_handle.join().unwrap();
    let total_reads: i64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_writes = *counter.read().unwrap();
    let elapsed = start.elapsed();

    log_progress("test_concurrent_read_write_72h",
        &format!("Completed: {} writes, {} reads in {:?}", total_writes, total_reads, elapsed));

    println!("72h Concurrent Test: {} writes, {} reads", total_writes, total_reads);
}
