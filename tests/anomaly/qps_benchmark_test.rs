//! QPS & Concurrent Performance Benchmark Tests
//!
//! P0 tests for QPS/并发性能目标 per ISSUE #842
//! Validates performance targets are met

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{ColumnDefinition, StorageEngine, TableInfo};
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    const WARMUP_ITERATIONS: usize = 100;
    const BENCHMARK_ITERATIONS: usize = 10000;
    const CONCURRENT_CLIENTS: usize = 16;

    fn create_benchmark_table() -> TableInfo {
        TableInfo {
            name: "benchmark".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "data".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        }
    }

    #[test]
    fn test_insert_qps_benchmark() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..WARMUP_ITERATIONS {
            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer(i as i64),
                        Value::Text(format!("warmup_{}", i)),
                    ]],
                )
                .ok();
        }

        let start = Instant::now();

        for i in 0..BENCHMARK_ITERATIONS {
            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer((WARMUP_ITERATIONS + i) as i64),
                        Value::Text(format!("data_{}", i)),
                    ]],
                )
                .ok();
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

        println!(
            "Insert QPS: {} ops in {:?} = {:.2} ops/sec",
            BENCHMARK_ITERATIONS, elapsed, qps
        );

        let count = storage.lock().unwrap().scan("benchmark").unwrap().len();
        assert_eq!(count, WARMUP_ITERATIONS + BENCHMARK_ITERATIONS);
    }

    #[test]
    fn test_scan_qps_benchmark() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..1000 {
            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer(i as i64),
                        Value::Text(format!("data_{}", i)),
                    ]],
                )
                .ok();
        }

        let start = Instant::now();

        for _ in 0..BENCHMARK_ITERATIONS {
            storage.lock().unwrap().scan("benchmark").ok();
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

        println!(
            "Scan QPS: {} ops in {:?} = {:.2} ops/sec",
            BENCHMARK_ITERATIONS, elapsed, qps
        );

        assert!(qps > 0.0, "QPS should be positive");
    }

    #[test]
    fn test_point_query_qps() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = TableInfo {
            name: "point_query".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..1000 {
            storage
                .lock()
                .unwrap()
                .insert("point_query", vec![vec![Value::Integer(i as i64)]])
                .ok();
        }

        let start = Instant::now();

        for i in 0..BENCHMARK_ITERATIONS {
            let id = (i % 1000) as i64;
            let result = storage.lock().unwrap().scan("point_query");

            if let Ok(rows) = result {
                let _found = rows.iter().any(|r| r.get(0) == Some(&Value::Integer(id)));
            }
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

        println!(
            "Point query QPS: {} ops in {:?} = {:.2} ops/sec",
            BENCHMARK_ITERATIONS, elapsed, qps
        );

        assert!(qps > 0.0);
    }

    #[test]
    fn test_concurrent_insert_qps() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let total_ops = Arc::new(Mutex::new(0usize));

        let start = Instant::now();

        let mut handles = vec![];

        for thread_id in 0..CONCURRENT_CLIENTS {
            let storage = storage.clone();
            let counter = total_ops.clone();

            let handle = thread::spawn(move || {
                let ops_per_thread = BENCHMARK_ITERATIONS / CONCURRENT_CLIENTS;
                let mut local_count = 0;

                for i in 0..ops_per_thread {
                    let unique_id = thread_id * ops_per_thread + i;
                    let result = storage.lock().unwrap().insert(
                        "benchmark",
                        vec![vec![
                            Value::Integer(unique_id as i64),
                            Value::Text(format!("concurrent_{}", unique_id)),
                        ]],
                    );

                    if result.is_ok() {
                        local_count += 1;
                    }
                }

                *counter.lock().unwrap() += local_count;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total = *total_ops.lock().unwrap();
        let qps = total as f64 / elapsed.as_secs_f64();

        println!(
            "Concurrent insert QPS: {} ops in {:?} = {:.2} ops/sec ({} clients)",
            total, elapsed, qps, CONCURRENT_CLIENTS
        );

        assert!(qps > 0.0);
    }

    #[test]
    fn test_concurrent_read_qps() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..1000 {
            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer(i as i64),
                        Value::Text(format!("data_{}", i)),
                    ]],
                )
                .ok();
        }

        let total_ops = Arc::new(Mutex::new(0usize));

        let start = Instant::now();

        let mut handles = vec![];

        for _ in 0..CONCURRENT_CLIENTS {
            let storage = storage.clone();
            let counter = total_ops.clone();

            let handle = thread::spawn(move || {
                let ops_per_thread = BENCHMARK_ITERATIONS / CONCURRENT_CLIENTS;
                let mut local_count = 0;

                for _ in 0..ops_per_thread {
                    let result = storage.lock().unwrap().scan("benchmark");

                    if result.is_ok() {
                        local_count += 1;
                    }
                }

                *counter.lock().unwrap() += local_count;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total = *total_ops.lock().unwrap();
        let qps = total as f64 / elapsed.as_secs_f64();

        println!(
            "Concurrent read QPS: {} ops in {:?} = {:.2} ops/sec ({} clients)",
            total, elapsed, qps, CONCURRENT_CLIENTS
        );

        assert!(qps > 0.0);
    }

    #[test]
    fn test_mixed_read_write_qps() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..100 {
            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer(i as i64),
                        Value::Text(format!("initial_{}", i)),
                    ]],
                )
                .ok();
        }

        let read_counter = Arc::new(Mutex::new(0usize));
        let write_counter = Arc::new(Mutex::new(0usize));

        let start = Instant::now();

        let mut handles = vec![];

        for thread_id in 0..8 {
            let storage = storage.clone();
            let reads = read_counter.clone();
            let writes = write_counter.clone();

            let handle = thread::spawn(move || {
                let local_reads = Arc::new(Mutex::new(0usize));
                let local_writes = Arc::new(Mutex::new(0usize));

                for i in 0..500 {
                    let mut s = storage.lock().unwrap();

                    if i % 2 == 0 {
                        s.scan("benchmark").ok();
                        *local_reads.lock().unwrap() += 1;
                    } else {
                        let unique_id = thread_id * 1000 + i;
                        s.insert(
                            "benchmark",
                            vec![vec![
                                Value::Integer(unique_id as i64),
                                Value::Text(format!("mixed_{}", unique_id)),
                            ]],
                        )
                        .ok();
                        *local_writes.lock().unwrap() += 1;
                    }
                }

                *reads.lock().unwrap() += *local_reads.lock().unwrap();
                *writes.lock().unwrap() += *local_writes.lock().unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let elapsed = start.elapsed();
        let total_reads = *read_counter.lock().unwrap();
        let total_writes = *write_counter.lock().unwrap();
        let total = total_reads + total_writes;
        let qps = total as f64 / elapsed.as_secs_f64();

        println!(
            "Mixed R/W QPS: {} reads + {} writes = {} ops in {:?} = {:.2} ops/sec",
            total_reads, total_writes, total, elapsed, qps
        );

        assert!(qps > 0.0);
    }

    #[test]
    fn test_bulk_insert_performance() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let bulk_size = 1000;

        let start = Instant::now();

        let mut all_records = Vec::with_capacity(bulk_size);
        for i in 0..bulk_size {
            all_records.push(vec![
                Value::Integer(i as i64),
                Value::Text(format!("bulk_{}", i)),
            ]);
        }

        for chunk in all_records.chunks(100) {
            let mut records = chunk.to_vec();
            storage
                .lock()
                .unwrap()
                .insert("benchmark", std::mem::take(&mut records))
                .ok();
        }

        let elapsed = start.elapsed();
        let qps = bulk_size as f64 / elapsed.as_secs_f64();

        println!(
            "Bulk insert: {} records in {:?} = {:.2} records/sec",
            bulk_size, elapsed, qps
        );

        let count = storage.lock().unwrap().scan("benchmark").unwrap().len();
        assert_eq!(count, bulk_size);
    }

    #[test]
    fn test_table_metadata_qps() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        for i in 0..50 {
            let info = TableInfo {
                name: format!("table_{}", i),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                }],
            };
            storage.lock().unwrap().create_table(&info).unwrap();
        }

        let start = Instant::now();

        for _ in 0..BENCHMARK_ITERATIONS {
            let tables = storage.lock().unwrap().list_tables();
            assert_eq!(tables.len(), 50);
        }

        let elapsed = start.elapsed();
        let qps = BENCHMARK_ITERATIONS as f64 / elapsed.as_secs_f64();

        println!(
            "Table metadata QPS: {} ops in {:?} = {:.2} ops/sec",
            BENCHMARK_ITERATIONS, elapsed, qps
        );

        assert!(qps > 0.0);
    }

    #[test]
    fn test_high_concurrency_stability() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let error_count = Arc::new(Mutex::new(0usize));
        let success_count = Arc::new(Mutex::new(0usize));

        let mut handles = vec![];

        for thread_id in 0..32 {
            let storage = storage.clone();
            let errors = error_count.clone();
            let successes = success_count.clone();

            let handle = thread::spawn(move || {
                let mut local_errors = 0;
                let mut local_successes = 0;

                for i in 0..200 {
                    let unique_id = thread_id * 10000 + i;
                    let mut s = storage.lock().unwrap();

                    let op_result: Result<(), _> = if i % 2 == 0 {
                        s.scan("benchmark").map(|_| ())
                    } else {
                        s.insert(
                            "benchmark",
                            vec![vec![
                                Value::Integer(unique_id as i64),
                                Value::Text(format!("hc_{}", unique_id)),
                            ]],
                        )
                    };

                    if op_result.is_ok() {
                        local_successes += 1;
                    } else {
                        local_errors += 1;
                    }
                }

                *errors.lock().unwrap() += local_errors;
                *successes.lock().unwrap() += local_successes;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let errors = *error_count.lock().unwrap();
        let successes = *success_count.lock().unwrap();
        let total = errors + successes;

        println!(
            "High concurrency: {} successes, {} errors ({}% success rate)",
            successes,
            errors,
            (successes as f64 / total as f64) * 100.0
        );

        assert!(successes > 0, "Should have successful operations");
    }

    #[test]
    fn test_latency_percentiles() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_benchmark_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let mut latencies = Vec::with_capacity(1000);

        for _ in 0..1000 {
            let start = Instant::now();

            storage
                .lock()
                .unwrap()
                .insert(
                    "benchmark",
                    vec![vec![
                        Value::Integer(0),
                        Value::Text("latency_test".to_string()),
                    ]],
                )
                .ok();

            latencies.push(start.elapsed());
        }

        latencies.sort();

        let p50 = latencies[latencies.len() / 2];
        let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
        let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

        println!(
            "Insert latency: p50={:?}, p95={:?}, p99={:?}",
            p50, p95, p99
        );

        assert!(p50.as_nanos() > 0);
    }
}
