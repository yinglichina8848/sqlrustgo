# SQLRustGo v2.1.0 API 文档

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、核心模块

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

#### MemoryStorage

```rust
pub struct MemoryStorage {
    tables: HashMap<u64, Table>,
    indexes: HashMap<String, BTreeIndex>,
}

impl MemoryStorage {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn insert(&self, table: &str, row: &Row) -> Result<(), SqlError>;
    pub fn update(&self, table: &str, row: &Row) -> Result<usize, SqlError>;
    pub fn delete(&self, table: &str, key: &Value) -> Result<usize, SqlError>;
    pub fn scan(&self, table: &str) -> Result<RowIterator, SqlError>;
    pub fn batch_scan(&self, table: &str, batch_size: usize) -> Result<BatchIterator, SqlError>;
}
```

---

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

---

### 1.3 Parser 模块

#### 新增 Token (v2.1.0)

```rust
pub enum Token {
    // v2.1.0 新增
    In,           // IN 关键字
    Uuid,         // UUID 关键字
    Array,        // ARRAY 关键字
    Enum,         // ENUM 关键字
    DateLiteral,  // DATE 字面量
    TimestampLiteral, // TIMESTAMP 字面量
    Between,      // BETWEEN 关键字
}
```

#### 新增 Expression (v2.1.0)

```rust
pub enum Expression {
    // v2.1.0 新增
    InList {
        expr: Box<Expression>,
        list: Vec<Expression>,
    },
    Between {
        expr: Box<Expression>,
        low: Box<Expression>,
        high: Box<Expression>,
    },
}
```

---

## 二、数据类型

### 2.1 DataType 枚举

```rust
pub enum DataType {
    // 基础类型
    Integer,
    BigInt,
    Float,
    Double,
    Varchar(usize),
    Char(usize),
    Boolean,
    
    // 日期时间
    Date,
    Timestamp,
    Time,
    
    // 复杂类型 (v2.1.0 新增)
    Uuid,
    Array(Box<DataType>),
    Enum(Vec<String>),
    
    // 存储类型
    Blob,
    Text,
}
```

### 2.2 Value 类型

```rust
pub enum Value {
    Null,
    Integer(i64),
    BigInt(i128),
    Float(f32),
    Double(f64),
    Varchar(String),
    Char(String),
    Boolean(bool),
    Date(NaiveDate),
    Timestamp(NaiveDateTime),
    
    // v2.1.0 新增
    Uuid(Uuid),
    Array(Vec<Value>),
    Enum(String),
}
```

---

## 三、Planner 模块

### 3.1 Expr 枚举

```rust
pub enum Expr {
    // 基础表达式
    Literal(Value),
    Column(String),
    Parameter(usize),
    
    // 运算表达式
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    
    // 函数
    Function {
        name: String,
        args: Vec<Expr>,
    },
    
    // 聚合
    Aggregate {
        func: AggregateFunction,
        args: Vec<Expr>,
        group_by: Vec<Expr>,
    },
    
    // v2.1.0 新增
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
    },
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
    },
}
```

---

## 四、工具 CLI

### 4.1 Physical Backup

```rust
pub struct PhysicalBackupOpts {
    pub command: PhysicalBackupCommand,
}

pub enum PhysicalBackupCommand {
    Backup {
        dir: PathBuf,
        data_dir: PathBuf,
        wal_dir: PathBuf,
        incremental: bool,
        compression: bool,
    },
    List {
        dir: PathBuf,
    },
    Verify {
        dir: PathBuf,
    },
    Restore {
        dir: PathBuf,
        target: PathBuf,
    },
    Prune {
        dir: PathBuf,
        keep: Option<usize>,
        keep_days: Option<usize>,
        dry_run: bool,
        force: bool,
    },
}

pub fn prune_physical_backups(
    dir: &Path,
    keep: Option<usize>,
    keep_days: Option<usize>,
    dry_run: bool,
    force: bool,
) -> Result<()>;
```

### 4.2 mysqldump

```rust
pub struct MysqldumpOpts {
    pub database: String,
    pub table: Option<String>,
    pub out: Option<PathBuf>,
    pub in: Option<PathBuf>,
}
```

---

## 五、可观测性 API

### 5.1 Health Endpoints

```rust
// GET /health - 综合健康检查
// GET /ready - 就绪检查

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub components: HashMap<String, ComponentStatus>,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: String,
    pub message: Option<String>,
}
```

### 5.2 Prometheus Metrics

```rust
// GET /metrics - Prometheus 格式指标

pub struct QueryMetrics {
    pub query_total: CounterVec,
    pub query_duration_seconds: HistogramVec,
}

pub struct ExecutorMetrics {
    pub rows_processed: CounterVec,
    pub operator_duration_seconds: HistogramVec,
}

pub struct StorageMetrics {
    pub read_bytes: Counter,
    pub write_bytes: Counter,
    pub page_cache_hits: Counter,
    pub page_cache_misses: Counter,
}
```

---

## 六、SQL Firewall

### 6.1 KILL Statement

```sql
-- 查看进程列表
SHOW PROCESSLIST;

-- 终止进程
KILL process_id;

-- 终止用户所有进程
KILL USER 'username';
```

### 6.2 Alert System

```rust
pub struct FirewallAlert {
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub query: String,
    pub client_ip: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}
```

---

*API 文档 v2.1.0*
