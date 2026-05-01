# MVCC SSI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement MVCC SSI (Serializable Snapshot Isolation) with row-level locks and commit-time conflict detection.

**Architecture:** Create new `ssi.rs` module in transaction crate with SsiDetector, SireadLock, and SerializationGraph. Integrate with existing MvccEngine and lock_manager.

**Tech Stack:** Rust, tokio async, existing transaction crate modules

---

## Task 1: Create ssi.rs with SsiError

**Files:**
- Create: `crates/transaction/src/ssi.rs`

**Step 1: Create the file with SsiError enum**

```rust
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
            SsiError::SerializationConflict { our_tx, conflicting_tx, reason } => {
                write!(f, "Serialization conflict: tx {} conflicts with tx {} - {}", our_tx, conflicting_tx, reason)
            }
            SsiError::LockTimeout => {
                write!(f, "SSI lock timeout")
            }
        }
    }
}

impl std::error::Error for SsiError {}
```

**Step 2: Verify file compiles**

Run: `cargo check -p sqlrustgo-transaction`
Expected: OK (empty module)

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "feat(transaction): add SSI error types"
```

---

## Task 2: Add SireadLock struct

**Files:**
- Modify: `crates/transaction/src/ssi.rs`

**Step 1: Add SireadLock struct**

```rust
/// SIREAD lock - records keys read by a transaction
#[derive(Debug, Clone)]
pub struct SireadLock {
    pub tx_id: TxId,
    pub keys: Vec<Vec<u8>>,
}

impl SireadLock {
    pub fn new(tx_id: TxId) -> Self {
        Self { tx_id, keys: Vec::new() }
    }

    pub fn add_key(&mut self, key: Vec<u8>) {
        self.keys.push(key);
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p sqlrustgo-transaction`
Expected: OK

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "feat(transaction): add SireadLock struct"
```

---

## Task 3: Add SerializationGraph struct

**Files:**
- Modify: `crates/transaction/src/ssi.rs`

**Step 1: Add SerializationGraph struct**

```rust
use std::collections::{HashMap, HashSet};

/// Serialization graph for detecting dangerous structures
#[derive(Debug, Clone)]
pub struct SerializationGraph {
    /// Maps tx_id -> set of tx_ids that this tx depends on (read-write)
    dependencies: HashMap<TxId, HashSet<TxId>>,
}

impl SerializationGraph {
    pub fn new() -> Self {
        Self { dependencies: HashMap::new() }
    }

    /// Add a dependency: tx1 depends on tx2 (tx1 read a key that tx2 wrote)
    pub fn add_dependency(&mut self, tx1: TxId, tx2: TxId) {
        self.dependencies.entry(tx1).or_default().insert(tx2);
    }

    /// Check if adding edge tx1->tx2 would create a cycle
    pub fn would_create_cycle(&self, tx1: TxId, tx2: TxId) -> bool {
        // If tx2 already depends on tx1, adding tx1->tx2 creates cycle
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
```

**Step 2: Verify compilation**

Run: `cargo check -p sqlrustgo-transaction`
Expected: OK

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "feat(transaction): add SerializationGraph for cycle detection"
```

---

## Task 4: Add SsiDetector struct with core methods

**Files:**
- Modify: `crates/transaction/src/ssi.rs`

**Step 1: Add SsiDetector struct**

```rust
use crate::lock_manager::{DistributedLockManager, LockKey, LockMode};
use crate::mvcc::TxId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// SSI Detector - detects serialization conflicts at commit time
pub struct SsiDetector {
    /// Read sets: tx_id -> set of keys read
    read_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    /// Write sets: tx_id -> set of keys written
    write_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    /// Serialization graph for cycle detection
    graph: RwLock<SerializationGraph>,
    /// Lock manager for row locks
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

    /// Record a read operation (SIREAD lock)
    pub async fn record_read(&self, tx_id: TxId, key: Vec<u8>) {
        let mut read_sets = self.read_sets.write().await;
        read_sets.entry(tx_id).or_default().insert(key);
    }

    /// Record a write operation and acquire X lock
    pub async fn record_write(&self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        // Add to write set
        {
            let mut write_sets = self.write_sets.write().await;
            write_sets.entry(tx_id).or_default().insert(key.clone());
        }

        // Try to acquire X lock
        let lock_key = LockKey::Row {
            table: String::new(), // Empty for now, key IS the row key
            row_key: key,
        };

        match self.locks.try_lock(&tx_id.into(), &lock_key).await {
            Ok(()) => Ok(()),
            Err(_) => Err(SsiError::LockTimeout),
        }
    }

    /// Validate commit - check for serialization conflicts
    pub async fn validate_commit(&self, tx_id: TxId) -> Result<(), SsiError> {
        let read_sets = self.read_sets.read().await;
        let write_sets = self.write_sets.read().await;

        let my_reads = read_sets.get(&tx_id).cloned().unwrap_or_default();
        let my_writes = write_sets.get(&tx_id).cloned().unwrap_or_default();

        // Check for conflicts with other active transactions
        for (other_tx, other_writes) in write_sets.iter() {
            if *other_tx == tx_id {
                continue;
            }

            // Check if other_tx wrote something we read (RW conflict)
            let rw_conflict = my_reads.intersection(other_writes).count() > 0;

            if rw_conflict {
                // Now check if we wrote something they read (WR conflict)
                if let Some(other_reads) = read_sets.get(other_tx) {
                    let wr_conflict = my_writes.intersection(other_reads).count() > 0;

                    if wr_conflict {
                        // Dangerous structure! Cycle detected
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

    /// Release all locks and clean up for a transaction
    pub async fn release(&self, tx_id: TxId) {
        // Release locks
        let _ = self.locks.unlock(&tx_id.into()).await;

        // Clean up read/write sets
        {
            let mut read_sets = self.read_sets.write().await;
            read_sets.remove(&tx_id);
        }
        {
            let mut write_sets = self.write_sets.write().await;
            write_sets.remove(&tx_id);
        }

        // Clean up graph
        {
            let mut graph = self.graph.write().await;
            graph.remove_tx(&tx_id);
        }
    }
}
```

**Step 2: Fix compilation errors**

Need to add imports and fix TxId -> GlobalTransactionId conversion. Update the file:

```rust
use crate::gid::GlobalTransactionId;

// In record_write, change:
match self.locks.try_lock(&tx_id.into(), &lock_key).await {
// to:
let gid = GlobalTransactionId::new(crate::NodeId(0)); // placeholder
match self.locks.try_lock(&gid, &lock_key).await {

// In release, change:
let _ = self.locks.unlock(&tx_id.into()).await;
// to:
let gid = GlobalTransactionId::new(crate::NodeId(0));
let _ = self.locks.unlock(&gid).await;
```

Run: `cargo check -p sqlrustgo-transaction`
Expected: OK

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "feat(transaction): add SsiDetector with core methods"
```

---

## Task 5: Update lib.rs to export ssi module

**Files:**
- Modify: `crates/transaction/src/lib.rs`

**Step 1: Add ssi module export**

```rust
pub mod mvcc;
pub mod lock_manager;
pub mod version_chain;
pub mod ssi;  // Add this

pub use mvcc::{Transaction, TxId, Snapshot, INVALID_TX_ID};
pub use ssi::{SsiDetector, SsiError};
```

**Step 2: Verify compilation**

Run: `cargo check -p sqlrustgo-transaction`
Expected: OK

**Step 3: Commit**

```bash
git add crates/transaction/src/lib.rs
git commit -m "feat(transaction): export ssi module"
```

---

## Task 6: Write unit tests for SsiError

**Files:**
- Modify: `crates/transaction/src/ssi.rs` (add tests module at end)

**Step 1: Add tests**

```rust
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
}
```

**Step 2: Run tests**

Run: `cargo test -p sqlrustgo-transaction ssi::tests`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "test(transaction): add ssi error tests"
```

---

## Task 7: Write unit tests for SerializationGraph

**Files:**
- Modify: `crates/transaction/src/ssi.rs` (add to tests module)

**Step 1: Add graph tests**

```rust
#[test]
fn test_serialization_graph_cycle_detection() {
    let graph = SerializationGraph::new();

    // No cycle initially
    assert!(!graph.would_create_cycle(TxId::new(1), TxId::new(2)));

    // Add edge 1 -> 2
    let mut graph = SerializationGraph::new();
    graph.add_dependency(TxId::new(1), TxId::new(2));

    // Adding 2 -> 1 would create cycle
    assert!(graph.would_create_cycle(TxId::new(2), TxId::new(1)));

    // Adding 1 -> 2 would not create cycle (already exists)
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
```

**Step 2: Run tests**

Run: `cargo test -p sqlrustgo-transaction ssi::tests`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "test(transaction): add serialization graph tests"
```

---

## Task 8: Write unit tests for SsiDetector

**Files:**
- Modify: `crates/transaction/src/ssi.rs` (add to tests module)

**Step 1: Add detector tests**

```rust
#[tokio::test]
async fn test_ssi_detector_no_conflict() {
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    let tx_id = TxId::new(1);
    detector.record_read(tx_id, b"key1".to_vec()).await;
    detector.record_write(tx_id, b"key2".to_vec()).await;

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
```

**Step 2: Run tests**

Run: `cargo test -p sqlrustgo-transaction ssi::tests`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/transaction/src/ssi.rs
git commit -m "test(transaction): add SsiDetector tests"
```

---

## Task 9: Integration test - SSI with FileStorage

**Files:**
- Create: `crates/transaction/tests/ssi_integration.rs`

**Step 1: Create integration test**

```rust
//! SSI integration tests with storage engines

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::{SsiDetector, SsiError, TxId};
    use sqlrustgo_storage::FileStorage;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ssi_concurrent_read_no_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let detector = SsiDetector::new(Arc::new(/* lock manager */));

        // Transaction 1 reads key1
        let tx1 = TxId::new(1);
        detector.record_read(tx1, b"key1".to_vec()).await;

        // Transaction 2 reads key1 (no conflict)
        let tx2 = TxId::new(2);
        detector.record_read(tx2, b"key1".to_vec()).await;

        // Both should commit successfully
        assert!(detector.validate_commit(tx1).await.is_ok());
        assert!(detector.validate_commit(tx2).await.is_ok());
    }
}
```

**Step 2: Verify compilation (may need adjustments)**

Run: `cargo check -p sqlrustgo-transaction --tests`
Expected: OK or errors to fix

**Step 3: Adjust test as needed and commit**

```bash
git add crates/transaction/tests/ssi_integration.rs
git commit -m "test(transaction): add SSI integration tests"
```

---

## Task 10: Final verification

**Step 1: Run all transaction tests**

Run: `cargo test -p sqlrustgo-transaction`
Expected: All tests pass

**Step 2: Check coverage**

Run: `cargo llvm-cov -p sqlrustgo-transaction --lib --summary-only`
Expected: SSI module coverage >80%

**Step 3: Final commit if needed**

```bash
git add -A && git commit -m "test(transaction): ensure >80% SSI coverage"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Create ssi.rs with SsiError | Create |
| 2 | Add SireadLock struct | Modify |
| 3 | Add SerializationGraph | Modify |
| 4 | Add SsiDetector | Modify |
| 5 | Export from lib.rs | Modify |
| 6 | Unit tests - SsiError | Modify |
| 7 | Unit tests - SerializationGraph | Modify |
| 8 | Unit tests - SsiDetector | Modify |
| 9 | Integration test | Create |
| 10 | Final verification | - |

**Total estimated tasks: 10**

---

## Plan Complete

Plan saved to `docs/plans/YYYY-MM-DD-mvcc-ssi-implementation-plan.md` (this file).
