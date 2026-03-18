# SQLRustGo v1.6.0 核心功能设计文档

> **版本**: v1.6.0
> **代号**: Production Preview（单机并发 + 可压测版本）
> **发布日期**: 2026-03
> **架构版本**: L3+ 事务隔离版

---

## 一、版本核心能力

### 1.1 三大核心能力

| 能力 | 说明 | 目标 |
|------|------|------|
| **高并发** | 多线程并发读写，无死锁 | 50+ 并发连接 |
| **可回滚** | 事务原子性，数据不丢失 | WAL 100% 恢复 |
| **可压测** | TPC-H Benchmark 可执行 | Q1/Q6 正确 |

### 1.2 架构定位

```
单机并发 + 可压测版本

目标: 让 SQLRustGo 支持真实的并发负载和性能基准测试

定位: 从 toy DB 演进到 production-ready 数据库
```

### 1.3 模块完成状态

| 模块 | 功能 | PR | 状态 |
|------|------|-----|------|
| **Transaction** | MVCC | #607 | ✅ |
| | 事务管理器 | #616 | ✅ |
| | READ COMMITTED | #620 | ✅ |
| | 行级锁 | #625 | ✅ |
| | 死锁检测 | #628 | ⏳ |
| **WAL** | 并发写入 | #608 | ✅ |
| | 检查点优化 | #613 | ✅ |
| | 归档功能 | #619 | ✅ |
| **Index** | 唯一索引 | #612 | ✅ |
| | 复合索引 | #615 | ✅ |
| | 索引统计 | #618 | ✅ |
| | 全文索引 | #626 | ✅ |
| **Performance** | 查询缓存 | #627 | ✅ |
| | 连接池 | #630 | ⏳ |
| | TPC-H | #631 | ⏳ |
| **Types** | DATE | #624 | ✅ |
| | TIMESTAMP | #634 | ✅ |

---

## 二、事务层 (Transaction Layer)

### 2.1 MVCC 引擎

**文件**: `crates/transaction/src/mvcc.rs`

#### 核心数据结构

```rust
// 事务ID
pub struct TxId(pub u64);
pub const INVALID_TX_ID: TxId = TxId(u64::MAX);

// 事务状态
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}

// 事务
pub struct Transaction {
    pub id: TxId,
    pub status: TransactionStatus,
    pub start_timestamp: u64,
    pub commit_timestamp: Option<u64>,
}

// 快照（用于隔离级别）
pub struct Snapshot {
    pub tx_id: TxId,
    pub snapshot_timestamp: u64,
    pub active_transactions: HashSet<TxId>,
}

// 行版本
pub struct RowVersion {
    pub tx_id: TxId,
    pub commit_timestamp: Option<u64>,
    pub data: Vec<u8>,
    pub next_version: Option<Box<RowVersion>>,
    pub is_deleted: bool,
}

// MVCC 引擎
pub struct MvccEngine {
    global_timestamp: u64,
    transactions: HashMap<TxId, Transaction>,
    version_chains: HashMap<Vec<u8>, Option<Box<RowVersion>>>,
    snapshots: HashMap<TxId, Snapshot>,
}
```

#### 关键 API

```rust
impl MvccEngine {
    // 事务生命周期
    pub fn begin_transaction(&mut self) -> TxId;
    pub fn commit_transaction(&mut self, tx_id: TxId) -> Option<u64>;
    pub fn abort_transaction(&mut self, tx_id: TxId) -> bool;
    
    // 快照管理
    pub fn create_snapshot(&self, tx_id: TxId) -> Snapshot;
    pub fn refresh_for_read_committed(&self, tx_id: TxId) -> Snapshot;
    
    // 可见性判断
    pub fn is_visible(&self, snapshot: &Snapshot, version: &RowVersion) -> bool;
    pub fn is_visible_read_committed(&self, snapshot: &Snapshot, version: &RowVersion) -> bool;
    
    // 版本链操作
    pub fn get_latest_version(&self, key: &[u8]) -> Option<&RowVersion>;
    pub fn add_version(&mut self, key: Vec<u8>, version: RowVersion);
}
```

#### 设计亮点

1. **双隔离级别支持**
   - `is_visible()`: Repeatable Read（事务内一致）
   - `is_visible_read_committed()`: Read Committed（每次查询刷新快照）

2. **全局单调递增时间戳**
   - `global_timestamp` 确保事务顺序性
   - commit_timestamp 保证可见性判断正确

3. **版本链结构**
   - 使用 `Box<RowVersion>` 实现链表追溯
   - 支持历史版本读取

---

### 2.2 事务管理器

**文件**: `crates/transaction/src/manager.rs`

#### 核心数据结构

```rust
// 隔离级别
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,      // 默认
    RepeatableRead,
    Serializable,
}

// 事务上下文
pub struct TransactionContext {
    pub tx_id: TxId,
    pub snapshot: Snapshot,
    pub isolation_level: IsolationLevel,
    pub read_only: bool,
}

// 事务管理器
pub struct TransactionManager {
    mvcc: Arc<RwLock<MvccEngine>>,
    current_tx: Option<TxId>,
    isolation_level: IsolationLevel,
}

// 错误类型
pub enum TransactionError {
    NoActiveTransaction,
    AlreadyCommitted,
    AlreadyAborted,
    ConcurrentModification,
}
```

#### 关键 API

```rust
impl TransactionManager {
    // 事务控制
    pub fn begin(&mut self) -> Result<TxId, TransactionError>;
    pub fn begin_with_isolation(&mut self, level: IsolationLevel) -> Result<TxId, TransactionError>;
    pub fn begin_read_only(&mut self) -> Result<TxId, TransactionError>;
    pub fn commit(&mut self) -> Result<Option<u64>, TransactionError>;
    pub fn rollback(&mut self) -> Result<(), TransactionError>;
    
    // 上下文获取
    pub fn get_transaction_context(&self) -> Result<TransactionContext, TransactionError>;
    pub fn get_transaction_context_for_query(&self) -> Result<TransactionContext, TransactionError>;
}
```

#### 设计亮点

1. **线程安全**: `Arc<RwLock<MvccEngine>>` 支持多线程并发
2. **读写分离**: 读操作使用 `RwLock` 允许并发
3. **查询级快照刷新**: `get_transaction_context_for_query()` 为 READ COMMITTED 每次查询刷新快照

---

### 2.3 行级锁 (T-04)

**文件**: `crates/transaction/src/lock_manager.rs`

#### 核心数据结构

```rust
pub enum LockType {
    Shared,      // 读锁，可并发
    Exclusive,   // 写锁，排他
}

pub struct LockState {
    owners: Vec<TxId>,
    waiters: Vec<TxId>,
    lock_type: LockType,
}

pub struct LockManager {
    locks: HashMap<RowId, LockState>,
    tx_locks: HashMap<TxId, HashSet<RowId>>,
}

pub type RowId = (TableId, u64);
pub type TableId = u64;
```

#### 锁兼容性矩阵

|          | S 持有 | X 持有 |
|----------|--------|--------|
| S 请求   | ✅ 兼容 | ❌ 阻塞 |
| X 请求   | ❌ 阻塞 | ❌ 阻塞 |

---

## 三、WAL 层 (Write-Ahead Logging)

**文件**: `crates/storage/src/wal.rs`

### 3.1 WAL 条目类型

```rust
pub enum WalEntryType {
    Begin = 1,
    Insert = 2,
    Update = 3,
    Delete = 4,
    Commit = 5,
    Rollback = 6,
    Checkpoint = 7,
}

pub struct WalEntry {
    pub tx_id: u64,
    pub entry_type: WalEntryType,
    pub table_id: u64,
    pub key: Option<Vec<u8>>,
    pub data: Option<Vec<u8>>,
    pub lsn: u64,          // Log Sequence Number
    pub timestamp: u64,
}
```

### 3.2 WAL 管理器

```rust
pub struct WalManager {
    dir: PathBuf,
    current_file: PathBuf,
    current_writer: BufWriter<File>,
    lsn_counter: AtomicU64,
}

impl WalManager {
    // 日志写入
    pub fn log_begin(&self, tx_id: u64) -> std::io::Result<u64>;
    pub fn log_insert(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> std::io::Result<u64>;
    pub fn log_update(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> std::io::Result<u64>;
    pub fn log_delete(&self, tx_id: u64, table_id: u64, key: Vec<u8>) -> std::io::Result<u64>;
    pub fn log_commit(&self, tx_id: u64) -> std::io::Result<u64>;
    pub fn log_rollback(&self, tx_id: u64) -> std::io::Result<u64>;
    
    // 检查点
    pub fn checkpoint(&self, tx_id: u64) -> std::io::Result<u64>;
    
    // 恢复
    pub fn recover(&self) -> std::io::Result<Vec<WalEntry>>;
}
```

### 3.3 WAL 归档

```rust
pub struct WalArchiveManager {
    archive_dir: PathBuf,
    archives: Vec<WalArchiveMetadata>,
    max_archive_age_days: u32,
    max_archive_size_mb: u64,
}

pub struct WalArchiveMetadata {
    pub archive_id: u64,
    pub original_file: String,
    pub archived_file: String,
    pub compressed: bool,
    pub original_size: u64,
    pub archived_size: u64,
    pub timestamp: u64,
    pub entry_count: u64,
}

impl WalArchiveManager {
    pub fn archive_wal(&mut self) -> std::io::Result<WalArchiveMetadata>;
    pub fn recover_from_archive(&self, archive_id: u64) -> std::io::Result<Vec<WalEntry>>;
    pub fn cleanup_old_archives(&self, keep_count: u32) -> std::io::Result<u32>;
}
```

### 3.4 设计亮点

1. **完整日志类型**: BEGIN/INSERT/UPDATE/DELETE/COMMIT/ROLLBACK/CHECKPOINT
2. **二进制序列化**: 固定字段+变长字段，高效存储
3. **归档管理**: 自动压缩 + 清理策略（默认7天/100MB阈值）
4. **恢复机制**: 支持从 WAL 和归档恢复数据

---

## 四、索引层 (Index Layer)

### 4.1 B+Tree 索引

**文件**: `crates/storage/src/bplus_tree/bplus_tree.rs`

#### 核心常量

```rust
const BTREE_ORDER: usize = 64;      // B+Tree 阶
const MAX_KEYS_PER_NODE: usize = 63; // 每节点最大键数
```

#### 节点结构

```rust
pub struct BTreeNode {
    pub is_leaf: bool,
    pub num_keys: u16,
    pub keys: Vec<i64>,
    pub values: Vec<u32>,
    pub children: Vec<u32>,      // 非叶子节点
    pub next_leaf: Option<u32>,  // 叶子节点链表
}
```

#### 关键 API

```rust
impl BTreeIndex {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: i64, value: u32);
    pub fn search(&self, key: i64) -> Option<u32>;
    pub fn delete(&mut self, key: i64) -> bool;
    pub fn range_query(&self, start: i64, end: i64) -> Vec<u32>;
    pub fn collect_stats(&self) -> IndexStats;
}
```

---

### 4.2 唯一索引 (I-03)

```rust
pub struct BTreeIndex {
    // ...
    is_unique: bool,
}

impl BTreeIndex {
    pub fn set_unique(&mut self, unique: bool);
    pub fn is_unique(&self) -> bool;
    
    pub fn insert_unique(&mut self, key: i64, value: u32) -> Result<(), UniqueConstraintViolation>;
}

#[derive(Error, Debug)]
pub enum UniqueConstraintViolation {
    #[error("Unique constraint violated for key: {0}")]
    KeyExists(i64),
}
```

---

### 4.3 复合索引 (I-04)

```rust
pub struct CompositeKey {
    pub columns: Vec<i64>,
}

impl CompositeKey {
    pub fn new(columns: Vec<i64>) -> Self;
}

pub struct CompositeBTreeIndex {
    num_columns: usize,
    inner: BTreeIndex,
}

impl CompositeBTreeIndex {
    pub fn new(num_columns: usize) -> Self;
    pub fn insert(&mut self, key: CompositeKey, value: u32);
    pub fn search(&self, key: &CompositeKey) -> Option<u32>;
    pub fn range_query(&self, start: &CompositeKey, end: &CompositeKey) -> Vec<u32>;
}
```

---

### 4.4 索引统计 (I-05)

```rust
pub struct IndexStats {
    pub num_entries: u64,
    pub num_leaf_nodes: u64,
    pub num_internal_nodes: u64,
    pub total_nodes: u64,
    pub height: u32,
    pub cardinality: u64,
    pub size_bytes: u64,
}

impl IndexStats {
    pub fn selectivity(&self) -> f64 {
        if self.num_entries == 0 {
            1.0
        } else {
            (self.cardinality as f64 / self.num_entries as f64).min(1.0)
        }
    }
}

impl BTreeIndex {
    pub fn collect_stats(&self) -> IndexStats;
    pub fn usage_stats(&self) -> IndexStats;
}
```

---

### 4.5 全文索引 (I-06)

**文件**: `crates/storage/src/bplus_tree/fulltext.rs`

```rust
pub struct FullTextIndex {
    metadata: FullTextMetadata,
    inverted_index: BTreeMap<String, PostingList>,
    deleted_docs: HashSet<u32>,
}

pub struct PostingList {
    pub doc_ids: Vec<u32>,
}

impl FullTextIndex {
    pub fn new(column_name: &str) -> Self;
    pub fn tokenize(text: &str) -> Vec<String>;
    pub fn insert(&mut self, doc_id: u32, text: &str);
    pub fn search(&self, query: &str) -> Vec<u32>;  // AND 查询
    pub fn delete(&mut self, doc_id: u32);  // 懒删除
    pub fn num_documents(&self) -> usize;
    pub fn num_terms(&self) -> usize;
}
```

**搜索示例**:
```rust
let mut index = FullTextIndex::new("content");
index.insert(1, "hello world");
index.insert(2, "hello rust");
index.search("hello");        // 返回 [1, 2]
index.search("hello world");  // 返回 [1] (AND 查询)
```

---

## 五、执行层 (Executor Layer)

### 5.1 查询缓存 (P-01)

**文件**: `crates/executor/src/query_cache.rs`

#### 缓存配置

```rust
pub struct QueryCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,       // 默认 1000
    pub max_memory_bytes: usize,  // 默认 100MB
    pub ttl_seconds: u64,         // 默认 300 秒
}

pub struct CacheKey {
    pub normalized_sql: String,
    pub params_hash: u64,
    pub table_versions: Vec<u64>,
}

pub struct CacheEntry {
    pub result: ExecutorResult,
    pub tables: Vec<String>,
    pub created_at: Instant,
    pub size_bytes: usize,
}
```

#### 缓存结构

```rust
pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    table_index: HashMap<String, HashSet<CacheKey>>,
    current_memory_bytes: usize,
}

impl QueryCache {
    pub fn new(config: QueryCacheConfig) -> Self;
    
    // 缓存操作
    pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult>;
    pub fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>);
    
    // 失效策略
    pub fn invalidate_table(&mut self, table: &str);
    pub fn clear(&mut self);
    
    // 统计
    pub fn stats(&self) -> QueryCacheStats;
}

pub struct QueryCacheStats {
    pub entries: usize,
    pub memory_bytes: usize,
    pub table_count: usize,
}
```

#### 缓存策略

1. **LRU 淘汰**: `lru_order` 维护访问顺序
2. **容量限制**: `max_entries` + `max_memory_bytes`
3. **表级失效**: `invalidate_table()` 清除相关缓存
4. **TTL**: 过期时间自动清理

---

## 二、高并发设计 (High Concurrency)

### 2.1 并发模型

```
┌─────────────────────────────────────────────────────────────────┐
│                    SQLRustGo 并发架构                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Thread Pool (tokio)                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐          │
│  │ Task 1  │  │ Task 2  │  │ Task 3  │  │ Task N  │          │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘          │
│       │              │              │              │              │
│       ▼              ▼              ▼              ▼              │
│  ┌─────────────────────────────────────────────────────┐         │
│  │              TransactionManager                        │         │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐             │         │
│  │  │ Snapshot │  │ Snapshot │  │ Snapshot │  ...       │         │
│  │  └─────────┘  └─────────┘  └─────────┘             │         │
│  └─────────────────────────────────────────────────────┘         │
│       │              │              │              │              │
│       ▼              ▼              ▼              ▼              │
│  ┌─────────────────────────────────────────────────────┐         │
│  │              LockManager (行级锁)                     │         │
│  │                                                       │         │
│  │  RowId → LockState { owners, waiters, lock_type }   │         │
│  │                                                       │         │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐             │         │
│  │  │ Deadlock │  │ Deadlock │  │ Deadlock │  检测      │         │
│  │  │Detector │──│Detector │──│Detector │             │         │
│  │  └─────────┘  └─────────┘  └─────────┘             │         │
│  └─────────────────────────────────────────────────────┘         │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 锁管理器 (T-04)

**文件**: `crates/transaction/src/lock_manager.rs`

#### 核心数据结构

```rust
// 行级锁标识
pub type RowId = (TableId, u64);
pub type TableId = u64;

// 锁类型
pub enum LockType {
    Shared,     // 读锁：多个事务可同时持有
    Exclusive, // 写锁：排他持有
}

// 锁状态
pub struct LockState {
    owners: Vec<TxId>,      // 当前持有锁的事务
    waiters: Vec<TxId>,     // 等待队列（FIFO）
    lock_type: LockType,
}

// 锁管理器
pub struct LockManager {
    locks: HashMap<RowId, LockState>,           // 行锁表
    tx_locks: HashMap<TxId, HashSet<RowId>>,  // 事务拥有的锁
}
```

#### 锁兼容性矩阵

|          | S 持有 | X 持有 |
|----------|--------|--------|
| S 请求   | ✅ 兼容 | ❌ 阻塞 |
| X 请求   | ❌ 阻塞 | ❌ 阻塞 |

#### 核心 API

```rust
impl LockManager {
    // 获取锁
    pub fn lock(&mut self, row_id: RowId, tx_id: TxId, lock_type: LockType) -> Result<(), LockError>;
    
    // 释放锁
    pub fn unlock(&mut self, tx_id: TxId);
    
    // 检查锁兼容性
    pub fn is_lock_compatible(&self, row_id: &RowId, lock_type: LockType) -> bool;
    
    // 锁等待超时
    pub fn lock_with_timeout(&mut self, row_id: RowId, tx_id: TxId, lock_type: LockType, timeout: Duration) -> Result<(), LockError>;
}
```

### 2.3 死锁检测 (T-05)

**文件**: `crates/concurrency/src/deadlock.rs`

#### Wait-For Graph

```rust
pub struct DeadlockDetector {
    // tx_id → 等待的事务ID集合
    wait_for_graph: HashMap<TxId, HashSet<TxId>>,
    // 事务等待开始时间
    wait_start: HashMap<TxId, Instant>,
}
```

#### 检测算法（DFS）

```rust
impl DeadlockDetector {
    // 检测死锁环
    pub fn detect_cycle(&self, start_tx: TxId) -> Option<Vec<TxId>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();
        
        self.dfs(start_tx, &mut visited, &mut recursion_stack, &mut path)
    }
    
    // 选择 victim（等待时间最长）
    pub fn select_victim(&self, cycle: &[TxId]) -> TxId {
        cycle.iter()
            .min_by_key(|tx| self.wait_start.get(tx))
            .copied()
            .unwrap_or(cycle[0])
    }
}
```

#### 触发策略

- **只在阻塞时触发**（非定时扫描）
- 减少不必要的检测开销

### 2.4 MVCC 读写并发

**核心优势**: 读不阻塞写，写不阻塞读

```rust
impl MvccEngine {
    // 读操作：无锁
    pub fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let version = self.get_latest_version(key)?;
        if snapshot.is_visible(&version) {
            Some(version.data.clone())
        } else {
            None
        }
    }
    
    // 写操作：只需行锁
    pub fn write(&mut self, key: &[u8], data: Vec<u8>, tx_id: TxId) {
        // 添加新版本，不阻塞其他读
        let new_version = RowVersion {
            tx_id,
            commit_timestamp: None,  // 未提交
            data,
            next_version: None,
            is_deleted: false,
        };
        self.add_version(key.to_vec(), new_version);
    }
}
```

### 2.5 并发性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 并发连接数 | ≥ 50 | 支持 50+ 同时连接 |
| 死锁检测时间 | < 10ms | 检测到回滚 |
| 锁粒度 | 行级 | 最小化冲突 |
| 读并发 | 无限制 | MVCC 支持 |
| 写并发 | 行级并行 | 冲突行串行 |

---

## 三、可回滚设计 (Rollback & Recovery)

### 3.1 事务回滚能力

```
事务生命周期
─────────────────────────────────────────────────────────

BEGIN ─────► READ/WRITE ─────► COMMIT
    │            │                │
    │            │                │
    │            ▼                ▼
    │       获取行锁          释放行锁
    │            │                │
    │            ▼                ▼
    │       写入 WAL         WAL 标记 COMMIT
    │            │                │
    │            ▼                ▼
    │       版本链更新       数据可见
    │            │                │
    │            ▼                ▼
    └────◄──── ROLLBACK ◄───────┘
              │
              ▼
         释放行锁
              │
              ▼
         版本链回滚（删除未提交版本）
              │
              ▼
         WAL 标记 ROLLBACK
```

### 3.2 WAL 完整设计

**文件**: `crates/storage/src/wal.rs`

#### WAL 条目类型

| 类型 | 说明 | 恢复影响 |
|------|------|----------|
| Begin | 事务开始 | 创建未完成事务 |
| Insert | 插入记录 | 应用插入 |
| Update | 更新记录 | 应用更新 |
| Delete | 删除记录 | 应用删除 |
| Commit | 事务提交 | 事务完成 |
| Rollback | 事务回滚 | 忽略未提交数据 |
| Checkpoint | 检查点 | 标记恢复起点 |

#### WAL 写入流程

```rust
impl WalManager {
    // 事务开始
    pub fn log_begin(&self, tx_id: u64) -> std::io::Result<u64> {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Begin,
            table_id: 0,
            key: None,
            data: None,
            lsn: self.next_lsn(),
            timestamp: now(),
        };
        self.write_entry(&entry)
    }
    
    // 数据变更
    pub fn log_insert(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> std::io::Result<u64> {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Insert,
            table_id,
            key: Some(key),
            data: Some(data),
            lsn: self.next_lsn(),
            timestamp: now(),
        };
        self.write_entry(&entry)
    }
    
    // 事务提交
    pub fn log_commit(&self, tx_id: u64) -> std::io::Result<u64> {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Commit,
            table_id: 0,
            key: None,
            data: None,
            lsn: self.next_lsn(),
            timestamp: now(),
        };
        self.write_entry(&entry)
    }
}
```

### 3.3 检查点 (Checkpoint)

```rust
impl WalManager {
    // 创建检查点
    pub fn checkpoint(&self, tx_id: u64) -> std::io::Result<u64> {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Checkpoint,
            table_id: 0,
            key: None,
            data: None,
            lsn: self.next_lsn(),
            timestamp: now(),
        };
        
        // 刷盘所有缓冲区
        self.flush_all()?;
        
        self.write_entry(&entry)
    }
}
```

### 3.4 恢复流程

```rust
impl WalManager {
    pub fn recover(&self) -> std::io::Result<Vec<WalEntry>> {
        let mut entries = Vec::new();
        let mut active_txs = HashSet::new();
        
        // 读取所有 WAL 文件
        for file in self.wal_files()? {
            entries.extend(self.read_file(&file)?);
        }
        
        // 按 LSN 排序
        entries.sort_by_key(|e| e.lsn);
        
        // 重放已提交事务
        let mut committed = HashSet::new();
        for entry in &entries {
            match entry.entry_type {
                WalEntryType::Begin => { active_txs.insert(entry.tx_id); }
                WalEntryType::Commit => { 
                    committed.insert(entry.tx_id); 
                    active_txs.remove(&entry.tx_id);
                }
                WalEntryType::Rollback => { 
                    active_txs.remove(&entry.tx_id);
                }
                WalEntryType::Checkpoint => {
                    // 检查点前的已提交事务已处理完
                }
                _ => {
                    // 数据操作：只有已提交事务才重放
                    if committed.contains(&entry.tx_id) {
                        self.replay_entry(entry)?;
                    }
                }
            }
        }
        
        // 丢弃未提交事务的数据（已由版本链处理）
        Ok(entries)
    }
}
```

### 3.5 归档管理

```rust
pub struct WalArchiveManager {
    archive_dir: PathBuf,
    max_archive_age_days: u32,  // 默认 7 天
    max_archive_size_mb: u64,   // 默认 100 MB
}

impl WalArchiveManager {
    // 归档
    pub fn archive_wal(&mut self) -> std::io::Result<WalArchiveMetadata> {
        // 1. 创建检查点
        // 2. 压缩 WAL 文件
        // 3. 移动到归档目录
        // 4. 记录元数据
    }
    
    // 清理旧归档
    pub fn cleanup_old_archives(&self, keep_count: u32) -> std::io::Result<u32> {
        // 1. 按时间排序归档
        // 2. 删除超过 keep_count 的旧归档
        // 3. 清理过期归档
    }
}
```

### 3.6 回滚性能指标

| 指标 | 目标 | 说明 |
|------|------|------|
| 恢复完整性 | 100% | 无数据丢失 |
| 恢复时间 | < 30s | 1GB WAL |
| 检查点间隔 | 5-10 min | 可配置 |
| 归档保留 | 7 天 | 可配置 |
| 压缩率 | ≥ 50% | gzip 压缩 |

---

## 四、性能设计 (Performance)

### 4.1 查询缓存 (P-01)

**文件**: `crates/executor/src/query_cache.rs`

#### 缓存结构

```rust
pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,           // LRU 链表
    table_index: HashMap<String, HashSet<CacheKey>>,  // 表 → 缓存键索引
    current_memory_bytes: usize,
}

pub struct CacheKey {
    pub normalized_sql: String,    // 归一化 SQL
    pub params_hash: u64,         // 参数哈希
    pub table_versions: Vec<u64>,  // 表版本（失效用）
}

pub struct CacheEntry {
    pub result: ExecutorResult,
    pub tables: Vec<String>,
    pub created_at: Instant,
    pub size_bytes: usize,
}
```

#### 缓存策略

| 策略 | 实现 | 说明 |
|------|------|------|
| LRU | `VecDeque<CacheKey>` | 最近最少使用淘汰 |
| 容量限制 | `max_entries` | 最大条目数（默认 1000） |
| 内存限制 | `max_memory_bytes` | 最大内存（默认 100MB） |
| TTL | `ttl_seconds` | 过期时间（默认 300s） |
| 表级失效 | `table_index` | 数据变更时清除相关缓存 |

#### 缓存命中流程

```rust
impl QueryCache {
    pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult> {
        if !self.config.enabled {
            return None;
        }
        
        // 检查 TTL
        let entry = self.cache.get_mut(key)?;
        if entry.is_expired(Duration::from_secs(self.config.ttl_seconds)) {
            self.remove(key);
            return None;
        }
        
        // 更新 LRU
        self.touch(key);
        
        Some(entry.result.clone())
    }
    
    // 写入缓存
    pub fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>) {
        // 1. 检查大小限制
        // 2. LRU 淘汰
        // 3. 插入缓存
        // 4. 更新表索引
    }
    
    // 表级失效
    pub fn invalidate_table(&mut self, table: &str) {
        if let Some(keys) = self.table_index.remove(table) {
            for key in keys {
                self.cache.remove(&key);
                self.lru_order.retain(|k| k != &key);
            }
        }
    }
}
```

### 4.2 TPC-H Benchmark (P-03)

#### Q1 - 价格汇总查询

```sql
SELECT
    l_returnflag,
    l_linestatus,
    SUM(l_quantity) AS sum_qty,
    SUM(l_extendedprice) AS sum_base_price,
    SUM(l_extendedprice * (1 - l_discount)) AS sum_disc_price,
    SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) AS sum_charge,
    AVG(l_quantity) AS avg_qty,
    AVG(l_extendedprice) AS avg_price,
    AVG(l_discount) AS avg_disc,
    COUNT(*) AS count_order
FROM lineitem
WHERE l_shipdate <= DATE '1998-09-02'
GROUP BY l_returnflag, l_linestatus
ORDER BY l_returnflag, l_linestatus;
```

**覆盖**: 聚合函数、日期过滤、多字段 GROUP BY、ORDER BY

#### Q6 - 折扣收入查询

```sql
SELECT
    SUM(l_extendedprice * l_discount) AS revenue
FROM lineitem
WHERE l_shipdate >= DATE '1994-01-01'
  AND l_shipdate < DATE '1995-01-01'
  AND l_discount BETWEEN 0.05 AND 0.07
  AND l_quantity < 24;
```

**覆盖**: 范围过滤、聚合函数

### 4.3 性能指标

| 指标 | 目标 | 对比基线 |
|------|------|----------|
| Q1 延迟 | < 500ms | 10K 行 |
| Q6 延迟 | < 200ms | 10K 行 |
| 缓存命中率 | ≥ 80% | 重复查询 |
| WAL 吞吐 | ≥ 50 MB/s | 顺序写入 |
| 并发 QPS | ≥ 100 | 50 并发 |
| 恢复时间 | < 30s | 1GB WAL |

---

## 五、数据类型 (Types)

### 6.1 DATE 类型

**文件**: `crates/types/src/value.rs`

```rust
pub enum Value {
    // ...
    Date(i32),  // days since UNIX epoch
}

impl Value {
    pub fn as_date(&self) -> Option<i32>;
}
```

### 6.2 TIMESTAMP 类型

**文件**: `crates/types/src/value.rs`

```rust
pub enum Value {
    // ...
    Timestamp(i64),  // microseconds since UNIX epoch
}

impl Value {
    // 构造函数
    pub fn timestamp(micros: i64) -> Self;
    
    // 获取器
    pub fn as_timestamp(&self) -> Option<i64>;
    
    // 格式化
    pub fn timestamp_to_string(&self) -> Option<String>;
}
```

**格式化算法**:
```rust
fn timestamp_to_datetime_string(micros: i64) -> String {
    // 微秒 -> 日期时间转换
    // 输出格式: "YYYY-MM-DD HH:MM:SS"
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
```

---

## 六、数据类型 (Types)

### 6.1 DATE 类型

```rust
pub enum Value {
    // ...
    Date(i32),  // days since UNIX epoch
}
```

### 6.2 TIMESTAMP 类型

```rust
pub enum Value {
    // ...
    Timestamp(i64),  // microseconds since UNIX epoch
}

impl Value {
    pub fn timestamp(micros: i64) -> Self;
    pub fn as_timestamp(&self) -> Option<i64>;
    pub fn timestamp_to_string(&self) -> Option<String>;
}
```

---

## 七、Parser 层

### 7.1 DATE/TIMESTAMP 关键字

**文件**: `crates/parser/src/token.rs`

```rust
pub enum Token {
    // Data Types
    Integer,
    Text,
    Float,
    Boolean,
    Blob,
    Null,
    Date,       // NEW
    Timestamp,   // NEW
    
    // Literals
    DateLiteral(String),      // NEW
    TimestampLiteral(String), // NEW
}
```

### 7.2 Lexer 支持

**文件**: `crates/parser/src/lexer.rs`

```rust
match ident.to_uppercase().as_str() {
    // ...
    "DATE" => Token::Date,
    "TIMESTAMP" => Token::Timestamp,
    // ...
}
```

---

## 八、执行流程图

### 8.1 查询执行流程

```
SQL Query
    ↓
[Lexer] → Token 流
    ↓
[Parser] → AST
    ↓
[Planner] → Logical Plan
    ↓
[Optimizer] → Physical Plan
    ↓
[Executor] 
    ├── 查询缓存检查 (P-01)
    └── 执行算子
         ↓
    [TransactionManager]
         ├── 快照获取
         └── 隔离级别
              ↓
         [LockManager] (T-04)
              ↓
         [MvccEngine]
              ↓
         [WAL]
              ↓
         [Storage + B+Tree]
```

### 8.2 事务执行流程

```
BEGIN
    ↓
创建 TxId
    ↓
获取快照 (READ COMMITTED / REPEATABLE READ)
    ↓
执行 SQL 操作
    ├── 获取行锁 (T-04)
    ├── 读写数据
    └── 记录 WAL
         ↓
COMMIT / ROLLBACK
    ├── 释放锁
    └── WAL 标记
```

---

## 九、模块依赖关系

```
┌─────────────────────────────────────────────────────────────┐
│                      Parser Layer                           │
│   Token ├── DATE ──────────────────────────────────────────►│
│   Lexer ├── TIMESTAMP ────────────────────────────────────►│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Planner Layer                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Executor Layer                         │
│   QueryCache ───────────────────────────────────────────►  │
└─────────────────────────────────────────────────────────────┘
          │                   │                   │
          ▼                   ▼                   ▼
┌─────────────────┐ ┌──────────────┐ ┌─────────────────┐
│ TransactionMgr   │ │ LockManager   │ │ MvccEngine      │
│ (manager.rs)     │ │ (T-04)       │ │ (mvcc.rs)        │
└─────────────────┘ └──────────────┘ └─────────────────┘
                              │                   │
                              ▼                   ▼
                    ┌─────────────────┐ ┌─────────────────┐
                    │ DeadlockDetector │ │     WAL          │
                    │ (T-05)          │ │ (wal.rs)         │
                    └─────────────────┘ └─────────────────┘
                                              │
                                              ▼
                    ┌─────────────────────────────────────────┐
                    │          Storage Layer                  │
                    │  B+Tree ├── 唯一索引 (I-03)           │
                    │        ├── 复合索引 (I-04)             │
                    │        ├── 索引统计 (I-05)             │
                    │        └── 全文索引 (I-06)             │
                    └─────────────────────────────────────────┘
```

---

## 十、完整性能指标汇总

### 10.1 高并发指标

| 指标 | 目标 | 验证方法 |
|------|------|----------|
| 并发连接数 | ≥ 50 | 连接池压测 |
| 死锁检测时间 | < 10ms | 单元测试 |
| 锁等待超时 | 可配置 | 超时测试 |
| 读并发 | 无限制 | MVCC |
| 写并发 | 行级并行 | 冲突行串行 |

### 10.2 可回滚指标

| 指标 | 目标 | 验证方法 |
|------|------|----------|
| 恢复完整性 | 100% | 破坏性测试 |
| 恢复时间 | < 30s (1GB WAL) | 计时测试 |
| 检查点间隔 | 5-10 min | 配置测试 |
| 归档保留 | 7 天 | 清理测试 |
| WAL 压缩率 | ≥ 50% | 压缩测试 |

### 10.3 TPC-H Benchmark 指标

| 指标 | 目标 | 数据规模 |
|------|------|----------|
| Q1 延迟 | < 500ms | 10K 行 |
| Q6 延迟 | < 200ms | 10K 行 |
| 结果正确性 | 与 SQLite 一致 | 对比测试 |
| QPS | ≥ 10 | 单并发 |

### 10.4 查询缓存指标

| 指标 | 目标 | 验证方法 |
|------|------|----------|
| 命中率 | ≥ 80% | 重复查询测试 |
| LRU 淘汰 | 正确 | 容量测试 |
| 表级失效 | 正确 | DML 后查询 |
| 内存限制 | ≤ 100MB | 内存监控 |

### 10.5 质量指标

| 指标 | Alpha | Beta | RC | GA |
|------|-------|------|-----|-----|
| 覆盖率 | ≥ 50% | ≥ 65% | ≥ 75% | ≥ 80% |
| Clippy | 零警告 | 零警告 | 零警告 | 零警告 |
| 编译 | 成功 | 成功 | 成功 | 成功 |

---

## 十一、版本验收标准

### 11.1 功能验收

| 功能 | 验收条件 | 对应模块 |
|------|----------|----------|
| 并发写 | 10 并发写入无冲突 | LockManager |
| 死锁检测 | 无死锁 hang | DeadlockDetector |
| 事务回滚 | abort 后数据正确 | TransactionManager |
| WAL 恢复 | 崩溃后可恢复 | WAL |
| 查询缓存 | 命中/失效正确 | QueryCache |
| TPC-H Q1 | 执行成功 | Executor |
| TPC-H Q6 | 执行成功 | Executor |

### 11.2 性能验收

| 指标 | 验收条件 |
|------|----------|
| 并发数 | 支持 50+ 连接 |
| 死锁处理 | 检测 + 回滚 < 100ms |
| 恢复 | 1GB WAL < 30s |
| Q1/Q6 | 延迟 < 目标值 |

### 11.3 工程验收

| 标准 | 验收条件 |
|------|----------|
| CI | 全绿 |
| 覆盖率 | ≥ 75% |
| 文档 | README + Benchmark 报告 |

---

## 十二、待完成功能

### 12.1 v1.6.0 剩余

| 功能 | 状态 | 说明 |
|------|------|------|
| T-05 死锁检测 | ⏳ | Wait-For Graph + DFS |
| P-02 连接池 | ⏳ | Semaphore 控制 |
| P-03 TPC-H | ⏳ | Q1/Q6 Benchmark |

### 12.2 v1.7.0 规划

| 功能 | 说明 |
|------|------|
| SAVEPOINT | 事务保存点 |
| SIMD | 向量化加速 |
| BLOB/BOOLEAN | 简单数据类型 |
| REPL 增强 | 交互体验 |

---

## 十三、相关文档

| 文档 | 说明 |
|------|------|
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 版本计划 |
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |
| [v1.6.0_gate_check_spec.md](./v1.6.0_gate_check_spec.md) | 门禁规范 |
| [v1.6.0_task_checklist.md](./v1.6.0_task_checklist.md) | 任务清单 |

---

## 十四、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v1.6.0 | 2026-03-19 | 完整核心功能设计文档 |
| v1.5.0 | 2026-03 | 持久化版 |
| v1.2.0 | 2026-03 | 架构重构 |
| v1.0.0 | 2026-02 | 基础 SQL 引擎 |

---

*本文档由 AI 辅助分析生成*
*分析日期: 2026-03-19*
