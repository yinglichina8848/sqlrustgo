//! XA-MVCC Bridge: Connects XA Transaction Coordinator with MVCC Storage Engine
//!
//! This module provides the bridge that allows XA transactions to participate in
//! MVCC snapshot isolation, combining the MySQL-compatible XA state machine with
//! the MVCC version chain storage.
//!
//! # Architecture
//!
//! ```text
//! XaCoordinator (state machine) <-- XaBridge --> MVCCStorageEngine (version chains)
//! ```
//!
//! # Key Integration Points
//!
//! - `xa_start`: Creates MVCC TxId and snapshot for the XA transaction
//! - `xa_prepare`: Commits MVCC versions (phase 1 - makes changes durable)
//! - `xa_commit`: Finalizes the XA transaction
//! - `xa_rollback`: Rolls back MVCC versions
//!
//! # XA Transaction Lifecycle with MVCC
//!
//! ```text
//! XA START (xid)           --> Create TxId, Snapshot in XaBridge
//! [SQL statements]          --> MVCC reads/writes using Snapshot
//! XA END (xid)             --> Mark SQL phase complete
//! XA PREPARE (xid)         --> Commit MVCC versions (phase 1)
//! XA COMMIT (xid)          --> Finalize XA transaction (phase 2)
//! ```

use crate::xa_coordinator::{XaCoordinator, XaError, Xid};
use sqlrustgo_transaction::manager::TransactionManager as MvccTransactionManager;
use sqlrustgo_transaction::mvcc::Snapshot;
use sqlrustgo_transaction::mvcc_storage::MVCCStorage;
use std::sync::{Arc, RwLock};

/// Bridge trait connecting XA transactions with MVCC storage engine
///
/// This trait allows XA transactions to participate in MVCC snapshot isolation
/// by providing methods to create, prepare, commit, and rollback MVCC versions
/// associated with XA transactions.
pub trait XaBridge: Send + Sync {
    /// Begin an XA transaction with MVCC support
    ///
    /// Creates a new TxId and associates it with the XA transaction.
    /// The XA transaction must be in ACTIVE state.
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Ok(TxId)` - The MVCC transaction ID associated with this XA transaction
    /// * `Err(XaError)` - If XA transaction not found or not in ACTIVE state
    fn begin_xa_transaction(&self, xid: &Xid) -> Result<TxId, XaError>;

    /// Prepare an XA transaction (phase 1)
    ///
    /// Commits the MVCC versions for the transaction, making changes durable.
    /// The XA transaction must be in IDLE state.
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Ok(u64)` - The prepare timestamp (commit_ts for MVCC versions)
    /// * `Err(XaError)` - If XA transaction not found or not in IDLE state
    fn prepare_xa_transaction(&self, xid: &Xid) -> Result<u64, XaError>;

    /// Commit an XA transaction (phase 2)
    ///
    /// Finalizes the XA transaction after it has been prepared.
    /// The XA transaction must be in PREPARED state.
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Ok(())` - Transaction committed successfully
    /// * `Err(XaError)` - If XA transaction not found or not in PREPARED state
    fn commit_xa_transaction(&self, xid: &Xid) -> Result<(), XaError>;

    /// Rollback an XA transaction
    ///
    /// Rolls back the MVCC versions created by the transaction.
    /// The XA transaction must be in ACTIVE, IDLE, or PREPARED state.
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Ok(())` - Transaction rolled back successfully
    /// * `Err(XaError)` - If XA transaction not found or in terminal state
    fn rollback_xa_transaction(&self, xid: &Xid) -> Result<(), XaError>;

    /// Read a key using the MVCC snapshot associated with an XA transaction
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    /// * `key` - The key to read
    ///
    /// # Returns
    /// * `Some(Vec<u8>)` - The value if found
    /// * `None` - If key not found or not visible
    fn read_xa(&self, xid: &Xid, key: &[u8]) -> Option<Vec<u8>>;

    /// Write a key-value pair in the XA transaction's MVCC context
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    /// * `key` - The key to write
    /// * `value` - The value to write
    fn write_xa(&self, xid: &Xid, key: Vec<u8>, value: Vec<u8>) -> Result<(), XaError>;

    /// Delete a key in the XA transaction's MVCC context
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    /// * `key` - The key to delete
    fn delete_xa(&self, xid: &Xid, key: Vec<u8>) -> Result<(), XaError>;

    /// Get the MVCC snapshot for an XA transaction
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Some(Snapshot)` - The MVCC snapshot
    /// * `None` - If XA transaction not found
    fn get_snapshot(&self, xid: &Xid) -> Option<Snapshot>;

    /// Get the TxId for an XA transaction
    ///
    /// # Arguments
    /// * `xid` - The XA transaction identifier
    ///
    /// # Returns
    /// * `Some(TxId)` - The MVCC transaction ID
    /// * `None` - If XA transaction not found
    fn get_tx_id(&self, xid: &Xid) -> Option<TxId>;
}

// Re-export TxId for convenience
pub use sqlrustgo_transaction::mvcc::TxId;

/// XA-MVCC Bridge implementation
///
/// This implementation connects the XaCoordinator with MVCCStorage and a TransactionManager
/// to provide full MVCC support for XA transactions.
pub struct XaMvccBridge<S: MVCCStorage> {
    /// The XA coordinator
    xa: Arc<XaCoordinator>,
    /// The MVCC storage engine
    storage: Arc<RwLock<S>>,
    /// Transaction manager for MVCC operations
    tx_manager: Arc<RwLock<MvccTransactionManager>>,
    /// Maps XID to MVCC TxId
    xid_to_tx: RwLock<std::collections::HashMap<Xid, TxId>>,
    /// Maps TxId to XID (reverse lookup)
    tx_to_xid: RwLock<std::collections::HashMap<TxId, Xid>>,
    /// Maps XID to MVCC Snapshot
    xid_to_snapshot: RwLock<std::collections::HashMap<Xid, Snapshot>>,
    /// Global timestamp for XA transactions
    global_timestamp: RwLock<u64>,
}

impl<S: MVCCStorage> XaMvccBridge<S> {
    /// Creates a new XA-MVCC Bridge
    pub fn new(xa: Arc<XaCoordinator>, storage: Arc<RwLock<S>>) -> Self {
        Self {
            xa,
            storage,
            tx_manager: Arc::new(RwLock::new(MvccTransactionManager::new())),
            xid_to_tx: RwLock::new(std::collections::HashMap::new()),
            tx_to_xid: RwLock::new(std::collections::HashMap::new()),
            xid_to_snapshot: RwLock::new(std::collections::HashMap::new()),
            global_timestamp: RwLock::new(1),
        }
    }

    /// Get next global timestamp
    fn next_timestamp(&self) -> u64 {
        let mut ts = self.global_timestamp.write().unwrap();
        let result = *ts;
        *ts += 1;
        result
    }
}

impl<S: MVCCStorage> XaBridge for XaMvccBridge<S> {
    fn begin_xa_transaction(&self, xid: &Xid) -> Result<TxId, XaError> {
        // Verify XA transaction exists and is in ACTIVE state
        let state = self.xa.get_state(xid).map_err(|e| e.clone())?;
        if state != crate::xa_coordinator::XaState::Active {
            return Err(XaError::InvalidStateTransition {
                current_state: state,
                attempted_operation: "XA BEGIN".to_string(),
            });
        }

        // Create MVCC transaction and get snapshot
        let (tx_id, snapshot) = {
            let mut tx_manager = self.tx_manager.write().unwrap();
            let tx_id = tx_manager
                .begin()
                .map_err(|e| XaError::InvalidForState(e.to_string()))?;
            let ctx = tx_manager
                .get_transaction_context_for_query()
                .map_err(|e| XaError::InvalidForState(e.to_string()))?;
            (tx_id, ctx.snapshot)
        };

        // Store mappings
        {
            let mut xid_to_tx = self.xid_to_tx.write().unwrap();
            xid_to_tx.insert(xid.clone(), tx_id);
        }
        {
            let mut tx_to_xid = self.tx_to_xid.write().unwrap();
            tx_to_xid.insert(tx_id, xid.clone());
        }
        {
            let mut xid_to_snapshot = self.xid_to_snapshot.write().unwrap();
            xid_to_snapshot.insert(xid.clone(), snapshot);
        }

        Ok(tx_id)
    }

    fn prepare_xa_transaction(&self, xid: &Xid) -> Result<u64, XaError> {
        // First, call XA PREPARE on the coordinator
        self.xa.xa_prepare(xid)?;

        // Get the TxId for this XA transaction
        let tx_id = {
            let xid_to_tx = self.xid_to_tx.read().unwrap();
            xid_to_tx.get(xid).copied()
        };

        let tx_id = tx_id.ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        // Commit MVCC versions
        let prepare_ts = self.next_timestamp();
        {
            let mut storage = self.storage.write().unwrap();
            storage
                .commit_versions(tx_id, prepare_ts)
                .map_err(|e| XaError::InvalidForState(e.to_string()))?;
        }

        // Commit the MVCC transaction in the manager (ends the transaction in manager)
        {
            let mut tx_manager = self.tx_manager.write().unwrap();
            tx_manager
                .commit()
                .map_err(|e| XaError::InvalidForState(e.to_string()))?;
        }

        Ok(prepare_ts)
    }

    fn commit_xa_transaction(&self, xid: &Xid) -> Result<(), XaError> {
        // Delegate to XA coordinator
        self.xa.xa_commit(xid)?;

        // Clean up mappings
        let tx_id = {
            let mut xid_to_tx = self.xid_to_tx.write().unwrap();
            xid_to_tx.remove(xid)
        };

        if let Some(tx_id) = tx_id {
            let mut tx_to_xid = self.tx_to_xid.write().unwrap();
            tx_to_xid.remove(&tx_id);
        }

        Ok(())
    }

    fn rollback_xa_transaction(&self, xid: &Xid) -> Result<(), XaError> {
        // Get the TxId for this XA transaction
        let tx_id = {
            let xid_to_tx = self.xid_to_tx.read().unwrap();
            xid_to_tx.get(xid).copied()
        };

        // Rollback MVCC versions if we have a TxId
        if let Some(tx_id) = tx_id {
            let mut storage = self.storage.write().unwrap();
            storage
                .rollback_versions(tx_id)
                .map_err(|e| XaError::InvalidForState(e.to_string()))?;
        }

        // Delegate to XA coordinator
        self.xa.xa_rollback(xid)?;

        // Clean up mappings
        {
            let mut xid_to_tx = self.xid_to_tx.write().unwrap();
            xid_to_tx.remove(xid);
        }
        if let Some(tx_id) = tx_id {
            let mut tx_to_xid = self.tx_to_xid.write().unwrap();
            tx_to_xid.remove(&tx_id);
        }
        {
            let mut xid_to_snapshot = self.xid_to_snapshot.write().unwrap();
            xid_to_snapshot.remove(xid);
        }

        Ok(())
    }

    fn read_xa(&self, xid: &Xid, key: &[u8]) -> Option<Vec<u8>> {
        let snapshot = self.get_snapshot(xid)?;
        let storage = self.storage.read().unwrap();
        storage.read(key, &snapshot)
    }

    fn write_xa(&self, xid: &Xid, key: Vec<u8>, value: Vec<u8>) -> Result<(), XaError> {
        let tx_id = {
            let xid_to_tx = self.xid_to_tx.read().unwrap();
            xid_to_tx.get(xid).copied()
        };

        let tx_id = tx_id.ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        let mut storage = self.storage.write().unwrap();
        storage.write_version(key, value, tx_id);
        Ok(())
    }

    fn delete_xa(&self, xid: &Xid, key: Vec<u8>) -> Result<(), XaError> {
        let tx_id = {
            let xid_to_tx = self.xid_to_tx.read().unwrap();
            xid_to_tx.get(xid).copied()
        };

        let tx_id = tx_id.ok_or_else(|| XaError::XidNotFound(xid.clone()))?;

        let mut storage = self.storage.write().unwrap();
        storage.delete_version(key, tx_id);
        Ok(())
    }

    fn get_snapshot(&self, xid: &Xid) -> Option<Snapshot> {
        let xid_to_snapshot = self.xid_to_snapshot.read().unwrap();
        xid_to_snapshot.get(xid).cloned()
    }

    fn get_tx_id(&self, xid: &Xid) -> Option<TxId> {
        let xid_to_tx = self.xid_to_tx.read().unwrap();
        xid_to_tx.get(xid).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xa_coordinator::{XaState, Xid};
    use sqlrustgo_transaction::mvcc_storage::MVCCStorageEngine;

    fn create_test_xid() -> Xid {
        Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec())
    }

    #[test]
    fn test_xa_mvcc_bridge_basic_lifecycle() {
        let xa = Arc::new(XaCoordinator::new());
        let storage = Arc::new(RwLock::new(MVCCStorageEngine::new()));
        let bridge: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa.clone(), storage.clone());

        let xid = create_test_xid();

        // Start XA transaction
        xa.xa_start(xid.clone()).unwrap();
        let tx_id = bridge.begin_xa_transaction(&xid).unwrap();
        assert!(tx_id.is_valid());

        // Write some data
        bridge
            .write_xa(&xid, b"key1".to_vec(), b"value1".to_vec())
            .unwrap();

        // Read back (own write should be visible)
        assert_eq!(bridge.read_xa(&xid, b"key1"), Some(b"value1".to_vec()));

        // End XA transaction
        xa.xa_end(&xid).unwrap();

        // Prepare (phase 1)
        let prepare_ts = bridge.prepare_xa_transaction(&xid).unwrap();
        assert!(prepare_ts > 0);

        // Commit (phase 2)
        bridge.commit_xa_transaction(&xid).unwrap();

        // Verify XA state
        assert_eq!(xa.get_state(&xid).unwrap(), XaState::Committed);
    }

    #[test]
    fn test_xa_mvcc_bridge_rollback() {
        let xa = Arc::new(XaCoordinator::new());
        let storage = Arc::new(RwLock::new(MVCCStorageEngine::new()));
        let bridge: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa.clone(), storage.clone());

        let xid = create_test_xid();

        // Start XA transaction
        xa.xa_start(xid.clone()).unwrap();
        bridge.begin_xa_transaction(&xid).unwrap();

        // Write some data
        bridge
            .write_xa(&xid, b"key1".to_vec(), b"value1".to_vec())
            .unwrap();

        // End and rollback
        xa.xa_end(&xid).unwrap();
        bridge.rollback_xa_transaction(&xid).unwrap();

        // Verify XA state
        assert_eq!(xa.get_state(&xid).unwrap(), XaState::RolledBack);

        // Verify storage is cleaned up - create new snapshot
        let snapshot = Snapshot::new(TxId::new(999), 1000, vec![]);
        let storage_guard = storage.read().unwrap();
        assert_eq!(storage_guard.read(b"key1", &snapshot), None);
    }

    #[test]
    fn test_xa_mvcc_bridge_snapshot_isolation() {
        let xa1 = Arc::new(XaCoordinator::new());
        let xa2 = Arc::new(XaCoordinator::new());
        let storage = Arc::new(RwLock::new(MVCCStorageEngine::new()));
        let bridge1: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa1.clone(), storage.clone());
        let bridge2: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa2.clone(), storage.clone());

        let xid1 = Xid::new(0, b"gtrid1".to_vec(), b"bqual1".to_vec());
        let xid2 = Xid::new(0, b"gtrid2".to_vec(), b"bqual2".to_vec());

        // Start and prepare first transaction
        xa1.xa_start(xid1.clone()).unwrap();
        bridge1.begin_xa_transaction(&xid1).unwrap();
        bridge1
            .write_xa(&xid1, b"key1".to_vec(), b"value1".to_vec())
            .unwrap();

        // Snapshot BEFORE prepare - should not see uncommitted value
        let snapshot_before_prepare = Snapshot::new(TxId::new(100), 50, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(storage_guard.read(b"key1", &snapshot_before_prepare), None);
        }

        xa1.xa_end(&xid1).unwrap();
        bridge1.prepare_xa_transaction(&xid1).unwrap();

        // After prepare, data IS visible (prepare = MVCC commit)
        let snapshot_after_prepare = Snapshot::new(TxId::new(101), 100, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(
                storage_guard.read(b"key1", &snapshot_after_prepare),
                Some(b"value1".to_vec())
            );
        }

        // Commit first transaction (doesn't change MVCC visibility)
        bridge1.commit_xa_transaction(&xid1).unwrap();

        // Still visible after commit
        let snapshot_after_commit = Snapshot::new(TxId::new(102), 150, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(
                storage_guard.read(b"key1", &snapshot_after_commit),
                Some(b"value1".to_vec())
            );
        }

        // Second XA transaction starts after first committed
        // Should see the committed value
        xa2.xa_start(xid2.clone()).unwrap();
        bridge2.begin_xa_transaction(&xid2).unwrap();

        let snapshot_during_tx2 = Snapshot::new(TxId::new(103), 200, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(
                storage_guard.read(b"key1", &snapshot_during_tx2),
                Some(b"value1".to_vec())
            );
        }
    }

    #[test]
    fn test_xa_mvcc_bridge_delete() {
        let xa = Arc::new(XaCoordinator::new());
        let storage = Arc::new(RwLock::new(MVCCStorageEngine::new()));
        let bridge: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa.clone(), storage.clone());

        let xid = create_test_xid();

        // Start, write, prepare, commit
        xa.xa_start(xid.clone()).unwrap();
        bridge.begin_xa_transaction(&xid).unwrap();
        bridge
            .write_xa(&xid, b"key1".to_vec(), b"value1".to_vec())
            .unwrap();
        xa.xa_end(&xid).unwrap();
        bridge.prepare_xa_transaction(&xid).unwrap();
        bridge.commit_xa_transaction(&xid).unwrap();

        // New transaction deletes the key
        let xid2 = Xid::new(0, b"gtrid2".to_vec(), b"bqual2".to_vec());
        xa.xa_start(xid2.clone()).unwrap();
        bridge.begin_xa_transaction(&xid2).unwrap();
        bridge.delete_xa(&xid2, b"key1".to_vec()).unwrap();
        xa.xa_end(&xid2).unwrap();
        bridge.prepare_xa_transaction(&xid2).unwrap();
        bridge.commit_xa_transaction(&xid2).unwrap();

        // Key should no longer be visible
        let snapshot = Snapshot::new(TxId::new(200), 300, vec![]);
        let storage_guard = storage.read().unwrap();
        assert_eq!(storage_guard.read(b"key1", &snapshot), None);
    }

    #[test]
    fn test_xa_mvcc_bridge_uncommitted_not_visible() {
        let xa = Arc::new(XaCoordinator::new());
        let storage = Arc::new(RwLock::new(MVCCStorageEngine::new()));
        let bridge: XaMvccBridge<MVCCStorageEngine> =
            XaMvccBridge::new(xa.clone(), storage.clone());

        let xid = create_test_xid();

        // Start transaction and write
        xa.xa_start(xid.clone()).unwrap();
        bridge.begin_xa_transaction(&xid).unwrap();
        bridge
            .write_xa(&xid, b"key1".to_vec(), b"value1".to_vec())
            .unwrap();

        // Create a snapshot from another "transaction" - should not see uncommitted
        let other_snapshot = Snapshot::new(TxId::new(99), 50, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(storage_guard.read(b"key1", &other_snapshot), None);
        }

        // End and prepare
        xa.xa_end(&xid).unwrap();
        bridge.prepare_xa_transaction(&xid).unwrap();

        // After prepare, the version IS committed in MVCC, so a snapshot
        // with snapshot_timestamp > prepare_ts should see it
        let snapshot_after_prepare = Snapshot::new(TxId::new(100), 100, vec![]);
        {
            let storage_guard = storage.read().unwrap();
            assert_eq!(
                storage_guard.read(b"key1", &snapshot_after_prepare),
                Some(b"value1".to_vec())
            );
        }
    }
}
