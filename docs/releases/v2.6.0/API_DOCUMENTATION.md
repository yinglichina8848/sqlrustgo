# API 文档

> **版本**: alpha/v2.6.0

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

## 5. 常用类型

### 5.1 Value

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

### 5.2 Record

数据记录。

```rust
pub struct Record {
    pub values: Vec<Value>,
}
```

### 5.3 DataType

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
}
```

---

## 6. 错误处理

### 6.1 SqlError

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

### 6.2 SqlResult

结果类型别名。

```rust
pub type SqlResult<T> = Result<T, SqlError>;
```

---

## 7. 示例

### 7.1 基本使用

```rust
use sqlrustgo::{ExecutionEngine, MemoryStorage};

let storage = MemoryStorage::new();
let engine = ExecutionEngine::new(Box::new(storage));

let result = engine.execute("SELECT * FROM users", vec![])?;
```

### 7.2 带参数

```rust
let params = vec![Value::Integer(1)];
let result = engine.execute("SELECT * FROM users WHERE id = ?", params)?;
```

---

## 8. 相关文档

- [用户手册](./oo/user-guide/USER_MANUAL.md)
- [升级指南](./UPGRADE_GUIDE.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
