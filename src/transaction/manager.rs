//! Transaction Manager
//! Handles transaction lifecycle: BEGIN, COMMIT, ROLLBACK

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
    pub snapshot: Vec<u64>,  // Snapshot of active transactions
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
        self.wal.append(&WalRecord::Begin { tx_id }).map_err(|e| e.to_string())?;

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
        self.wal.append(&WalRecord::Commit { tx_id }).map_err(|e| e.to_string())?;

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
        self.wal.append(&WalRecord::Rollback { tx_id }).map_err(|e| e.to_string())?;

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
}
