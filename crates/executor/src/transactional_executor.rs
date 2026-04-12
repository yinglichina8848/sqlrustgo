use parking_lot::RwLock;
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_transaction::{TransactionError, TransactionManager, TxId};
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
