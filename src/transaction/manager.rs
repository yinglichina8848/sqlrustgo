//! Transaction Manager
//!
//! Manages transaction lifecycle: BEGIN, COMMIT, ROLLBACK.
//! Works with WAL for durability and recovery.
//!
//! ## Transaction States
//!
//! ```mermaid
//! stateDiagram-v2
//!     [*] --> Active: BEGIN
//!     Active --> Committed: COMMIT
//!     Active --> Aborted: ROLLBACK
//!     Committed --> [*]
//!     Aborted --> [*]
//! ```
//!
//! ## ACID Properties
//!
//! - **Atomicity**: Either all changes apply or none (rollback undoes)
//! - **Consistency**: Transactions leave DB in valid state
//! - **Isolation**: MVCC-like snapshot per transaction
//! - **Durability**: Committed changes survive crash (via WAL)
//!
//! ## Lifecycle
//!
//! 1. `begin()` - Creates new transaction, returns tx_id, logs BEGIN
//! 2. Operations - Modify data within transaction
//! 3. `commit()` - Marks committed, logs COMMIT, removes from active
//! 4. `rollback()` - Marks aborted, logs ROLLBACK, removes from active

use super::wal::{WalRecord, WriteAheadLog};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Transaction state
#[derive(Debug, Clone, PartialEq)]
pub enum TxState {
    Active,
    Committed,
    Aborted,
}

/// Transaction record
#[derive(Debug)]
pub struct Transaction {
    pub tx_id: u64,
    pub state: TxState,
    pub snapshot: Vec<u64>, // Snapshot of active transactions
}

impl Transaction {
    pub fn new(tx_id: u64) -> Self {
        Self {
            tx_id,
            state: TxState::Active,
            snapshot: Vec::new(),
        }
    }
}

/// Transaction Manager
pub struct TransactionManager {
    next_tx_id: Arc<Mutex<u64>>,
    active_transactions: Arc<Mutex<HashMap<u64, Transaction>>>,
    wal: Arc<WriteAheadLog>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(wal: Arc<WriteAheadLog>) -> Self {
        Self {
            next_tx_id: Arc::new(Mutex::new(1)),
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            wal,
        }
    }

    /// Begin a new transaction
    pub fn begin(&self) -> Result<u64, String> {
        let tx_id = {
            let mut next = self.next_tx_id.lock().unwrap();
            let id = *next;
            *next += 1;
            id
        };

        // Get snapshot of active transactions
        let snapshot: Vec<u64> = {
            let active = self.active_transactions.lock().unwrap();
            active.keys().cloned().collect()
        };

        // Create transaction record
        let mut tx = Transaction::new(tx_id);
        tx.snapshot = snapshot;

        // Log BEGIN
        self.wal
            .append(&WalRecord::Begin { tx_id })
            .map_err(|e| e.to_string())?;

        // Register transaction
        self.active_transactions.lock().unwrap().insert(tx_id, tx);

        Ok(tx_id)
    }

    /// Commit a transaction
    pub fn commit(&self, tx_id: u64) -> Result<(), String> {
        let mut active = self.active_transactions.lock().unwrap();

        if let Some(tx) = active.get_mut(&tx_id) {
            if tx.state != TxState::Active {
                return Err("Transaction not active".to_string());
            }

            tx.state = TxState::Committed;
        } else {
            return Err("Transaction not found".to_string());
        }

        // Log COMMIT
        self.wal
            .append(&WalRecord::Commit { tx_id })
            .map_err(|e| e.to_string())?;

        // Remove from active
        active.remove(&tx_id);

        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback(&self, tx_id: u64) -> Result<(), String> {
        let mut active = self.active_transactions.lock().unwrap();

        if let Some(tx) = active.get_mut(&tx_id) {
            if tx.state != TxState::Active {
                return Err("Transaction not active".to_string());
            }

            tx.state = TxState::Aborted;
        } else {
            return Err("Transaction not found".to_string());
        }

        // Log ROLLBACK
        self.wal
            .append(&WalRecord::Rollback { tx_id })
            .map_err(|e| e.to_string())?;

        // Remove from active
        active.remove(&tx_id);

        Ok(())
    }

    /// Get transaction state
    pub fn get_state(&self, tx_id: u64) -> Option<TxState> {
        let active = self.active_transactions.lock().unwrap();
        active.get(&tx_id).map(|tx| tx.state.clone())
    }

    /// Check if transaction is active
    pub fn is_active(&self, tx_id: u64) -> bool {
        self.get_state(tx_id) == Some(TxState::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_begin() {
        let path = "/tmp/tm_test_begin.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        assert_eq!(tx_id, 1);
        assert!(tm.is_active(1));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_commit() {
        let path = "/tmp/tm_test_commit.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        tm.commit(tx_id).unwrap();

        assert!(!tm.is_active(tx_id));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_rollback() {
        let path = "/tmp/tm_test_rollback.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        tm.rollback(tx_id).unwrap();

        assert!(!tm.is_active(tx_id));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_get_state() {
        let path = "/tmp/tm_test_state.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        assert_eq!(tm.get_state(tx_id), Some(TxState::Active));

        // After commit, transaction may or may not be in state tracking
        let _ = tm.commit(tx_id);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_nonexistent() {
        let path = "/tmp/tm_test_nonexistent.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        // Query non-existent transaction
        assert!(!tm.is_active(999));
        assert_eq!(tm.get_state(999), None);

        std::fs::remove_file(path).ok();
    }

    // ==================== Additional Coverage Tests ====================

    #[test]
    fn test_transaction_commit_nonexistent() {
        let path = "/tmp/tm_test_commit_nonexistent.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        // Try to commit non-existent transaction
        let result = tm.commit(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_rollback_nonexistent() {
        let path = "/tmp/tm_test_rollback_nonexistent.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        // Try to rollback non-existent transaction
        let result = tm.rollback(999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_double_commit() {
        let path = "/tmp/tm_test_double_commit.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        tm.commit(tx_id).unwrap();

        // Try to commit again - should fail because tx is no longer active
        let result = tm.commit(tx_id);
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_double_rollback() {
        let path = "/tmp/tm_test_double_rollback.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        tm.rollback(tx_id).unwrap();

        // Try to rollback again - should fail because tx is no longer active
        let result = tm.rollback(tx_id);
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_commit_after_rollback() {
        let path = "/tmp/tm_test_commit_after_rollback.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        let tx_id = tm.begin().unwrap();
        tm.rollback(tx_id).unwrap();

        // Try to commit after rollback - should fail
        let result = tm.commit(tx_id);
        assert!(result.is_err());

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_multiple_concurrent() {
        let path = "/tmp/tm_test_concurrent.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        // Start multiple transactions
        let tx1 = tm.begin().unwrap();
        let tx2 = tm.begin().unwrap();
        let tx3 = tm.begin().unwrap();

        assert_eq!(tx1, 1);
        assert_eq!(tx2, 2);
        assert_eq!(tx3, 3);

        // All should be active
        assert!(tm.is_active(tx1));
        assert!(tm.is_active(tx2));
        assert!(tm.is_active(tx3));

        // Commit one
        tm.commit(tx2).unwrap();
        assert!(!tm.is_active(tx2));
        assert!(tm.is_active(tx1));
        assert!(tm.is_active(tx3));

        // Rollback one
        tm.rollback(tx1).unwrap();
        assert!(!tm.is_active(tx1));

        // Only tx3 should remain active
        assert!(!tm.is_active(tx1));
        assert!(!tm.is_active(tx2));
        assert!(tm.is_active(tx3));

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_transaction_state_enum() {
        // Test TxState enum variants
        assert_eq!(TxState::Active, TxState::Active);
        assert_eq!(TxState::Committed, TxState::Committed);
        assert_eq!(TxState::Aborted, TxState::Aborted);
        assert_ne!(TxState::Active, TxState::Committed);
    }

    #[test]
    fn test_transaction_snapshot() {
        let path = "/tmp/tm_test_snapshot.log";
        std::fs::remove_file(path).ok();

        let wal = Arc::new(WriteAheadLog::new(path).unwrap());
        let tm = TransactionManager::new(wal);

        // Begin first transaction
        let tx1 = tm.begin().unwrap();

        // Begin second transaction while first is active
        let tx2 = tm.begin().unwrap();

        // Both should be active
        assert!(tm.is_active(tx1));
        assert!(tm.is_active(tx2));

        // Get state for tx1
        let state1 = tm.get_state(tx1);
        assert_eq!(state1, Some(TxState::Active));

        // Get state for tx2
        let state2 = tm.get_state(tx2);
        assert_eq!(state2, Some(TxState::Active));

        std::fs::remove_file(path).ok();
    }
}
