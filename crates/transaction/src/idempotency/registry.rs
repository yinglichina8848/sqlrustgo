use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum IdempotencyState {
    Pending,
    Committed,
    Rejected { reason: String },
}

#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: String,
    pub request_hash: [u8; 32],
    pub state: IdempotencyState,
    pub created_at: u64,
    pub updated_at: u64,
    pub tx_id: u64,
}

#[derive(Debug, Clone)]
pub struct IdempotencyRegistry {
    records: Arc<RwLock<HashMap<String, IdempotencyRecord>>>,
}

impl IdempotencyRegistry {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for IdempotencyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl IdempotencyRegistry {
    pub fn check_and_register(
        &self,
        key: &str,
        request_hash: [u8; 32],
        tx_id: u64,
    ) -> Result<bool, IdempotencyError> {
        let mut records = self
            .records
            .write()
            .map_err(|_| IdempotencyError::LockError)?;

        if let Some(existing) = records.get(key) {
            if existing.request_hash == request_hash {
                match existing.state {
                    IdempotencyState::Committed => Ok(true),
                    IdempotencyState::Pending => Ok(false),
                    IdempotencyState::Rejected { ref reason } => {
                        Err(IdempotencyError::PreviouslyRejected(reason.clone()))
                    }
                }
            } else {
                Err(IdempotencyError::HashMismatch(format!(
                    "Same key '{}' used with different request payload",
                    key
                )))
            }
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            records.insert(
                key.to_string(),
                IdempotencyRecord {
                    key: key.to_string(),
                    request_hash,
                    state: IdempotencyState::Pending,
                    created_at: now,
                    updated_at: now,
                    tx_id,
                },
            );
            Ok(false)
        }
    }

    pub fn mark_committed(&self, key: &str) -> Result<(), IdempotencyError> {
        let mut records = self
            .records
            .write()
            .map_err(|_| IdempotencyError::LockError)?;
        if let Some(record) = records.get_mut(key) {
            record.state = IdempotencyState::Committed;
            record.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        Ok(())
    }

    pub fn mark_rejected(&self, key: &str, reason: &str) -> Result<(), IdempotencyError> {
        let mut records = self
            .records
            .write()
            .map_err(|_| IdempotencyError::LockError)?;
        if let Some(record) = records.get_mut(key) {
            record.state = IdempotencyState::Rejected {
                reason: reason.to_string(),
            };
            record.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        Ok(())
    }

    pub fn get_state(&self, key: &str) -> Result<Option<IdempotencyState>, IdempotencyError> {
        let records = self
            .records
            .read()
            .map_err(|_| IdempotencyError::LockError)?;
        Ok(records.get(key).map(|r| r.state.clone()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    #[error("Lock error")]
    LockError,
    #[error("Same key '{0}' used with different request payload")]
    HashMismatch(String),
    #[error("Request was previously rejected: {0}")]
    PreviouslyRejected(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request_not_idempotent() {
        let registry = IdempotencyRegistry::new();
        let hash = [0u8; 32];
        let result = registry.check_and_register("txn-1", hash, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_same_request_is_idempotent() {
        let registry = IdempotencyRegistry::new();
        let hash = [0u8; 32];

        registry.check_and_register("txn-1", hash, 1).unwrap();
        let result = registry.check_and_register("txn-1", hash, 2);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_different_hash_same_key_rejected() {
        let registry = IdempotencyRegistry::new();

        registry.check_and_register("txn-1", [0u8; 32], 1).unwrap();
        let result = registry.check_and_register("txn-1", [1u8; 32], 2);

        assert!(matches!(result, Err(IdempotencyError::HashMismatch(_))));
    }

    #[test]
    fn test_mark_committed() {
        let registry = IdempotencyRegistry::new();
        let hash = [0u8; 32];

        registry.check_and_register("txn-1", hash, 1).unwrap();
        registry.mark_committed("txn-1").unwrap();

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(state, IdempotencyState::Committed));
    }

    #[test]
    fn test_mark_rejected() {
        let registry = IdempotencyRegistry::new();
        let hash = [0u8; 32];

        registry.check_and_register("txn-1", hash, 1).unwrap();
        registry
            .mark_rejected("txn-1", "validation failed")
            .unwrap();

        let state = registry.get_state("txn-1").unwrap().unwrap();
        assert!(matches!(
            state,
            IdempotencyState::Rejected { reason } if reason == "validation failed"
        ));
    }

    #[test]
    fn test_previously_rejected_error() {
        let registry = IdempotencyRegistry::new();
        let hash = [0u8; 32];

        registry.check_and_register("txn-1", hash, 1).unwrap();
        registry.mark_rejected("txn-1", "first rejection").unwrap();

        let result = registry.check_and_register("txn-1", hash, 2);
        assert!(matches!(
            result,
            Err(IdempotencyError::PreviouslyRejected(msg)) if msg == "first rejection"
        ));
    }
}
