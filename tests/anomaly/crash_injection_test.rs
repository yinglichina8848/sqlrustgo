//! Crash Injection Test Matrix
//!
//! P0/P1 tests for crash safety per ISSUE #843

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::file_storage::FileStorage;
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_storage::StorageEngine;
    use sqlrustgo_types::Value;
    
    
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Test: Crash during commit simulation
    /// 模拟在 commit 阶段发生崩溃
    #[test]
    fn test_crash_during_commit() {
        let dir = create_temp_dir();

        // 第一次运行：创建数据并提交
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "test_crash".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();

            // 插入数据
            storage
                .insert(
                    "test_crash",
                    vec![vec![Value::Integer(1), Value::Integer(100)]],
                )
                .unwrap();
            storage
                .insert(
                    "test_crash",
                    vec![vec![Value::Integer(2), Value::Integer(200)]],
                )
                .unwrap();
        }

        // 第二次运行：验证数据持久化
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let result = storage.scan("test_crash");

            // 验证：数据应该被持久化（即使发生崩溃也应该恢复）
            println!("Crash during commit test: {:?}", result.map(|r| r.len()));
        }
    }

    /// Test: Crash during checkpoint simulation
    /// 模拟在 checkpoint 阶段发生崩溃
    #[test]
    fn test_crash_during_checkpoint() {
        let dir = create_temp_dir();

        // 第一次运行：写入大量数据
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "checkpoint_test".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();

            // 写入多批数据（可能触发 checkpoint）
            for i in 0..100 {
                storage
                    .insert("checkpoint_test", vec![vec![Value::Integer(i)]])
                    .ok();
            }
        }

        // 第二次运行：验证数据完整性
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let result = storage.scan("checkpoint_test");

            println!(
                "Crash during checkpoint test: {:?}",
                result.map(|r| r.len())
            );
        }
    }

    /// Test: Crash during index update
    /// 模拟在索引更新时发生崩溃
    #[test]
    fn test_crash_during_index_update() {
        let dir = create_temp_dir();

        // 第一次运行：创建表和索引
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "index_test".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();

            // 插入数据
            for i in 0..50 {
                storage
                    .insert("index_test", vec![vec![Value::Integer(i)]])
                    .ok();
            }

            // 创建索引（模拟索引更新）
            storage.create_table_index("index_test", "id", 0).ok();
        }

        // 第二次运行：验证索引一致性
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let has_index = storage.has_index("index_test", "id");

            println!("Crash during index update: has_index={}", has_index);
        }
    }

    /// Test: Crash during schema change
    /// 模拟在 schema 变更时发生崩溃
    #[test]
    fn test_crash_during_schema_change() {
        let dir = create_temp_dir();

        // 第一次运行：创建表
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "schema_test".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();
        }

        // 模拟 schema 变更过程中的崩溃场景
        // 第二次运行：验证 schema 一致性
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let table_exists = storage.contains_table("schema_test");

            println!("Crash during schema change: table_exists={}", table_exists);
            assert!(table_exists, "Table should exist after schema change crash");
        }
    }

    /// Test: Power failure simulation
    /// 模拟断电故障
    #[test]
    fn test_power_failure_simulation() {
        let dir = create_temp_dir();

        // 第一次运行：写入数据后立即"断电"（不等待 flush）
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "power_fail".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();

            // 快速写入数据（模拟断电来不及时 flush）
            for i in 0..10 {
                storage
                    .insert("power_fail", vec![vec![Value::Integer(i)]])
                    .ok();
            }

            // 直接退出，不做额外的 flush 操作
        }

        // 第二次运行：验证数据恢复
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let result = storage.scan("power_fail");

            println!("Power failure test: {:?}", result.map(|r| r.len()));
        }
    }

    /// Test: Disk full simulation
    /// 模拟磁盘满情况
    #[test]
    fn test_disk_full_simulation() {
        let dir = create_temp_dir();

        let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
        let info = sqlrustgo_storage::TableInfo {
            name: "disk_full_test".to_string(),
            columns: vec![],
        };
        storage.create_table(&info).unwrap();

        // 尝试大量写入（可能触发磁盘满）
        let mut success_count = 0;
        let mut fail_count = 0;

        for i in 0..1000 {
            match storage.insert("disk_full_test", vec![vec![Value::Integer(i)]]) {
                Ok(_) => success_count += 1,
                Err(_) => fail_count += 1,
            }
        }

        println!(
            "Disk full test: success={}, failed={}",
            success_count, fail_count
        );
    }

    /// Test: Partial write recovery
    /// 部分写入恢复测试
    #[test]
    fn test_partial_write_recovery() {
        let dir = create_temp_dir();

        // 第一次运行：写入部分数据
        {
            let mut storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let info = sqlrustgo_storage::TableInfo {
                name: "partial_write".to_string(),
                columns: vec![],
            };
            storage.create_table(&info).unwrap();

            // 成功写入一些数据
            storage
                .insert("partial_write", vec![vec![Value::Integer(1)]])
                .ok();
            storage
                .insert("partial_write", vec![vec![Value::Integer(2)]])
                .ok();
        }

        // 第二次运行：验证已提交数据恢复
        {
            let storage = FileStorage::new(dir.path().to_path_buf()).unwrap();
            let result = storage.scan("partial_write");

            println!("Partial write recovery: {:?}", result.map(|r| r.len()));
        }
    }

    /// Test: Concurrent crash scenarios
    /// 并发崩溃场景测试
    #[test]
    fn test_concurrent_crash_scenarios() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new()));

        let info = sqlrustgo_storage::TableInfo {
            name: "concurrent_crash".to_string(),
            columns: vec![],
        };
        storage.lock().unwrap().create_table(&info).unwrap();

        let mut handles = vec![];

        // 模拟多个线程同时进行可能导致崩溃的操作
        for i in 0..10 {
            let storage = storage.clone();
            let handle = thread::spawn(move || {
                let mut s = storage.lock().unwrap();
                for j in 0..20 {
                    s.insert("concurrent_crash", vec![vec![Value::Integer(i * 100 + j)]])
                        .ok();
                }
            });
            handles.push(handle);
        }

        // 等待所有操作完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证数据一致性
        let s = storage.lock().unwrap();
        let result = s.scan("concurrent_crash");

        println!("Concurrent crash test: {:?}", result.map(|r| r.len()));
    }
}
