# SQLRustGo v1.6.0 API 文档

> **版本**: v1.6.0
> **发布日期**: 2026-03-19

---

## 一、核心 API

### 1.1 Transaction 模块

#### TransactionManager

```rust
pub struct TransactionManager {
    mvcc: Arc<RwLock<MvccEngine>>,
    current_tx: Option<TxId>,
    isolation_level: IsolationLevel,
}

impl TransactionManager {
    /// 开始事务
    pub fn begin(&mut self) -> Result<TxId, TransactionError>;
    
    /// 带隔离级别开始事务
    pub fn begin_with_isolation(&mut self, level: IsolationLevel) -> Result<TxId, TransactionError>;
    
    /// 提交事务
    pub fn commit(&mut self) -> Result<Option<u64>, TransactionError>;
    
    /// 回滚事务
    pub fn rollback(&mut self) -> Result<(), TransactionError>;
}
```

#### IsolationLevel

```rust
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,  // 默认
    RepeatableRead,
    Serializable,
}
```

### 1.2 MVCC 模块

#### MvccEngine

```rust
pub struct MvccEngine {
    transactions: HashMap<TxId, Transaction>,
    next_tx_id: u64,
    global_timestamp: u64,
}

impl MvccEngine {
    /// 开始事务
    pub fn begin_transaction(&mut self) -> TxId;
    
    /// 提交事务
    pub fn commit_transaction(&mut self, tx_id: TxId) -> Option<u64>;
    
    /// 中止事务
    pub fn abort_transaction(&mut self, tx_id: TxId) -> bool;
    
    /// 创建快照
    pub fn create_snapshot(&self, tx_id: TxId) -> Snapshot;
}
```

### 1.3 锁模块

#### LockManager

```rust
pub struct LockManager {
    locks: HashMap<Vec<u8>, LockInfo>,
    tx_locks: HashMap<TxId, HashSet<Vec<u8>>>,
}

impl LockManager {
    /// 获取锁
    pub fn acquire_lock(
        &mut self,
        tx_id: TxId,
        key: Vec<u8>,
        mode: LockMode,
    ) -> Result<LockGrantMode, LockError>;
    
    /// 释放锁
    pub fn release_lock(&mut self, tx_id: TxId, key: &Vec<u8>) -> Result<(), LockError>;
    
    /// 释放事务所有锁
    pub fn release_all_locks(&mut self, tx_id: TxId) -> Result<Vec<Vec<u8>>, LockError>;
}
```

#### LockMode

```rust
pub enum LockMode {
    Shared,     // 读锁
    Exclusive, // 写锁
}
```

### 1.4 死锁检测

#### DeadlockDetector

```rust
pub struct DeadlockDetector {
    waits_for: HashMap<TxId, HashSet<TxId>>,
    lock_wait_timeout: Duration,
}

impl DeadlockDetector {
    /// 添加等待边
    pub fn add_edge(&mut self, blocked: TxId, holder: TxId);
    
    /// 检测死锁环
    pub fn detect_cycle(&self, start: TxId) -> Option<Vec<TxId>>;
    
    /// 移除事务的所有边
    pub fn remove_edges_for(&mut self, tx_id: TxId);
}
```

---

## 二、存储模块

### 2.1 WAL

#### WalManager

```rust
pub struct WalManager {
    dir: PathBuf,
    current_file: PathBuf,
    current_writer: BufWriter<File>,
    lsn_counter: AtomicU64,
}

impl WalManager {
    /// 记录事务开始
    pub fn log_begin(&self, tx_id: u64) -> std::io::Result<u64>;
    
    /// 记录插入
    pub fn log_insert(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> std::io::Result<u64>;
    
    /// 记录提交
    pub fn log_commit(&self, tx_id: u64) -> std::io::Result<u64>;
    
    /// 记录回滚
    pub fn log_rollback(&self, tx_id: u64) -> std::io::Result<u64>;
    
    /// 创建检查点
    pub fn checkpoint(&self, tx_id: u64) -> std::io::Result<u64>;
    
    /// 恢复
    pub fn recover(&self) -> std::io::Result<Vec<WalEntry>>;
}
```

---

## 三、执行模块

### 3.1 查询缓存

#### QueryCache

```rust
pub struct QueryCache {
    config: QueryCacheConfig,
    cache: HashMap<CacheKey, CacheEntry>,
    lru_order: VecDeque<CacheKey>,
    table_index: HashMap<String, HashSet<CacheKey>>,
    current_memory_bytes: usize,
}

impl QueryCache {
    /// 创建新缓存
    pub fn new(config: QueryCacheConfig) -> Self;
    
    /// 获取缓存
    pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult>;
    
    /// 放入缓存
    pub fn put(&mut self, key: CacheKey, entry: CacheEntry, tables: Vec<String>);
    
    /// 失效表
    pub fn invalidate_table(&mut self, table: &str);
    
    /// 清空缓存
    pub fn clear(&mut self);
}
```

#### QueryCacheConfig

```rust
pub struct QueryCacheConfig {
    pub enabled: bool,
    pub max_entries: usize,        // 默认 1000
    pub max_memory_bytes: usize,  // 默认 100MB
    pub ttl_seconds: u64,         // 默认 30 秒
}
```

### 3.2 LocalExecutor

```rust
pub struct LocalExecutor<'a> {
    storage: &'a dyn StorageEngine,
    cache: Arc<RwLock<QueryCache>>,
    cache_config: QueryCacheConfig,
}

impl<'a> LocalExecutor<'a> {
    /// 创建执行器
    pub fn new(storage: &'a dyn StorageEngine) -> Self;
    
    /// 执行物理计划
    pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;
    
    /// 执行带缓存
    pub fn execute_with_cache(
        &self,
        plan: &dyn PhysicalPlan,
        sql: &str,
        params: &[Value],
    ) -> SqlResult<ExecutorResult>;
    
    /// 失效表缓存
    pub fn invalidate_table(&self, table: &str);
}
```

---

## 四、服务器模块

### 4.1 连接池

#### ConnectionPool

```rust
pub struct ConnectionPool {
    shared_storage: Arc<MemoryStorage>,
    sessions: Arc<Vec<PooledSession>>,
    available: Sender<PooledSession>,
    received: Receiver<PooledSession>,
    config: PoolConfig,
}

impl ConnectionPool {
    /// 创建连接池
    pub fn new(config: PoolConfig) -> Self;
    
    /// 获取连接
    pub fn acquire(&self) -> PooledConnection;
    
    /// 池大小
    pub fn size(&self) -> usize;
}
```

#### PoolConfig

```rust
pub struct PoolConfig {
    pub size: usize,           // 最大连接数
    pub timeout: Duration,   // 获取连接超时
}
```

---

## 五、数据类型

### 5.1 Value

```rust
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Date(i32),      // 天数 (UNIX epoch)
    Timestamp(i64), // 微秒 (UNIX epoch)
}

impl Value {
    pub fn as_integer(&self) -> Option<i64>;
    pub fn as_text(&self) -> Option<&String>;
    pub fn as_date(&self) -> Option<i32>;
    pub fn as_timestamp(&self) -> Option<i64>;
}
```

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-19*
