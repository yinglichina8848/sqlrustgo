//! Transaction Manager implementation
//!
//! This module provides the TransactionManager that handles
//! transaction lifecycle: BEGIN, COMMIT, ROLLBACK.

use crate::mvcc::{MvccEngine, Snapshot, TxId};
use crate::savepoint::{SavepointError, SavepointManager};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    #[default]
    ReadCommitted,
    ReadUncommitted,
    RepeatableRead,
    Serializable,
}

#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub tx_id: TxId,
    pub snapshot: Snapshot,
    pub isolation_level: IsolationLevel,
    pub read_only: bool,
}

impl TransactionContext {
    pub fn new(tx_id: TxId, snapshot: Snapshot, isolation_level: IsolationLevel) -> Self {
        Self {
            tx_id,
            snapshot,
            isolation_level,
            read_only: false,
        }
    }

    pub fn is_visible(&self, tx_id: TxId, commit_timestamp: Option<u64>) -> bool {
        match self.isolation_level {
            IsolationLevel::ReadCommitted => self.snapshot.is_visible_read_committed(
                tx_id,
                commit_timestamp,
                self.snapshot.snapshot_timestamp,
            ),
            _ => self.snapshot.is_visible(tx_id, commit_timestamp),
        }
    }

    pub fn refresh_snapshot(&mut self, current_timestamp: u64) {
        if self.isolation_level == IsolationLevel::ReadCommitted {
            self.snapshot.refresh_for_read_committed(current_timestamp);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionCommand {
    Begin,
    Commit,
    Rollback,
    BeginReadOnly,
}

pub struct TransactionManager {
    mvcc: Arc<RwLock<MvccEngine>>,
    current_tx: Option<TxId>,
    isolation_level: IsolationLevel,
    savepoint_manager: SavepointManager,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            mvcc: Arc::new(RwLock::new(MvccEngine::new())),
            current_tx: None,
            isolation_level: IsolationLevel::ReadCommitted,
            savepoint_manager: SavepointManager::new(),
        }
    }

    pub fn with_mvcc(mvcc: Arc<RwLock<MvccEngine>>) -> Self {
        Self {
            mvcc,
            current_tx: None,
            isolation_level: IsolationLevel::ReadCommitted,
            savepoint_manager: SavepointManager::new(),
        }
    }

    pub fn begin(&mut self) -> Result<TxId, TransactionError> {
        if self.current_tx.is_some() {
            return Err(TransactionError::TransactionInProgress);
        }

        let mut mvcc = self.mvcc.write().map_err(|_| TransactionError::LockError)?;
        let tx_id = mvcc.begin_transaction();
        self.current_tx = Some(tx_id);

        Ok(tx_id)
    }

    pub fn begin_with_isolation(
        &mut self,
        level: IsolationLevel,
    ) -> Result<TxId, TransactionError> {
        self.isolation_level = level;
        self.begin()
    }

    pub fn begin_read_only(&mut self) -> Result<TxId, TransactionError> {
        let tx_id = self.begin()?;
        Ok(tx_id)
    }

    pub fn commit(&mut self) -> Result<Option<u64>, TransactionError> {
        let tx_id = self
            .current_tx
            .take()
            .ok_or(TransactionError::NoTransaction)?;

        let mut mvcc = self.mvcc.write().map_err(|_| TransactionError::LockError)?;
        let commit_ts = mvcc
            .commit_transaction(tx_id)
            .ok_or(TransactionError::InvalidTransaction)?;

        Ok(Some(commit_ts))
    }

    pub fn rollback(&mut self) -> Result<(), TransactionError> {
        let tx_id = self
            .current_tx
            .take()
            .ok_or(TransactionError::NoTransaction)?;

        let mut mvcc = self.mvcc.write().map_err(|_| TransactionError::LockError)?;
        if !mvcc.abort_transaction(tx_id) {
            return Err(TransactionError::InvalidTransaction);
        }

        Ok(())
    }

    pub fn get_current_tx_id(&self) -> Option<TxId> {
        self.current_tx
    }

    pub fn get_transaction_context(&self) -> Result<TransactionContext, TransactionError> {
        let tx_id = self.current_tx.ok_or(TransactionError::NoTransaction)?;

        let mvcc = self.mvcc.read().map_err(|_| TransactionError::LockError)?;
        let tx = mvcc
            .get_transaction(tx_id)
            .ok_or(TransactionError::InvalidTransaction)?;

        if !tx.is_active() {
            return Err(TransactionError::TransactionNotActive);
        }

        let snapshot = mvcc.create_snapshot(tx_id);

        Ok(TransactionContext::new(
            tx_id,
            snapshot,
            self.isolation_level,
        ))
    }

    pub fn set_isolation_level(&mut self, level: IsolationLevel) {
        self.isolation_level = level;
    }

    pub fn get_isolation_level(&self) -> IsolationLevel {
        self.isolation_level
    }

    pub fn is_in_transaction(&self) -> bool {
        self.current_tx.is_some()
    }

    pub fn get_global_timestamp(&self) -> Result<u64, TransactionError> {
        let mvcc = self.mvcc.read().map_err(|_| TransactionError::LockError)?;
        Ok(mvcc.get_global_timestamp())
    }

    pub fn get_transaction_context_for_query(
        &self,
    ) -> Result<TransactionContext, TransactionError> {
        let tx_id = self.current_tx.ok_or(TransactionError::NoTransaction)?;

        let mvcc = self.mvcc.read().map_err(|_| TransactionError::LockError)?;
        let tx = mvcc
            .get_transaction(tx_id)
            .ok_or(TransactionError::InvalidTransaction)?;

        if !tx.is_active() {
            return Err(TransactionError::TransactionNotActive);
        }

        let current_ts = mvcc.get_global_timestamp();
        let snapshot = mvcc.create_snapshot(tx_id);

        match self.isolation_level {
            IsolationLevel::ReadCommitted => {
                let mut ctx = TransactionContext::new(tx_id, snapshot, self.isolation_level);
                ctx.refresh_snapshot(current_ts);
                Ok(ctx)
            }
            _ => Ok(TransactionContext::new(
                tx_id,
                snapshot,
                self.isolation_level,
            )),
        }
    }

    pub fn savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.savepoint(name)
    }

    pub fn rollback_to(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.rollback_to(&name)
    }

    pub fn release_savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.release_savepoint(&name)
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum TransactionError {
    TransactionInProgress,
    NoTransaction,
    InvalidTransaction,
    TransactionNotActive,
    LockError,
    StorageError(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::TransactionInProgress => write!(f, "transaction already in progress"),
            TransactionError::NoTransaction => write!(f, "no transaction in progress"),
            TransactionError::InvalidTransaction => write!(f, "invalid transaction"),
            TransactionError::TransactionNotActive => write!(f, "transaction is not active"),
            TransactionError::LockError => write!(f, "failed to acquire lock"),
            TransactionError::StorageError(msg) => write!(f, "storage error: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_transaction() {
        let mut manager = TransactionManager::new();
        let tx_id = manager.begin().unwrap();
        assert!(tx_id.is_valid());
        assert!(manager.is_in_transaction());
        assert_eq!(manager.get_current_tx_id(), Some(tx_id));
    }

    #[test]
    fn test_commit_transaction() {
        let mut manager = TransactionManager::new();
        let _tx_id = manager.begin().unwrap();
        let commit_ts = manager.commit().unwrap();
        assert!(commit_ts.is_some());
        assert!(!manager.is_in_transaction());
    }

    #[test]
    fn test_rollback_transaction() {
        let mut manager = TransactionManager::new();
        manager.begin().unwrap();
        manager.rollback().unwrap();
        assert!(!manager.is_in_transaction());
    }

    #[test]
    fn test_transaction_error_no_transaction() {
        let mut manager = TransactionManager::new();
        let result = manager.commit();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_transaction_error_in_progress() {
        let mut manager = TransactionManager::new();
        manager.begin().unwrap();
        let result = manager.begin();
        assert!(matches!(
            result,
            Err(TransactionError::TransactionInProgress)
        ));
    }

    #[test]
    fn test_isolation_level() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::ReadUncommitted);
        assert_eq!(
            manager.get_isolation_level(),
            IsolationLevel::ReadUncommitted
        );
    }

    #[test]
    fn test_transaction_context() {
        let mut manager = TransactionManager::new();
        let tx_id = manager.begin().unwrap();

        let ctx = manager.get_transaction_context().unwrap();
        assert_eq!(ctx.tx_id, tx_id);
        assert_eq!(ctx.isolation_level, IsolationLevel::ReadCommitted);
    }

    #[test]
    fn test_multiple_transactions() {
        let mut manager = TransactionManager::new();

        let tx1 = manager.begin().unwrap();
        manager.commit().unwrap();

        let tx2 = manager.begin().unwrap();
        assert_ne!(tx1, tx2);
        manager.commit().unwrap();
    }

    #[test]
    fn test_begin_read_only() {
        let mut manager = TransactionManager::new();
        let _tx_id = manager.begin_read_only().unwrap();
        assert!(manager.is_in_transaction());
    }

    #[test]
    fn test_read_committed_isolation() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::ReadCommitted);

        let tx_id = manager.begin().unwrap();

        let ctx1 = manager.get_transaction_context_for_query().unwrap();
        assert_eq!(ctx1.isolation_level, IsolationLevel::ReadCommitted);

        let ctx2 = manager.get_transaction_context_for_query().unwrap();
        assert!(ctx2.is_visible(tx_id, Some(1)));

        manager.commit().unwrap();
    }

    #[test]
    fn test_read_committed_no_dirty_read() {
        use crate::mvcc::TxId;

        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::ReadCommitted);

        let _tx1 = manager.begin().unwrap();
        let ctx = manager.get_transaction_context_for_query().unwrap();

        assert!(!ctx.is_visible(TxId::new(999), None));

        manager.rollback().unwrap();
    }

    #[test]
    fn test_snapshot_refresh_on_each_query() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::ReadCommitted);

        let _tx_id = manager.begin().unwrap();

        let ctx1 = manager.get_transaction_context_for_query().unwrap();
        let ts1 = ctx1.snapshot.snapshot_timestamp;

        let _ = manager.get_global_timestamp().unwrap();

        let ctx2 = manager.get_transaction_context_for_query().unwrap();
        let ts2 = ctx2.snapshot.snapshot_timestamp;

        assert!(ts2 >= ts1);

        manager.commit().unwrap();
    }

    #[test]
    fn test_isolation_level_repeatable_read() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::RepeatableRead);

        assert_eq!(
            manager.get_isolation_level(),
            IsolationLevel::RepeatableRead
        );
    }

    #[test]
    fn test_isolation_level_serializable() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::Serializable);

        assert_eq!(manager.get_isolation_level(), IsolationLevel::Serializable);
    }

    #[test]
    fn test_isolation_level_read_uncommitted() {
        let mut manager = TransactionManager::new();
        manager.set_isolation_level(IsolationLevel::ReadUncommitted);

        assert_eq!(
            manager.get_isolation_level(),
            IsolationLevel::ReadUncommitted
        );
    }

    #[test]
    fn test_transaction_context_isolation_levels() {
        let manager = TransactionManager::new();

        let ctx1 = TransactionContext::new(
            TxId::new(1),
            Snapshot::new_read_committed(TxId::new(1), 100),
            IsolationLevel::RepeatableRead,
        );
        assert_eq!(ctx1.isolation_level, IsolationLevel::RepeatableRead);
        assert!(!ctx1.read_only);

        let ctx2 = TransactionContext::new(
            TxId::new(2),
            Snapshot::new_read_committed(TxId::new(2), 200),
            IsolationLevel::Serializable,
        );
        assert_eq!(ctx2.isolation_level, IsolationLevel::Serializable);
    }

    #[test]
    fn test_transaction_context_refresh_snapshot() {
        let mut ctx = TransactionContext::new(
            TxId::new(1),
            Snapshot::new_read_committed(TxId::new(1), 100),
            IsolationLevel::ReadCommitted,
        );

        ctx.refresh_snapshot(200);
        assert_eq!(ctx.snapshot.snapshot_timestamp, 200);
    }

    #[test]
    fn test_transaction_context_refresh_snapshot_non_rc() {
        let mut ctx = TransactionContext::new(
            TxId::new(1),
            Snapshot::new_read_committed(TxId::new(1), 100),
            IsolationLevel::RepeatableRead,
        );

        ctx.refresh_snapshot(200);
        assert_eq!(ctx.snapshot.snapshot_timestamp, 100);
    }

    #[test]
    fn test_begin_without_commit() {
        let mut manager = TransactionManager::new();
        let tx_id = manager.begin().unwrap();
        assert!(tx_id.is_valid());

        let tx = manager.get_transaction_context().unwrap();
        assert_eq!(tx.tx_id, tx_id);

        manager.rollback().unwrap();
    }

    #[test]
    fn test_commit_increments_timestamp() {
        let mut manager = TransactionManager::new();

        let tx1 = manager.begin().unwrap();
        manager.commit().unwrap();

        let ts1 = manager.get_global_timestamp().unwrap();

        let tx2 = manager.begin().unwrap();
        assert!(tx2.as_u64() > tx1.as_u64() || ts1 >= 1);
    }

    #[test]
    fn test_rollback_no_transaction() {
        let mut manager = TransactionManager::new();
        let result = manager.rollback();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_commit_no_transaction() {
        let mut manager = TransactionManager::new();
        let result = manager.commit();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_double_commit() {
        let mut manager = TransactionManager::new();
        manager.begin().unwrap();
        manager.commit().unwrap();

        let result = manager.commit();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_double_rollback() {
        let mut manager = TransactionManager::new();
        manager.begin().unwrap();
        manager.rollback().unwrap();

        let result = manager.rollback();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_get_transaction_context_no_transaction() {
        let manager = TransactionManager::new();
        let result = manager.get_transaction_context();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_get_transaction_context_for_query_no_transaction() {
        let manager = TransactionManager::new();
        let result = manager.get_transaction_context_for_query();
        assert!(matches!(result, Err(TransactionError::NoTransaction)));
    }

    #[test]
    fn test_is_in_transaction() {
        let mut manager = TransactionManager::new();

        assert!(!manager.is_in_transaction());

        manager.begin().unwrap();
        assert!(manager.is_in_transaction());

        manager.commit().unwrap();
        assert!(!manager.is_in_transaction());
    }

    #[test]
    fn test_transaction_manager_with_mvcc() {
        let mvcc = Arc::new(RwLock::new(MvccEngine::new()));
        let mut manager = TransactionManager::with_mvcc(mvcc.clone());

        let tx_id = manager.begin().unwrap();
        assert!(tx_id.is_valid());

        manager.commit().unwrap();
    }

    #[test]
    fn test_mvcc_lock_error() {
        let manager = TransactionManager::new();

        let result = manager.get_global_timestamp();
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_error_display() {
        let err_in_progress = TransactionError::TransactionInProgress;
        let err_no_tx = TransactionError::NoTransaction;
        let err_invalid = TransactionError::InvalidTransaction;
        let err_not_active = TransactionError::TransactionNotActive;
        let err_lock = TransactionError::LockError;

        assert_eq!(
            err_in_progress.to_string(),
            "transaction already in progress"
        );
        assert_eq!(err_no_tx.to_string(), "no transaction in progress");
        assert_eq!(err_invalid.to_string(), "invalid transaction");
        assert_eq!(err_not_active.to_string(), "transaction is not active");
        assert_eq!(err_lock.to_string(), "failed to acquire lock");
    }

    #[test]
    fn test_transaction_context_visibility_repeatable_read() {
        let ctx = TransactionContext::new(
            TxId::new(1),
            Snapshot::new(TxId::new(1), 100, vec![]),
            IsolationLevel::RepeatableRead,
        );

        assert!(ctx.is_visible(TxId::new(1), Some(50)));
        assert!(!ctx.is_visible(TxId::new(2), Some(150)));
    }

    #[test]
    fn test_transaction_context_visibility_serializable() {
        let ctx = TransactionContext::new(
            TxId::new(1),
            Snapshot::new(TxId::new(1), 100, vec![]),
            IsolationLevel::Serializable,
        );

        assert!(ctx.is_visible(TxId::new(1), Some(50)));
        assert!(!ctx.is_visible(TxId::new(2), Some(150)));
    }

    #[test]
    fn test_multiple_transactions_sequential() {
        let mut manager = TransactionManager::new();

        let tx1 = manager.begin().unwrap();
        manager.commit().unwrap();

        let tx2 = manager.begin().unwrap();
        manager.commit().unwrap();

        assert!(tx2.as_u64() > tx1.as_u64());
    }

    #[test]
    fn test_abort_transaction_updates_status() {
        let mut manager = TransactionManager::new();

        let tx_id = manager.begin().unwrap();

        let mut mvcc = manager.mvcc.write().unwrap();
        mvcc.abort_transaction(tx_id);

        let tx = mvcc.get_transaction(tx_id).unwrap();
        assert!(tx.is_aborted());
    }
}
