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

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::gid::GlobalTransactionId;
use crate::lock_manager::{DistributedLockManager, LockKey};

pub struct SsiDetector {
    read_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    write_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    graph: RwLock<SerializationGraph>,
    locks: Arc<DistributedLockManager>,
}

impl SsiDetector {
    pub fn new(locks: Arc<DistributedLockManager>) -> Self {
        Self {
            read_sets: RwLock::new(HashMap::new()),
            write_sets: RwLock::new(HashMap::new()),
            graph: RwLock::new(SerializationGraph::new()),
            locks,
        }
    }

    pub async fn record_read(&self, tx_id: TxId, key: Vec<u8>) {
        let mut read_sets = self.read_sets.write().await;
        read_sets.entry(tx_id).or_default().insert(key);
    }

    pub async fn record_write(&self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        {
            let mut write_sets = self.write_sets.write().await;
            write_sets.entry(tx_id).or_default().insert(key.clone());
        }

        let lock_key = LockKey::Row {
            table: String::new(),
            row_key: key,
        };

        let gid = GlobalTransactionId::new(crate::gid::NodeId(0));
        match self.locks.try_lock(&gid, &lock_key).await {
            Ok(()) => Ok(()),
            Err(_) => Err(SsiError::LockTimeout),
        }
    }

    pub async fn validate_commit(&self, tx_id: TxId) -> Result<(), SsiError> {
        let read_sets = self.read_sets.read().await;
        let write_sets = self.write_sets.read().await;

        let my_reads = read_sets.get(&tx_id).cloned().unwrap_or_default();
        let my_writes = write_sets.get(&tx_id).cloned().unwrap_or_default();

        for (other_tx, other_writes) in write_sets.iter() {
            if *other_tx == tx_id {
                continue;
            }

            let rw_conflict = my_reads.intersection(other_writes).count() > 0;

            if rw_conflict {
                if let Some(other_reads) = read_sets.get(other_tx) {
                    let wr_conflict = my_writes.intersection(other_reads).count() > 0;

                    if wr_conflict {
                        return Err(SsiError::SerializationConflict {
                            our_tx: tx_id,
                            conflicting_tx: *other_tx,
                            reason: "RW-WR cycle detected".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn release(&self, tx_id: TxId) {
        let gid = GlobalTransactionId::new(crate::gid::NodeId(0));
        let _ = self.locks.unlock(&gid).await;

        {
            let mut read_sets = self.read_sets.write().await;
            read_sets.remove(&tx_id);
        }
        {
            let mut write_sets = self.write_sets.write().await;
            write_sets.remove(&tx_id);
        }

        {
            let mut graph = self.graph.write().await;
            graph.remove_tx(&tx_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssi_error_display() {
        let err = SsiError::LockTimeout;
        assert_eq!(err.to_string(), "SSI lock timeout");

        let err = SsiError::SerializationConflict {
            our_tx: TxId::new(1),
            conflicting_tx: TxId::new(2),
            reason: "test".to_string(),
        };
        assert!(err.to_string().contains("Serialization conflict"));
    }

    #[test]
    fn test_serialization_graph_cycle_detection() {
        let graph = SerializationGraph::new();
        assert!(!graph.would_create_cycle(TxId::new(1), TxId::new(2)));

        let mut graph = SerializationGraph::new();
        graph.add_dependency(TxId::new(1), TxId::new(2));
        assert!(graph.would_create_cycle(TxId::new(2), TxId::new(1)));
        assert!(!graph.would_create_cycle(TxId::new(1), TxId::new(2)));
    }

    #[test]
    fn test_serialization_graph_remove_tx() {
        let mut graph = SerializationGraph::new();
        graph.add_dependency(TxId::new(1), TxId::new(2));
        graph.add_dependency(TxId::new(2), TxId::new(3));

        graph.remove_tx(&TxId::new(2));

        assert!(!graph.would_create_cycle(TxId::new(1), TxId::new(2)));
        assert!(!graph.would_create_cycle(TxId::new(1), TxId::new(3)));
    }

    #[tokio::test]
    async fn test_ssi_detector_no_conflict() {
        let locks = Arc::new(DistributedLockManager::new());
        let detector = SsiDetector::new(locks);

        let tx_id = TxId::new(1);
        detector.record_read(tx_id, b"key1".to_vec()).await;
        let _ = detector.record_write(tx_id, b"key2".to_vec()).await;

        let result = detector.validate_commit(tx_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ssi_detector_record_read() {
        let locks = Arc::new(DistributedLockManager::new());
        let detector = SsiDetector::new(locks);

        let tx_id = TxId::new(1);
        detector.record_read(tx_id, b"key1".to_vec()).await;
        detector.record_read(tx_id, b"key2".to_vec()).await;

        let read_sets = detector.read_sets.read().await;
        let reads = read_sets.get(&tx_id).unwrap();
        assert_eq!(reads.len(), 2);
    }

    #[tokio::test]
    async fn test_ssi_detector_record_write() {
        let locks = Arc::new(DistributedLockManager::new());
        let detector = SsiDetector::new(locks);

        let tx_id = TxId::new(1);
        let result = detector.record_write(tx_id, b"key1".to_vec()).await;
        assert!(result.is_ok());

        let write_sets = detector.write_sets.read().await;
        let writes = write_sets.get(&tx_id).unwrap();
        assert!(writes.contains(&b"key1".to_vec()));
    }
}
