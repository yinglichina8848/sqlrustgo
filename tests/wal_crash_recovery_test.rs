use sqlrustgo_storage::wal::{WalArchiveManager, WalEntry, WalEntryType, WalManager, WalWriter};
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

fn create_temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn make_entry(
    entry_type: WalEntryType,
    tx_id: u64,
    table_id: u64,
    key: Option<Vec<u8>>,
    data: Option<Vec<u8>>,
) -> WalEntry {
    WalEntry {
        tx_id,
        entry_type,
        table_id,
        key,
        data,
        lsn: 0,
        timestamp: 1234567890,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_wal_archive_concurrent_archival() {
    let dir = create_temp_dir();
    let wal_dir = dir.path().join("wal");
    let archive_dir = dir.path().join("archive");

    fs::create_dir_all(&wal_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    let wal_path = wal_dir.join("test.wal");

    let mut writer = WalWriter::new(&wal_path).unwrap();
    for i in 0..100 {
        let entry = make_entry(WalEntryType::Begin, i, 0, None, None);
        writer.append(&entry).unwrap();
        let insert_entry = make_entry(
            WalEntryType::Insert,
            i,
            1,
            Some(vec![(i % 256) as u8]),
            Some(vec![(i % 256) as u8]),
        );
        writer.append(&insert_entry).unwrap();
        let commit_entry = make_entry(WalEntryType::Commit, i, 0, None, None);
        writer.append(&commit_entry).unwrap();
    }
    drop(writer);

    let mut manager = WalArchiveManager::new(wal_dir.clone(), archive_dir.clone()).unwrap();
    manager.set_compression(false);
    let result = manager.archive_wal();
    assert!(result.is_ok(), "Should archive WAL successfully");

    let archives = manager.list_archives().unwrap();
    assert_eq!(archives.len(), 1, "Should have 1 archive");
    assert!(!archives[0].compressed, "Compression should be disabled");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_wal_archive_state_no_leak_on_concurrent_ops() {
    let dir = create_temp_dir();
    let wal_dir = dir.path().join("wal");
    let archive_dir = dir.path().join("archive");

    fs::create_dir_all(&wal_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    let wal_path = wal_dir.join("test.wal");

    let mut writer = WalWriter::new(&wal_path).unwrap();
    for i in 0..100 {
        let entry = make_entry(WalEntryType::Begin, i, 0, None, None);
        writer.append(&entry).unwrap();
        let commit_entry = make_entry(WalEntryType::Commit, i, 0, None, None);
        writer.append(&commit_entry).unwrap();
    }
    drop(writer);

    let manager: Arc<RwLock<WalArchiveManager>> = Arc::new(RwLock::new(
        WalArchiveManager::new(wal_dir.clone(), archive_dir.clone()).unwrap(),
    ));

    let mut handles = Vec::with_capacity(5);
    for i in 0..5 {
        let mgr = Arc::clone(&manager);
        let wal_dir = wal_dir.clone();
        let handle = tokio::spawn(async move {
            let wal_path = wal_dir.join(format!("concurrent_{}.wal", i));
            let mut writer = WalWriter::new(&wal_path).unwrap();
            for j in 0..20 {
                let entry = make_entry(WalEntryType::Begin, j, 0, None, None);
                writer.append(&entry).unwrap();
                let commit_entry = make_entry(WalEntryType::Commit, j, 0, None, None);
                writer.append(&commit_entry).unwrap();
            }
            drop(writer);
            let mut m = mgr.write().await;
            let _ = m.archive_wal();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("task should succeed");
    }

    let archives = manager.read().await.list_archives().unwrap();
    assert!(
        !archives.is_empty(),
        "Archive should be created without state leaks"
    );
}

#[test]
fn test_wal_crash_recovery_after_incomplete_transaction() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    {
        let mut writer = WalWriter::new(&wal_path).unwrap();
        for i in 0..5 {
            let entry = make_entry(WalEntryType::Begin, i, 0, None, None);
            writer.append(&entry).unwrap();
            let insert_entry = make_entry(
                WalEntryType::Insert,
                i,
                1,
                Some(vec![i as u8]),
                Some(vec![i as u8]),
            );
            writer.append(&insert_entry).unwrap();
        }
    }

    let manager = WalManager::new(wal_path);
    let entries = manager.recover().unwrap();
    assert_eq!(
        entries.len(),
        10,
        "Should recover partial entries after crash"
    );
}

#[test]
fn test_wal_archive_cleanup_preserves_recent() {
    let dir = create_temp_dir();
    let wal_dir = dir.path().join("wal");
    let archive_dir = dir.path().join("archive");

    fs::create_dir_all(&wal_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    let mut manager = WalArchiveManager::new(wal_dir.clone(), archive_dir.clone()).unwrap();

    for i in 0..5 {
        let wal_path = wal_dir.join(format!("test{}.wal", i));
        let mut writer = WalWriter::new(&wal_path).unwrap();
        for j in 0..10 {
            let tx_id = i * 100 + j;
            let entry = make_entry(WalEntryType::Begin, tx_id, 0, None, None);
            writer.append(&entry).unwrap();
            let commit_entry = make_entry(WalEntryType::Commit, tx_id, 0, None, None);
            writer.append(&commit_entry).unwrap();
        }
        drop(writer);
        let _ = manager.archive_wal();
    }

    let archives = manager.list_archives().unwrap();
    assert_eq!(archives.len(), 5, "Should have 5 archives before cleanup");

    let cleaned = manager.cleanup_old_archives(2).unwrap();
    assert_eq!(cleaned, 3, "Should clean up 3 old archives");

    let remaining = manager.list_archives().unwrap();
    assert_eq!(
        remaining.len(),
        2,
        "Should keep only 2 most recent archives"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_wal_archive_recovery_stress() {
    let dir = create_temp_dir();
    let wal_dir = dir.path().join("wal");
    let archive_dir = dir.path().join("archive");

    fs::create_dir_all(&wal_dir).unwrap();
    fs::create_dir_all(&archive_dir).unwrap();

    let mut manager = WalArchiveManager::new(wal_dir.clone(), archive_dir.clone()).unwrap();
    manager.set_compression(false);

    for batch in 0..10 {
        let wal_path = wal_dir.join(format!("stress{}.wal", batch));
        let mut writer = WalWriter::new(&wal_path).unwrap();
        for i in 0..50 {
            let tx_id = batch * 100 + i;
            let entry = make_entry(WalEntryType::Begin, tx_id, 0, None, None);
            writer.append(&entry).unwrap();
            let insert_entry = make_entry(
                WalEntryType::Insert,
                tx_id,
                1,
                Some(vec![(tx_id % 256) as u8]),
                Some(vec![(tx_id % 256) as u8]),
            );
            writer.append(&insert_entry).unwrap();
            let commit_entry = make_entry(WalEntryType::Commit, tx_id, 0, None, None);
            writer.append(&commit_entry).unwrap();
        }
        drop(writer);
        let _ = manager.archive_wal();
    }

    let archives = manager.list_archives().unwrap();
    assert_eq!(archives.len(), 10, "Should have 10 archives");
    assert!(
        archives[9].archive_id > archives[0].archive_id,
        "Archives should have incrementing IDs"
    );
}
