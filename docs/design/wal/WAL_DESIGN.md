# Write-Ahead Logging (WAL) Design

## Overview

The WAL provides durability by logging all database modifications before applying them to data pages. This ensures crash recovery capability.

## WAL Entry Types

```rust
pub enum WalEntryType {
    Begin = 1,       // Transaction begin
    Insert = 2,     // Row insert
    Update = 3,     // Row update
    Delete = 4,     // Row delete
    Commit = 5,     // Transaction commit
    Rollback = 6,   // Transaction rollback
    Checkpoint = 7, // Checkpoint marker
    Prepare = 8,    // 2PC prepare phase
}
```

## WAL Entry Structure

```rust
pub struct WalEntry {
    pub tx_id: u64,              // Transaction ID
    pub entry_type: WalEntryType,
    pub table_id: u64,
    pub key: Option<Vec<u8>>,    // Row key (for update/delete)
    pub data: Option<Vec<u8>>,   // Row data (for insert/update)
    pub lsn: u64,               // Log Sequence Number
    pub timestamp: u64,          // Unix timestamp (ms)
}
```

### Serialization Format

```
+--------+--------+--------+--------+--------+--------+--------+--------+
|   LSN (8 bytes)  | Timestamp (8) |  TXID (8)  | Type | TableID (8) |
+--------+--------+--------+--------+--------+--------+--------+--------+
| Key Len (4) | Key Data (variable) | Data Len (4) | Data (variable) |
+--------+--------+--------+--------+--------+--------+--------+--------+
```

## WAL Manager

```rust
pub struct WalManager {
    wal_path: PathBuf,
}
```

### Key Operations

```rust
impl WalManager {
    pub fn log_begin(&self, tx_id: u64) -> Result<()>;
    pub fn log_insert(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> Result<()>;
    pub fn log_update(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> Result<()>;
    pub fn log_delete(&self, tx_id: u64, table_id: u64, key: Vec<u8>) -> Result<()>;
    pub fn log_commit(&self, tx_id: u64) -> Result<()>;
    pub fn log_rollback(&self, tx_id: u64) -> Result<()>;
    pub fn checkpoint(&self, tx_id: u64) -> Result<()>;
}
```

### LSN Tracking

Each entry gets a unique LSN (Log Sequence Number) that increases monotonically. The LSN is used for:
- Recovery ordering
- Checkpoint identification
- Replication position tracking

## Crash Recovery

### Recovery Process

```
1. Read WAL from beginning
2. Build list of transactions
3. For each entry:
   - If BEGIN: start new transaction
   - If INSERT/UPDATE/DELETE: add to transaction's pending operations
   - If COMMIT: apply all pending operations to data pages
   - If ROLLBACK: discard pending operations
4. For incomplete transactions (no COMMIT): discard pending operations
```

### Recovery Algorithm

```rust
pub fn recover(&self) -> Result<Vec<WalEntry>> {
    let entries = self.read_all_entries()?;
    let mut committed = Vec::new();
    let mut pending: HashMap<u64, Vec<WalEntry>> = HashMap::new();
    
    for entry in entries {
        match entry.entry_type {
            WalEntryType::Begin => {
                pending.entry(entry.tx_id).or_default();
            }
            WalEntryType::Commit => {
                if let Some(ops) = pending.remove(&entry.tx_id) {
                    committed.extend(ops);
                }
            }
            WalEntryType::Rollback => {
                pending.remove(&entry.tx_id);
            }
            _ => {
                // INSERT, UPDATE, DELETE
                if let Some(ops) = pending.get_mut(&entry.tx_id) {
                    ops.push(entry);
                }
            }
        }
    }
    
    Ok(committed)
}
```

### Recovery after Crash Simulation

```
Before crash:
  T1: BEGIN, INSERT (1,100), COMMIT
  T2: BEGIN, INSERT (2,200)  <- crashed here (no COMMIT)

After recovery:
  T1's INSERT is applied (committed before crash)
  T2's INSERT is discarded (not committed)
```

## Checkpoint Mechanism

### Checkpoint Types

1. **Full Checkpoint**: All dirty pages written to data files
2. **Incremental Checkpoint**: Only modified pages since last checkpoint

### Checkpoint Configuration

```rust
pub struct CheckpointConfig {
    pub interval: Duration,        // Time between checkpoints (default: 5 min)
    pub max_wal_size_mb: u64,      // WAL size threshold (default: 100 MB)
    pub incremental: bool,         // Enable incremental checkpoints
}
```

### Checkpoint Manager

```rust
pub struct CheckpointManager {
    config: CheckpointConfig,
    last_checkpoint: Arc<RwLock<Option<CheckpointMetadata>>>,
    checkpoint_dir: PathBuf,
}

pub struct CheckpointMetadata {
    pub lsn: u64,
    pub timestamp: u64,
    pub tx_count: u64,
    pub dirty_pages: u64,
    pub file_path: PathBuf,
}
```

### Checkpoint Decision

```rust
pub fn needs_checkpoint(&self) -> bool {
    // Time-based
    if elapsed >= self.config.interval {
        return true;
    }
    // WAL size-based (TODO)
    false
}
```

### Checkpoint Workflow

```
1. Request checkpoint
2. WAL manager writes CHECKPOINT entry with current LSN
3. All dirty buffer pool pages are flushed to disk
4. Checkpoint metadata is saved
5. Old WAL segments before checkpoint LSN can be archived/deleted
```

### Checkpoint Metadata JSON Format

```json
{
  "lsn": 12345,
  "timestamp": 1609459200000,
  "tx_count": 100,
  "dirty_pages": 50,
  "file_path": "/path/to/checkpoint.1"
}
```

### WAL Truncation

After a successful checkpoint:
- WAL entries with LSN < checkpoint LSN are eligible for archival
- Old WAL files can be removed or archived for PITR

## Point-in-Time Recovery (PITR)

### Recovery Targets

```rust
pub enum RecoveryTarget {
    LSN(u64),                 // Recover to specific LSN
    Timestamp(u64),           // Recover to specific timestamp
    TransactionId(u64),       // Recover to specific transaction
}
```

### PITR Recovery Manager

```rust
pub struct PITRRecovery {
    wal_path: PathBuf,
    backup_path: PathBuf,
    current_lsn: Arc<RwLock<u64>>,
}
```

### Recovery Planning

```rust
pub fn prepare_recovery(&self, target: RecoveryTarget) -> SqlResult<RecoveryPlan> {
    let recovery_point = self.find_recovery_point(target)?;
    let plan = self.build_recovery_plan(&recovery_point)?;
    Ok(plan)
}

pub struct RecoveryPlan {
    pub recovery_point: RecoveryPoint,
    pub base_backup_required: bool,
    pub wal_replay_required: bool,
    pub affected_table_ids: Vec<u64>,
    pub estimated_rollback_entries: usize,
}
```

### Finding Recovery Point by Timestamp

```rust
fn find_lsn_by_timestamp(&self, timestamp: u64) -> SqlResult<u64> {
    let entries = self.read_wal_entries(0)?;
    
    for entry in entries.iter().rev() {
        if entry.timestamp <= timestamp {
            return Ok(entry.lsn);
        }
    }
    Ok(0)
}
```

### PITR Workflow

```
1. Take base backup of data files
2. Archive WAL continuously
3. On failure, determine target recovery point
4. Restore base backup
5. Replay WAL entries up to target point
6. Database is now at exact point in time
```

### Partial Table Recovery

```rust
pub fn build_recovery_plan(&self, point: &RecoveryPoint) -> SqlResult<RecoveryPlan> {
    let lsn = /* derived from point */;
    let entries = self.read_wal_entries(0)?;
    
    let mut affected_tables = HashSet::new();
    for entry in &entries {
        if entry.lsn <= lsn {
            affected_tables.insert(entry.table_id);
        }
    }
    
    Ok(RecoveryPlan {
        affected_table_ids: affected_tables.into_iter().collect(),
        // ...
    })
}
```

## WAL Storage Format

### File Structure

```
WAL File: wal.binlog
+--------+----------+--------+----------+
| Header | Entry 1  | Entry 2| Entry N  |
+--------+----------+--------+----------+
```

- Header: Magic number (0x57414C01) + Version (1)
- Entries: Variable-length binary records

### WAL Writer

```rust
pub struct WalWriter {
    file: BufWriter<File>,
    current_lsn: u64,
}
```

### WAL Reader

```rust
pub struct WalReader {
    file: BufReader<File>,
}
```

## WAL Integration with MVCC

```
Transaction T1:
  BEGIN (LSN 100)
  INSERT (LSN 101) -> WAL
  UPDATE (LSN 102) -> WAL
  COMMIT (LSN 103) -> WAL

During recovery:
  - LSN 100: T1 begins
  - LSN 101-102: T1's modifications logged
  - LSN 103: T1 commits -> apply modifications to MVCC
```

## Concurrency

- Single WAL writer per database (serialized)
- Multiple readers can read WAL concurrently
- Checkpoint can run concurrently with queries

## Configuration

```rust
pub struct WalConfig {
    pub wal_dir: PathBuf,           // WAL file directory
    pub wal_file_size_mb: u64,      // Individual WAL file size
    pub checkpoint_interval_secs: u64,
    pub enable_pitr: bool,          // Enable point-in-time recovery
}
```

## Example: Crash During Multi-Statement Transaction

```sql
BEGIN;
INSERT INTO accounts VALUES (1, 1000);
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
-- CRASH --
```

After recovery:
- Transaction has no COMMIT in WAL
- All modifications from this transaction are rolled back
- Account balance remains unchanged (1000)

## Example: PITR to Timestamp

```sql
-- At 10:00 AM: Accidental DELETE FROM orders
-- At 10:05 AM: Discovery of problem

PITR Recovery:
  Target: Timestamp of 09:59:59
  1. Restore base backup
  2. Replay WAL up to target timestamp
  3. Database restored to state before accident
```
