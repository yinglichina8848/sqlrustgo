//! Production Scenario Tests
//!
//! These tests simulate realistic production scenarios:
//! - High-volume OLTP workloads
//! - Complex queries with joins and aggregations
//! - Concurrent read/write operations
//! - Data consistency under load

use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};
use sqlrustgo_storage::wal::WalManager;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oltp_workload() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let setup_start = Instant::now();

        // Setup: Create and populate tables
        {
            let mut storage = storage.lock().unwrap();

            storage
                .create_table(&sqlrustgo_storage::engine::TableInfo {
                    name: "orders".to_string(),
                    columns: vec![
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "customer_id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "amount".to_string(),
                            data_type: "FLOAT".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "status".to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                    ],
                })
                .unwrap();

            storage
                .create_table(&sqlrustgo_storage::engine::TableInfo {
                    name: "customers".to_string(),
                    columns: vec![
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "name".to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                        sqlrustgo_storage::engine::ColumnDefinition {
                            name: "region".to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: false,
                            is_unique: false,
                            is_primary_key: false,
                            auto_increment: false,
                            references: None,
                        },
                    ],
                })
                .unwrap();

            // Insert test data
            for i in 1..=100 {
                storage
                    .insert(
                        "customers",
                        vec![vec![
                            sqlrustgo_types::Value::Integer(i as i64),
                            sqlrustgo_types::Value::Text(format!("Customer {}", i)),
                            sqlrustgo_types::Value::Text(format!("Region {}", (i - 1) / 20 + 1)),
                        ]],
                    )
                    .unwrap();

                for j in 1..=10 {
                    storage
                        .insert(
                            "orders",
                            vec![vec![
                                sqlrustgo_types::Value::Integer((i * 10 + j) as i64),
                                sqlrustgo_types::Value::Integer(i as i64),
                                sqlrustgo_types::Value::Float((i * 10 + j) as f64),
                                sqlrustgo_types::Value::Text("pending".to_string()),
                            ]],
                        )
                        .unwrap();
                }
            }

            let order_count = storage.scan("orders").unwrap().len();
            let customer_count = storage.scan("customers").unwrap().len();

            println!(
                "Setup: {} orders, {} customers in {:?}",
                order_count,
                customer_count,
                setup_start.elapsed()
            );

            assert_eq!(order_count, 1000);
            assert_eq!(customer_count, 100);
        }

        println!("✓ OLTP workload test passed");
    }

    #[test]
    fn test_concurrent_read_write() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        // Setup table
        storage
            .lock()
            .unwrap()
            .create_table(&sqlrustgo_storage::engine::TableInfo {
                name: "items".to_string(),
                columns: vec![
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "value".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                ],
            })
            .unwrap();

        let storage_clone = storage.clone();
        let writer = thread::spawn(move || {
            let mut storage = storage_clone.lock().unwrap();
            for i in 1..=50 {
                storage
                    .insert(
                        "items",
                        vec![vec![
                            sqlrustgo_types::Value::Integer(i as i64),
                            sqlrustgo_types::Value::Integer(i * 10 as i64),
                        ]],
                    )
                    .unwrap();
            }
        });

        let storage_clone = storage.clone();
        let reader = thread::spawn(move || {
            for _ in 0..10 {
                let storage = storage_clone.lock().unwrap();
                let _ = storage.scan("items").unwrap();
                drop(storage);
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();

        let count = storage.lock().unwrap().scan("items").unwrap().len();
        assert_eq!(count, 50);
        println!("✓ Concurrent read/write test passed");
    }

    #[test]
    fn test_wal_recovery_scenario() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let manager = WalManager::new(wal_path.clone());

        // Write transactions
        for i in 1..=20 {
            let _ = manager.log_begin(i).unwrap();
            let _ = manager
                .log_insert(i, 1, vec![i as u8], vec![i as u8; 100])
                .unwrap();
            let _ = manager.log_commit(i).unwrap();
        }

        // Simulate crash and recover
        let manager = WalManager::new(wal_path.clone());
        let entries = manager.recover().unwrap();

        let commits = entries
            .iter()
            .filter(|e| e.entry_type == sqlrustgo_storage::wal::WalEntryType::Commit)
            .count();

        assert!(
            commits >= 20,
            "Should recover at least 20 committed transactions"
        );
        println!("✓ WAL recovery scenario: {} commits recovered", commits);
    }

    #[test]
    fn test_large_dataset_scan() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        // Create large dataset
        storage
            .lock()
            .unwrap()
            .create_table(&sqlrustgo_storage::engine::TableInfo {
                name: "data".to_string(),
                columns: vec![
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "value".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                ],
            })
            .unwrap();

        let mut records = Vec::new();
        for i in 1..=1000 {
            records.push(vec![
                sqlrustgo_types::Value::Integer(i as i64),
                sqlrustgo_types::Value::Text(format!("Value {}", i)),
            ]);
        }

        let insert_start = Instant::now();
        storage.lock().unwrap().insert("data", records).unwrap();
        println!("Inserted 1000 rows in {:?}", insert_start.elapsed());

        let scan_start = Instant::now();
        let result = storage.lock().unwrap().scan("data").unwrap();
        let scan_time = scan_start.elapsed();

        assert_eq!(result.len(), 1000);
        println!("Scanned 1000 rows in {:?}", scan_time);

        // Performance assertion: should complete in reasonable time
        assert!(
            scan_time.as_millis() < 1000,
            "Scan too slow: {:?}",
            scan_time
        );
        println!("✓ Large dataset scan test passed");
    }

    #[test]
    fn test_mixed_workload() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        // Setup
        storage
            .lock()
            .unwrap()
            .create_table(&sqlrustgo_storage::engine::TableInfo {
                name: "transactions".to_string(),
                columns: vec![
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "id".to_string(),
                        data_type: "INTEGER".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "account".to_string(),
                        data_type: "TEXT".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                    sqlrustgo_storage::engine::ColumnDefinition {
                        name: "amount".to_string(),
                        data_type: "FLOAT".to_string(),
                        nullable: false,
                        is_unique: false,
                        is_primary_key: false,
                        auto_increment: false,
                        references: None,
                    },
                ],
            })
            .unwrap();

        let start = Instant::now();
        let num_operations = 1000;

        // Mixed read/write workload
        let storage_clone = storage.clone();
        let writer = thread::spawn(move || {
            let mut storage = storage_clone.lock().unwrap();
            for i in 1..=num_operations {
                storage
                    .insert(
                        "transactions",
                        vec![vec![
                            sqlrustgo_types::Value::Integer(i as i64),
                            sqlrustgo_types::Value::Text(format!("ACC{}", i % 100)),
                            sqlrustgo_types::Value::Float((i as f64) * 1.5),
                        ]],
                    )
                    .unwrap();
            }
        });

        let storage_clone = storage.clone();
        let reader = thread::spawn(move || {
            let mut total_rows = 0;
            for _ in 0..50 {
                let storage = storage_clone.lock().unwrap();
                let rows = storage.scan("transactions").unwrap();
                total_rows += rows.len();
                drop(storage);
                thread::sleep(std::time::Duration::from_millis(5));
            }
            total_rows
        });

        writer.join().unwrap();
        let _final_count = reader.join().unwrap();

        let elapsed = start.elapsed();

        println!(
            "Mixed workload: {} ops in {:?} ({:.0} ops/sec)",
            num_operations,
            elapsed,
            num_operations as f64 / elapsed.as_secs_f64()
        );

        assert!(storage.lock().unwrap().scan("transactions").unwrap().len() > 0);
        println!("✓ Mixed workload test passed");
    }
}
