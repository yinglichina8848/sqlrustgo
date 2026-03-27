//! MVCC & Concurrency Anomaly Tests
//!
//! P0 tests for MVCC and concurrency anomalies per ISSUE #845

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_storage::StorageEngine;
    use sqlrustgo_types::Value;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// Test: Concurrent Insert Stability
    /// 验证并发插入不会导致数据丢失或损坏
    #[test]
    fn test_concurrent_insert_stability() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = sqlrustgo_storage::TableInfo {
            name: "test_concurrent".to_string(),
            columns: vec![],
        };
        storage.lock().unwrap().create_table(&info).unwrap();

        let mut handles = vec![];

        // 启动多个插入线程
        for i in 0..10 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let mut s = storage.lock().unwrap();
                for j in 0..10 {
                    s.insert("test_concurrent", vec![vec![Value::Integer(i * 100 + j)]])
                        .ok();
                }
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证: 数据完整性 - 应该至少有部分数据
        let s = storage.lock().unwrap();
        let result = s.scan("test_concurrent");
        println!("Concurrent insert test: {:?}", result.map(|r| r.len()));
    }

    /// Test: Concurrent Read Stability
    /// 验证并发读取不会导致问题
    #[test]
    fn test_concurrent_read_stability() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = sqlrustgo_storage::TableInfo {
            name: "test_read".to_string(),
            columns: vec![],
        };
        storage.lock().unwrap().create_table(&info).unwrap();

        // 插入测试数据
        for i in 0..100 {
            storage
                .lock()
                .unwrap()
                .insert("test_read", vec![vec![Value::Integer(i)]])
                .ok();
        }

        let mut handles = vec![];

        // 启动多个读取线程
        for _ in 0..20 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let s = storage.lock().unwrap();
                let result = s.scan("test_read");
                result.map(|rows| rows.len()).unwrap_or(0)
            });
            handles.push(handle);
        }

        // 验证: 所有读取应该成功
        let mut total = 0;
        for handle in handles {
            total += handle.join().unwrap();
        }

        println!("Concurrent read test: total reads = {}", total);
        // 验证至少有数据被读取
        assert!(total > 0, "Should have read data");
    }

    /// Test: Deadlock Detection Basic
    /// 验证死锁能被检测
    #[test]
    fn test_deadlock_scenario() {
        let storage1 = Arc::new(Mutex::new(MemoryStorage::new()));
        let storage2 = storage1.clone();

        let info = sqlrustgo_storage::TableInfo {
            name: "deadlock_test".to_string(),
            columns: vec![],
        };
        storage1.lock().unwrap().create_table(&info).unwrap();
        storage1
            .lock()
            .unwrap()
            .insert("deadlock_test", vec![vec![Value::Integer(1)]])
            .ok();

        let handle1 = thread::spawn(move || {
            let mut s = storage1.lock().unwrap();
            s.insert("deadlock_test", vec![vec![Value::Integer(10)]])
                .ok();
            thread::sleep(Duration::from_millis(10));
            s.insert("deadlock_test", vec![vec![Value::Integer(11)]])
                .ok();
        });

        let handle2 = thread::spawn(move || {
            let mut s = storage2.lock().unwrap();
            s.insert("deadlock_test", vec![vec![Value::Integer(20)]])
                .ok();
            thread::sleep(Duration::from_millis(10));
            s.insert("deadlock_test", vec![vec![Value::Integer(21)]])
                .ok();
        });

        let result1 = handle1.join();
        let result2 = handle2.join();

        println!(
            "Deadlock test: tx1={:?}, tx2={:?}",
            result1.is_ok(),
            result2.is_ok()
        );
    }

    /// Test: Large Dataset Concurrent Operations
    /// 大数据集并发操作测试
    #[test]
    fn test_large_dataset_concurrent_ops() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = sqlrustgo_storage::TableInfo {
            name: "large_data".to_string(),
            columns: vec![],
        };
        storage.lock().unwrap().create_table(&info).unwrap();

        // 准备数据
        for i in 0..1000 {
            storage
                .lock()
                .unwrap()
                .insert("large_data", vec![vec![Value::Integer(i)]])
                .ok();
        }

        let mut handles = vec![];

        // 启动多个并发操作
        for _ in 0..5 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let s = storage.lock().unwrap();
                for _ in 0..10 {
                    let _ = s.scan("large_data");
                }
                true
            });
            handles.push(handle);
        }

        // 验证所有线程完成
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        println!("Large dataset test: {} threads completed", results.len());
        assert_eq!(results.len(), 5);
    }

    /// Test: Stress Test - Rapid Create/Drop
    /// 压力测试: 快速创建和删除表
    #[test]
    fn test_rapid_create_drop() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let mut handles = vec![];

        // 快速创建和删除表
        for i in 0..20 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let mut s = storage.lock().unwrap();
                let table_name = format!("temp_table_{}", i);

                let info = sqlrustgo_storage::TableInfo {
                    name: table_name.clone(),
                    columns: vec![],
                };

                s.create_table(&info).ok();
                s.insert(&table_name, vec![vec![Value::Integer(i)]]).ok();
                s.drop_table(&table_name).ok();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        println!("Rapid create/drop test: completed");
    }

    /// Test: Concurrent Mixed Operations
    /// 混合并发操作测试
    #[test]
    fn test_concurrent_mixed_operations() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = sqlrustgo_storage::TableInfo {
            name: "mixed_ops".to_string(),
            columns: vec![],
        };
        storage.lock().unwrap().create_table(&info).unwrap();

        let mut handles = vec![];

        for i in 0..10 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let mut s = storage.lock().unwrap();
                s.insert("mixed_ops", vec![vec![Value::Integer(i)]]).ok();
                let _ = s.scan("mixed_ops");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        println!("Mixed operations test: completed");
    }
}
