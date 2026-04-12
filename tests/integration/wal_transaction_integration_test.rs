use sqlrustgo_storage::engine::MemoryStorage;
use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager};
use sqlrustgo_storage::{StorageEngine, WalStorage};
use sqlrustgo_types::Value;
use tempfile::TempDir;

fn create_wal_storage() -> (TempDir, WalStorage<MemoryStorage>) {
    let dir = TempDir::new().unwrap();
    let inner = MemoryStorage::new();
    let wal_path = dir.path().join("test.wal");
    let storage = WalStorage::new(inner, wal_path).unwrap();
    (dir, storage)
}

#[test]
fn test_single_transaction_commit() {
    let (_dir, mut storage) = create_wal_storage();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "users",
            vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
        )
        .unwrap();
    storage.commit_transaction().unwrap();

    let entries = storage.recover().unwrap();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), 1);

    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    assert_eq!(inserts.len(), 1);
}

#[test]
fn test_single_transaction_rollback() {
    let (_dir, mut storage) = create_wal_storage();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "users",
            vec![vec![Value::Integer(1), Value::Text("Bob".to_string())]],
        )
        .unwrap();
    storage.rollback_transaction().unwrap();

    let entries = storage.recover().unwrap();
    let rollbacks: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Rollback)
        .collect();
    assert_eq!(rollbacks.len(), 1);

    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    assert_eq!(inserts.len(), 1);
}

#[test]
fn test_multiple_transactions_sequential() {
    let (_dir, mut storage) = create_wal_storage();

    for i in 1..=5 {
        storage.begin_transaction().unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(i), Value::Text(format!("User{}", i))]],
            )
            .unwrap();
        storage.commit_transaction().unwrap();
    }

    let entries = storage.recover().unwrap();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), 5);

    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    assert_eq!(inserts.len(), 5);
}

#[test]
fn test_mixed_operations() {
    let (_dir, mut storage) = create_wal_storage();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "t1",
            vec![vec![Value::Integer(1), Value::Text("A".to_string())]],
        )
        .unwrap();
    storage.commit_transaction().unwrap();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "t1",
            vec![vec![Value::Integer(2), Value::Text("B".to_string())]],
        )
        .unwrap();
    storage.rollback_transaction().unwrap();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "t1",
            vec![vec![Value::Integer(3), Value::Text("C".to_string())]],
        )
        .unwrap();
    storage.commit_transaction().unwrap();

    let entries = storage.recover().unwrap();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    let rollbacks: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Rollback)
        .collect();
    assert_eq!(commits.len(), 2);
    assert_eq!(rollbacks.len(), 1);
}

#[test]
fn test_transaction_isolation() {
    let (_dir, mut storage) = create_wal_storage();

    storage.begin_transaction().unwrap();
    storage
        .insert("t1", vec![vec![Value::Integer(1), Value::Integer(100)]])
        .unwrap();
    storage.commit_transaction().unwrap();

    storage.begin_transaction().unwrap();
    storage
        .insert("t1", vec![vec![Value::Integer(2), Value::Integer(200)]])
        .unwrap();

    let result = storage.begin_transaction();
    assert!(result.is_err());

    storage.commit_transaction().unwrap();
}

#[test]
fn test_concurrent_writers() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("test.wal");

    for i in 1..=20 {
        let inner = MemoryStorage::new();
        let mut storage = WalStorage::new(inner, wal_path.clone()).unwrap();
        storage.begin_transaction().unwrap();
        storage
            .insert("t1", vec![vec![Value::Integer(i), Value::Integer(i * 10)]])
            .unwrap();
        storage.commit_transaction().unwrap();
    }

    let entries = {
        let storage = WalStorage::new(MemoryStorage::new(), wal_path).unwrap();
        storage.recover().unwrap()
    };
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), 20);
}

#[test]
fn test_wal_disabled_mode() {
    let inner = MemoryStorage::new();
    let mut storage = WalStorage::new_without_wal(inner);

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "t1",
            vec![vec![Value::Integer(1), Value::Text("test".to_string())]],
        )
        .unwrap();
    storage.commit_transaction().unwrap();

    let entries = storage.recover().unwrap_or_default();
    assert!(entries.is_empty());
}

#[test]
fn test_wal_entry_types_all_logged() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("test.wal");
    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
    manager.log_update(1, 1, vec![1], vec![100]).unwrap();
    manager.log_delete(1, 1, vec![1]).unwrap();
    manager.log_commit(1).unwrap();

    let entries = manager.recover().unwrap();
    let begins: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Begin)
        .collect();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    let updates: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Update)
        .collect();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();

    assert_eq!(begins.len(), 1);
    assert_eq!(inserts.len(), 1);
    assert_eq!(updates.len(), 1);
    assert_eq!(commits.len(), 1);

    let deletes: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Delete)
        .collect();
    assert_eq!(deletes.len(), 1);
}

#[test]
fn test_wal_with_large_payload() {
    let (_dir, mut storage) = create_wal_storage();

    let large_data = Value::Text("x".repeat(10000));

    storage.begin_transaction().unwrap();
    storage
        .insert("t1", vec![vec![Value::Integer(1), large_data]])
        .unwrap();
    storage.commit_transaction().unwrap();

    let entries = storage.recover().unwrap();
    assert_eq!(entries.len(), 3);

    let insert_entry = entries
        .iter()
        .find(|e| e.entry_type == WalEntryType::Insert)
        .unwrap();
    assert!(insert_entry.data.is_some());
    assert!(insert_entry.data.as_ref().unwrap().len() > 1000);
}

#[test]
fn test_wal_recovery_after_crash_simulation() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("test.wal");

    {
        let mut storage = WalStorage::new(MemoryStorage::new(), wal_path.clone()).unwrap();
        storage.begin_transaction().unwrap();
        storage
            .insert("t1", vec![vec![Value::Integer(1), Value::Integer(100)]])
            .unwrap();
        storage.commit_transaction().unwrap();

        storage.begin_transaction().unwrap();
        storage
            .insert("t1", vec![vec![Value::Integer(2), Value::Integer(200)]])
            .unwrap();
    }

    let entries = {
        let storage = WalStorage::new(MemoryStorage::new(), wal_path).unwrap();
        storage.recover().unwrap()
    };

    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();

    assert_eq!(commits.len(), 1);
    assert_eq!(inserts.len(), 2);
}

#[test]
fn test_wal_performance_bulk_insert() {
    let (_dir, mut storage) = create_wal_storage();

    let start = std::time::Instant::now();
    storage.begin_transaction().unwrap();

    for i in 1..=1000 {
        storage
            .insert("t1", vec![vec![Value::Integer(i), Value::Integer(i * 2)]])
            .unwrap();
    }

    storage.commit_transaction().unwrap();
    let elapsed = start.elapsed();

    let entries = storage.recover().unwrap();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();

    assert_eq!(inserts.len(), 1000);
    assert!(elapsed.as_secs_f64() < 10.0);
}

#[test]
fn test_wal_manager_separate_usage() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("standalone.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![10]).unwrap();
    manager.log_commit(1).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_insert(2, 1, vec![2], vec![20]).unwrap();
    manager.log_rollback(2).unwrap();

    let entries = manager.recover().unwrap();
    assert_eq!(entries.len(), 6);
}

#[test]
fn test_wal_empty_recovery() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("empty.wal");

    std::fs::write(&wal_path, &[]).ok();

    let manager = WalManager::new(wal_path);
    let result = manager.recover();

    match result {
        Ok(entries) => assert!(entries.is_empty()),
        Err(_) => {}
    }
}

#[test]
fn test_wal_entry_serialization_roundtrip() {
    let entry = WalEntry {
        tx_id: 42,
        entry_type: WalEntryType::Insert,
        table_id: 5,
        key: Some(vec![1, 2, 3]),
        data: Some(vec![10, 20, 30]),
        lsn: 100,
        timestamp: 1234567890,
    };

    let bytes = entry.to_bytes();
    let restored = WalEntry::from_bytes(&bytes).unwrap();

    assert_eq!(restored.tx_id, entry.tx_id);
    assert_eq!(restored.entry_type, entry.entry_type);
    assert_eq!(restored.table_id, entry.table_id);
    assert_eq!(restored.key, entry.key);
    assert_eq!(restored.data, entry.data);
}

#[test]
fn test_wal_truncated_entry_handling() {
    let entry = WalEntry {
        tx_id: 1,
        entry_type: WalEntryType::Insert,
        table_id: 1,
        key: Some(vec![1]),
        data: Some(vec![10]),
        lsn: 0,
        timestamp: 0,
    };

    let bytes = entry.to_bytes();
    let truncated = &bytes[..bytes.len() / 2];

    assert!(WalEntry::from_bytes(truncated).is_none());
}

#[test]
fn test_wal_multiple_tables() {
    let (_dir, mut storage) = create_wal_storage();

    storage.begin_transaction().unwrap();
    storage
        .insert(
            "users",
            vec![vec![Value::Integer(1), Value::Text("U1".to_string())]],
        )
        .unwrap();
    storage
        .insert("orders", vec![vec![Value::Integer(1), Value::Integer(100)]])
        .unwrap();
    storage.commit_transaction().unwrap();

    let entries = storage.recover().unwrap();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    assert_eq!(inserts.len(), 2);
}

#[test]
fn test_wal_stress_many_small_transactions() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("stress.wal");
    let manager = WalManager::new(wal_path);

    let count = 500usize;
    for i in 1..=count {
        manager.log_begin(i as u64).unwrap();
        manager
            .log_insert(i as u64, 1, vec![i as u8], vec![i as u8])
            .unwrap();
        manager.log_commit(i as u64).unwrap();
    }

    let entries = manager.recover().unwrap();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), count);
}

#[test]
fn test_wal_stress_large_transaction() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("stress.wal");
    let manager = WalManager::new(wal_path);

    let tx_id = 1;
    manager.log_begin(tx_id).unwrap();

    let count = 10000usize;
    for i in 0..count {
        manager
            .log_insert(tx_id, 1, vec![i as u8], vec![i as u8; 100])
            .unwrap();
    }

    manager.log_commit(tx_id).unwrap();

    let entries = manager.recover().unwrap();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    assert_eq!(inserts.len(), count);
}

#[test]
fn test_wal_rapid_commit_rollback_cycles() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("stress.wal");
    let manager = WalManager::new(wal_path);

    for i in 1..=100u64 {
        manager.log_begin(i).unwrap();
        manager
            .log_insert(i, 1, vec![i as u8], vec![i as u8])
            .unwrap();
        if i % 2 == 0 {
            manager.log_commit(i).unwrap();
        } else {
            manager.log_rollback(i).unwrap();
        }
    }

    let entries = manager.recover().unwrap();
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    let rollbacks: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Rollback)
        .collect();
    assert_eq!(commits.len(), 50);
    assert_eq!(rollbacks.len(), 50);
}

#[test]
fn test_wal_max_payload() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("stress.wal");
    let manager = WalManager::new(wal_path);

    let tx_id = 1;
    manager.log_begin(tx_id).unwrap();

    let large_data = vec![0x42u8; 1_000_000];
    manager.log_insert(tx_id, 1, vec![1], large_data).unwrap();

    manager.log_commit(tx_id).unwrap();

    let entries = manager.recover().unwrap();
    let insert_entry = entries
        .iter()
        .find(|e| e.entry_type == WalEntryType::Insert)
        .unwrap();
    assert_eq!(insert_entry.data.as_ref().unwrap().len(), 1_000_000);
}

#[test]
fn test_wal_lsn_monotonically_increasing() {
    use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalWriter};

    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("stress.wal");

    let mut writer = WalWriter::new(&wal_path).unwrap();

    let tx_id = 1;
    let mut last_lsn = 0u64;
    for i in 0..100 {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Insert,
            table_id: 1,
            key: Some(vec![i as u8]),
            data: Some(vec![i as u8]),
            lsn: last_lsn + 1,
            timestamp: 0,
        };
        let lsn = writer.append(&entry).unwrap();
        assert!(lsn >= last_lsn);
        last_lsn = lsn;
    }
}

#[test]
fn test_wal_recovery_performance() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("perf.wal");
    let manager = WalManager::new(wal_path.clone());

    let count = 5000usize;
    for i in 1..=count {
        manager.log_begin(i as u64).unwrap();
        manager
            .log_insert(i as u64, 1, vec![i as u8], vec![i as u8; 100])
            .unwrap();
        manager.log_commit(i as u64).unwrap();
    }

    let start = std::time::Instant::now();
    let entries = manager.recover().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(entries.len(), count * 3);
    assert!(elapsed.as_secs_f64() < 5.0);
}

#[test]
fn test_wal_mixed_transaction_sizes() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("mixed.wal");
    let manager = WalManager::new(wal_path);

    for i in 1..=10u64 {
        manager.log_begin(i).unwrap();
        let op_count = (i * 100) as usize;
        for j in 0..op_count {
            manager
                .log_insert(i, 1, vec![j as u8], vec![j as u8; 10])
                .unwrap();
        }
        manager.log_commit(i).unwrap();
    }

    let entries = manager.recover().unwrap();
    let inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Insert)
        .collect();
    let expected: usize = (1..=10).map(|i| i * 100).sum();
    assert_eq!(inserts.len(), expected);
}
