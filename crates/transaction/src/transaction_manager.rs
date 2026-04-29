use std::collections::HashMap;

use crate::mvcc::{Snapshot, TxId};
use crate::ssi::{SsiDetectorSync, SsiError};

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    /// Snapshot Isolation - readers see consistent snapshot, writers use first-committer-wins
    #[default]
    SnapshotIsolation,
    /// Serializable - ensures strict serial execution order
    Serializable,
}

/// Current state of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is actively executing
    Active,
    /// Transaction has been committed successfully
    Committed,
    /// Transaction was aborted (rolled back)
    Aborted,
}

/// Active transaction with its metadata
#[derive(Debug)]
pub struct ActiveTransaction {
    /// Unique transaction identifier
    pub tx_id: TxId,
    /// MVCC snapshot for this transaction
    pub snapshot: Snapshot,
    /// Current state of the transaction
    pub state: TransactionState,
    /// Keys read by this transaction
    pub read_keys: Vec<Vec<u8>>,
    /// Keys written by this transaction
    pub write_keys: Vec<Vec<u8>>,
}

impl ActiveTransaction {
    /// Create a new active transaction
    pub fn new(tx_id: TxId, snapshot: Snapshot) -> Self {
        Self {
            tx_id,
            snapshot,
            state: TransactionState::Active,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
        }
    }
}

/// Transaction manager with SSI (Serializable Snapshot Isolation) support
pub struct TransactionManager {
    ssi_detector: SsiDetectorSync,
    active_transactions: HashMap<TxId, ActiveTransaction>,
    next_tx_id: u64,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            ssi_detector: SsiDetectorSync::new(),
            active_transactions: HashMap::new(),
            next_tx_id: 1,
        }
    }

    /// Begin a new transaction with the specified isolation level
    ///
    /// # Arguments
    /// * `isolation` - Isolation level for the new transaction
    ///
    /// # Returns
    /// * `Ok(TxId)` - Transaction ID if successful
    /// * `Err(SsiError)` - If transaction cannot be started
    pub fn begin_transaction(&mut self, _isolation: IsolationLevel) -> Result<TxId, SsiError> {
        let tx_id = TxId::new(self.next_tx_id);
        self.next_tx_id += 1;

        let snapshot_timestamp = tx_id.as_u64();
        let snapshot = Snapshot::new_read_committed(tx_id, snapshot_timestamp);

        let active_tx = ActiveTransaction::new(tx_id, snapshot);
        self.active_transactions.insert(tx_id, active_tx);

        Ok(tx_id)
    }

    /// Record a read operation for SSI detection
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    /// * `key` - Key being read
    pub fn record_read(&mut self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        self.ssi_detector.record_read(tx_id, key.clone());

        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.read_keys.push(key);
        }

        Ok(())
    }

    /// Record a write operation for SSI detection
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    /// * `key` - Key being written
    pub fn record_write(&mut self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        self.ssi_detector.record_write(tx_id, key.clone());

        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.write_keys.push(key);
        }

        Ok(())
    }

    /// Commit a transaction
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID to commit
    ///
    /// # Returns
    /// * `Ok(())` - If commit succeeds
    /// * `Err(SsiError)` - If serialization failure detected
    pub fn commit(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        self.ssi_detector.validate_commit(tx_id)?;

        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Committed;
        }

        self.ssi_detector.release(tx_id);
        self.active_transactions.remove(&tx_id);

        Ok(())
    }

    pub fn rollback(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Aborted;
        }

        self.ssi_detector.release(tx_id);
        self.active_transactions.remove(&tx_id);

        Ok(())
    }

    /// Abort (rollback) a transaction
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID to abort
    pub fn abort(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Aborted;
        }

        self.ssi_detector.release(tx_id);
        self.active_transactions.remove(&tx_id);

        Ok(())
    }

    /// Get the snapshot for a transaction
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    /// * `Some(Snapshot)` - If transaction is active
    /// * `None` - If transaction not found
    pub fn get_snapshot(&self, tx_id: TxId) -> Option<Snapshot> {
        self.active_transactions
            .get(&tx_id)
            .map(|at| at.snapshot.clone())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_transaction() {
        let mut mgr = TransactionManager::new();
        let tx_id = mgr.begin_transaction(IsolationLevel::SnapshotIsolation);
        assert!(tx_id.is_ok());
        assert_eq!(tx_id.unwrap().as_u64(), 1);
    }

    #[test]
    fn test_record_read_write() {
        let mut mgr = TransactionManager::new();
        let tx_id = mgr
            .begin_transaction(IsolationLevel::SnapshotIsolation)
            .unwrap();

        mgr.record_read(tx_id, b"key1".to_vec()).unwrap();
        mgr.record_write(tx_id, b"key2".to_vec()).unwrap();

        let active = mgr.active_transactions.get(&tx_id).unwrap();
        assert_eq!(active.read_keys, vec![b"key1".to_vec()]);
        assert_eq!(active.write_keys, vec![b"key2".to_vec()]);
    }

    #[test]
    fn test_commit_transaction() {
        let mut mgr = TransactionManager::new();
        let tx_id = mgr
            .begin_transaction(IsolationLevel::SnapshotIsolation)
            .unwrap();

        mgr.record_read(tx_id, b"key1".to_vec()).unwrap();
        let result = mgr.commit(tx_id);
        assert!(result.is_ok());

        assert!(mgr.active_transactions.get(&tx_id).is_none());
    }

    #[test]
    fn test_rollback_transaction() {
        let mut mgr = TransactionManager::new();
        let tx_id = mgr
            .begin_transaction(IsolationLevel::SnapshotIsolation)
            .unwrap();

        mgr.record_read(tx_id, b"key1".to_vec()).unwrap();
        let result = mgr.rollback(tx_id);
        assert!(result.is_ok());

        assert!(mgr.active_transactions.get(&tx_id).is_none());
    }

    #[test]
    fn test_get_snapshot() {
        let mut mgr = TransactionManager::new();
        let tx_id = mgr
            .begin_transaction(IsolationLevel::SnapshotIsolation)
            .unwrap();

        let snapshot = mgr.get_snapshot(tx_id);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().tx_id, tx_id);
    }

    #[test]
    fn test_get_snapshot_none() {
        let mgr = TransactionManager::new();
        let snapshot = mgr.get_snapshot(TxId::new(999));
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_isolation_level_default() {
        assert_eq!(IsolationLevel::default(), IsolationLevel::SnapshotIsolation);
    }

    #[test]
    fn test_multiple_transactions() {
        let mut mgr = TransactionManager::new();

        let tx1 = mgr
            .begin_transaction(IsolationLevel::SnapshotIsolation)
            .unwrap();
        let tx2 = mgr.begin_transaction(IsolationLevel::Serializable).unwrap();

        assert_eq!(tx1.as_u64(), 1);
        assert_eq!(tx2.as_u64(), 2);

        mgr.record_read(tx1, b"key1".to_vec()).unwrap();
        mgr.record_write(tx2, b"key2".to_vec()).unwrap();

        mgr.commit(tx1).unwrap();
        mgr.commit(tx2).unwrap();
    }
}
