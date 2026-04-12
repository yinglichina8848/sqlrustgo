use parking_lot::RwLock;
use sqlrustgo_storage::{StorageEngine, WalStorage};
use sqlrustgo_transaction::{TransactionError, TransactionManager, TxId};
use sqlrustgo_types::SqlError;
use std::path::PathBuf;
use std::sync::Arc;

pub struct TransactionalExecutor<S: StorageEngine> {
    storage: Arc<RwLock<S>>,
    tx_manager: Arc<RwLock<TransactionManager>>,
}

impl<S: StorageEngine> TransactionalExecutor<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            tx_manager: Arc::new(RwLock::new(TransactionManager::new())),
        }
    }

    pub fn storage(&self) -> Arc<RwLock<S>> {
        self.storage.clone()
    }

    pub fn begin(&self) -> Result<TxId, TransactionError> {
        self.tx_manager.write().begin()
    }

    pub fn commit(&self) -> Result<Option<u64>, TransactionError> {
        self.tx_manager.write().commit()
    }

    pub fn rollback(&self) -> Result<(), TransactionError> {
        self.tx_manager.write().rollback()
    }

    pub fn is_in_transaction(&self) -> bool {
        self.tx_manager.read().is_in_transaction()
    }

    pub fn current_tx_id(&self) -> Option<TxId> {
        self.tx_manager.read().get_current_tx_id()
    }

    pub fn get_transaction_context(
        &self,
    ) -> Result<sqlrustgo_transaction::TransactionContext, TransactionError> {
        self.tx_manager.read().get_transaction_context()
    }

    pub fn set_isolation_level(&self, level: sqlrustgo_transaction::IsolationLevel) {
        self.tx_manager.write().set_isolation_level(level)
    }

    pub fn execute_read<F, T>(&self, f: F) -> Result<T, TransactionError>
    where
        F: FnOnce(&dyn StorageEngine) -> T,
    {
        let ctx = self
            .tx_manager
            .read()
            .get_transaction_context_for_query()
            .map_err(|e| TransactionError::StorageError(e.to_string()))?;
        let storage = self.storage.read();
        Ok(f(&*storage))
    }
}

impl<S: StorageEngine> Drop for TransactionalExecutor<S> {
    fn drop(&mut self) {
        if self.tx_manager.read().is_in_transaction() {
            let _ = self.tx_manager.write().rollback();
        }
    }
}

/// WAL-enabled transactional executor with automatic WAL logging
pub struct WalTransactionalExecutor<S: StorageEngine> {
    storage: Arc<RwLock<WalStorage<S>>>,
    tx_manager: Arc<RwLock<TransactionManager>>,
}

impl<S: StorageEngine> WalTransactionalExecutor<S> {
    pub fn new(inner: S, wal_path: PathBuf) -> Result<Self, SqlError> {
        let wal_storage = WalStorage::new(inner, wal_path)
            .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
        Ok(Self {
            storage: Arc::new(RwLock::new(wal_storage)),
            tx_manager: Arc::new(RwLock::new(TransactionManager::new())),
        })
    }

    pub fn new_without_wal(inner: S) -> Self {
        let wal_storage = WalStorage::new_without_wal(inner);
        Self {
            storage: Arc::new(RwLock::new(wal_storage)),
            tx_manager: Arc::new(RwLock::new(TransactionManager::new())),
        }
    }

    pub fn storage(&self) -> Arc<RwLock<WalStorage<S>>> {
        self.storage.clone()
    }

    pub fn begin(&self) -> Result<TxId, TransactionError> {
        let mut storage = self.storage.write();
        storage
            .begin_transaction()
            .map_err(|e| TransactionError::StorageError(e.to_string()))?;
        self.tx_manager.write().begin()
    }

    pub fn commit(&self) -> Result<Option<u64>, TransactionError> {
        let mut storage = self.storage.write();
        storage
            .commit_transaction()
            .map_err(|e| TransactionError::StorageError(e.to_string()))?;
        self.tx_manager.write().commit()
    }

    pub fn rollback(&self) -> Result<(), TransactionError> {
        let mut storage = self.storage.write();
        storage
            .rollback_transaction()
            .map_err(|e| TransactionError::StorageError(e.to_string()))?;
        self.tx_manager.write().rollback()
    }

    pub fn is_in_transaction(&self) -> bool {
        self.tx_manager.read().is_in_transaction()
    }

    pub fn current_tx_id(&self) -> Option<TxId> {
        self.tx_manager.read().get_current_tx_id()
    }

    pub fn get_transaction_context(
        &self,
    ) -> Result<sqlrustgo_transaction::TransactionContext, TransactionError> {
        self.tx_manager.read().get_transaction_context()
    }

    pub fn set_isolation_level(&self, level: sqlrustgo_transaction::IsolationLevel) {
        self.tx_manager.write().set_isolation_level(level)
    }

    pub fn execute_read<F, T>(&self, f: F) -> Result<T, TransactionError>
    where
        F: FnOnce(&dyn StorageEngine) -> T,
    {
        let ctx = self
            .tx_manager
            .read()
            .get_transaction_context_for_query()
            .map_err(|e| TransactionError::StorageError(e.to_string()))?;
        let storage = self.storage.read();
        Ok(f(&*storage))
    }

    pub fn recover(&self) -> Result<Vec<sqlrustgo_storage::WalEntry>, SqlError> {
        let storage = self.storage.read();
        storage
            .recover()
            .map_err(|e| SqlError::ExecutionError(e.to_string()))
    }

    pub fn set_wal_enabled(&self, enabled: bool) {
        let mut storage = self.storage.write();
        storage.set_wal_enabled(enabled);
    }
}

impl<S: StorageEngine> Drop for WalTransactionalExecutor<S> {
    fn drop(&mut self) {
        if self.tx_manager.read().is_in_transaction() {
            let mut storage = self.storage.write();
            let _ = storage.rollback_transaction();
            let _ = self.tx_manager.write().rollback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::engine::MemoryStorage;

    #[test]
    fn test_transactional_executor_basic() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        let tx_id = exec.begin().unwrap();
        assert!(tx_id.is_valid());
        assert!(exec.is_in_transaction());
        assert_eq!(exec.current_tx_id(), Some(tx_id));

        let commit_ts = exec.commit().unwrap();
        assert!(commit_ts.is_some());
        assert!(!exec.is_in_transaction());
    }

    #[test]
    fn test_transactional_executor_rollback() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        let _tx_id = exec.begin().unwrap();
        assert!(exec.is_in_transaction());

        exec.rollback().unwrap();
        assert!(!exec.is_in_transaction());
    }

    #[test]
    fn test_transactional_executor_auto_rollback_on_drop() {
        let storage = MemoryStorage::new();
        {
            let exec = TransactionalExecutor::new(storage);
            exec.begin().unwrap();
        }
    }

    #[test]
    fn test_transactional_executor_no_commit_without_begin() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        let result = exec.commit();
        assert!(result.is_err());
    }

    #[test]
    fn test_transactional_executor_no_rollback_without_begin() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        let result = exec.rollback();
        assert!(result.is_err());
    }

    #[test]
    fn test_transactional_executor_isolation_level() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        exec.set_isolation_level(sqlrustgo_transaction::IsolationLevel::RepeatableRead);

        let tx_id = exec.begin().unwrap();
        let ctx = exec.get_transaction_context().unwrap();

        assert_eq!(
            ctx.isolation_level,
            sqlrustgo_transaction::IsolationLevel::RepeatableRead
        );
        assert_eq!(ctx.tx_id, tx_id);

        exec.commit().unwrap();
    }

    #[test]
    fn test_transactional_executor_double_begin_error() {
        let storage = MemoryStorage::new();
        let exec = TransactionalExecutor::new(storage);

        exec.begin().unwrap();
        let result = exec.begin();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod wal_transactional_executor_tests {
    use super::*;
    use sqlrustgo_storage::engine::MemoryStorage;
    use tempfile::TempDir;

    #[test]
    fn test_wal_transactional_executor_basic() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();

        let tx_id = exec.begin().unwrap();
        assert!(tx_id.is_valid());
        assert!(exec.is_in_transaction());
        assert_eq!(exec.current_tx_id(), Some(tx_id));

        let commit_ts = exec.commit().unwrap();
        assert!(commit_ts.is_some());
        assert!(!exec.is_in_transaction());
    }

    #[test]
    fn test_wal_transactional_executor_rollback() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();

        let _tx_id = exec.begin().unwrap();
        assert!(exec.is_in_transaction());

        exec.rollback().unwrap();
        assert!(!exec.is_in_transaction());
    }

    #[test]
    fn test_wal_transactional_executor_recovery() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let wal_path = dir.path().join("test.wal");
        {
            let exec = WalTransactionalExecutor::new(storage, wal_path.clone()).unwrap();
            exec.begin().unwrap();
            exec.commit().unwrap();
        }

        let storage2 = MemoryStorage::new();
        let exec2 = WalTransactionalExecutor::new(storage2, wal_path).unwrap();
        let entries = exec2.recover().unwrap();
        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == sqlrustgo_storage::WalEntryType::Commit)
            .collect();
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn test_wal_transactional_executor_wal_disabled() {
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new_without_wal(storage);

        exec.begin().unwrap();
        exec.commit().unwrap();

        let entries = exec.recover().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_wal_transactional_executor_double_begin_error() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();

        exec.begin().unwrap();
        let result = exec.begin();

        assert!(result.is_err());
    }

    #[test]
    fn test_wal_transactional_executor_auto_rollback_on_drop() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        {
            let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();
            exec.begin().unwrap();
        }
    }

    #[test]
    fn test_wal_transactional_executor_no_commit_without_begin() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();

        let result = exec.commit();
        assert!(result.is_err());
    }

    #[test]
    fn test_wal_transactional_executor_crash_recovery() {
        let dir = TempDir::new().unwrap();
        let wal_path = dir.path().join("crash_test.wal");

        {
            let storage = MemoryStorage::new();
            let mut exec = WalTransactionalExecutor::new(storage, wal_path.clone()).unwrap();
            exec.begin().unwrap();
            exec.commit().unwrap();

            exec.begin().unwrap();
            exec.rollback().unwrap();
        }

        let storage2 = MemoryStorage::new();
        let exec2 = WalTransactionalExecutor::new(storage2, wal_path).unwrap();
        let entries = exec2.recover().unwrap();

        let commits: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == sqlrustgo_storage::WalEntryType::Commit)
            .collect();
        let rollbacks: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type == sqlrustgo_storage::WalEntryType::Rollback)
            .collect();

        assert_eq!(commits.len(), 1);
        assert_eq!(rollbacks.len(), 1);
    }

    #[test]
    fn test_wal_transactional_executor_set_wal_enabled() {
        let dir = TempDir::new().unwrap();
        let storage = MemoryStorage::new();
        let exec = WalTransactionalExecutor::new(storage, dir.path().join("test.wal")).unwrap();

        exec.set_wal_enabled(false);
        exec.begin().unwrap();
        exec.commit().unwrap();

        let entries = exec.recover();
        assert!(entries.is_ok());
    }
}
