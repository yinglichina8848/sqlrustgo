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

    pub fn is_visible_read_committed(
        &self,
        tx_id: TxId,
        commit_timestamp: Option<u64>,
        current_timestamp: u64,
    ) -> bool {
        if tx_id == self.tx_id {
            return true;
        }

        match commit_timestamp {
            Some(ts) => ts < current_timestamp,
            None => false,
        }
    }

    pub fn refresh_for_read_committed(&mut self, current_timestamp: u64) {
        self.snapshot_timestamp = current_timestamp;
        self.active_transactions.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChain {
    pub row_key: Vec<u8>,
    pub versions: Vec<RowVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowVersion {
    pub value: Vec<u8>,
    pub created_by: TxId,
    pub created_commit_ts: Option<u64>,
    pub deleted_by: Option<TxId>,
    pub deleted_commit_ts: Option<u64>,
}

impl RowVersion {
    pub fn new(tx_id: TxId, value: Vec<u8>) -> Self {
        Self {
            value,
            created_by: tx_id,
            created_commit_ts: None,
            deleted_by: None,
            deleted_commit_ts: None,
        }
    }

    pub fn new_deleted(tx_id: TxId) -> Self {
        Self {
            value: Vec::new(),
            created_by: tx_id,
            created_commit_ts: None,
            deleted_by: Some(tx_id),
            deleted_commit_ts: None,
        }
    }

    pub fn commit(&mut self, timestamp: u64) {
        self.created_commit_ts = Some(timestamp);
    }

    pub fn mark_deleted(&mut self, tx_id: TxId, timestamp: u64) {
        self.deleted_by = Some(tx_id);
        self.deleted_commit_ts = Some(timestamp);
    }

    pub fn is_visible(&self, snapshot: &Snapshot) -> bool {
        if self.created_by == snapshot.tx_id {
            return true;
        }
        let created_ts = match self.created_commit_ts {
            Some(ts) => ts,
            None => return false,
        };
        if created_ts > snapshot.snapshot_timestamp {
            return false;
        }
        if let Some(deleted_ts) = self.deleted_commit_ts {
            if deleted_ts <= snapshot.snapshot_timestamp {
                return false;
            }
        }
        true
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
        assert!(version.created_commit_ts.is_none());

        version.commit(100);
        assert_eq!(version.created_commit_ts, Some(100));
    }

    #[test]
    fn test_row_version_is_visible_own_uncommitted() {
        let version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        let snapshot = Snapshot::new(TxId::new(1), 100, vec![TxId::new(1)]);
        assert!(version.is_visible(&snapshot));
    }

    #[test]
    fn test_row_version_is_visible_other_uncommitted() {
        let version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        let snapshot = Snapshot::new(TxId::new(2), 100, vec![TxId::new(1)]);
        assert!(!version.is_visible(&snapshot));
    }

    #[test]
    fn test_row_version_is_visible_committed_before_snapshot() {
        let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        version.commit(50);
        let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
        assert!(version.is_visible(&snapshot));
    }

    #[test]
    fn test_row_version_is_visible_committed_after_snapshot() {
        let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        version.commit(150);
        let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
        assert!(!version.is_visible(&snapshot));
    }

    #[test]
    fn test_row_version_is_visible_deleted_after_snapshot() {
        let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        version.commit(50);
        version.mark_deleted(TxId::new(2), 125);
        let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
        assert!(version.is_visible(&snapshot));
    }

    #[test]
    fn test_row_version_is_visible_deleted_before_snapshot() {
        let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
        version.commit(50);
        version.mark_deleted(TxId::new(2), 75);
        let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
        assert!(!version.is_visible(&snapshot));
    }
}
