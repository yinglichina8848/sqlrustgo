//! XA Transaction Coordinator
//!
//! Provides MySQL 5.7 compatible XA transaction support.
//!
//! # XA Transaction States
//!
//! - `Active`: Transaction in progress
//! - `Idle`: XA END (suspend)
//! - `Prepared`: XA PREPARE successful, waiting for commit/rollback
//! - `Committed`: XA COMMIT successful
//! - `RolledBack`: XA ROLLBACK successful
//!
//! # Example
//!
//! ```ignore
//! use sqlrustgo_distributed::xa::{XACoordinator, XaTransactionState};
//!
//! let mut coordinator = XACoordinator::new(1);
//! let xid = coordinator.xa_start("my-xid");
//! // ... do work ...
//! coordinator.xa_end(&xid);
//! coordinator.xa_prepare(&xid)?;
//! coordinator.xa_commit(&xid)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum XaError {
    #[error("Transaction {0} not found")]
    TransactionNotFound(String),
    #[error("Invalid state transition from {0} to {1}")]
    InvalidStateTransition(String, String),
    #[error("Transaction {0} already exists")]
    AlreadyExists(String),
    #[error("XA timeout for transaction {0}")]
    Timeout(String),
    #[error("Deadlock detected for transaction {0}")]
    Deadlock(String),
    #[error("IO error: {0}")]
    IoError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum XaTransactionState {
    #[default]
    Active,
    Idle,
    Prepared,
    Committed,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum XaIsolationLevel {
    #[default]
    Serializable,
    ReadCommitted,
    RepeatableRead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Xid {
    pub gtrid: u64,
    pub bqual: u64,
    pub format_id: i32,
}

impl Xid {
    pub fn new(gtrid: u64, bqual: u64, format_id: i32) -> Self {
        Self {
            gtrid,
            bqual,
            format_id,
        }
    }

    pub fn from_string(s: &str) -> Self {
        let hash = Self::hash_string(s);
        Self {
            gtrid: hash,
            bqual: 0,
            format_id: 1,
        }
    }

    fn hash_string(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl std::fmt::Display for Xid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}','{}',{}", self.gtrid, self.bqual, self.format_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaTransaction {
    pub xid: Xid,
    pub state: XaTransactionState,
    pub isolation_level: XaIsolationLevel,
    pub created_at_ms: u64,
    pub last_update_ms: u64,
    pub timeout_ms: u64,
    pub participant_nodes: Vec<u64>,
    pub sql_statements: Vec<String>,
    pub locks_held: Vec<String>,
    pub error_reason: Option<String>,
}

impl XaTransaction {
    pub fn new(xid: Xid, timeout_ms: u64) -> Self {
        let now = current_timestamp_ms();
        Self {
            xid,
            state: XaTransactionState::Active,
            isolation_level: XaIsolationLevel::Serializable,
            created_at_ms: now,
            last_update_ms: now,
            timeout_ms,
            participant_nodes: Vec::new(),
            sql_statements: Vec::new(),
            locks_held: Vec::new(),
            error_reason: None,
        }
    }

    pub fn is_timed_out(&self) -> bool {
        current_timestamp_ms() > self.last_update_ms + self.timeout_ms
    }

    pub fn update_timestamp(&mut self) {
        self.last_update_ms = current_timestamp_ms();
    }

    pub fn add_lock(&mut self, lock: String) {
        if !self.locks_held.contains(&lock) {
            self.locks_held.push(lock);
        }
    }

    pub fn release_locks(&mut self) {
        self.locks_held.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaRecoverItem {
    pub xid: Xid,
    pub committed: bool,
}

pub struct XACoordinator {
    node_id: u64,
    transactions: RwLock<HashMap<String, XaTransaction>>,
    #[allow(dead_code)]
    next_gtrid: RwLock<u64>,
    deadlock_detection_enabled: bool,
}

impl XACoordinator {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            transactions: RwLock::new(HashMap::new()),
            next_gtrid: RwLock::new(1),
            deadlock_detection_enabled: true,
        }
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    pub fn xa_start(&self, xid: Xid) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        if txs.contains_key(&key) {
            return Err(XaError::AlreadyExists(key));
        }

        let timeout_ms = 30_000;
        let tx = XaTransaction::new(xid, timeout_ms);
        txs.insert(key, tx);

        Ok(())
    }

    pub fn xa_end(&self, xid: &Xid) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;

        if tx.state != XaTransactionState::Active {
            return Err(XaError::InvalidStateTransition(
                format!("{:?}", tx.state),
                "Idle".to_string(),
            ));
        }

        tx.state = XaTransactionState::Idle;
        tx.update_timestamp();
        tx.release_locks();

        Ok(())
    }

    pub fn xa_prepare(&self, xid: &Xid) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;

        if tx.state != XaTransactionState::Idle {
            return Err(XaError::InvalidStateTransition(
                format!("{:?}", tx.state),
                "Prepared".to_string(),
            ));
        }

        if self.deadlock_detection_enabled {
            if let Some(deadlock_xid) = self.detect_deadlock(tx) {
                return Err(XaError::Deadlock(deadlock_xid.to_string()));
            }
        }

        tx.state = XaTransactionState::Prepared;
        tx.update_timestamp();

        Ok(())
    }

    pub fn xa_commit(&self, xid: &Xid) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;

        if tx.state != XaTransactionState::Prepared {
            return Err(XaError::InvalidStateTransition(
                format!("{:?}", tx.state),
                "Committed".to_string(),
            ));
        }

        tx.state = XaTransactionState::Committed;
        tx.update_timestamp();

        Ok(())
    }

    pub fn xa_rollback(&self, xid: &Xid) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;

        match tx.state {
            XaTransactionState::Active | XaTransactionState::Idle => {
                tx.state = XaTransactionState::RolledBack;
                tx.update_timestamp();
                tx.release_locks();
                Ok(())
            }
            XaTransactionState::Prepared => {
                tx.state = XaTransactionState::RolledBack;
                tx.update_timestamp();
                tx.release_locks();
                Ok(())
            }
            _ => Err(XaError::InvalidStateTransition(
                format!("{:?}", tx.state),
                "RolledBack".to_string(),
            )),
        }
    }

    pub fn xa_recover(&self) -> Vec<XaRecoverItem> {
        let txs = self
            .transactions
            .read()
            .map_err(|e| XaError::IoError(e.to_string()))
            .unwrap();

        txs.values()
            .filter(|tx| {
                matches!(
                    tx.state,
                    XaTransactionState::Prepared
                        | XaTransactionState::Active
                        | XaTransactionState::Idle
                )
            })
            .map(|tx| {
                let committed = matches!(tx.state, XaTransactionState::Prepared);
                XaRecoverItem {
                    xid: tx.xid.clone(),
                    committed,
                }
            })
            .collect()
    }

    pub fn get_transaction(&self, xid: &Xid) -> Option<XaTransaction> {
        let key = xid.to_string();
        self.transactions
            .read()
            .ok()
            .and_then(|txs| txs.get(&key).cloned())
    }

    pub fn get_state(&self, xid: &Xid) -> Option<XaTransactionState> {
        self.get_transaction(xid).map(|tx| tx.state)
    }

    pub fn add_statement(&self, xid: &Xid, sql: String) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;

        if tx.state != XaTransactionState::Active {
            return Err(XaError::InvalidStateTransition(
                format!("{:?}", tx.state),
                "Active".to_string(),
            ));
        }

        tx.sql_statements.push(sql);
        tx.update_timestamp();
        Ok(())
    }

    pub fn add_lock(&self, xid: &Xid, lock: String) -> Result<(), XaError> {
        let key = xid.to_string();
        let mut txs = self
            .transactions
            .write()
            .map_err(|e| XaError::IoError(e.to_string()))?;

        let tx = txs
            .get_mut(&key)
            .ok_or(XaError::TransactionNotFound(key.clone()))?;
        tx.add_lock(lock);
        tx.update_timestamp();
        Ok(())
    }

    pub fn get_pending_transactions(&self) -> Vec<XaTransaction> {
        let txs = match self.transactions.read() {
            Ok(txs) => txs,
            Err(_) => return Vec::new(),
        };

        txs.values()
            .filter(|tx| {
                matches!(
                    tx.state,
                    XaTransactionState::Active | XaTransactionState::Idle
                ) && tx.is_timed_out()
            })
            .cloned()
            .collect()
    }

    pub fn cleanup_completed_transactions(&self) -> usize {
        let mut txs = match self.transactions.write() {
            Ok(txs) => txs,
            Err(_) => return 0,
        };

        let before = txs.len();
        txs.retain(|_, tx| {
            !matches!(
                tx.state,
                XaTransactionState::Committed | XaTransactionState::RolledBack
            )
        });
        before - txs.len()
    }

    fn detect_deadlock(&self, _tx: &XaTransaction) -> Option<String> {
        None
    }

    pub fn set_deadlock_detection(&mut self, enabled: bool) {
        self.deadlock_detection_enabled = enabled;
    }

    pub fn stats(&self) -> XaCoordinatorStats {
        let txs = match self.transactions.read() {
            Ok(txs) => txs,
            Err(_) => return XaCoordinatorStats::default(),
        };

        XaCoordinatorStats {
            total_transactions: txs.len(),
            active: txs
                .values()
                .filter(|tx| tx.state == XaTransactionState::Active)
                .count(),
            idle: txs
                .values()
                .filter(|tx| tx.state == XaTransactionState::Idle)
                .count(),
            prepared: txs
                .values()
                .filter(|tx| tx.state == XaTransactionState::Prepared)
                .count(),
            committed: txs
                .values()
                .filter(|tx| tx.state == XaTransactionState::Committed)
                .count(),
            rolled_back: txs
                .values()
                .filter(|tx| tx.state == XaTransactionState::RolledBack)
                .count(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct XaCoordinatorStats {
    pub total_transactions: usize,
    pub active: usize,
    pub idle: usize,
    pub prepared: usize,
    pub committed: usize,
    pub rolled_back: usize,
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xid_display() {
        let xid = Xid::new(123, 456, 1);
        assert_eq!(format!("{}", xid), "'123','456',1");
    }

    #[test]
    fn test_xid_from_string() {
        let xid = Xid::from_string("test-xid-123");
        assert_eq!(xid.bqual, 0);
        assert_eq!(xid.format_id, 1);
        assert_ne!(xid.gtrid, 0);
    }

    #[test]
    fn test_xa_basic_flow() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        coordinator.xa_start(xid.clone()).unwrap();

        coordinator.xa_end(&xid).unwrap();

        coordinator.xa_prepare(&xid).unwrap();

        coordinator.xa_commit(&xid).unwrap();

        assert_eq!(
            coordinator.get_state(&xid),
            Some(XaTransactionState::Committed)
        );
    }

    #[test]
    fn test_xa_rollback() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_rollback(&xid).unwrap();

        assert_eq!(
            coordinator.get_state(&xid),
            Some(XaTransactionState::RolledBack)
        );
    }

    #[test]
    fn test_xa_recover() {
        let coordinator = XACoordinator::new(1);
        let xid1 = Xid::new(1, 0, 1);
        let xid2 = Xid::new(2, 0, 1);

        coordinator.xa_start(xid1.clone()).unwrap();
        coordinator.xa_start(xid2.clone()).unwrap();
        coordinator.xa_end(&xid1).unwrap();
        coordinator.xa_end(&xid2).unwrap();
        coordinator.xa_prepare(&xid1).unwrap();

        let recover_list = coordinator.xa_recover();
        assert_eq!(recover_list.len(), 2);
    }

    #[test]
    fn test_xa_already_exists() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        coordinator.xa_start(xid.clone()).unwrap();
        let result = coordinator.xa_start(xid.clone());
        assert!(matches!(result, Err(XaError::AlreadyExists(_))));
    }

    #[test]
    fn test_xa_invalid_state_transition() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        let result = coordinator.xa_prepare(&xid);
        assert!(matches!(result, Err(XaError::TransactionNotFound(_))));
    }

    #[test]
    fn test_xa_stats() {
        let coordinator = XACoordinator::new(1);
        let xid1 = Xid::new(1, 0, 1);
        let xid2 = Xid::new(2, 0, 1);

        coordinator.xa_start(xid1.clone()).unwrap();
        coordinator.xa_start(xid2.clone()).unwrap();
        coordinator.xa_end(&xid1).unwrap();

        let stats = coordinator.stats();
        assert_eq!(stats.total_transactions, 2);
        assert_eq!(stats.active, 1);
        assert_eq!(stats.idle, 1);
    }

    #[test]
    fn test_xa_add_statement() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator
            .add_statement(&xid, "INSERT INTO t VALUES (1)".to_string())
            .unwrap();

        let tx = coordinator.get_transaction(&xid).unwrap();
        assert_eq!(tx.sql_statements.len(), 1);
    }

    #[test]
    fn test_xa_cleanup() {
        let coordinator = XACoordinator::new(1);
        let xid = Xid::new(1, 0, 1);

        coordinator.xa_start(xid.clone()).unwrap();
        coordinator.xa_end(&xid).unwrap();
        coordinator.xa_prepare(&xid).unwrap();
        coordinator.xa_commit(&xid).unwrap();

        let cleaned = coordinator.cleanup_completed_transactions();
        assert_eq!(cleaned, 1);

        let stats = coordinator.stats();
        assert_eq!(stats.total_transactions, 0);
    }
}
