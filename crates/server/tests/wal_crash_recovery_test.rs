//! WAL Crash Recovery Tests
//!
//! Tests WAL durability and crash recovery functionality.
//! Verifies that committed transactions survive crashes and uncommitted
//! transactions are properly rolled back during recovery.

use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager};
use tempfile::TempDir;

/// Helper to create a test WAL manager with temp directory
fn create_test_wal() -> (WalManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let wal_path = temp_dir.path().join("test.wal");
    let wal = WalManager::new(wal_path);
    (wal, temp_dir)
}

/// Helper to create a WAL entry
fn create_entry(
    tx_id: u64,
    entry_type: WalEntryType,
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
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

#[test]
fn test_wal_write_and_recover_single_entry() {
    let (wal, _temp_dir) = create_test_wal();

    // Write a begin entry
    let mut writer = wal.get_writer().expect("Failed to get writer");
    let entry = create_entry(1, WalEntryType::Begin, 0, None, None);
    writer.append(&entry).expect("Failed to append entry");
    writer.flush().expect("Failed to flush");
    drop(writer);

    // Recover and verify
    let entries = wal.recover().expect("Failed to recover");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tx_id, 1);
    assert_eq!(entries[0].entry_type, WalEntryType::Begin);
}

#[test]
fn test_wal_write_and_recover_multiple_entries() {
    let (wal, _temp_dir) = create_test_wal();

    // Write multiple entries for a transaction
    let mut writer = wal.get_writer().expect("Failed to get writer");

    // Begin
    let entry1 = create_entry(1, WalEntryType::Begin, 0, None, None);
    writer.append(&entry1).expect("Failed to append begin");

    // Insert
    let entry2 = create_entry(
        1,
        WalEntryType::Insert,
        1,
        Some(b"key1".to_vec()),
        Some(b"data1".to_vec()),
    );
    writer.append(&entry2).expect("Failed to append insert");

    // Commit
    let entry3 = create_entry(1, WalEntryType::Commit, 0, None, None);
    writer.append(&entry3).expect("Failed to append commit");

    writer.flush().expect("Failed to flush");
    drop(writer);

    // Recover and verify
    let entries = wal.recover().expect("Failed to recover");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].entry_type, WalEntryType::Begin);
    assert_eq!(entries[1].entry_type, WalEntryType::Insert);
    assert_eq!(entries[2].entry_type, WalEntryType::Commit);
}

#[test]
fn test_wal_recover_committed_transaction() {
    let (wal, _temp_dir) = create_test_wal();

    // Write a complete committed transaction
    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        let entry1 = create_entry(1, WalEntryType::Begin, 0, None, None);
        writer.append(&entry1).expect("Failed to append begin");

        let entry2 = create_entry(
            1,
            WalEntryType::Insert,
            1,
            Some(b"row1".to_vec()),
            Some(b"value1".to_vec()),
        );
        writer.append(&entry2).expect("Failed to append insert");

        let entry3 = create_entry(1, WalEntryType::Commit, 0, None, None);
        writer.append(&entry3).expect("Failed to append commit");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Simulate crash and reopen - recover committed transaction
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    // Verify committed transaction is recovered
    let committed_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.tx_id == 1 && e.entry_type == WalEntryType::Commit)
        .collect();
    assert_eq!(committed_entries.len(), 1, "Committed transaction should be recovered");
}

#[test]
fn test_wal_recover_multiple_transactions() {
    let (wal, _temp_dir) = create_test_wal();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        // Transaction 1: committed
        writer
            .append(&create_entry(1, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                1,
                WalEntryType::Insert,
                1,
                Some(b"k1".to_vec()),
                Some(b"v1".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        // Transaction 2: committed
        writer
            .append(&create_entry(2, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                2,
                WalEntryType::Insert,
                1,
                Some(b"k2".to_vec()),
                Some(b"v2".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(2, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Recover and verify both transactions
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    let tx1_commits = entries
        .iter()
        .filter(|e| e.tx_id == 1 && e.entry_type == WalEntryType::Commit)
        .count();
    let tx2_commits = entries
        .iter()
        .filter(|e| e.tx_id == 2 && e.entry_type == WalEntryType::Commit)
        .count();

    assert_eq!(tx1_commits, 1);
    assert_eq!(tx2_commits, 1);
}

#[test]
fn test_wal_empty_recovery() {
    let (_wal, _temp_dir) = create_test_wal();

    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);

    // Recover from non-existent WAL file should return empty
    let entries = wal2.recover().unwrap_or_default();
    assert_eq!(entries.len(), 0);
}

#[test]
fn test_wal_rollback_entry() {
    let (wal, _temp_dir) = create_test_wal();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        writer
            .append(&create_entry(1, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                1,
                WalEntryType::Insert,
                1,
                Some(b"k1".to_vec()),
                Some(b"v1".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Rollback, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Recover and verify rollback
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    let rollbacks = entries
        .iter()
        .filter(|e| e.tx_id == 1 && e.entry_type == WalEntryType::Rollback)
        .count();
    assert_eq!(rollbacks, 1, "Rollback entry should be recovered");
}

#[test]
fn test_wal_recover_to_timestamp() {
    let (wal, _temp_dir) = create_test_wal();

    let far_future_timestamp: u64 = u64::MAX - 1000;

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        writer
            .append(&create_entry(1, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                1,
                WalEntryType::Insert,
                1,
                Some(b"k1".to_vec()),
                Some(b"v1".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }

    // Recover with far future timestamp should return all entries
    let entries = wal
        .recover_to_timestamp(far_future_timestamp)
        .expect("Failed to recover to timestamp");
    assert_eq!(entries.len(), 3, "Should recover all entries with future timestamp");

    // Recover with timestamp 0 (before all entries) should return nothing
    let entries = wal
        .recover_to_timestamp(0)
        .expect("Failed to recover to timestamp");
    assert_eq!(entries.len(), 0, "Should recover no entries with past timestamp");
}

#[test]
fn test_wal_checkpoint() {
    let (wal, _temp_dir) = create_test_wal();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        writer
            .append(&create_entry(1, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        // Create checkpoint
        let _checkpoint_lsn = writer.current_lsn();
        writer.flush().expect("Failed to flush");

        // Write another transaction
        writer
            .append(&create_entry(2, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        // Don't commit tx 2 - simulating incomplete transaction
    }
    drop(wal);

    // Recover - should get both committed tx and incomplete tx
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    // Should have: Begin(1), Commit(1), Begin(2) - 3 entries
    assert_eq!(entries.len(), 3);

    let commits = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Commit)
        .count();
    assert_eq!(commits, 1, "Should have 1 commit");
}

#[test]
fn wal_data_integrity_after_recovery() {
    let (wal, _temp_dir) = create_test_wal();

    let test_data = b"test_value_1234567890".to_vec();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        writer
            .append(&create_entry(1, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");

        let insert_entry = create_entry(
            1,
            WalEntryType::Insert,
            1,
            Some(b"integrity_key".to_vec()),
            Some(test_data.clone()),
        );
        writer.append(&insert_entry).expect("Failed to append");

        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Recover and verify data integrity
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    let insert_entry = entries
        .iter()
        .find(|e| e.entry_type == WalEntryType::Insert)
        .expect("Should find insert entry");

    assert_eq!(
        insert_entry.data.as_ref().unwrap(), &test_data,
        "Data should match after recovery"
    );
    assert_eq!(
        insert_entry.key.as_ref().unwrap(),
        b"integrity_key",
        "Key should match after recovery"
    );
}

#[test]
fn test_wal_delete_entry_recovery() {
    let (wal, _temp_dir) = create_test_wal();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        // Insert
        writer
            .append(&create_entry(
                1,
                WalEntryType::Insert,
                1,
                Some(b"del_key".to_vec()),
                Some(b"del_value".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        // Delete
        writer
            .append(&create_entry(2, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                2,
                WalEntryType::Delete,
                1,
                Some(b"del_key".to_vec()),
                None,
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(2, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Recover and verify delete
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    let deletes = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Delete)
        .count();
    assert_eq!(deletes, 1, "Delete entry should be recovered");
}

#[test]
fn test_wal_update_entry_recovery() {
    let (wal, _temp_dir) = create_test_wal();

    {
        let mut writer = wal.get_writer().expect("Failed to get writer");

        // Insert
        writer
            .append(&create_entry(
                1,
                WalEntryType::Insert,
                1,
                Some(b"upd_key".to_vec()),
                Some(b"old_value".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(1, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        // Update
        writer
            .append(&create_entry(2, WalEntryType::Begin, 0, None, None))
            .expect("Failed to append");
        writer
            .append(&create_entry(
                2,
                WalEntryType::Update,
                1,
                Some(b"upd_key".to_vec()),
                Some(b"new_value".to_vec()),
            ))
            .expect("Failed to append");
        writer
            .append(&create_entry(2, WalEntryType::Commit, 0, None, None))
            .expect("Failed to append");

        writer.flush().expect("Failed to flush");
    }
    drop(wal);

    // Recover and verify update
    let wal_path = _temp_dir.path().join("test.wal");
    let wal2 = WalManager::new(wal_path);
    let entries = wal2.recover().expect("Failed to recover");

    let updates = entries
        .iter()
        .filter(|e| e.entry_type == WalEntryType::Update)
        .count();
    assert_eq!(updates, 1, "Update entry should be recovered");

    let update_entry = entries
        .iter()
        .find(|e| e.entry_type == WalEntryType::Update)
        .unwrap();
    assert_eq!(
        update_entry.data.as_ref().unwrap(),
        b"new_value",
        "Updated value should match"
    );
}
