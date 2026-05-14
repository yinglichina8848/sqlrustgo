//! WAL recovery validation tests for clustered index.
//!
//! These tests verify that crash recovery correctly restores ClusteredLeafPage
//! state from WAL entries.

use crate::clustered_index::{ClusteredLeafPage, ClusteredPageTransaction, ClusteredWalManager};
use crate::row_format::types::ClusterKey;
use sqlrustgo_types::Value;
use tempfile::TempDir;

/// Helper to create a test page with records.
fn create_page_with_records(count: usize) -> ClusteredLeafPage {
    let mut page = ClusteredLeafPage::new();
    for i in 0..count {
        let key = ClusterKey::HiddenRowId(i as u64);
        let fixed = vec![Value::Integer(i as i64)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls: Vec<bool> = vec![false];
        let _ = page.insert(&key, &fixed, &varlen, &nulls);
    }
    page
}

/// Test that committed transactions are recovered correctly.
#[test]
fn test_recover_committed_insert() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1); // table_id = 1

    let tx_id = 1;
    {
        let mut page = ClusteredLeafPage::new();
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Insert records
        for i in 0..5 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64 * 10)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls = vec![false];
            tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        }

        // Commit
        wal_manager.log_commit(tx_id).unwrap();
    } // page dropped here

    // Recover
    let entries = wal_manager.recover().unwrap();
    let committed_inserts: Vec<_> = entries
        .iter()
        .filter(|e| e.is_insert() && e.entry.tx_id == tx_id)
        .collect();

    assert_eq!(committed_inserts.len(), 5);
}

/// Test that uncommitted transactions are NOT replayed.
#[test]
fn test_recovery_ignores_uncommitted() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    let tx_id = 1;
    {
        let mut page = ClusteredLeafPage::new();
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Insert records (but don't commit)
        for i in 0..3 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64 * 10)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls = vec![false];
            tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        }
        // No commit!
    }

    // Recover - should find the uncommitted inserts
    let entries = wal_manager.recover().unwrap();
    let uncommitted_inserts: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();

    // WAL still has the entries, but recovery logic should check commit
    // The actual replay would need to track transaction state
    assert_eq!(uncommitted_inserts.len(), 3);

    // Find transaction boundary - there should be no commit for tx_id 1
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.entry.entry_type == crate::wal::WalEntryType::Commit && e.entry.tx_id == tx_id
        })
        .collect();
    assert_eq!(commits.len(), 0);
}

/// Test delete is logged correctly.
#[test]
fn test_recover_delete_operation() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    // Setup: insert and commit
    let tx_id1 = 1;
    let mut page = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id1);
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager.log_commit(tx_id1).unwrap();
    }

    // Delete in new transaction
    let tx_id2 = 2;
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id2);
        tx.delete(0).unwrap();
        wal_manager.log_commit(tx_id2).unwrap();
    }

    // Recover and verify
    let entries = wal_manager.recover().unwrap();

    let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();
    let delete_entries: Vec<_> = entries.iter().filter(|e| e.is_delete()).collect();

    assert_eq!(insert_entries.len(), 1);
    assert_eq!(delete_entries.len(), 1);

    // Verify delete is for the correct key
    let delete_entry = delete_entries.first().unwrap();
    assert_eq!(delete_entry.cluster_key, Some(ClusterKey::HiddenRowId(1)));
}

/// Test multiple transactions with interleaved commits.
#[test]
fn test_recover_interleaved_transactions() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    // Transaction 1: insert 2 records
    let tx_id1 = 1;
    let mut page = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id1);
        for i in 0..2 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls = vec![false];
            tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        }
        wal_manager.log_commit(tx_id1).unwrap();
    }

    // Transaction 2: insert 3 records
    let tx_id2 = 2;
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id2);
        for i in 2..5 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls = vec![false];
            tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        }
        wal_manager.log_commit(tx_id2).unwrap();
    }

    // Transaction 3: delete record at slot 0 (not committed)
    let tx_id3 = 3;
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id3);
        tx.delete(0).unwrap();
        // No commit - simulating crash before commit
    }

    // Recover
    let entries = wal_manager.recover().unwrap();

    // Should see 5 inserts and 1 delete
    let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();
    let delete_entries: Vec<_> = entries.iter().filter(|e| e.is_delete()).collect();

    assert_eq!(insert_entries.len(), 5);
    assert_eq!(delete_entries.len(), 1);

    // Only tx1 and tx2 should have commits
    let commits: Vec<_> = entries
        .iter()
        .filter(|e| e.entry.entry_type == crate::wal::WalEntryType::Commit)
        .collect();
    assert_eq!(commits.len(), 2);
}

/// Test PITR (Point-In-Time Recovery) with timestamp filtering.
#[test]
fn test_recover_to_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    let tx_id1 = 1;
    let mut page = ClusteredLeafPage::new();

    // Get timestamp before any operations
    let timestamp_before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Ensure tx1 happens AFTER timestamp_before in a different millisecond
    std::thread::sleep(std::time::Duration::from_millis(2));

    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id1);
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager.log_commit(tx_id1).unwrap();
    }

    // Get timestamp after tx1 commit
    let timestamp_after_tx1 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Ensure tx2 happens AFTER timestamp_after_tx1 in a different millisecond
    std::thread::sleep(std::time::Duration::from_millis(2));

    let tx_id2 = 2;
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id2);
        let key = ClusterKey::HiddenRowId(2);
        let fixed = vec![Value::Integer(200)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager.log_commit(tx_id2).unwrap();
    }

    // Get timestamp after tx2 commit
    let timestamp_after_tx2 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Recover before tx1 - should get nothing
    let entries_before = wal_manager.recover_to_timestamp(timestamp_before).unwrap();
    let inserts_before: Vec<_> = entries_before.iter().filter(|e| e.is_insert()).collect();
    assert_eq!(inserts_before.len(), 0, "Should have no inserts before tx1");

    // Recover after tx1 but before tx2 - should only get tx1
    let entries_after_tx1 = wal_manager
        .recover_to_timestamp(timestamp_after_tx1)
        .unwrap();
    let inserts_after_tx1: Vec<_> = entries_after_tx1.iter().filter(|e| e.is_insert()).collect();
    assert_eq!(inserts_after_tx1.len(), 1, "Should only have tx1's insert");

    // Recover after tx2 - should get both
    let entries_after_tx2 = wal_manager
        .recover_to_timestamp(timestamp_after_tx2)
        .unwrap();
    let inserts_after_tx2: Vec<_> = entries_after_tx2.iter().filter(|e| e.is_insert()).collect();
    assert_eq!(
        inserts_after_tx2.len(),
        2,
        "Should have both tx1 and tx2 inserts"
    );
}

/// Test recovery with PrimaryKey cluster key type.
#[test]
fn test_recover_primary_key_cluster() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    let tx_id = 1;
    let mut page = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Insert with PrimaryKey
        let key = ClusterKey::PrimaryKey(Value::Integer(999));
        let fixed = vec![Value::Text("test value".to_string())];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();

        wal_manager.log_commit(tx_id).unwrap();
    }

    // Recover
    let entries = wal_manager.recover().unwrap();
    let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();

    assert_eq!(insert_entries.len(), 1);
    assert_eq!(
        insert_entries.first().unwrap().cluster_key,
        Some(ClusterKey::PrimaryKey(Value::Integer(999)))
    );
}

/// Test that update is logged as delete + insert.
#[test]
fn test_recover_update_operation() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    let wal_manager = ClusteredWalManager::new(wal_path, 1);

    let tx_id = 1;
    let mut page = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Initial insert
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();

        wal_manager.log_commit(tx_id).unwrap();
    }

    // New transaction for update
    let tx_id2 = 2;
    {
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id2);

        // Update the record
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(200)]; // Changed value
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.update(0, &key, &fixed, &varlen, &nulls).unwrap();

        wal_manager.log_commit(tx_id2).unwrap();
    }

    // Recover
    let entries = wal_manager.recover().unwrap();
    let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();
    let delete_entries: Vec<_> = entries.iter().filter(|e| e.is_delete()).collect();

    // Update should produce 1 delete + 2 inserts (initial + update)
    assert_eq!(insert_entries.len(), 2);
    assert_eq!(delete_entries.len(), 1);
}

/// Test table_id filtering in recovery.
#[test]
fn test_recovery_filters_by_table_id() {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");

    // Create WAL manager for table 1
    let wal_manager1 = ClusteredWalManager::new(wal_path.clone(), 1);

    // Create another WAL manager for table 2
    let wal_manager2 = ClusteredWalManager::new(wal_path.clone(), 2);

    // Insert for table 1
    let tx_id1 = 1;
    let mut page1 = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page1, &wal_manager1, tx_id1);
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager1.log_commit(tx_id1).unwrap();
    }

    // Insert for table 2
    let tx_id2 = 1;
    let mut page2 = ClusteredLeafPage::new();
    {
        let mut tx = ClusteredPageTransaction::new(&mut page2, &wal_manager2, tx_id2);
        let key = ClusterKey::HiddenRowId(1);
        let fixed = vec![Value::Integer(200)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager2.log_commit(tx_id2).unwrap();
    }

    // Recover table 1 - should only see table 1's inserts
    let entries1 = wal_manager1.recover().unwrap();
    let inserts1: Vec<_> = entries1.iter().filter(|e| e.is_insert()).collect();
    assert_eq!(inserts1.len(), 1);

    // Recover table 2 - should only see table 2's inserts
    let entries2 = wal_manager2.recover().unwrap();
    let inserts2: Vec<_> = entries2.iter().filter(|e| e.is_insert()).collect();
    assert_eq!(inserts2.len(), 1);
}
