use super::registry::{IdempotencyRecord, IdempotencyRegistry};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum WalEntry {
    IdempotentPending { key: String, hash: [u8; 32] },
    IdempotentCommit { key: String },
    IdempotentReject { key: String, reason: String },
}

pub struct IdempotencyWalAdapter {
    registry: Arc<IdempotencyRegistry>,
}

impl IdempotencyWalAdapter {
    pub fn new(registry: Arc<IdempotencyRegistry>) -> Self {
        Self { registry }
    }

    pub fn log_pending(
        &self,
        record: &IdempotencyRecord,
    ) -> Result<(), super::registry::IdempotencyError> {
        log::debug!(
            "WAL: logging idempotent pending key={} hash={}",
            record.key,
            hex::encode(record.request_hash)
        );
        Ok(())
    }

    pub fn log_commit(&self, key: &str) -> Result<(), super::registry::IdempotencyError> {
        log::debug!("WAL: logging idempotent commit key={}", key);
        self.registry.mark_committed(key)
    }

    pub fn log_reject(
        &self,
        key: &str,
        reason: &str,
    ) -> Result<(), super::registry::IdempotencyError> {
        log::debug!(
            "WAL: logging idempotent reject key={} reason={}",
            key,
            reason
        );
        self.registry.mark_rejected(key, reason)
    }

    pub fn recover_from_wal(
        &self,
        wal_entries: &[WalEntry],
    ) -> Result<usize, super::registry::IdempotencyError> {
        let mut recovered = 0;
        for entry in wal_entries {
            match entry {
                WalEntry::IdempotentPending { key, hash } => {
                    self.registry.check_and_register(key, *hash, 0)?;
                    recovered += 1;
                }
                WalEntry::IdempotentCommit { key } => {
                    self.registry.mark_committed(key)?;
                    recovered += 1;
                }
                WalEntry::IdempotentReject { key, reason } => {
                    self.registry.mark_rejected(key, reason)?;
                    recovered += 1;
                }
            }
        }
        Ok(recovered)
    }
}

#[cfg(test)]
mod wal_tests {
    use super::*;

    #[test]
    fn test_log_pending() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        let record = IdempotencyRecord {
            key: "txn-1".to_string(),
            request_hash: [0u8; 32],
            state: super::super::registry::IdempotencyState::Pending,
            created_at: 0,
            updated_at: 0,
            tx_id: 1,
        };

        let result = adapter.log_pending(&record);
        assert!(result.is_ok());
    }

    #[test]
    fn test_log_commit() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        registry.check_and_register("txn-1", [0u8; 32], 1).unwrap();

        let result = adapter.log_commit("txn-1");
        assert!(result.is_ok());

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            super::super::registry::IdempotencyState::Committed
        ));
    }

    #[test]
    fn test_log_reject() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        registry.check_and_register("txn-1", [0u8; 32], 1).unwrap();

        let result = adapter.log_reject("txn-1", "validation failed");
        assert!(result.is_ok());

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            super::super::registry::IdempotencyState::Rejected { reason }
            if reason == "validation failed"
        ));
    }

    #[test]
    fn test_recover_from_wal_pending() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        let entries = vec![WalEntry::IdempotentPending {
            key: "txn-1".to_string(),
            hash: [0u8; 32],
        }];

        let recovered = adapter.recover_from_wal(&entries).unwrap();
        assert_eq!(recovered, 1);

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            super::super::registry::IdempotencyState::Pending
        ));
    }

    #[test]
    fn test_recover_from_wal_commit() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        let entries = vec![
            WalEntry::IdempotentPending {
                key: "txn-1".to_string(),
                hash: [0u8; 32],
            },
            WalEntry::IdempotentCommit {
                key: "txn-1".to_string(),
            },
        ];

        let recovered = adapter.recover_from_wal(&entries).unwrap();
        assert_eq!(recovered, 2);

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            super::super::registry::IdempotencyState::Committed
        ));
    }

    #[test]
    fn test_recover_from_wal_reject() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let adapter = IdempotencyWalAdapter::new(registry);

        let entries = vec![
            WalEntry::IdempotentPending {
                key: "txn-1".to_string(),
                hash: [0u8; 32],
            },
            WalEntry::IdempotentReject {
                key: "txn-1".to_string(),
                reason: "validation failed".to_string(),
            },
        ];

        let recovered = adapter.recover_from_wal(&entries).unwrap();
        assert_eq!(recovered, 2);

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            super::super::registry::IdempotencyState::Rejected { reason }
            if reason == "validation failed"
        ));
    }

    #[test]
    fn test_recover_from_wal_multiple_entries() {
        let registry = Arc::new(IdempotencyRegistry::new());
        let registry_for_check = registry.clone();
        let adapter = IdempotencyWalAdapter::new(registry);

        let entries = vec![
            WalEntry::IdempotentPending {
                key: "txn-1".to_string(),
                hash: [0u8; 32],
            },
            WalEntry::IdempotentCommit {
                key: "txn-1".to_string(),
            },
            WalEntry::IdempotentPending {
                key: "txn-2".to_string(),
                hash: [1u8; 32],
            },
            WalEntry::IdempotentReject {
                key: "txn-2".to_string(),
                reason: "bad request".to_string(),
            },
        ];

        let recovered = adapter.recover_from_wal(&entries).unwrap();
        assert_eq!(recovered, 4);

        let state1 = registry_for_check.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state1,
            super::super::registry::IdempotencyState::Committed
        ));

        let state2 = registry_for_check.get_state("txn-2").unwrap().unwrap();
        assert!(matches!(
            state2,
            super::super::registry::IdempotencyState::Rejected { reason }
            if reason == "bad request"
        ));
    }
}
