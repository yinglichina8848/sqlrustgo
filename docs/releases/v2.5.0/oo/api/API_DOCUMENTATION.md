# SQLRustGo v2.5.0 API 文档

**版本**: v2.5.0

---

## 一、核心 API

### 1.1 存储引擎 API

```rust
// 存储引擎 trait
pub trait StorageEngine {
    fn insert(&self, table_id: u64, row: &[Value]) -> Result<TxId>;
    fn update(&self, table_id: u64, key: &[u8], row: &[Value]) -> Result<()>;
    fn delete(&self, table_id: u64, key: &[u8]) -> Result<()>;
    fn get(&self, table_id: u64, key: &[u8]) -> Result<Option<Vec<Value>>>;
    fn scan(&self, table_id: u64, predicate: &Predicate) -> Result<Vec<Vec<Value>>>;
}
```

### 1.2 事务 API

```rust
// 事务管理器
pub trait TransactionManager {
    fn begin(&self) -> Result<TransactionId>;
    fn commit(&self, tx_id: TransactionId) -> Result<u64>;
    fn rollback(&self, tx_id: TransactionId) -> Result<()>;
    fn get_snapshot(&self, tx_id: TransactionId) -> Snapshot;
}
```

### 1.3 MVCC 存储 API

```rust
// MVCC 存储
pub trait MVCCStorage {
    fn get(&self, key: &[u8], snapshot: &Snapshot) -> Result<Option<Vec<Value>>>;
    fn insert(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn update(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&self, key: &[u8]) -> Result<()>;
}
```

---

## 二、WAL API

### 2.1 WAL 管理器

```rust
pub trait WalManager {
    fn append(&self, entry: &WalEntry) -> Result<u64>;
    fn read(&self, start_lsn: u64) -> Result<Vec<WalEntry>>;
    fn recover(&self) -> Result<Vec<WalEntry>>;
    fn sync(&self) -> Result<()>;
}
```

### 2.2 PITR 恢复

```rust
pub trait PITRRecovery {
    fn recover_to_timestamp(&self, timestamp: i64) -> Result<RecoveryResult>;
    fn recover_table_to_timestamp(&self, table_id: u64, timestamp: i64) -> Result<RecoveryResult>;
    fn find_archive_for_timestamp(&self, timestamp: i64) -> Result<Option<ArchiveId>>;
}
```

---

## 三、向量 API

### 3.1 向量索引 trait

```rust
pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, id: u64, vector: &[f32]) -> Result<()>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>>;
    fn build_index(&mut self) -> Result<()>;
    fn save(&self, path: &Path) -> Result<()>;
    fn load(&mut self, path: &Path) -> Result<()>;
}
```

### 3.2 向量存储

```rust
pub trait VectorStore {
    fn insert_vector(&self, id: u64, vector: &[f32], metadata: &Metadata) -> Result<()>;
    fn search_similar(&self, query: &[f32], k: usize, filter: Option<&Filter>) -> Result<Vec<VectorResult>>;
    fn delete_vector(&self, id: u64) -> Result<()>;
}
```

---

## 四、图 API

### 4.1 图引擎

```rust
pub trait GraphEngine {
    fn execute(&self, query: &str) -> Result<GraphResult>;
    fn execute_plan(&self, plan: &GraphPlan) -> Result<GraphResult>;
    
    fn insert_node(&self, node: &Node) -> Result<NodeId>;
    fn insert_edge(&self, edge: &Edge) -> Result<EdgeId>;
    fn delete_node(&self, node_id: NodeId) -> Result<()>;
}
```

### 4.2 Cypher 解析

```rust
pub trait CypherParser {
    fn parse(&self, query: &str) -> Result<GraphPlan>;
    fn validate(&self, plan: &GraphPlan) -> Result<()>;
}
```

---

## 五、查询 API

### 5.1 统一查询

```rust
pub trait UnifiedQueryEngine {
    fn execute(&self, request: &UnifiedQueryRequest) -> Result<UnifiedQueryResponse>;
    fn route(&self, request: &UnifiedQueryRequest) -> QueryPlan;
}
```

### 5.2 SQL 执行

```rust
pub trait QueryExecutor {
    fn execute(&self, plan: &PhysicalPlan) -> Result<RecordSet>;
    fn explain(&self, plan: &PhysicalPlan) -> ExplainResult;
}
```

---

## 六、HTTP API 端点

### 6.1 SQL 端点

```bash
# SQL 查询
POST /sql
Content-Type: application/json
{"query": "SELECT * FROM users", "params": []}
```

### 6.2 向量端点

```bash
# 向量搜索
POST /vector
Content-Type: application/json
{"table": "docs", "column": "embedding", "query": [0.1, 0.2, ...], "k": 10}
```

### 6.3 图端点

```bash
# 图查询
POST /graph
Content-Type: application/json
{"query": "MATCH (a)-[:KNOWS]->(b) RETURN b"}
```

### 6.4 统一查询端点

```bash
# 统一查询
POST /api/v2/query/unified
Content-Type: application/json
{"query": "...", "mode": "SqlVector", "weights": {...}}
```

---

## 七、配置 API

### 7.1 配置结构

```rust
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub wal: WalConfig,
    pub vector: VectorConfig,
    pub connection_pool: PoolConfig,
}
```

### 7.2 系统表

```sql
-- 查看版本
SELECT * FROM system.version;

-- 查看配置
SELECT * FROM system.config;

-- 查看统计
SELECT * FROM system.stats;

-- 查看锁
SELECT * FROM system.locks;

-- 查看表信息
SELECT * FROM system.tables;
```

---

*文档版本: 1.0*
*最后更新: 2026-04-16*