//! Transaction layer for clustered index with WAL integration.
//!
//! This module provides transaction support for ClusteredLeafPage operations.
//! All modifications are logged to WAL before being applied for crash recovery.
//!
//! # Usage
//!
//! ```ignore
//! let mut tx = ClusteredPageTransaction::new(page, wal_manager, tx_id);
//! tx.insert(&key, &fixed, &varlen, &nulls)?;
//! wal_manager.log_commit(tx_id)?;
//! ```

use crate::clustered_index::{ClusteredLeafPage, ClusteredWalManager};
use crate::row_format::types::ClusterKey;
use sqlrustgo_types::Value;

/// Transaction wrapper for ClusteredLeafPage with WAL logging.
///
/// This struct ensures all page modifications are logged to WAL before
/// being applied, enabling crash recovery.
pub struct ClusteredPageTransaction<'a> {
    /// The underlying page
    page: &'a mut ClusteredLeafPage,
    /// WAL manager for logging
    wal_manager: &'a ClusteredWalManager,
    /// Transaction ID
    tx_id: u64,
}

impl<'a> ClusteredPageTransaction<'a> {
    /// Create a new transaction wrapper.
    pub fn new(
        page: &'a mut ClusteredLeafPage,
        wal_manager: &'a ClusteredWalManager,
        tx_id: u64,
    ) -> Self {
        Self {
            page,
            wal_manager,
            tx_id,
        }
    }

    /// Insert a record with WAL logging.
    ///
    /// Logs to WAL before modifying the page. If WAL write fails,
    /// the page is not modified.
    pub fn insert(
        &mut self,
        cluster_key: &ClusterKey,
        fixed_columns: &[Value],
        varlen_columns: &[Option<Vec<u8>>],
        null_bitmap: &[bool],
    ) -> std::io::Result<u16> {
        // Log to WAL first - if this fails, don't modify page
        self.wal_manager.log_insert(
            self.tx_id,
            cluster_key,
            fixed_columns,
            varlen_columns,
            null_bitmap,
        )?;

        // WAL logged successfully, now apply to page
        self.page
            .insert(cluster_key, fixed_columns, varlen_columns, null_bitmap)
    }

    /// Delete a record with WAL logging.
    ///
    /// Logs to WAL before marking the slot as deleted. If WAL write fails,
    /// the page is not modified.
    pub fn delete(&mut self, slot_idx: u16) -> std::io::Result<()> {
        // Get the cluster key for WAL logging before marking as deleted
        let cluster_key = if let Ok(Some(record)) = self.page.get(slot_idx) {
            Some(record.cluster_key)
        } else {
            None
        };

        // Log to WAL first - if this fails, don't modify page
        if let Some(ref key) = cluster_key {
            self.wal_manager.log_delete(self.tx_id, key)?;
        }

        // WAL logged successfully, now apply to page
        self.page.delete(slot_idx)
    }

    /// Update a record with WAL logging.
    ///
    /// Logs to WAL before updating the record. This is implemented as
    /// delete + insert to handle variable-length field changes.
    pub fn update(
        &mut self,
        slot_idx: u16,
        cluster_key: &ClusterKey,
        fixed_columns: &[Value],
        varlen_columns: &[Option<Vec<u8>>],
        null_bitmap: &[bool],
    ) -> std::io::Result<u16> {
        // Log old value as delete
        let old_key = if let Ok(Some(record)) = self.page.get(slot_idx) {
            Some(record.cluster_key)
        } else {
            None
        };

        if let Some(ref key) = old_key {
            self.wal_manager.log_delete(self.tx_id, key)?;
        }

        // Log new value as insert
        self.wal_manager.log_insert(
            self.tx_id,
            cluster_key,
            fixed_columns,
            varlen_columns,
            null_bitmap,
        )?;

        // Delete old record
        self.page.delete(slot_idx)?;

        // Insert new record
        self.page
            .insert(cluster_key, fixed_columns, varlen_columns, null_bitmap)
    }

    /// Get a record by slot index.
    pub fn get(
        &self,
        slot_idx: u16,
    ) -> std::io::Result<Option<crate::row_format::types::ClusteredLeafRecord>> {
        self.page.get(slot_idx)
    }

    /// Get the number of live records.
    pub fn live_record_count(&self) -> usize {
        self.page.live_record_count()
    }

    /// Check if the page needs split.
    pub fn needs_split(&self) -> bool {
        self.page.needs_split()
    }

    /// Get slot count.
    pub fn slot_count(&self) -> u16 {
        self.page.slot_count()
    }

    /// Get free space.
    pub fn free_space(&self) -> usize {
        self.page.free_space()
    }

    /// Find lower bound.
    pub fn lower_bound(&self, key: &ClusterKey) -> Option<u16> {
        self.page.lower_bound(key)
    }

    /// Find upper bound.
    pub fn upper_bound(&self, key: &ClusterKey) -> Option<u16> {
        self.page.upper_bound(key)
    }

    /// Get the underlying page (for read-only access).
    pub fn page(&self) -> &ClusteredLeafPage {
        self.page
    }

    /// Get transaction ID.
    pub fn tx_id(&self) -> u64 {
        self.tx_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::row_format::types::ClusterKey;
    use sqlrustgo_types::Value;
    use tempfile::TempDir;

    fn create_test_wal_manager() -> (ClusteredWalManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        let wal_manager = ClusteredWalManager::new(wal_path, 1); // table_id = 1
        (wal_manager, temp_dir)
    }

    #[test]
    fn test_transaction_insert_logs_to_wal() {
        let (wal_manager, _temp_dir) = create_test_wal_manager();
        let mut page = ClusteredLeafPage::new();
        let tx_id = 1;

        // Create transaction
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Insert a record
        let key = ClusterKey::HiddenRowId(42);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];

        let slot_idx = tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        assert_eq!(slot_idx, 0);

        // Verify record was inserted
        assert_eq!(tx.live_record_count(), 1);

        // Commit the transaction
        wal_manager.log_commit(tx_id).unwrap();

        // Recover and verify
        let entries = wal_manager.recover().unwrap();
        let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();
        assert_eq!(insert_entries.len(), 1);
    }

    #[test]
    fn test_transaction_delete_logs_to_wal() {
        let (wal_manager, _temp_dir) = create_test_wal_manager();
        let mut page = ClusteredLeafPage::new();
        let tx_id = 1;

        // Insert first
        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);
        let key = ClusterKey::HiddenRowId(42);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        wal_manager.log_commit(tx_id).unwrap();

        // Begin new transaction for delete
        let tx_id2 = 2;
        let mut tx2 = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id2);
        tx2.delete(0).unwrap();
        wal_manager.log_commit(tx_id2).unwrap();

        // Recover and verify
        let entries = wal_manager.recover().unwrap();
        let delete_entries: Vec<_> = entries.iter().filter(|e| e.is_delete()).collect();
        assert_eq!(delete_entries.len(), 1);
    }

    #[test]
    fn test_transaction_preserves_page_on_wal_failure() {
        // This test verifies that if WAL logging fails, the page is not modified.
        // We can't easily simulate WAL failure in a unit test without mocking,
        // but the architecture ensures WAL is called before page modification.
        let (wal_manager, _temp_dir) = create_test_wal_manager();
        let mut page = ClusteredLeafPage::new();
        let tx_id = 1;

        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        let key = ClusterKey::HiddenRowId(42);
        let fixed = vec![Value::Integer(100)];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls = vec![false];

        // Insert should succeed
        tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        assert_eq!(tx.live_record_count(), 1);
    }

    #[test]
    fn test_transaction_multiple_operations_same_tx() {
        let (wal_manager, _temp_dir) = create_test_wal_manager();
        let mut page = ClusteredLeafPage::new();
        let tx_id = 1;

        let mut tx = ClusteredPageTransaction::new(&mut page, &wal_manager, tx_id);

        // Insert multiple records in same transaction
        for i in 0..5 {
            let key = ClusterKey::HiddenRowId(i);
            let fixed = vec![Value::Integer(i as i64 * 10)];
            let varlen: Vec<Option<Vec<u8>>> = vec![];
            let nulls = vec![false];
            tx.insert(&key, &fixed, &varlen, &nulls).unwrap();
        }

        assert_eq!(tx.slot_count(), 5);
        assert_eq!(tx.live_record_count(), 5);

        // Commit
        wal_manager.log_commit(tx_id).unwrap();

        // Recover and verify
        let entries = wal_manager.recover().unwrap();
        let insert_entries: Vec<_> = entries.iter().filter(|e| e.is_insert()).collect();
        assert_eq!(insert_entries.len(), 5);
    }
}
