//! SSI (Serializable Snapshot Isolation) implementation

use crate::mvcc::TxId;

/// SSI conflict error
#[derive(Debug, Clone)]
pub enum SsiError {
    SerializationConflict {
        our_tx: TxId,
        conflicting_tx: TxId,
        reason: String,
    },
    LockTimeout,
}

impl std::fmt::Display for SsiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsiError::SerializationConflict {
                our_tx,
                conflicting_tx,
                reason,
            } => {
                write!(
                    f,
                    "Serialization conflict: tx {} conflicts with tx {} - {}",
                    our_tx, conflicting_tx, reason
                )
            }
            SsiError::LockTimeout => {
                write!(f, "SSI lock timeout")
            }
        }
    }
}

impl std::error::Error for SsiError {}

/// SIREAD lock - records keys read by a transaction
#[derive(Debug, Clone)]
pub struct SireadLock {
    pub tx_id: TxId,
    pub keys: Vec<Vec<u8>>,
}

impl SireadLock {
    pub fn new(tx_id: TxId) -> Self {
        Self {
            tx_id,
            keys: Vec::new(),
        }
    }

    pub fn add_key(&mut self, key: Vec<u8>) {
        self.keys.push(key);
    }
}
