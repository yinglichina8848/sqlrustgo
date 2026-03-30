# SQLRustGo v2.0.0 API 文档

**版本**: v2.0.0
**更新日期**: 2026-03-29

---

## 一、核心模块 API

### 1.1 Storage 模块

#### StorageEngine Trait

```rust
pub trait StorageEngine {
    // 读取操作
    fn read_page(&self, table_id: u64, page_id: u64) -> Result<Page, SqlError>;
    fn write_page(&self, table_id: u64, page: &Page) -> Result<(), SqlError>;

    // 表操作
    fn create_table(&self, schema: &Schema) -> Result<u64, SqlError>;
    fn drop_table(&self, table_id: u64) -> Result<(), SqlError>;
    fn get_table(&self, name: &str) -> Result<Table, SqlError>;

    // 事务
    fn begin_transaction(&self) -> Result<TransactionId, SqlError>;
    fn commit(&self, tx_id: TransactionId) -> Result<(), SqlError>;
    fn rollback(&self, tx_id: TransactionId) -> Result<(), SqlError>;
}
```

#### RowStorage (行式存储)

```rust
pub struct RowStorage {
    buffer_pool: BufferPool,
    page_manager: PageManager,
}

impl RowStorage {
    pub fn new(config: StorageConfig) -> Self;
    pub fn insert(&self, table: &str, row: &Row) -> Result<(), SqlError>;
    pub fn update(&self, table: &str, row: &Row) -> Result<(), SqlError>;
    pub fn delete(&self, table: &str, key: &Value) -> Result<(), SqlError>;
    pub fn scan(&self, table: &str) -> Result<RowIterator, SqlError>;
}
```

#### ColumnarStorage (列式存储)

```rust
pub struct ColumnarStorage {
    segments: HashMap<u64, ColumnSegment>,
    buffer_pool: BufferPool,
}

impl ColumnarStorage {
    pub fn new(config: StorageConfig) -> Self;

    // 列式操作
    pub fn write_column(&self, table: &str, column: &Column) -> Result<(), SqlError>;
    pub fn read_column(&self, table: &str, column: &str) -> Result<Column, SqlError>;
    pub fn projection_pushdown(&self, table: &str, columns: &[&str]) -> Result<ColumnIterator, SqlError>;
}
```

### 1.2 Executor 模块

#### Executor Trait

```rust
pub trait Executor {
    fn execute(&self, plan: PhysicalPlan) -> Result<ExecutionResult, SqlError>;
    fn explain(&self, plan: PhysicalPlan) -> Result<String, SqlError>;
}
```

#### ParallelExecutor

```rust
pub struct ParallelExecutor {
    scheduler: TaskScheduler,
    workers: usize,
}

impl ParallelExecutor {
    pub fn new(workers: usize) -> Self;
    pub fn execute_parallel(&self, plan: PhysicalPlan) -> Result<ExecutionResult, SqlError>;
}

pub trait TaskScheduler {
    fn schedule(&self, task: Task) -> TaskId;
    fn wait(&self, task_id: TaskId) -> Result<(), SqlError>;
}
```

#### 执行器节点

```rust
// Scan 节点
pub struct SeqScan {
    table: String,
    projection: Vec<String>,
    filter: Option<Expr>,
}

// Join 节点
pub struct HashJoin {
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    condition: JoinCondition,
}

// 聚合节点
pub struct HashAgg {
    group_by: Vec<String>,
    aggregates: Vec<AggregateFunction>,
}
```

### 1.3 Transaction 模块

#### TransactionManager

```rust
pub trait TransactionManager {
    fn begin(&self) -> Result<TxId, SqlError>;
    fn commit(&self, tx_id: TxId) -> Result<(), SqlError>;
    fn rollback(&self, tx_id: TxId) -> Result<(), SqlError>;
    fn get_status(&self, tx_id: TxId) -> TxStatus;
}
```

#### DistributedTransactionManager (2PC)

```rust
pub struct DistributedTransactionManager {
    coordinator: CoordinatorClient,
    participants: Vec<ParticipantClient>,
}

impl DistributedTransactionManager {
    pub fn new(coordinator_addr: &str) -> Self;

    // 2PC 协议
    pub fn begin_distributed(&self) -> Result<DtxId, SqlError>;
    pub fn prepare(&self, dtx_id: DtxId) -> Result<PrepareResult, SqlError>;
    pub fn commit(&self, dtx_id: DtxId) -> Result<(), SqlError>;
    pub fn rollback(&self, dtx_id: DtxId) -> Result<(), SqlError>;
}

// Coordinator API
pub trait Coordinator {
    fn create_transaction(&self) -> DtxId;
    fn add_participant(&self, dtx_id: DtxId, participant: &str);
    fn prepare(&self, dtx_id: DtxId) -> PhaseResult;
    fn commit(&self, dtx_id: DtxId) -> PhaseResult;
    fn rollback(&self, dtx_id: DtxId) -> PhaseResult;
}
```

### 1.4 Parser 模块

#### Parser

```rust
pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(sql: &str) -> Self;
    pub fn parse(&self) -> Result<Statement, SqlError>;
    pub fn parse_statements(&self) -> Result<Vec<Statement>, SqlError>;
}

// COPY 语句 (v2.0 新增)
#[derive(Debug)]
pub enum CopyStatement {
    Import {
        table: String,
        source: FileSource,
        format: FileFormat,
    },
    Export {
        table: String,
        dest: FileDest,
        format: FileFormat,
    },
}

pub enum FileFormat {
    Parquet,
    Csv,
    Json,
}
```

### 1.5 Catalog 模块

#### Catalog

```rust
pub trait Catalog {
    fn get_table(&self, name: &str) -> Result<TableSchema, SqlError>;
    fn get_tables(&self) -> Result<Vec<String>, SqlError>;
    fn create_table(&self, schema: TableSchema) -> Result<(), SqlError>;
    fn drop_table(&self, name: &str) -> Result<(), SqlError>;
}

pub struct DatabaseCatalog {
    tables: HashMap<String, TableSchema>,
}

impl DatabaseCatalog {
    pub fn new() -> Self;
    pub fn add_table(&mut self, schema: TableSchema) -> Result<(), SqlError>;
}
```

### 1.6 Types 模块

#### Value

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Boolean(bool),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
    Varchar(String),
    Date(i32),
    Timestamp(i64),
}

impl Value {
    pub fn type_info(&self) -> DataType;
    pub fn cast_to(&self, target: DataType) -> Result<Value, SqlError>;
    pub fn serialize(&self) -> Vec<u8>;
    pub fn deserialize(data: &[u8]) -> Result<Self, SqlError>;
}
```

### 1.7 Security 模块

#### Authenticator

```rust
pub trait Authenticator {
    fn authenticate(&self, user: &str, password: &str) -> Result<UserId, SqlError>;
    fn create_user(&self, user: &str, password: &str, role: Role) -> Result<(), SqlError>;
    fn grant(&self, who: UserId, what: Permission, on: ObjectId) -> Result<(), SqlError>;
    fn revoke(&self, who: UserId, what: Permission, on: ObjectId) -> Result<(), SqlError>;
}

#[derive(Debug, Clone)]
pub enum Permission {
    Select,
    Insert,
    Update,
    Delete,
    Grant,
}

#[derive(Debug, Clone)]
pub struct Role {
    name: String,
    permissions: Vec<Permission>,
}
```

---

## 二、配置文件格式

### 2.1 完整配置示例

```toml
[server]
bind = "0.0.0.0:3306"
max_connections = 100

[storage]
type = "columnar"              # "row" 或 "columnar"
buffer_pool_size = 8192        # 页缓存大小
data_dir = "/var/lib/sqlrustgo"

[transaction]
distributed = true             # 启用 2PC
coordinator_address = "grpc://localhost:50051"

[parquet]
enabled = true
chunk_size = 8388608           # 8MB

[executor]
parallel_enabled = true
max_threads = 8

[security]
tls_enabled = false
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[log]
level = "info"
path = "/var/log/sqlrustgo"
```

---

## 三、错误代码

### SqlError 类型

```rust
pub enum SqlError {
    // 解析错误 (1xxx)
    ParseError(String),
    SyntaxError(String),

    // 执行错误 (2xxx)
    ExecutionError(String),
    ColumnNotFound(String),
    TableNotFound(String),
    TypeMismatch { expected: DataType, actual: DataType },

    // 存储错误 (3xxx)
    IoError(String),
    PageNotFound { table: u64, page: u64 },
    ChecksumMismatch { expected: u32, actual: u32 },

    // 事务错误 (4xxx)
    TransactionNotFound(TxId),
    TransactionAborted(TxId),
    DeadlockDetected { tx1: TxId, tx2: TxId },

    // 分布式错误 (5xxx)
    CoordinatorUnreachable(String),
    ParticipantTimeout(String),
    DtxNotFound(DtxId),

    // 权限错误 (6xxx)
    Unauthorized(String),
    PermissionDenied { user: String, action: String },
}
```

---

## 四、使用示例

### 4.1 基本使用

```rust
use sqlrustgo::{Engine, Config};

let config = Config::from_file("config.toml")?;
let engine = Engine::new(config)?;

engine.execute("CREATE TABLE users (id INT, name VARCHAR)")?;
engine.execute("INSERT INTO users VALUES (1, 'Alice')")?;

let result = engine.execute("SELECT * FROM users WHERE id = 1")?;
for row in result {
    println!("{:?}", row);
}
```

### 4.2 COPY Parquet

```rust
// 导入
engine.execute(
    "COPY users FROM '/data/users.parquet' (FORMAT PARQUET)"
)?;

// 导出
engine.execute(
    "COPY users TO '/export/users.parquet' (FORMAT PARQUET)"
)?;
```

### 4.3 分布式事务

```rust
let mut dtx = engine.begin_distributed()?;

dtx.execute("INSERT INTO orders VALUES (1, 'order1')")?;
dtx.execute("UPDATE inventory SET stock = stock - 1 WHERE product_id = 1")?;

dtx.commit()?;
```

---

## 五、变更日志

### v2.0.0 新增 API

| 模块 | 新增 API | 说明 |
|------|----------|------|
| storage | ColumnarStorage | 列式存储引擎 |
| storage | ParquetCompat | Parquet 兼容格式 |
| executor | ParallelExecutor | 并行执行器 |
| executor | TaskScheduler | 任务调度器 |
| transaction | DistributedTM | 2PC 分布式事务 |
| parser | CopyStatement | COPY 语句 |
| catalog | DatabaseCatalog | 目录服务 |
| security | RBAC | 权限管理 |

---

*最后更新: 2026-03-29*
