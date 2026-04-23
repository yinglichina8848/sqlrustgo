//! Long-Run Stability Tests - 72 Hour Actual Run
//!
//! P0 tests for 72h stability per ISSUE #847
//! This version runs for ACTUAL 72 hours (not accelerated)
//!
//! Run with --ignored flag:
//!   cargo test --test long_run_stability_72h_test --release -- --ignored

use sqlrustgo::{ExecutionEngine, MemoryExecutionEngine};
use sqlrustgo_storage::{FileStorage, MemoryStorage};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicI64, Ordering},
    Arc, RwLock,
};
use std::time::{Duration, Instant};

const CONCURRENT_THREADS: usize = 8;
const TEST_DURATION_HOURS: u64 = 72;
const LOG_FILE: &str = "test_results_72h/72h_test_progress.log";
const MONITOR_INTERVAL_SECS: u64 = 600;

fn create_engine() -> ExecutionEngine<FileStorage> {
    let data_dir = PathBuf::from("test_results_72h/file_storage_data");
    std::fs::create_dir_all(&data_dir).ok();
    let storage = FileStorage::new(data_dir).expect("Failed to create FileStorage");
    ExecutionEngine::new(Arc::new(RwLock::new(storage)))
}

fn create_memory_engine() -> MemoryExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    MemoryExecutionEngine::new(storage)
}

fn setup_table(engine: &mut ExecutionEngine<FileStorage>) {
    let _ = engine.execute("CREATE TABLE IF NOT EXISTS stability_test (id INTEGER, value TEXT)");
}

fn setup_memory_table(engine: &mut MemoryExecutionEngine) {
    let _ = engine.execute("CREATE TABLE IF NOT EXISTS stability_test (id INTEGER, value TEXT)");
}

fn log_progress(test_name: &str, message: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        writeln!(
            file,
            "[{:?}] {}: {}",
            std::time::SystemTime::now(),
            test_name,
            message
        )
        .ok();
    }
}

fn get_process_stats() -> (u64, f64) {
    let pid = std::process::id() as u64;
    let stat_file = format!("/proc/{}/stat", pid);

    let mut rss_bytes: u64 = 0;
    let mut cpu_pct: f64 = 0.0;

    if let Ok(content) = std::fs::read_to_string(&stat_file) {
        if let Some(last) = content.split(')').next_back() {
            let parts: Vec<&str> = last.split_whitespace().collect();
            if parts.len() >= 24 {
                let utime: u64 = parts[11].parse().unwrap_or(0);
                let stime: u64 = parts[12].parse().unwrap_or(0);
                if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
                    if let Some(first) = uptime.split_whitespace().next() {
                        if let Ok(uptime_secs) = first.parse::<f64>() {
                            let clk_tck: f64 = 100.0;
                            if let Some(start_time_idx) = content.find("pow(") {
                                if let Some(st) =
                                    content[start_time_idx..].split_whitespace().nth(1)
                                {
                                    if let Ok(st_usec) = st.parse::<u64>() {
                                        let start_secs = st_usec as f64 / 1_000_000.0 / clk_tck;
                                        let secs = uptime_secs - start_secs;
                                        if secs > 0.0 {
                                            let total_time = utime + stime;
                                            cpu_pct = (total_time as f64 / clk_tck / secs) * 100.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let statm_file = format!("/proc/{}/statm", pid);
    if let Ok(content) = std::fs::read_to_string(&statm_file) {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(size) = first.parse::<u64>() {
                rss_bytes = size * 4096;
            }
        }
    }

    (rss_bytes, cpu_pct)
}

fn get_disk_usage(path: &str) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn log_system_stats(test_name: &str, ops_count: i64, elapsed_secs: u64, remaining_secs: u64) {
    let (rss, cpu_pct) = get_process_stats();
    let rss_mb = rss / 1024 / 1024;
    let disk_bytes = get_disk_usage("test_results_72h");
    let disk_mb = disk_bytes / 1024 / 1024;

    log_progress(
        test_name,
        &format!(
            "STATS|ops={}|elapsed={}s|remaining={}s|RSS={}MB|cpu={:.1}%|disk={}MB",
            ops_count, elapsed_secs, remaining_secs, rss_mb, cpu_pct, disk_mb
        ),
    );
}

fn spawn_monitor_thread(
    test_name: String,
    ops_counter: Arc<AtomicI64>,
    running: Arc<AtomicBool>,
    duration: Duration,
) {
    std::thread::spawn(move || {
        let start = Instant::now();
        while running.load(Ordering::Relaxed) && start.elapsed() < duration {
            std::thread::sleep(Duration::from_secs(MONITOR_INTERVAL_SECS));
            if !running.load(Ordering::Relaxed) {
                break;
            }
            let ops = ops_counter.load(Ordering::Relaxed);
            let elapsed = start.elapsed().as_secs();
            let remaining = duration.as_secs().saturating_sub(elapsed);
            log_system_stats(&test_name, ops, elapsed, remaining);
        }
    });
}

#[test]
#[ignore]
fn test_sustained_write_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);
    let start = Instant::now();
    let ops_counter = Arc::new(AtomicI64::new(0));
    let running = Arc::new(AtomicBool::new(true));

    log_progress(
        "test_sustained_write_72h",
        "Starting 72h write test (FileStorage with circular buffer)",
    );

    let mut engine = create_engine();
    setup_table(&mut engine);

    let ops_counter_clone = Arc::clone(&ops_counter);
    let running_clone = Arc::clone(&running);
    spawn_monitor_thread(
        "test_sustained_write_72h".to_string(),
        ops_counter_clone,
        running_clone,
        duration,
    );

    let mut total_inserted = 0i64;
    const CIRCULAR_SIZE: i64 = 100_000;

    while start.elapsed() < duration {
        let circular_id = total_inserted % CIRCULAR_SIZE;

        let result = engine.execute(&format!(
            "INSERT INTO stability_test VALUES ({}, 'value_{}')",
            circular_id, total_inserted
        ));
        assert!(
            result.is_ok(),
            "Insert should succeed at iteration {}",
            total_inserted
        );
        total_inserted += 1;
        ops_counter.fetch_add(1, Ordering::Relaxed);

        if total_inserted % 10000 == 0 {
            let elapsed = start.elapsed().as_secs();
            let remaining = duration.as_secs().saturating_sub(elapsed);
            log_progress(
                "test_sustained_write_72h",
                &format!(
                    "Progress: {} iterations (circular id: {}), elapsed: {}s, remaining: {}s",
                    total_inserted, circular_id, elapsed, remaining
                ),
            );
        }

        if total_inserted > 0 && total_inserted % CIRCULAR_SIZE == 0 {
            log_progress(
                "test_sustained_write_72h",
                "Circular buffer full - truncating table",
            );
            let _ = engine.execute("DROP TABLE IF EXISTS stability_test");
            setup_table(&mut engine);
        }
    }

    running.store(false, Ordering::Relaxed);
    let elapsed = start.elapsed();
    let ops_per_sec = total_inserted as f64 / elapsed.as_secs_f64();

    log_progress(
        "test_sustained_write_72h",
        &format!(
            "Completed: {} iterations in {:?}, ops/sec: {:.2}",
            total_inserted, elapsed, ops_per_sec
        ),
    );

    println!(
        "72h Write Test Complete: {} iterations, {:.2} ops/sec",
        total_inserted, ops_per_sec
    );
}

#[test]
#[ignore]
fn test_sustained_write_concurrent_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);
    let start = Instant::now();
    let ops_counter = Arc::new(AtomicI64::new(0));
    let counter = Arc::new(AtomicI64::new(0));
    let running = Arc::new(AtomicBool::new(true));

    log_progress(
        "test_sustained_write_concurrent_72h",
        "Starting 72h concurrent write test (8 threads)",
    );

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = create_memory_engine();
    setup_memory_table(&mut engine);

    let ops_counter_clone = Arc::clone(&ops_counter);
    let running_clone = Arc::clone(&running);
    spawn_monitor_thread(
        "test_sustained_write_concurrent_72h".to_string(),
        ops_counter_clone,
        running_clone,
        duration,
    );

    let mut handles = vec![];
    for i in 0..CONCURRENT_THREADS {
        let storage_clone = Arc::clone(&storage);
        let counter_clone = Arc::clone(&counter);
        let ops_clone = Arc::clone(&ops_counter);
        let run_clone = Arc::clone(&running);
        let dur = duration;
        let start_clone = start;
        handles.push(std::thread::spawn(move || {
            let mut local_count = counter_clone.fetch_add(1000000 * i as i64, Ordering::Relaxed);
            let mut local_inserted = 0i64;
            while start_clone.elapsed() < dur && run_clone.load(Ordering::Relaxed) {
                let mut engine = MemoryExecutionEngine::new(Arc::clone(&storage_clone));
                let result = engine.execute(&format!(
                    "INSERT INTO stability_test VALUES ({}, 'value_{}')",
                    local_count, local_count
                ));
                if result.is_ok() {
                    local_count += 1;
                    local_inserted += 1;
                    ops_clone.fetch_add(1, Ordering::Relaxed);
                } else {
                    std::thread::sleep(Duration::from_micros(100));
                }
            }
            local_inserted
        }));
    }

    let total_inserted: i64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    running.store(false, Ordering::Relaxed);
    let elapsed = start.elapsed();
    let ops_per_sec = total_inserted as f64 / elapsed.as_secs_f64();

    log_progress(
        "test_sustained_write_concurrent_72h",
        &format!(
            "Completed: {} iterations in {:?}, ops/sec: {:.2}",
            total_inserted, elapsed, ops_per_sec
        ),
    );

    println!(
        "72h Concurrent Write Test Complete: {} iterations, {:.2} ops/sec",
        total_inserted, ops_per_sec
    );
}

#[test]
#[ignore]
fn test_sustained_read_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);

    let mut engine = create_memory_engine();
    setup_memory_table(&mut engine);

    log_progress(
        "test_sustained_read_72h",
        "Populating table with initial data...",
    );
    for i in 0..10000 {
        let result = engine.execute(&format!(
            "INSERT INTO stability_test VALUES ({}, 'value_{}')",
            i, i
        ));
        assert!(result.is_ok());
    }

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    for i in 0..10000 {
        let mut eng = MemoryExecutionEngine::new(Arc::clone(&storage));
        let _ = eng.execute(&format!(
            "INSERT INTO stability_test VALUES ({}, 'value_{}')",
            i, i
        ));
    }

    let start = Instant::now();
    let ops_counter = Arc::new(AtomicI64::new(0));
    let running = Arc::new(AtomicBool::new(true));

    let ops_counter_clone = Arc::clone(&ops_counter);
    let running_clone = Arc::clone(&running);
    spawn_monitor_thread(
        "test_sustained_read_72h".to_string(),
        ops_counter_clone,
        running_clone,
        duration,
    );

    let mut handles = vec![];
    for _ in 0..CONCURRENT_THREADS {
        let storage_clone = Arc::clone(&storage);
        let run_clone = Arc::clone(&running);
        let dur = duration;
        let start_clone = start;
        handles.push(std::thread::spawn(move || {
            let mut local_reads = 0i64;
            while start_clone.elapsed() < dur && run_clone.load(Ordering::Relaxed) {
                let mut eng = MemoryExecutionEngine::new(Arc::clone(&storage_clone));
                let _ = eng.execute("SELECT * FROM stability_test WHERE id % 100 = 0");
                local_reads += 1;
            }
            local_reads
        }));
    }

    let total_read: i64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    ops_counter.store(total_read, Ordering::Relaxed);

    running.store(false, Ordering::Relaxed);
    let elapsed = start.elapsed();
    let ops_per_sec = total_read as f64 / elapsed.as_secs_f64();

    log_progress(
        "test_sustained_read_72h",
        &format!(
            "Completed: {} reads in {:?}, ops/sec: {:.2}",
            total_read, elapsed, ops_per_sec
        ),
    );

    println!(
        "72h Read Test Complete: {} reads, {:.2} ops/sec",
        total_read, ops_per_sec
    );
}

#[test]
#[ignore]
fn test_concurrent_read_write_72h() {
    let duration = Duration::from_secs(TEST_DURATION_HOURS * 3600);
    let start = Instant::now();

    log_progress(
        "test_concurrent_read_write_72h",
        "Starting 72h concurrent test",
    );

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let counter = Arc::new(AtomicI64::new(0));
    let running = Arc::new(AtomicBool::new(true));
    let start_time = Instant::now();

    let ops_counter = Arc::new(AtomicI64::new(0));
    let ops_counter_clone = Arc::clone(&ops_counter);
    let running_clone = Arc::clone(&running);
    spawn_monitor_thread(
        "test_concurrent_read_write_72h".to_string(),
        ops_counter_clone,
        running_clone,
        duration,
    );

    let writer_counter = Arc::clone(&counter);
    let writer_storage = Arc::clone(&storage);
    let writer_running = Arc::clone(&running);
    let writer_start = start_time;
    let writer_dur = duration;
    let writer_ops = Arc::clone(&ops_counter);
    let writer_handle = std::thread::spawn(move || {
        let mut local_counter = writer_counter.load(Ordering::Relaxed);
        while writer_start.elapsed() < writer_dur && writer_running.load(Ordering::Relaxed) {
            let mut engine = MemoryExecutionEngine::new(Arc::clone(&writer_storage));
            let result = engine.execute(&format!(
                "INSERT INTO stability_test VALUES ({}, 'value_{}')",
                local_counter, local_counter
            ));
            if result.is_ok() {
                local_counter += 1;
                writer_counter.store(local_counter, Ordering::Relaxed);
                writer_ops.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let mut reader_handles = vec![];
    for _ in 0..CONCURRENT_THREADS {
        let storage_clone = Arc::clone(&storage);
        let run_clone = Arc::clone(&running);
        let start_clone = start_time;
        let dur_clone = duration;
        let handle = std::thread::spawn(move || {
            let mut local_reads = 0i64;
            while start_clone.elapsed() < dur_clone && run_clone.load(Ordering::Relaxed) {
                let mut engine = MemoryExecutionEngine::new(Arc::clone(&storage_clone));
                let _ = engine.execute("SELECT * FROM stability_test WHERE id % 100 = 0");
                local_reads += 1;
            }
            local_reads
        });
        reader_handles.push(handle);
    }

    writer_handle.join().unwrap();
    running.store(false, Ordering::Relaxed);
    let total_reads: i64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_writes = counter.load(Ordering::Relaxed);
    let elapsed = start.elapsed();

    log_progress(
        "test_concurrent_read_write_72h",
        &format!(
            "Completed: {} writes, {} reads in {:?}",
            total_writes, total_reads, elapsed
        ),
    );

    println!(
        "72h Concurrent Test: {} writes, {} reads",
        total_writes, total_reads
    );
}
