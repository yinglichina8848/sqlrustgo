//! Long-Run Stability Tests
//!
//! P0 tests for 72h stability per ISSUE #847
//! Simulates extended runtime stability with accelerated testing

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{ColumnDefinition, StorageEngine, TableInfo};
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    const STABILITY_ITERATIONS: usize = 1000;
    const CONCURRENT_THREADS: usize = 8;

    fn create_test_table() -> TableInfo {
        TableInfo {
            name: "stability_test".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                },
                ColumnDefinition {
                    name: "value".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: true,
                    is_unique: false,
                    references: None,
                },
            ],
        }
    }

    #[test]
    fn test_sustained_write_load() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let start = Instant::now();
        let mut total_inserted = 0;

        for iteration in 0..STABILITY_ITERATIONS {
            let storage = storage.clone();
            let result = storage.lock().unwrap().insert(
                "stability_test",
                vec![vec![
                    Value::Integer(iteration as i64),
                    Value::Text(format!("value_{}", iteration)),
                ]],
            );

            assert!(
                result.is_ok(),
                "Insert should succeed at iteration {}",
                iteration
            );
            total_inserted += 1;
        }

        let elapsed = start.elapsed();
        println!(
            "Sustained write test: {} inserts in {:?} ({} ops/sec)",
            total_inserted,
            elapsed,
            total_inserted as f64 / elapsed.as_secs_f64()
        );

        let count = storage
            .lock()
            .unwrap()
            .scan("stability_test")
            .unwrap()
            .len();
        assert_eq!(count, total_inserted);
    }

    #[test]
    fn test_sustained_read_load() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for i in 0..100 {
            storage
                .lock()
                .unwrap()
                .insert(
                    "stability_test",
                    vec![vec![Value::Integer(i), Value::Text(format!("value_{}", i))]],
                )
                .unwrap();
        }

        let start = Instant::now();
        let mut total_reads = 0;

        for _ in 0..STABILITY_ITERATIONS {
            let storage = storage.clone();
            let result = storage.lock().unwrap().scan("stability_test");

            assert!(result.is_ok(), "Scan should succeed");
            total_reads += result.unwrap().len();
        }

        let elapsed = start.elapsed();
        println!(
            "Sustained read test: {} scans in {:?} ({} ops/sec)",
            total_reads,
            elapsed,
            total_reads as f64 / elapsed.as_secs_f64()
        );
    }

    #[test]
    fn test_concurrent_read_write_stability() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));
        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let write_counter = Arc::new(Mutex::new(0usize));
        let read_counter = Arc::new(Mutex::new(0usize));
        let error_counter = Arc::new(Mutex::new(0usize));

        let mut handles = vec![];

        for thread_id in 0..CONCURRENT_THREADS {
            let storage = storage.clone();
            let counter = write_counter.clone();
            let errors = error_counter.clone();

            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let mut s = storage.lock().unwrap();
                    let unique_id = thread_id * 1000 + i;
                    let result = s.insert(
                        "stability_test",
                        vec![vec![
                            Value::Integer(unique_id as i64),
                            Value::Text(format!("concurrent_{}", unique_id)),
                        ]],
                    );

                    if result.is_ok() {
                        *counter.lock().unwrap() += 1;
                    } else {
                        *errors.lock().unwrap() += 1;
                    }
                }
            });
            handles.push(handle);
        }

        for _ in 0..CONCURRENT_THREADS {
            let storage = storage.clone();
            let counter = read_counter.clone();

            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let s = storage.lock().unwrap();
                    let result = s.scan("stability_test");

                    if result.is_ok() {
                        *counter.lock().unwrap() += 1;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let writes = *write_counter.lock().unwrap();
        let reads = *read_counter.lock().unwrap();
        let errors = *error_counter.lock().unwrap();

        println!(
            "Concurrent stability: {} writes, {} reads, {} errors",
            writes, reads, errors
        );

        assert_eq!(
            errors, 0,
            "No errors should occur during concurrent operations"
        );
    }

    #[test]
    fn test_repeated_create_drop_stability() {
        let start = Instant::now();

        for i in 0..100 {
            let mut storage = MemoryStorage::new();

            let info = TableInfo {
                name: format!("temp_table_{}", i),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    references: None,
                }],
            };

            let result = storage.create_table(&info);
            assert!(
                result.is_ok(),
                "Create table should succeed at iteration {}",
                i
            );

            let exists = storage.has_table(&info.name);
            assert!(exists, "Table should exist after create");

            storage.drop_table(&info.name).ok();

            let exists_after = storage.has_table(&info.name);
            assert!(!exists_after, "Table should not exist after drop");
        }

        let elapsed = start.elapsed();
        println!("Repeated create/drop test: 100 cycles in {:?}", elapsed);
    }

    #[test]
    fn test_memory_stability_under_load() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = TableInfo {
            name: "memory_test".to_string(),
            columns: vec![ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                references: None,
            }],
        };

        storage.lock().unwrap().create_table(&info).unwrap();

        let initial_size = {
            let s = storage.lock().unwrap();
            s.scan("memory_test").unwrap().len()
        };

        for batch in 0..50 {
            let storage = storage.clone();
            let mut handles = vec![];

            for i in 0..20 {
                let storage = storage.clone();
                let handle = thread::spawn(move || {
                    let mut s = storage.lock().unwrap();
                    for j in 0..10 {
                        s.insert(
                            "memory_test",
                            vec![vec![Value::Integer((batch * 200 + i * 10 + j) as i64)]],
                        )
                        .ok();
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }

        let final_size = {
            let s = storage.lock().unwrap();
            s.scan("memory_test").unwrap().len()
        };

        println!("Memory stability: {} -> {} rows", initial_size, final_size);

        assert!(final_size > initial_size, "Data should be inserted");
    }

    #[test]
    fn test_table_info_consistency_under_load() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        for _ in 0..STABILITY_ITERATIONS {
            let retrieved = storage
                .lock()
                .unwrap()
                .get_table_info("stability_test")
                .unwrap();

            assert_eq!(retrieved.name, "stability_test");
            assert_eq!(retrieved.columns.len(), 2);
        }
    }

    #[test]
    fn test_list_tables_stability() {
        let mut storage = MemoryStorage::new();

        for i in 0..50 {
            let info = TableInfo {
                name: format!("table_{}", i),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();
        }

        for _ in 0..1000 {
            let tables = storage.list_tables();
            assert_eq!(tables.len(), 50);
        }
    }

    #[test]
    fn test_interleaved_read_write_consistency() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let mut handles = vec![];

        for iteration in 0..10 {
            let storage = storage.clone();

            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let mut s = storage.lock().unwrap();

                    s.insert(
                        "stability_test",
                        vec![vec![
                            Value::Integer((iteration * 100 + i) as i64),
                            Value::Text(format!("iter{}_{}", iteration, i)),
                        ]],
                    )
                    .ok();

                    s.scan("stability_test").ok();

                    s.get_table_info("stability_test").ok();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_count = storage
            .lock()
            .unwrap()
            .scan("stability_test")
            .unwrap()
            .len();
        assert!(final_count > 0, "Should have inserted data");
    }

    #[test]
    fn test_rapid_burst_writes() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = create_test_table();
        storage.lock().unwrap().create_table(&info).unwrap();

        let start = Instant::now();

        for burst in 0..10 {
            let storage = storage.clone();
            let mut handles = vec![];

            for i in 0..100 {
                let storage = storage.clone();
                let handle = thread::spawn(move || {
                    let mut s = storage.lock().unwrap();
                    s.insert(
                        "stability_test",
                        vec![vec![
                            Value::Integer((burst * 100 + i) as i64),
                            Value::Text(format!("burst{}_{}", burst, i)),
                        ]],
                    )
                    .ok();
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            thread::sleep(Duration::from_millis(10));
        }

        let elapsed = start.elapsed();
        let count = storage
            .lock()
            .unwrap()
            .scan("stability_test")
            .unwrap()
            .len();

        println!("Burst writes test: {} inserts in {:?}", count, elapsed);

        assert!(count > 0);
    }

    #[test]
    fn test_stress_table_operations() {
        let mut storage = MemoryStorage::new();

        for i in 0..20 {
            let info = TableInfo {
                name: format!("stress_table_{}", i),
                columns: vec![ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    references: None,
                }],
            };

            storage.create_table(&info).unwrap();

            for j in 0..50 {
                storage
                    .insert(
                        &format!("stress_table_{}", i),
                        vec![vec![Value::Integer(j)]],
                    )
                    .ok();
            }
        }

        for i in 0..20 {
            let info = storage
                .get_table_info(&format!("stress_table_{}", i))
                .unwrap();
            assert_eq!(info.name, format!("stress_table_{}", i));

            let data = storage.scan(&format!("stress_table_{}", i)).unwrap();
            assert_eq!(data.len(), 50);
        }
    }
}
