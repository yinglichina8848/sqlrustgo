# Transaction Design

This document describes the transaction management system for SqlRustGo, covering ACID properties, transaction manager, lock manager, and deadlock detection.

## ACID Properties

SqlRustGo implements full ACID transaction semantics:

### Atomicity

Transactions are atomic: all operations succeed or all fail together.

**Implementation**:
- Write-Ahead Logging (WAL) records all modifications
- Transaction commit/abort based on log flush
- Automatic rollback on failure

```
BEGIN;
  UPDATE accounts SET balance = balance - 100 WHERE id = 1;
  UPDATE accounts SET balance = balance + 100 WHERE id = 2;
COMMIT;
-- If any fails, both rolled back
```

### Consistency

The database transitions from one consistent state to another.

**Implementation**:
- Constraints checked before transaction commits
- Foreign key relationships enforced
- Unique indices maintained
- Triggers run atomically

```
-- Constraint ensures balance never goes negative
ALTER TABLE accounts ADD CHECK (balance >= 0);
```

### Isolation

Concurrent transactions appear serialized. SqlRustGo supports multiple isolation levels:

| Level | Dirty Read | Non-repeatable Read | Phantom |
|-------|------------|---------------------|---------|
| `READ UNCOMMITTED` | Possible | Possible | Possible |
| `READ COMMITTED` | Prevented | Possible | Possible |
| `REPEATABLE READ` | Prevented | Prevented | Possible |
| `SERIALIZABLE` | Prevented | Prevented | Prevented |

**Implementation**: MVCC with snapshot isolation at `READ COMMITTED` and above.

### Durability

Committed transactions survive system crashes.

**Implementation**:
- WAL flushed to disk on commit
- Checkpointing for recovery
- Group commit for throughput

## Transaction Manager

### Overview

The Transaction Manager (TM) coordinates transaction lifecycle, logging, and recovery.

### Transaction States

```
                    +---------+
    BEGIN           | active  |
       +----------->+---------+
                    |
       +------------+------------+
       |                         |
       v                         v
  +--------+               +-----------+
  | abort  |<--ROLLBACK/   | committing|
  +--------+   ERROR       +-----------+
                            |           |
                            v           v
                       +--------+  +---------+
                       |aborted |  |committed|
                       +--------+  +---------+
```

### Transaction Context

Each transaction maintains:

```rust
struct Transaction {
    xid: TransactionId,           // Unique transaction ID
    xmin: TransactionId,         // Snapshot xmin (for MVCC)
    snapshot: Snapshot,           // Visibility snapshot
    start_time: Timestamp,        // Start timestamp
    isolation_level: Isolation,   // Requested isolation
    state: TransState,           // Current state
    ResourceManagers: Vec<RM>,   // Participating RMs
    logger: Logger,              // WAL handle
}
```

### Two-Phase Commit (2PC)

For distributed transactions:

```
Phase 1: Prepare
  Coordinator -> Participants: "PREPARE"
  Participant -> Coordinator: "VOTE COMMIT" or "VOTE ABORT"

Phase 2: Commit/Rollback
  If all vote COMMIT:
    Coordinator -> Participants: "COMMIT"
  Else:
    Coordinator -> Participants: "ABORT"
```

### Transaction Table

Active transactions tracked in shared memory:

```
TransTable
├── Active transactions (xid -> xmin, status)
├── Oldest active xid (for vacuum)
├── Transaction age tracking
└── Distributed coordinator info
```

## Lock Manager

### Overview

The Lock Manager provides concurrency control through fine-grained locking.

### Lock Types

| Lock | Mode | Compatibility |
|------|------|---------------|
| `SHARE` | Shared read | Compatible with SHARE, conflict with EXCLUSIVE |
| `EXCLUSIVE` | Write | Conflicts with all except ACCESS |
| `ACCESS SHARE` | Row-level auto | Compatible with all |
| `ROW SHARE` | Select for update | Conflict with EXCLUSIVE |
| `ROW EXCLUSIVE` | Update/delete | Conflict with SHARE, EXCLUSIVE |

### Lock Modes Compatibility Matrix

```
         SHARE  EXCLUSIVE  ROW_SHARE  ROW_EXCL  ACCESS_SHARE  ACCESS_EXCL
SHARE      +        -         +          -           +             -
EXCLUSIVE  -        -         -          -           -             -
ROW_SHARE  +        -         +          -           +             -
ROW_EXCL   -        -         -          -           -             -
ACCESS_SHR +        -         +          -           +             -
ACCESS_EXC -        -         -          -           -             +
```

### Lock Granularity

SqlRustGo supports multi-granularity locking:

```
Database
  └── Table (Intention locks)
        └── Page
              └── Row
```

**Intention Locks**:
- `INTENTION SHARE (IS)`: Row lock coming
- `INTENTION EXCLUSIVE (IX)`: Row update coming
- `SHARE INTENTION EXCLUSIVE (SIX)`: Table scan + updates

### Lock Data Structures

**Lock Hash Table**:
```
LockTable
└── hash(bucket) -> LockChain
                      ├── LockRequest (transaction, mode, granted)
                      ├── LockRequest
                      └── LockRequest
```

**Lock Acquisition**:

```rust
fn acquire_lock(txn: TransactionId, resource: Resource, mode: LockMode) -> Result<Lock> {
    // 1. Compute lock hash bucket
    // 2. Walk lock chain for resource
    // 3. Check compatibility with granted locks
    // 4. Add request to wait queue
    // 5. Return when granted or deadlock detected
}
```

### Lock Escalation

To prevent lock table overflow, automatically escalate:
- Many row locks on same table -> table lock
- Threshold: 50 row locks default

```
50 RowLocks -> TableIXLock (or upgrade existing)
```

## Deadlock Detection

### Overview

Deadlock occurs when two or more transactions wait for each other to release locks.

### Wait-For Graph

Deadlock detection uses a directed wait-for graph:

```
Transaction A --waits for--> Transaction B
      ^                              |
      |                              v
      +-------waits for-------------+
```

A cycle in this graph indicates deadlock.

### Detection Algorithm

Sql饿死 detection runs periodically:

```
1. Build wait-for graph from lock requests
2. For each transaction in graph:
     - Mark as unvisited
     - DFS following wait edges
     - If we reach a marked node -> cycle found
3. On cycle: select victim (youngest/shortest), abort
```

### Detection Interval

- Default: Check every 1 second
- Configurable via `deadlock_timeout`
- Also triggered on lock wait timeout

### Victim Selection

When deadlock detected, select victim based on:

| Criterion | Description |
|-----------|-------------|
| `youngest` | Abort transaction with highest xid |
| `shortest` | Abort transaction with fewest locks |
| `minimal_cost` | Abort transaction with least work done |

### Deadlock Avoidance

**Wait-Die**: Older transaction waits; younger transaction dies (pre-emptive abort)

```
T1 (older) holds lock, T2 (younger) wants it -> T2 dies
T1 (older) wants lock, T2 (younger) holds it -> T1 waits
```

**Wound-Wait**: Older transaction wounds (preempts) younger; younger waits

```
T1 (older) wants lock, T2 (younger) holds it -> T2 wounded (abort)
T1 (older) holds lock, T2 (younger) wants it -> T2 waits
```

## MVCC Implementation

### Snapshot

Each transaction sees a consistent snapshot:

```rust
struct Snapshot {
    xmin: TransactionId,  // Oldest active tx when snapshot taken
    xmax: TransactionId,  // Current max txid
    active: Vec<TransactionId>, // Active txids at snapshot time
}
```

### Tuple Visibility

```
tuple.xmin == IN_PROGRESS -> invisible (unless own txn)
tuple.xmax == ABORTED -> invisible
tuple.xmax == IN_PROGRESS -> invisible
tuple committed before snapshot.xmin -> visible
tuple committed after snapshot.xmin -> invisible
```

### Write Path

1. Check tuple visibility
2. Mark old tuple as expired (xmax = current xid)
3. Insert new tuple version
4. Both changes logged to WAL

### Read Path

1. Get snapshot
2. Scan tuples, checking visibility
3. Return latest visible version

## Recovery

### Write-Ahead Logging

All modifications logged before applying to data pages:

```
LOG: [xid, page_id, offset, old_value, new_value]
```

### Checkpoint

Periodic checkpoints for faster recovery:

```
CHECKPOINT:
  - Force dirty pages to disk
  - Write checkpoint record to WAL
  - Update control file
```

### Recovery Process

On startup after crash:

```
1. Read checkpoint from control file
2. Read last good checkpoint location
3. Redo: Apply all WAL records after checkpoint
4. Undo: Rollback uncommitted transactions
```

### WAL Segments

WAL organized in segments (typically 16MB):

```
pg_wal/
  ├── 000000010000000000000001
  ├── 000000010000000000000002
  └── ...
```

### Archive Mode

Optional WAL archiving:

```sql
ALTER SYSTEM SET wal_level = archive;
ALTER SYSTEM SET archive_mode = on;
```

Archives to `archive_command` (cp to remote, S3, etc.)

## Transaction Isolation Levels Detail

### READ COMMITTED

- Each statement sees snapshot at statement start
- Row locks released after statement
- Non-repeatable reads possible within transaction

### REPEATABLE READ

- Transaction sees snapshot at first statement
- Row locks held until commit
- `PostgresSQL` uses `SERIALIZABLE` for this level (historically)

### SERIALIZABLE

- Full snapshot at transaction start
- Predicate locks prevent phantoms
- May abort with `could not serialize` error
- Retry required

## Performance Considerations

### Bypass for Read-Only

Read-only transactions:
- Skip WAL writes
- Use snapshot without logging
- 2x faster for analytical queries

### Autocommit

Each statement is implicit transaction:

```sql
-- Equivalent to:
BEGIN;
  SELECT ...;
COMMIT;
```

Disable with `SET autocommit = off;`

### Savepoints

Named savepoints within transaction:

```sql
BEGIN;
  UPDATE ...;
  SAVEPOINT sp1;
  UPDATE ...;
  ROLLBACK TO SAVEPOINT sp1;
  UPDATE ...;
COMMIT;
```
