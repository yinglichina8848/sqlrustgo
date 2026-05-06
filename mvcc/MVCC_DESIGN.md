# Multi-Version Concurrency Control (MVCC) Design

## Overview

MVCC provides concurrent transaction support by maintaining multiple versions of data rows. Each transaction sees a consistent snapshot of the database without blocking readers.

## Transaction Identifiers

```rust
pub struct TxId(u64);
pub const INVALID_TX_ID: u64 = 0;
```

Transactions are identified by a monotonically increasing `TxId`. The `MvccEngine` maintains:
- `next_tx_id`: Next available transaction ID
- `global_timestamp`: Monotonically increasing logical timestamp

## Transaction States

```
     Active
       |
   commit()        abort()
       |              |
       v              v
  Committed -----> Aborted
```

```rust
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}
```

## Snapshot Isolation

### Snapshot Structure

```rust
pub struct Snapshot {
    pub tx_id: TxId,                       // Owner transaction
    pub snapshot_timestamp: u64,           // All reads see data before this time
    pub active_transactions: Vec<TxId>,     // Transactions running at snapshot time
}
```

### Visibility Rules

A row version is visible if:

1. **Own transaction**: `created_by == snapshot.tx_id` — always visible (even uncommitted)
2. **Active transaction**: `created_by` is in `active_transactions` — NOT visible
3. **Committed before snapshot**: `created_commit_ts < snapshot_timestamp` — visible
4. **Deleted check**: If row has a delete marker, `deleted_commit_ts <= snapshot_timestamp` — NOT visible

```rust
pub fn is_visible(&self, snapshot: &Snapshot) -> bool {
    // Own uncommitted version is visible
    if self.created_by == snapshot.tx_id {
        return true;
    }
    // Active uncommitted version is not visible
    for active in &snapshot.active_transactions {
        if *active == self.created_by {
            return false;
        }
    }
    // Must be committed
    let created_ts = match self.created_commit_ts {
        Some(ts) => ts,
        None => return false,
    };
    // Committed after snapshot is not visible
    if created_ts > snapshot.snapshot_timestamp {
        return false;
    }
    // Check delete marker
    if !self.value.is_empty() {
        if let Some(deleted_ts) = self.deleted_commit_ts {
            if deleted_ts <= snapshot.snapshot_timestamp {
                return false;
            }
        }
    }
    true
}
```

## Version Chain Management

### Structure

```rust
pub struct RowVersion {
    pub value: Vec<u8>,                    // Row data
    pub created_by: TxId,                  // Creating transaction
    pub created_commit_ts: Option<u64>,    // Commit timestamp (None = uncommitted)
    pub deleted_by: Option<TxId>,          // Deleting transaction
    pub deleted_commit_ts: Option<u64>,    // Delete commit timestamp
}

pub struct VersionChainMap {
    chains: RwLock<HashMap<Vec<u8>, Vec<RowVersion>>>,
}
```

### Version Storage Layout

```
Key: "row_key"
Chain (oldest -> newest):
  [RowVersion{v1, tx=1, commit_ts=10}, 
   RowVersion{v2, tx=2, commit_ts=20}, 
   RowVersion{empty, tx=3, commit_ts=30}]  <- delete marker
```

- Versions stored oldest-to-newest in the chain
- `find_visible()` searches newest-to-oldest, returns first visible
- Empty `value` with `deleted_by` set indicates a deletion

### Commit Operation

```rust
pub fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) {
    for versions in chains.values_mut() {
        for version in versions.iter_mut() {
            if version.created_by == tx_id && version.created_commit_ts.is_none() {
                version.created_commit_ts = Some(commit_ts);
                // Delete markers: creation commit also serves as delete commit
                if version.value.is_empty() {
                    version.deleted_commit_ts = Some(commit_ts);
                }
            }
        }
    }
}
```

### Rollback Operation

```rust
pub fn rollback_versions(&mut self, tx_id: TxId) {
    for versions in chains.values_mut() {
        versions.retain(|v| v.created_by != tx_id || v.created_commit_ts.is_some());
    }
}
```

### Garbage Collection

Old versions are garbage collected when:
- Version is a delete marker (`value.is_empty()`)
- Deletion is committed (`deleted_commit_ts` is Some)
- No active transactions depend on it
- Deletion is "old enough" (gap from oldest_snapshot_ts > threshold)

```rust
const RECENT_GAP_THRESHOLD: u64 = 5;

pub fn gc(&mut self, active_transactions: &[TxId], oldest_snapshot_ts: u64) -> usize {
    // ... for each key ...
    let gap = oldest_snapshot_ts.saturating_sub(deleted_ts);
    if gap <= Self::RECENT_GAP_THRESHOLD {
        return false;  // preserve
    }
    true  // safe to collect
}
```

## SSI (Serializable Snapshot Isolation)

SSI detects serialization conflicts at commit time by tracking read-write dependencies.

### Conflict Types

1. **RW Conflict**: T1 reads key X, T2 writes key X (after T1's read)
2. **WR Conflict**: T1 writes key Y, T2 reads key Y (after T1's write)
3. **WW Conflict**: T1 writes key Z, T2 writes key Z

### SIREAD Locks

Records keys read by each transaction:

```rust
pub struct SireadLock {
    pub tx_id: TxId,
    pub keys: Vec<Vec<u8>>,
}
```

### Serialization Graph

Tracks dependencies between transactions:

```rust
pub struct SerializationGraph {
    dependencies: HashMap<TxId, HashSet<TxId>>,
}
```

An edge `tx1 -> tx2` means tx1 depends on tx2 (tx1 read a key that tx2 wrote).

### Cycle Detection

```rust
pub fn would_create_cycle(&self, tx1: TxId, tx2: TxId) -> bool {
    if let Some(deps) = self.dependencies.get(&tx2) {
        return deps.contains(&tx1);
    }
    false
}
```

A dangerous structure (RW-WR cycle) indicates a serialization conflict.

### Validation at Commit

```rust
pub fn validate_commit(&self, tx_id: TxId) -> Result<(), SsiError> {
    for (other_tx, other_writes) in write_sets.iter() {
        if *other_tx == tx_id { continue; }
        
        // Check RW conflict: our reads vs their writes
        let rw_conflict = my_reads.intersection(other_writes).count() > 0;
        
        if rw_conflict {
            // Check WR conflict: their reads vs our writes
            if let Some(other_reads) = read_sets.get(other_tx) {
                let wr_conflict = my_writes.intersection(other_reads).count() > 0;
                
                if wr_conflict {
                    return Err(SsiError::SerializationConflict {
                        reason: "RW-WR cycle detected",
                    });
                }
            }
        }
    }
    Ok(())
}
```

## Integration with Lock Manager

The `LockManager` provides row-level locking that complements MVCC:

```rust
pub enum LockMode {
    Shared,      // Multiple readers
    Exclusive,   // Single writer
}
```

- Shared locks allow concurrent readers
- Exclusive locks block readers and other writers
- Lock upgrade: Shared -> Exclusive (when no other waiters)

## Isolation Levels

| Level | Dirty Reads | Non-Repeatable Reads | Phantoms |
|-------|-------------|---------------------|----------|
| Read Uncommitted | Possible | Possible | Possible |
| Read Committed | Prevented | Possible | Possible |
| Snapshot (SI) | Prevented | Prevented | Possible |
| Serializable (SSI) | Prevented | Prevented | Prevented |

SQLRustGo implements:
- **Read Committed**: Fresh snapshot per statement
- **Snapshot Isolation**: Consistent snapshot per transaction
- **Serializable**: SSI with read-write conflict detection

## Example: T1=(Read A, Write B), T2=(Read B, Write A)

This creates a dangerous cycle:

1. T1 reads A (records A in read set, dependency T1 -> T2 if T2 wrote A)
2. T2 reads B (records B in read set, dependency T2 -> T1 if T1 wrote B)
3. T1 writes B
4. T2 writes A

At commit:
- T1 sees T2's write to B (RW conflict) and T2's read of A (WR conflict)
- This RW-WR cycle indicates serialization failure
- One transaction must be aborted

## Configuration

```rust
pub struct MvccConfig {
    pub gc_threshold: u64,        // Version GC threshold
    pub max_versions_per_key: usize, // Maximum versions to retain
}
```
