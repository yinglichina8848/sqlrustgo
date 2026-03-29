# SQLRustGo v2.0.0 Transaction 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-transaction

---

## 1. 模块概述

Transaction 模块负责事务管理，包括单机 MVCC 和分布式 2PC 协议。

## 2. 核心组件

### 2.1 TransactionManager

```rust
pub trait TransactionManager: Send + Sync {
    fn begin(&self, isolation: IsolationLevel) -> Result<TransactionId>;
    fn commit(&self, txid: TransactionId) -> Result<()>;
    fn rollback(&self, txid: TransactionId) -> Result<()>;
    fn get_status(&self, txid: TransactionId) -> Result<TransactionStatus>;
    fn get_transaction(&self, txid: TransactionId) -> Result<Transaction>;
}

pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

pub struct TransactionId(u64);

pub enum TransactionStatus {
    Active,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}
```

### 2.2 Transaction

```rust
pub struct Transaction {
    pub id: TransactionId,
    pub isolation_level: IsolationLevel,
    pub start_timestamp: Timestamp,
    pub commit_timestamp: Option<Timestamp>,
    pub status: TransactionStatus,
    pub snapshot: Snapshot,
    pub write_set: WriteSet,
    pub participants: Vec<NodeId>,
}

pub struct Snapshot {
    pub txid: TransactionId,
    pub commit_timestamp: Timestamp,
    pub active_txids: Vec<TransactionId>,
}

pub struct WriteSet {
    pub inserts: Vec<WriteRecord>,
    pub updates: Vec<WriteRecord>,
    pub deletes: Vec<WriteRecord>,
}

pub struct WriteRecord {
    pub table_id: TableId,
    pub key: Key,
    pub value: Vec<u8>,
    pub txid: TransactionId,
}
```

---

## 3. MVCC 实现

### 3.1 MVCC Manager

```rust
pub struct MVCC {
    version_chain: RwLock<HashMap<Key, Vec<Version>>>,
    garbage_collector: Arc<GarbageCollector>,
}

impl MVCC {
    pub fn new() -> Self;
    pub fn read(&self, key: &Key, txid: TransactionId) -> Result<Option<Value>>;
    pub fn write(&self, key: &Key, value: Value, txid: TransactionId) -> Result<()>;
    pub fn commit(&self, txid: TransactionId) -> Result<Timestamp>;
    pub fn abort(&self, txid: TransactionId) -> Result<()>;
}

pub struct Version {
    pub txid: TransactionId,
    pub start_ts: Timestamp,
    pub commit_ts: Timestamp,
    pub value: Vec<u8>,
    pub is_deleted: bool,
    pub next_version: Option<Box<Version>>,
}
```

### 3.2 快照隔离

```rust
impl MVCC {
    pub fn snapshot_read(&self, key: &Key, snapshot: &Snapshot) -> Result<Option<Value>> {
        let versions = self.version_chain.read().unwrap();
        
        if let Some(chain) = versions.get(key) {
            for version in chain.iter().rev() {
                if version.commit_ts <= snapshot.commit_timestamp 
                    && version.txid != snapshot.txid 
                    && !version.is_deleted 
                {
                    return Ok(Some(version.value.clone()));
                }
            }
        }
        Ok(None)
    }
}
```

---

## 4. Lock Manager

### 4.1 LockManager Trait

```rust
pub trait LockManager: Send + Sync {
    fn acquire_lock(&self, txid: TransactionId, key: &Key, mode: LockMode) -> Result<()>;
    fn release_lock(&self, txid: TransactionId, key: &Key) -> Result<()>;
    fn release_all_locks(&self, txid: TransactionId) -> Result<()>;
    fn is_locked(&self, key: &Key, mode: LockMode) -> bool;
}

pub enum LockMode {
    Shared,
    Exclusive,
    IntentionShared,
    IntentionExclusive,
}
```

### 4.2 DeadlockDetector

```rust
pub struct DeadlockDetector {
    waits_for: RwLock<HashMap<TransactionId, HashMap<Key, TransactionId>>>,
}

impl DeadlockDetector {
    pub fn new() -> Self;
    pub fn add_wait(&self, waiter: TransactionId, holder: TransactionId, key: &Key);
    pub fn remove_wait(&self, waiter: TransactionId);
    pub fn detect_deadlock(&self) -> Result<Vec<TransactionId>>;
}
```

---

## 5. 分布式事务 2PC

### 5.1 Coordinator

```rust
pub trait Coordinator: Send + Sync {
    fn begin_distributed(&self, isolation: IsolationLevel) -> Result<TransactionId>;
    fn add_participant(&self, txid: TransactionId, participant: NodeId) -> Result<()>;
    fn prepare(&self, txid: TransactionId) -> Result<PrepareResult>;
    fn commit(&self, txid: TransactionId) -> Result<CommitResult>;
    fn rollback(&self, txid: TransactionId) -> Result<()>;
}

pub struct CoordinatorState {
    pub transaction_id: TransactionId,
    pub participants: Vec<ParticipantInfo>,
    pub phase: TwoPhasePhase,
    pub isolation_level: IsolationLevel,
}

pub enum TwoPhasePhase {
    Initial,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}

pub struct PrepareResult {
    pub txid: TransactionId,
    pub vote: Vote,
    pub message: String,
}

pub enum Vote {
    VoteCommit,
    VoteAbort,
}

pub struct CommitResult {
    pub txid: TransactionId,
    pub success: bool,
    pub message: String,
}
```

### 5.2 Participant

```rust
pub trait Participant: Send + Sync {
    fn vote_prepare(&self, txid: TransactionId) -> Result<Vote>;
    fn commit(&self, txid: TransactionId) -> Result<()>;
    fn rollback(&self, txid: TransactionId) -> Result<()>;
    fn recover(&self) -> Result<Vec<TransactionId>>;
}

pub struct ParticipantState {
    pub node_id: NodeId,
    pub wal_manager: Arc<WALManager>,
    pub transaction_log: HashMap<TransactionId, ParticipantTransactionState>,
}

pub struct ParticipantTransactionState {
    pub txid: TransactionId,
    pub phase: TwoPhasePhase,
    pub write_set: WriteSet,
    pub coordinator_id: NodeId,
}
```

### 5.3 WAL 集成

```rust
pub struct RecoveryWALManager {
    wal_manager: Arc<WALManager>,
    coordinator_client: Arc<CoordinatorClient>,
}

impl RecoveryWALManager {
    pub fn new(wal: Arc<WALManager>, coordinator: Arc<CoordinatorClient>) -> Self;
    pub fn replay_prepared_transactions(&self) -> Result<Vec<TransactionId>>;
    pub fn checkpoint(&self) -> Result<()>;
}
```

---

## 6. 分布式锁

### 6.1 DistributedLockManager

```rust
pub trait DistributedLockManager: Send + Sync {
    fn acquire_lock(&self, key: &Key, txid: TransactionId, timeout: Duration) -> Result<()>;
    fn release_lock(&self, key: &Key, txid: TransactionId) -> Result<()>;
    fn extend_lock(&self, key: &Key, txid: TransactionId, timeout: Duration) -> Result<()>;
}

pub struct TwoPhaseLockManager {
    local_locks: Arc<LockManager>,
    coordinator: Arc<dyn Coordinator>,
}

impl DistributedLockManager for TwoPhaseLockManager {
    fn acquire_lock(&self, key: &Key, txid: TransactionId, timeout: Duration) -> Result<()>;
    fn release_lock(&self, key: &Key, txid: TransactionId) -> Result<()>;
}
```

---

## 7. 故障恢复

### 7.1 Recovery Manager

```rust
pub struct RecoveryManager {
    wal_manager: Arc<WALManager>,
    storage: Arc<dyn StorageEngine>,
    coordinator: Arc<dyn Coordinator>,
}

impl RecoveryManager {
    pub fn recover(&self) -> Result<RecoveryReport>;
    pub fn replay_wal(&self) -> Result<usize>;
    pub fn resolve_in_doubt_transactions(&self) -> Result<Vec<TransactionId>>;
}

pub struct RecoveryReport {
    pub transactions_recovered: usize,
    pub transactions_committed: usize,
    pub transactions_aborted: usize,
    pub in_doubt_transactions: Vec<TransactionId>,
}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
