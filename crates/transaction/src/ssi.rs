//! SSI (Serializable Snapshot Isolation) implementation

use std::collections::{HashMap, HashSet};

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

/// Serialization graph for detecting dangerous structures
#[derive(Debug, Clone)]
pub struct SerializationGraph {
    /// Maps tx_id -> set of tx_ids that this tx depends on (read-write)
    dependencies: HashMap<TxId, HashSet<TxId>>,
}

impl SerializationGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    /// Add a dependency: tx1 depends on tx2 (tx1 read a key that tx2 wrote)
    pub fn add_dependency(&mut self, tx1: TxId, tx2: TxId) {
        self.dependencies.entry(tx1).or_default().insert(tx2);
    }

    /// Check if adding edge tx1->tx2 would create a cycle
    pub fn would_create_cycle(&self, tx1: TxId, tx2: TxId) -> bool {
        if let Some(deps) = self.dependencies.get(&tx2) {
            return deps.contains(&tx1);
        }
        false
    }

    /// Remove a transaction from the graph
    pub fn remove_tx(&mut self, tx_id: &TxId) {
        self.dependencies.remove(tx_id);
        for deps in self.dependencies.values_mut() {
            deps.remove(tx_id);
        }
    }
}

impl Default for SerializationGraph {
    fn default() -> Self {
        Self::new()
    }
}
