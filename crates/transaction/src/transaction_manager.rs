use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use crate::idempotency::{IdempotencyError, IdempotencyRegistry};
use crate::lock::{LockGrantMode, LockManager, LockMode, LockTarget};
use crate::mvcc::{Snapshot, TxId};
use crate::ssi::{SsiDetectorSync, SsiError};
use sqlrustgo_observability::tables::OBSERVABILITY;

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationLevel {
    /// Snapshot Isolation - readers see consistent snapshot, writers use first-committer-wins
    #[default]
    SnapshotIsolation,
    /// Repeatable Read - readers see data as of first read timestamp (MySQL default)
    RepeatableRead,
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
    /// Idempotency key for this transaction (if any)
    pub idempotency_key: Option<String>,
}

impl ActiveTransaction {
    pub fn new(tx_id: TxId, snapshot: Snapshot, idempotency_key: Option<String>) -> Self {
        Self {
            tx_id,
            snapshot,
            state: TransactionState::Active,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            idempotency_key,
        }
    }
}

/// Result of an idempotent begin transaction
#[derive(Debug)]
pub enum IdempotentBeginResult {
    /// Transaction was already committed (idempotent success)
    IdempotentSuccess { key: String },
    /// New transaction created successfully
    NewTransaction { tx_id: TxId },
}

/// Error type combining idempotency errors with SSI errors
#[derive(Debug)]
pub enum IdempotencyOrSsiError {
    IdempotencyError(String),
    HashError(SsiError),
    SsiError(SsiError),
}

impl From<SsiError> for IdempotencyOrSsiError {
    fn from(err: SsiError) -> Self {
        IdempotencyOrSsiError::SsiError(err)
    }
}

/// Transaction manager with SSI (Serializable Snapshot Isolation) support
pub struct TransactionManager {
    ssi_detector: SsiDetectorSync,
    lock_manager: LockManager,
    active_transactions: HashMap<TxId, ActiveTransaction>,
    next_tx_id: u64,
    global_timestamp: u64,
    idempotency_registry: Arc<IdempotencyRegistry>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            ssi_detector: SsiDetectorSync::new(),
            lock_manager: LockManager::new(),
            active_transactions: HashMap::new(),
            next_tx_id: 1,
            global_timestamp: 1,
            idempotency_registry: Arc::new(IdempotencyRegistry::new()),
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
    pub fn begin_transaction(&mut self, isolation: IsolationLevel) -> Result<TxId, SsiError> {
        let tx_id = TxId::new(self.next_tx_id);
        self.next_tx_id += 1;

        let snapshot_timestamp = self.global_timestamp;
        self.global_timestamp += 1;

        let snapshot = match isolation {
            IsolationLevel::RepeatableRead => {
                Snapshot::new_repeatable_read_from_start(tx_id, snapshot_timestamp, Vec::new())
            }
            _ => Snapshot::new_read_committed(tx_id, snapshot_timestamp),
        };

        let active_tx = ActiveTransaction::new(tx_id, snapshot, None);
        self.active_transactions.insert(tx_id, active_tx);

        if let Ok(mut history) = OBSERVABILITY.transaction_history.write() {
            let entry =
                sqlrustgo_observability::tables::transaction_history::TransactionHistoryEntry::new(
                    tx_id.as_u64(),
                    format!("{:?}", isolation),
                );
            history.append(entry);
        }

        Ok(tx_id)
    }

    fn compute_request_hash(
        statement: &sqlrustgo_parser::TransactionStatement,
    ) -> Result<[u8; 32], SsiError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        statement.hash(&mut hasher);
        let hash = hasher.finish();

        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result[8..16].copy_from_slice(&hash.to_le_bytes());
        result[16..24].copy_from_slice(&hash.to_le_bytes());
        result[24..32].copy_from_slice(&hash.to_le_bytes());
        Ok(result)
    }

    pub fn begin_transaction_idempotent(
        &mut self,
        key: &str,
        statement: &sqlrustgo_parser::TransactionStatement,
        isolation: IsolationLevel,
    ) -> Result<IdempotentBeginResult, IdempotencyOrSsiError> {
        let tx_id = TxId::new(self.next_tx_id);
        self.next_tx_id += 1;
        let msg = format!(
            "DEBUG begin_transaction_idempotent: key={}, tx_id={}\n",
            key,
            tx_id.as_u64()
        );
        std::fs::write("/tmp/debug.log", &msg).ok();
        std::io::stderr().write_all(msg.as_bytes()).ok();
        std::io::stderr().flush().ok();

        let request_hash =
            Self::compute_request_hash(statement).map_err(IdempotencyOrSsiError::HashError)?;

        match self
            .idempotency_registry
            .check_and_register(key, request_hash, tx_id.as_u64())
        {
            Ok(is_idempotent) => {
                eprintln!(
                    "DEBUG begin_transaction_idempotent: key={}, is_idempotent={}",
                    key, is_idempotent
                );
                if is_idempotent {
                    return Ok(IdempotentBeginResult::IdempotentSuccess {
                        key: key.to_string(),
                    });
                }
            }
            Err(IdempotencyError::HashMismatch(_)) => {
                return Err(IdempotencyOrSsiError::IdempotencyError(format!(
                    "Transaction with key '{}' already exists with different content",
                    key
                )));
            }
            Err(IdempotencyError::PreviouslyRejected(reason)) => {
                return Err(IdempotencyOrSsiError::IdempotencyError(reason));
            }
            Err(IdempotencyError::LockError) => {
                return Err(IdempotencyOrSsiError::IdempotencyError(
                    "Lock error".to_string(),
                ));
            }
        }

        let snapshot_timestamp = self.global_timestamp;
        self.global_timestamp += 1;

        let snapshot = match isolation {
            IsolationLevel::RepeatableRead => {
                Snapshot::new_repeatable_read_from_start(tx_id, snapshot_timestamp, Vec::new())
            }
            _ => Snapshot::new_read_committed(tx_id, snapshot_timestamp),
        };

        eprintln!(
            "DEBUG begin_transaction_idempotent: creating active_tx with key={}",
            key
        );
        let active_tx = ActiveTransaction::new(tx_id, snapshot, Some(key.to_string()));
        self.active_transactions.insert(tx_id, active_tx);

        Ok(IdempotentBeginResult::NewTransaction { tx_id })
    }

    pub fn mark_idempotent_committed(&self, key: &str) -> Result<(), IdempotencyError> {
        self.idempotency_registry.mark_committed(key)
    }

    pub fn get_idempotency_registry(&self) -> Arc<IdempotencyRegistry> {
        self.idempotency_registry.clone()
    }

    /// Record a read operation for SSI detection and snapshot management
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    /// * `key` - Key being read
    /// * `isolation` - Isolation level (determines if first-read timestamp is tracked)
    pub fn record_read(
        &mut self,
        tx_id: TxId,
        key: Vec<u8>,
        isolation: IsolationLevel,
    ) -> Result<(), SsiError> {
        self.ssi_detector.record_read(tx_id, key.clone());

        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.read_keys.push(key);

            if isolation == IsolationLevel::RepeatableRead {
                let read_ts = self.global_timestamp;
                self.global_timestamp += 1;
                active_tx.snapshot.snapshot_timestamp = read_ts;
            }
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

    /// Acquire a lock on a key for a transaction
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    /// * `key` - Key to lock
    /// * `mode` - Lock mode (Shared or Exclusive)
    ///
    /// # Returns
    /// * `Ok(LockGrantMode)` - Whether lock was granted or waiter
    /// * `Err(LockError)` - If deadlock detected or other error
    pub fn acquire_lock(
        &mut self,
        tx_id: TxId,
        key: Vec<u8>,
        mode: LockMode,
    ) -> Result<LockGrantMode, crate::lock::LockError> {
        self.lock_manager.acquire_lock(tx_id, key, mode)
    }

    /// Acquire a lock with a target (supports Gap and NextKey locks)
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    /// * `target` - Lock target (Record, Gap, or NextKey)
    /// * `mode` - Lock mode (Shared or Exclusive)
    ///
    /// # Returns
    /// * `Ok(LockGrantMode)` - Whether lock was granted or waiter
    /// * `Err(LockError)` - If deadlock detected or other error
    pub fn acquire_lock_with_target(
        &mut self,
        tx_id: TxId,
        target: LockTarget,
        mode: LockMode,
    ) -> Result<LockGrantMode, crate::lock::LockError> {
        self.lock_manager
            .acquire_lock_with_target(tx_id, target, mode)
    }

    /// Release all locks held by a transaction
    ///
    /// # Arguments
    /// * `tx_id` - Transaction ID
    ///
    /// # Returns
    /// * `Ok(())` - If locks released successfully
    pub fn release_all_locks(&mut self, tx_id: TxId) -> Result<(), crate::lock::LockError> {
        self.lock_manager.release_all_locks_full(tx_id)?;
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

        let idempotency_key = if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Committed;
            let key = active_tx.idempotency_key.clone();
            eprintln!(
                "DEBUG commit: tx_id={}, idempotency_key={:?}",
                tx_id.as_u64(),
                key
            );
            key
        } else {
            eprintln!("DEBUG commit: tx_id={}, no active_tx found", tx_id.as_u64());
            None
        };

        self.ssi_detector.release(tx_id);
        self.lock_manager.release_all_locks_full(tx_id).ok();
        self.active_transactions.remove(&tx_id);

        if let Ok(mut history) = OBSERVABILITY.transaction_history.write() {
            history.update_status(
                tx_id.as_u64(),
                sqlrustgo_observability::tables::transaction_history::TransactionStatus::Committed,
            );
        }

        if let Some(ref key) = idempotency_key {
            eprintln!("DEBUG commit: calling mark_committed for key={}", key);
            self.mark_idempotent_committed(key).ok();
        }

        Ok(())
    }

    pub fn rollback(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Aborted;
        }

        self.ssi_detector.release(tx_id);
        self.lock_manager.release_all_locks_full(tx_id).ok();
        self.active_transactions.remove(&tx_id);

        if let Ok(mut history) = OBSERVABILITY.transaction_history.write() {
            history.update_status(
                tx_id.as_u64(),
                sqlrustgo_observability::tables::transaction_history::TransactionStatus::Aborted,
            );
        }

        Ok(())
    }

    pub fn abort(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        if let Some(active_tx) = self.active_transactions.get_mut(&tx_id) {
            active_tx.state = TransactionState::Aborted;
        }

        self.ssi_detector.release(tx_id);
        self.lock_manager.release_all_locks_full(tx_id).ok();
        self.active_transactions.remove(&tx_id);

        if let Ok(mut history) = OBSERVABILITY.transaction_history.write() {
            history.update_status(
                tx_id.as_u64(),
                sqlrustgo_observability::tables::transaction_history::TransactionStatus::Aborted,
            );
        }

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

    /// Get count of active transactions (for state leak detection)
    pub fn active_transaction_count(&self) -> usize {
        self.active_transactions.len()
    }

    /// Get all active transaction IDs (for debugging)
    pub fn get_active_tx_ids(&self) -> Vec<TxId> {
        self.active_transactions.keys().cloned().collect()
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

        mgr.record_read(tx_id, b"key1".to_vec(), IsolationLevel::SnapshotIsolation)
            .unwrap();
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

        mgr.record_read(tx_id, b"key1".to_vec(), IsolationLevel::SnapshotIsolation)
            .unwrap();
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

        mgr.record_read(tx_id, b"key1".to_vec(), IsolationLevel::SnapshotIsolation)
            .unwrap();
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

        mgr.record_read(tx1, b"key1".to_vec(), IsolationLevel::SnapshotIsolation)
            .unwrap();
        mgr.record_write(tx2, b"key2".to_vec()).unwrap();

        mgr.commit(tx1).unwrap();
        mgr.commit(tx2).unwrap();
    }

    #[test]
    fn test_begin_transaction_idempotent_new_key() {
        use sqlrustgo_parser::TransactionStatement;

        let mut mgr = TransactionManager::new();
        let key = "txn-test-1";
        let statement = TransactionStatement::Begin {
            work: false,
            isolation_level: None,
        };

        let result =
            mgr.begin_transaction_idempotent(key, &statement, IsolationLevel::SnapshotIsolation);

        assert!(result.is_ok());
        match result.unwrap() {
            IdempotentBeginResult::NewTransaction { tx_id } => {
                assert_eq!(tx_id.as_u64(), 1);
            }
            IdempotentBeginResult::IdempotentSuccess { .. } => {
                panic!("Expected new transaction, got idempotent success");
            }
        }
    }

    #[test]
    fn test_begin_transaction_idempotent_same_key_idempotent() {
        use sqlrustgo_parser::TransactionStatement;

        let mut mgr = TransactionManager::new();
        let key = "txn-test-2";
        let statement = TransactionStatement::Begin {
            work: false,
            isolation_level: None,
        };

        let result1 =
            mgr.begin_transaction_idempotent(key, &statement, IsolationLevel::SnapshotIsolation);

        assert!(result1.is_ok());
        match result1.unwrap() {
            IdempotentBeginResult::NewTransaction { tx_id: _ } => {}
            IdempotentBeginResult::IdempotentSuccess { .. } => {
                panic!("Expected new transaction on first call");
            }
        }

        mgr.mark_idempotent_committed(key).unwrap();

        let result2 =
            mgr.begin_transaction_idempotent(key, &statement, IsolationLevel::SnapshotIsolation);

        assert!(result2.is_ok());
        match result2.unwrap() {
            IdempotentBeginResult::IdempotentSuccess { key: k } => {
                assert_eq!(k, key);
            }
            IdempotentBeginResult::NewTransaction { .. } => {
                panic!("Expected idempotent success on second call");
            }
        }
    }

    #[test]
    fn test_begin_transaction_idempotent_different_content_rejected() {
        use sqlrustgo_parser::TransactionStatement;

        let mut mgr = TransactionManager::new();
        let key = "txn-test-3";

        let statement1 = TransactionStatement::Begin {
            work: false,
            isolation_level: None,
        };

        let result1 =
            mgr.begin_transaction_idempotent(key, &statement1, IsolationLevel::SnapshotIsolation);

        assert!(result1.is_ok());

        let statement2 = TransactionStatement::Begin {
            work: true,
            isolation_level: None,
        };

        let result2 =
            mgr.begin_transaction_idempotent(key, &statement2, IsolationLevel::SnapshotIsolation);

        assert!(result2.is_err());
        match result2.unwrap_err() {
            IdempotencyOrSsiError::IdempotencyError(msg) => {
                assert!(msg.contains("already exists with different content"));
            }
            _ => panic!("Expected idempotency error"),
        }
    }

    #[test]
    fn test_mark_idempotent_committed() {
        use sqlrustgo_parser::TransactionStatement;

        let mgr = TransactionManager::new();
        let key = "txn-test-4";
        let statement = TransactionStatement::Begin {
            work: false,
            isolation_level: None,
        };

        let registry = mgr.get_idempotency_registry();
        registry.check_and_register(key, [0u8; 32], 1).unwrap();

        let result = mgr.mark_idempotent_committed(key);
        assert!(result.is_ok());

        let state = registry.get_state(key).unwrap().unwrap();
        assert!(matches!(
            state,
            crate::idempotency::IdempotencyState::Committed
        ));
    }

    #[test]
    fn test_get_idempotency_registry() {
        let mgr = TransactionManager::new();
        let registry = mgr.get_idempotency_registry();
        assert!(!registry.get_state("nonexistent").unwrap().is_some());
    }
}
