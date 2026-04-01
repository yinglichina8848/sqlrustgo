use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager, WalReader, WalWriter};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

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

#[test]
fn test_wal_single_transaction() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_commit(1).unwrap();

    let entries = manager.recover().unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn test_wal_multiple_transactions() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_commit(1).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_insert(2, 1, vec![2], vec![200]).unwrap();
    manager.log_commit(2).unwrap();

    let entries = manager.recover().unwrap();
    assert_eq!(entries.len(), 6);
}

#[test]
fn test_wal_writer_reader_sequential() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let mut writer = WalWriter::new(&wal_path).unwrap();

    let entry = make_entry(WalEntryType::Begin, 1, 0, None, None);
    writer.append(&entry).unwrap();

    drop(writer);

    let manager = WalManager::new(wal_path);
    let entries = manager.recover().unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn test_wal_entry_serialization_roundtrip() {
    let entry = make_entry(
        WalEntryType::Insert,
        1,
        1,
        Some(vec![1, 2, 3]),
        Some(vec![10, 20, 30]),
    );
    let bytes = entry.to_bytes();
    let restored = WalEntry::from_bytes(&bytes).unwrap();

    assert_eq!(restored.entry_type, entry.entry_type);
    assert_eq!(restored.tx_id, entry.tx_id);
    assert_eq!(restored.table_id, entry.table_id);
    assert_eq!(restored.key, entry.key);
    assert_eq!(restored.data, entry.data);
}

#[test]
fn test_wal_entry_with_null_data() {
    let entry = make_entry(WalEntryType::Insert, 1, 1, None, None);
    let bytes = entry.to_bytes();
    let restored = WalEntry::from_bytes(&bytes).unwrap();

    assert_eq!(restored.entry_type, entry.entry_type);
    assert!(restored.key.is_none());
    assert!(restored.data.is_none());
}

#[test]
fn test_wal_entry_large_payload() {
    let large_key: Vec<u8> = (0..100u8).collect();
    let large_value: Vec<u8> = (0..200u8).collect();

    let entry = make_entry(
        WalEntryType::Insert,
        1,
        1,
        Some(large_key.clone()),
        Some(large_value.clone()),
    );
    let bytes = entry.to_bytes();
    let restored = WalEntry::from_bytes(&bytes).unwrap();

    assert_eq!(restored.key, Some(large_key));
    assert_eq!(restored.data, Some(large_value));
}

#[test]
fn test_wal_checkpoint_recovery() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.checkpoint(1).unwrap();
    manager.log_commit(1).unwrap();

    let entries = manager.recover().unwrap();
    let checkpoints: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Checkpoint)
        .collect();
    assert_eq!(checkpoints.len(), 1);
}

#[test]
fn test_wal_rollback_recovery() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_rollback(1).unwrap();

    let entries = manager.recover().unwrap();
    let rollbacks: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Rollback)
        .collect();
    assert_eq!(rollbacks.len(), 1);
}

#[test]
fn test_wal_recovery_after_crash() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_commit(1).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_insert(2, 1, vec![2], vec![200]).unwrap();
    let entries = manager.recover().unwrap();
    assert_eq!(entries.len(), 5);
}

#[test]
fn test_wal_recovery_with_pending_transaction() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_commit(1).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_insert(2, 1, vec![2], vec![200]).unwrap();
    let entries = manager.recover().unwrap();
    assert!(entries.len() >= 4);
}

#[test]
fn test_wal_concurrent_transactions_isolation() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_insert(2, 1, vec![2], vec![200]).unwrap();

    manager.log_commit(1).unwrap();
    manager.log_commit(2).unwrap();

    let entries = manager.recover().unwrap();
    assert!(
        entries.len() >= 6,
        "Expected at least 6 entries, got {}",
        entries.len()
    );
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), 2, "Expected 2 commits");
}

#[test]
fn test_wal_mixed_operations() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let manager = WalManager::new(wal_path);

    manager.log_begin(1).unwrap();
    manager.log_insert(1, 1, vec![1], vec![100]).unwrap();
    manager.log_update(1, 1, vec![1], vec![150]).unwrap();
    manager.log_commit(1).unwrap();

    manager.log_begin(2).unwrap();
    manager.log_delete(2, 1, vec![2]).unwrap();
    manager.log_commit(2).unwrap();

    let entries = manager.recover().unwrap();
    assert!(
        entries.len() >= 6,
        "Expected at least 6 entries, got {}",
        entries.len()
    );
    let commits = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .count();
    assert_eq!(commits, 2, "Expected 2 commits");
}

#[test]
fn test_wal_archive_metadata_roundtrip() {
    let dir = create_temp_dir();
    let archive_path = dir.path().join("archive.dat");

    let metadata = vec![
        ("file1.wal".to_string(), 1024u64),
        ("file2.wal".to_string(), 2048u64),
    ];
    let bytes = metadata.iter().fold(Vec::new(), |mut acc, (name, size)| {
        acc.extend_from_slice(name.as_bytes());
        acc.extend_from_slice(&size.to_le_bytes());
        acc
    });

    fs::write(&archive_path, &bytes).unwrap();
    let read_bytes = fs::read(&archive_path).unwrap();

    assert_eq!(read_bytes.len(), bytes.len());
}

#[test]
fn test_wal_archive_metadata_empty_file() {
    let dir = create_temp_dir();
    let archive_path = dir.path().join("empty_archive.dat");

    fs::write(&archive_path, &[]).unwrap();
    let read_bytes = fs::read(&archive_path).unwrap();
    assert!(read_bytes.is_empty());
}

#[test]
fn test_wal_writer_lsn_tracking() {
    let dir = create_temp_dir();
    let wal_path = dir.path().join("test.wal");

    let mut writer = WalWriter::new(&wal_path).unwrap();

    let lsn1 = writer.current_lsn();
    assert_eq!(lsn1, 0);

    let entry = make_entry(WalEntryType::Begin, 1, 0, None, None);
    writer.append(&entry).unwrap();

    let lsn2 = writer.current_lsn();
    assert!(lsn2 > lsn1);
}

#[test]
fn test_wal_reader_empty_file() {
    let dir = create_temp_dir();
    let empty_path = dir.path().join("empty.wal");

    std::fs::write(&empty_path, &[]).unwrap();

    let manager = WalManager::new(empty_path);
    let entries = manager.recover().unwrap();
    assert!(entries.is_empty());
}
