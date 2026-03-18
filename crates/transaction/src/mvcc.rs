//! MVCC (Multi-Version Concurrency Control) implementation
//!
//! This module provides snapshot isolation, version chain management,
//! and visibility checking for concurrent transaction support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const INVALID_TX_ID: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxId(u64);

impl TxId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn invalid() -> Self {
        Self(INVALID_TX_ID)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != INVALID_TX_ID
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for TxId {
    fn default() -> Self {
        Self::invalid()
    }
}

impl std::fmt::Display for TxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TxId,
    pub status: TransactionStatus,
    pub start_timestamp: u64,
    pub commit_timestamp: Option<u64>,
}

impl Transaction {
    pub fn new(id: TxId, start_timestamp: u64) -> Self {
        Self {
            id,
            status: TransactionStatus::Active,
            start_timestamp,
            commit_timestamp: None,
        }
    }

    pub fn commit(&mut self, commit_timestamp: u64) {
        self.status = TransactionStatus::Committed;
        self.commit_timestamp = Some(commit_timestamp);
    }

    pub fn abort(&mut self) {
        self.status = TransactionStatus::Aborted;
    }

    pub fn is_active(&self) -> bool {
        self.status == TransactionStatus::Active
    }

    pub fn is_committed(&self) -> bool {
        self.status == TransactionStatus::Committed
    }

    pub fn is_aborted(&self) -> bool {
        self.status == TransactionStatus::Aborted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Snapshot {
    pub tx_id: TxId,
    pub snapshot_timestamp: u64,
    pub active_transactions: Vec<TxId>,
}

impl Snapshot {
    pub fn new(tx_id: TxId, snapshot_timestamp: u64, active: Vec<TxId>) -> Self {
        Self {
            tx_id,
            snapshot_timestamp,
            active_transactions: active,
        }
    }

    pub fn new_read_committed(tx_id: TxId, snapshot_timestamp: u64) -> Self {
        Self {
            tx_id,
            snapshot_timestamp,
            active_transactions: Vec::new(),
        }
    }

    pub fn is_visible(&self, tx_id: TxId, commit_timestamp: Option<u64>) -> bool {
        if tx_id == self.tx_id {
            return true;
        }

        for active in &self.active_transactions {
            if *active == tx_id {
                return false;
            }
        }

        match commit_timestamp {
            Some(ts) => ts < self.snapshot_timestamp,
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChain {
    pub row_key: Vec<u8>,
    pub versions: Vec<RowVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowVersion {
    pub tx_id: TxId,
    pub commit_timestamp: Option<u64>,
    pub data: Vec<u8>,
    pub next_version: Option<Box<RowVersion>>,
    pub is_deleted: bool,
}

impl RowVersion {
    pub fn new(tx_id: TxId, data: Vec<u8>) -> Self {
        Self {
            tx_id,
            commit_timestamp: None,
            data,
            next_version: None,
            is_deleted: false,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.is_deleted = true;
    }

    pub fn commit(&mut self, timestamp: u64) {
        self.commit_timestamp = Some(timestamp);
    }
}

pub struct MvccEngine {
    transactions: HashMap<TxId, Transaction>,
    next_tx_id: u64,
    global_timestamp: u64,
}

impl MvccEngine {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            next_tx_id: 1,
            global_timestamp: 1,
        }
    }

    pub fn begin_transaction(&mut self) -> TxId {
        let tx_id = TxId::new(self.next_tx_id);
        self.next_tx_id += 1;

        let start_ts = self.global_timestamp;
        self.global_timestamp += 1;

        let tx = Transaction::new(tx_id, start_ts);
        self.transactions.insert(tx_id, tx);

        tx_id
    }

    pub fn commit_transaction(&mut self, tx_id: TxId) -> Option<u64> {
        if let Some(tx) = self.transactions.get_mut(&tx_id) {
            let commit_ts = self.global_timestamp;
            self.global_timestamp += 1;
            tx.commit(commit_ts);
            Some(commit_ts)
        } else {
            None
        }
    }

    pub fn abort_transaction(&mut self, tx_id: TxId) -> bool {
        if let Some(tx) = self.transactions.get_mut(&tx_id) {
            tx.abort();
            true
        } else {
            false
        }
    }

    pub fn get_transaction(&self, tx_id: TxId) -> Option<&Transaction> {
        self.transactions.get(&tx_id)
    }

    pub fn create_snapshot(&self, tx_id: TxId) -> Snapshot {
        let active: Vec<TxId> = self
            .transactions
            .values()
            .filter(|t| t.is_active())
            .map(|t| t.id)
            .collect();

        Snapshot::new(tx_id, self.global_timestamp, active)
    }

    pub fn get_global_timestamp(&self) -> u64 {
        self.global_timestamp
    }
}

impl Default for MvccEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let mut engine = MvccEngine::new();

        let tx_id = engine.begin_transaction();
        assert!(tx_id.is_valid());

        let tx = engine.get_transaction(tx_id).unwrap();
        assert!(tx.is_active());

        let commit_ts = engine.commit_transaction(tx_id).unwrap();
        let tx = engine.get_transaction(tx_id).unwrap();
        assert!(tx.is_committed());
        assert_eq!(tx.commit_timestamp, Some(commit_ts));
    }

    #[test]
    fn test_transaction_abort() {
        let mut engine = MvccEngine::new();

        let tx_id = engine.begin_transaction();
        assert!(engine.abort_transaction(tx_id));

        let tx = engine.get_transaction(tx_id).unwrap();
        assert!(tx.is_aborted());
    }

    #[test]
    fn test_snapshot_visibility() {
        let mut engine = MvccEngine::new();

        let tx1 = engine.begin_transaction();
        engine.commit_transaction(tx1).unwrap();

        let tx2 = engine.begin_transaction();
        let snapshot = engine.create_snapshot(tx2);

        assert!(snapshot.is_visible(tx1, Some(1)));
    }

    #[test]
    fn test_row_version_commit() {
        let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        assert!(version.commit_timestamp.is_none());

        version.commit(100);
        assert_eq!(version.commit_timestamp, Some(100));
    }
}
