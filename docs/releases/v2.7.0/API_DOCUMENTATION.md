# API 文档

> **版本**: alpha/v2.7.0

---

## 1. 核心 API

### 1.1 ExecutionEngine

高级 SQL 执行引擎 API。

```rust
use sqlrustgo::ExecutionEngine;

pub struct ExecutionEngine {
    storage: Arc<RwLock<Box<dyn StorageEngine>>>,
    catalog: Arc<Catalog>,
}

impl ExecutionEngine {
    pub fn new(storage: Box<dyn StorageEngine>) -> Self;
    pub fn execute(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Record>, Error>;
    pub fn execute_batch(&self, sql: &str) -> Result<(), Error>;
}
```

### 1.2 StorageEngine

存储引擎接口。

```rust
pub trait StorageEngine {
    fn scan(&self, table: &str) -> Result<Vec<Record>, Error>;
    fn insert(&self, table: &str, records: Vec<Record>) -> Result<(), Error>;
    fn update(&self, table: &str, records: Vec<Record>, key: &str) -> Result<(), Error>;
    fn delete(&self, table: &str, keys: Vec<Value>) -> Result<(), Error>;
}
```

### 1.3 TransactionManager

事务管理器。

```rust
pub trait TransactionManager {
    fn begin(&self) -> Result<TransactionId, Error>;
    fn commit(&self, tx_id: TransactionId) -> Result<(), Error>;
    fn rollback(&self, tx_id: TransactionId) -> Result<(), Error>;
}
```

---

## 2. Parser API

### 2.1 Parser

SQL 解析器。

```rust
pub struct Parser;

impl Parser {
    pub fn new(sql: &str) -> Self;
    pub fn parse(&self) -> Result<Statement, Error>;
}
```

### 2.2 Statement

解析后的 SQL 语句。

```rust
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    CreateIndex(CreateIndexStatement),
    // v2.7.0新增
    CreateVectorIndex(CreateVectorIndexStatement),
    SyncToQmd(SyncToQmdStatement),
}
```

---

## 3. Executor API

### 3.1 Executor

查询执行器。

```rust
pub trait Executor {
    fn execute(&self, plan: PhysicalPlan) -> Result<Vec<Record>, Error>;
}
```

### 3.2 PhysicalPlan

物理执行计划。

```rust
pub enum PhysicalPlan {
    SeqScan(SeqScanExec),
    IndexScan(IndexScanExec),
    Insert(InsertExec),
    Update(UpdateExec),
    Delete(DeleteExec),
    Join(HashJoinExec),
    Aggregate(AggregateExec),
    Sort(SortExec),
    Limit(LimitExec),
    // v2.7.0新增
    VectorScan(VectorScanExec),
    GraphMatch(GraphMatchExec),
    HybridSearch(HybridSearchExec),
}
```

---

## 4. Storage API

### 4.1 FileStorage

文件存储引擎。

```rust
pub struct FileStorage {
    path: String,
}

impl FileStorage {
    pub fn new(path: &str) -> Self;
    pub fn new_with_wal(path: &str) -> Self;
}
```

### 4.2 MemoryStorage

内存存储引擎。

```rust
pub struct MemoryStorage {
    data: RwLock<HashMap<String, Vec<Record>>>,
}
```

### 4.3 ColumnarStorage

列式存储引擎。

```rust
pub struct ColumnarStorage {
    columns: RwLock<HashMap<String, Column>>,
}
```

---

## 5. qmd-bridge API (v2.7.0 新增)

> qmd-bridge 是 SQLRustGo 与 QMD (Query Memory Database) 的双向数据桥梁

### 5.1 QmdBridge Trait

```rust
use sqlrustgo::qmd_bridge::{QmdBridge, HybridQuery, QmdQuery, QmdData};

pub trait QmdBridge {
    /// 同步数据到 QMD
    fn sync_to_qmd(&mut self, data: &QmdData) -> SqlResult<()>;

    /// 从 QMD 检索
    fn search_from_qmd(&self, query: &QmdQuery) -> SqlResult<QmdResult>;

    /// 混合检索 (向量 + 图谱 + 全文)
    fn hybrid_search(&self, query: &HybridQuery) -> SqlResult<HybridResult>;

    /// 同步状态检查
    fn sync_status(&self) -> SqlResult<SyncStatus>;
}
```

### 5.2 数据类型

```rust
/// QMD 数据格式
pub struct QmdData {
    pub id: String,
    pub data_type: QmdDataType,  // Vector, Graph, Document
    pub content: Vec<f32>,       // 向量数据
    pub metadata: HashMap<String, String>,
    pub timestamp: i64,
}

/// QMD 查询格式
pub struct QmdQuery {
    pub query_type: QueryType,    // Knn, BFS, DFS, Hybrid
    pub vector: Option<Vec<f32>>,
    pub graph_pattern: Option<GraphPattern>,
    pub filters: Vec<Filter>,
    pub limit: usize,
}

/// 混合检索结果
pub struct HybridResult {
    pub vector_results: Vec<SearchResult>,
    pub graph_results: Vec<SearchResult>,
    pub reranked_results: Vec<SearchResult>,
    pub scores: Vec<f32>,
}
```

### 5.3 HybridQuery

混合检索查询结构。

```rust
pub struct HybridQuery {
    pub vector: Option<Vec<f32>>,                    // 向量搜索
    pub graph_pattern: Option<String>,                // 图谱模式
    pub text_query: Option<String>,                   // 关键词搜索
    pub filters: Vec<Filter>,                        // SQL 过滤器
    pub weights: Option<Vec<f32>>,                   // 权重分配 [vec, graph, text]
    pub limit: usize,                                 // 返回结果数
}
```

---

## 6. 向量检索 API (v2.7.0 新增)

### 6.1 VectorIndex Trait

```rust
pub trait VectorIndex {
    fn insert(&mut self, id: &str, vector: &[f32]) -> SqlResult<()>;
    fn search(&self, query: &[f32], k: usize) -> SqlResult<Vec<SearchResult>>;
    fn delete(&mut self, id: &str) -> SqlResult<()>;
}
```

### 6.2 HnswIndex

HNSW 向量索引。

```rust
pub struct HnswIndex {
    dimension: usize,
    max_elements: usize,
    ef_construction: usize,
    m: usize,
}

impl HnswIndex {
    pub fn new(dimension: usize) -> Self;
    pub fn with_params(dimension: usize, ef_construction: usize, m: usize) -> Self;
    pub fn set_ef(&mut self, ef: usize);  // 设置搜索时的 ef 参数
    pub fn set_distance_type(&mut self, distance: DistanceType);  // cosine, l2, dot
}
```

### 6.3 IvfPqIndex

IVF-PQ 向量索引。

```rust
pub struct IvfPqIndex {
    dimension: usize,
    nlist: usize,       // 倒排列表数量
    nprobe: usize,      // 查询时探查的簇数
    m: usize,           // PQ 子空间数
}

impl IvfPqIndex {
    pub fn new(dimension: usize, nlist: usize) -> Self;
    pub fn set_nprobe(&mut self, nprobe: usize);
}
```

### 6.4 DistanceType

```rust
pub enum DistanceType {
    Cosine,   // 余弦距离
    L2,       // 欧氏距离
    Dot,      // 点积
}
```

---

## 7. GMP 图谱 API (v2.7.0 新增)

> GMP (Graph Memory Processor) 是 SQLRustGo 的图谱处理引擎

### 7.1 GraphEngine Trait

```rust
pub trait GraphEngine {
    fn create_graph(&mut self, name: &str) -> SqlResult<()>;
    fn add_edge(&mut self, graph: &str, src: &str, dst: &str, rel: &str) -> SqlResult<()>;
    fn query(&self, graph: &str, pattern: &GraphPattern) -> SqlResult<Vec<GraphResult>>;
}
```

### 7.2 GraphPattern

图查询模式。

```rust
pub struct GraphPattern {
    pub nodes: Vec<NodePattern>,
    pub edges: Vec<EdgePattern>,
    pub conditions: Vec<Condition>,
}

impl GraphPattern {
    pub fn parse(pattern: &str) -> SqlResult<Self>;
}
```

### 7.3 GMP Top10

热点数据查询。

```rust
pub struct GmpTop10 {
    pub metric: Top10Metric,  //热度/最新/推荐
    pub window: TimeWindow,
}

pub enum Top10Metric {
    Popularity,  // 热度
    Latest,      // 最新
    Recommended, // 推荐
}
```

---

## 8. SQL 语法参考

### 8.1 向量检索语法

```sql
-- 创建带向量列的表
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    title VARCHAR(200),
    content TEXT,
    embedding VECTOR(768)  -- 768维向量
);

-- HNSW 向量检索
SELECT id, title, content, 
       VECTOR_DISTANCE(embedding, '[0.1, 0.2, ...]') AS distance
FROM documents
WHERE VECTOR_SEARCH(
    embedding, 
    '[0.1, 0.2, ...]',
    'hnsw',
    distance_type => 'cosine',
    limit => 10
)
ORDER BY distance;

-- IVF-PQ 向量检索
SELECT id, title, content
FROM documents
WHERE VECTOR_SEARCH(
    embedding, 
    '[0.1, 0.2, ...]',
    'ivfpq',
    nlist => 100,
    nprobe => 10,
    limit => 20
);
```

### 8.2 图谱查询语法

```sql
-- 启用图谱
ALTER TABLE friendships ADD GRAPH (user_id, friend_id);

-- 社交网络好友推荐 (二度人脉)
GRAPH MATCH (me)-[:friendship]->(friend)-[:friendship]->(suggestion)
WHERE me.id = @current_user_id
  AND suggestion.id != @current_user_id
RETURN suggestion.name, COUNT(*) AS common_friends
ORDER BY common_friends DESC
LIMIT 10;

-- 知识图谱多跳查询
GRAPH MATCH (university)-[:located_in]->(city)<-[:located_in]-(org)
       <-[:works_for]-(student)<-[:teaches]-(professor)
WHERE university.name = '清华大学'
RETURN DISTINCT org.name AS company;
```

### 8.3 混合检索语法

```sql
-- 混合搜索: 向量 + 图谱 + 全文
SELECT * FROM documents 
WHERE HYBRID_SEARCH(
    embedding => '[0.1, 0.2, ...]',           -- 向量搜索
    graph_pattern => 'MATCH (a)-[r]->(b)',     -- 图谱模式
    text_query => 'Rust 编程',                 -- 关键词搜索
    weights => [0.4, 0.3, 0.3],               -- 权重 [向量, 图谱, 全文]
    limit => 10
);
```

### 8.4 qmd-bridge 语法

```sql
-- 同步数据到 QMD
SYNC TO QMD FROM documents WHERE condition;

-- 检查同步状态
SELECT * FROM qmd_sync_status();
```

---

## 9. 配置选项

### 9.1 配置文件格式

```toml
# /etc/sqlrustgo/config.toml 或 ./config.toml

[server]
host = "0.0.0.0"
port = 3306
max_connections = 100

[storage]
data_dir = "/var/lib/sqlrustgo"
temp_dir = "/tmp/sqlrustgo"

[storage.engine]
type = "buffer_pool"
max_memory_mb = 2048

[vector]
default_index = "hnsw"
hnsw.ef_construction = 200
hnsw.m = 16
ivfpq.nlist = 100
ivfpq.nprobe = 10

[qmd_bridge]
enabled = true
qmd_url = "http://localhost:8080"
sync_interval_ms = 1000
timeout_ms = 5000

[gmp]
graph_cache_size_mb = 1024
top_k = 10

[logging]
level = "info"
output = "stdout"

[security]
enable_auth = true
```

### 9.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_HOST` | 监听地址 | 0.0.0.0 |
| `SQLRUSTGO_PORT` | 监听端口 | 3306 |
| `SQLRUSTGO_DATA_DIR` | 数据目录 | /data |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 | info |
| `SQLRUSTGO_MAX_CONNECTIONS` | 最大连接数 | 100 |
| `SQLRUSTGO_QMD_ENABLED` | 启用 qmd-bridge | false |
| `SQLRUSTGO_QMD_URL` | QMD 服务地址 | localhost:8080 |

---

## 10. CLI 参考

### 10.1 基础命令

```bash
# 启动 REPL
sqlrustgo

# 启动服务器
sqlrustgo server --config /etc/sqlrustgo/config.toml

# 版本信息
sqlrustgo --version

# 帮助信息
sqlrustgo --help
```

### 10.2 管理命令

```bash
# 初始化存储
sqlrustgo init --data-dir /var/lib/sqlrustgo

# 健康检查
sqlrustgo health

# 备份数据
sqlrustgo-tools backup --output /backup/v2.7.0

# 恢复数据
sqlrustgo-tools restore --input /backup/v2.7.0

# 迁移 WAL 格式
sqlrustgo-tools migrate-wal --database mydb

# 验证数据库
sqlrustgo-tools verify --database mydb
```

### 10.3 开发命令

```bash
# 构建
cargo build --all-features

# 测试
cargo test --all-features

# Clippy 检查
cargo clippy --all-features -- -D warnings

# 格式化
cargo fmt --all

# 生成文档
cargo doc --all-features --no-deps
```

### 10.4 Docker 部署

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v2.7.0

# 运行容器
docker run -d \
  --name sqlrustgo \
  -p 3306:3306 \
  -v sqlrustgo-data:/data \
  minzuuniversity/sqlrustgo:v2.7.0

# Docker Compose
docker-compose up -d
```

---

## 11. 常用类型

### 11.1 Value

SQL 值类型。

```rust
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Blob(Vec<u8>),
}
```

### 11.2 Record

数据记录。

```rust
pub struct Record {
    pub values: Vec<Value>,
}
```

### 11.3 DataType

数据类型。

```rust
pub enum DataType {
    Integer,
    Float,
    Varchar(usize),
    Boolean,
    Blob,
    Date,
    Timestamp,
    Vector(usize),  // v2.7.0 新增
}
```

---

## 12. 错误处理

### 12.1 SqlError

SQL 执行错误。

```rust
pub struct SqlError {
    message: String,
}

impl std::error::Error for SqlError {}
impl std::fmt::Display for SqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}
```

### 12.2 SqlResult

结果类型别名。

```rust
pub type SqlResult<T> = Result<T, SqlError>;
```

---

## 13. 示例

### 13.1 基本使用

```rust
use sqlrustgo::{ExecutionEngine, MemoryStorage};

let storage = MemoryStorage::new();
let engine = ExecutionEngine::new(Box::new(storage));

let result = engine.execute("SELECT * FROM users", vec![])?;
```

### 13.2 带参数

```rust
let params = vec![Value::Integer(1)];
let result = engine.execute("SELECT * FROM users WHERE id = ?", params)?;
```

### 13.3 向量检索 (v2.7.0)

```rust
use sqlrustgo::vector::{HnswIndex, DistanceType};

let mut index = HnswIndex::new(768);
index.insert("doc1", &[0.1, 0.2, /* ... */])?;
let results = index.search(&[0.1, 0.2, /* ... */], 10)?;
```

### 13.4 混合检索 (v2.7.0)

```rust
use sqlrustgo::qmd_bridge::{QmdBridge, HybridQuery};

let query = HybridQuery {
    vector: Some(query_vector),
    graph_pattern: Some("MATCH (a)-[r]->(b)".to_string()),
    text_query: Some("Rust".to_string()),
    filters: vec![],
    limit: 10,
};

let result = qmd.hybrid_search(&query)?;
```

---

## 14. 相关文档

- [用户手册](./oo/user-guide/USER_MANUAL.md)
- [升级指南](./UPGRADE_GUIDE.md)
- [qmd-bridge 设计文档](./qmd-bridge-design.md)
- [GMP Top10 场景](./gmp-top10-scenarios.md)
- [快速开始](./QUICK_START.md)
- [部署指南](./DEPLOYMENT_GUIDE.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-22*
