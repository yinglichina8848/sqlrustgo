# SQLRustGo v1.1.0 API 文档

> 版本：v1.1.0
> 发布日期：2026-03-03

---

## 一、核心模块

### 1.1 执行器模块 (executor)

#### ExecutionEngine

主执行引擎，负责执行物理查询计划。

```rust
use sqlrustgo::executor::ExecutionEngine;

/// 创建新的执行引擎实例
let mut engine = ExecutionEngine::new();

/// 使用数据目录创建执行引擎
let engine = ExecutionEngine::with_data_dir("/path/to/data")?;

/// 执行物理计划
let result = engine.execute(physical_plan)?;
```

**方法**:

| 方法 | 说明 | 返回类型 |
|------|------|----------|
| `new()` | 创建内存模式执行引擎 | `ExecutionEngine` |
| `with_data_dir(path)` | 创建持久化模式执行引擎 | `SqlResult<ExecutionEngine>` |
| `execute(plan)` | 执行物理查询计划 | `SqlResult<QueryResult>` |
| `get_table(name)` | 获取表引用 | `Option<&Table>` |
| `create_index(table, column)` | 创建索引 | `SqlResult<()>` |

#### QueryResult

查询执行结果。

```rust
/// 查询结果结构
pub struct QueryResult {
    /// 列名列表
    pub columns: Vec<String>,
    /// 数据行
    pub rows: Vec<Vec<Value>>,
    /// 受影响的行数（INSERT/UPDATE/DELETE）
    pub affected_rows: usize,
}
```

---

### 1.2 规划器模块 (planner)

#### LogicalPlan

逻辑查询计划，表示查询的逻辑结构。

```rust
use sqlrustgo::planner::LogicalPlan;

/// 从解析结果创建逻辑计划
let logical_plan = LogicalPlan::from_parse_result(parse_result)?;

/// 转换为物理计划
let physical_plan = logical_plan.to_physical()?;

/// 优化逻辑计划
let optimized = logical_plan.optimize()?;
```

**计划类型**:

| 类型 | 说明 |
|------|------|
| `Select` | SELECT 查询 |
| `Insert` | INSERT 语句 |
| `Update` | UPDATE 语句 |
| `Delete` | DELETE 语句 |
| `CreateTable` | 创建表 |
| `DropTable` | 删除表 |

#### PhysicalPlan

物理查询计划，表示查询的执行方式。

```rust
use sqlrustgo::planner::PhysicalPlan;

/// 物理计划 trait
pub trait PhysicalPlan: Send + Sync {
    /// 执行计划
    fn execute(&self, engine: &ExecutionEngine) -> SqlResult<QueryResult>;
    
    /// 获取计划成本估算
    fn cost(&self) -> f64;
    
    /// 获取输出列
    fn output_columns(&self) -> Vec<String>;
}
```

---

### 1.3 解析器模块 (parser)

#### parse

解析 SQL 语句。

```rust
use sqlrustgo::parser::parse;

/// 解析 SQL 字符串
let result = parse("SELECT * FROM users WHERE id = 1")?;

/// 结果类型
pub enum ParseResult {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    CreateIndex(CreateIndexStatement),
    DropIndex(DropIndexStatement),
}
```

---

### 1.4 存储模块 (storage)

#### FileStorage

文件存储引擎。

```rust
use sqlrustgo::storage::FileStorage;

/// 创建文件存储实例
let storage = FileStorage::new("/path/to/data")?;

/// 创建表
storage.create_table("users", &columns)?;

/// 插入数据
storage.insert("users", &row)?;

/// 查询数据
let rows = storage.scan("users", &predicate)?;

/// 创建索引
storage.create_index("users", "id")?;
```

**方法**:

| 方法 | 说明 |
|------|------|
| `new(path)` | 创建存储实例 |
| `create_table(name, columns)` | 创建表 |
| `drop_table(name)` | 删除表 |
| `insert(table, row)` | 插入数据 |
| `delete(table, predicate)` | 删除数据 |
| `update(table, updates, predicate)` | 更新数据 |
| `scan(table, predicate)` | 扫描表 |
| `create_index(table, column)` | 创建索引 |
| `drop_index(table, column)` | 删除索引 |

---

### 1.5 网络模块 (network)

#### Server

MySQL 协议服务器。

```rust
use sqlrustgo::network::Server;

/// 创建服务器
let server = Server::bind("0.0.0.0:3306")?;

/// 运行服务器
server.run().await?;
```

**配置**:

```rust
/// 服务器配置
pub struct ServerConfig {
    /// 绑定地址
    pub bind: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时（秒）
    pub timeout: u64,
}
```

#### Client

MySQL 协议客户端。

```rust
use sqlrustgo::network::Client;

/// 连接服务器
let mut client = Client::connect("127.0.0.1:3306").await?;

/// 执行查询
let result = client.query("SELECT * FROM users").await?;

/// 关闭连接
client.close().await?;
```

---

### 1.6 类型模块 (types)

#### Value

数据值类型。

```rust
use sqlrustgo::types::Value;

/// 值类型枚举
pub enum Value {
    /// 空值
    Null,
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Boolean(bool),
    /// 二进制数据
    Bytes(Vec<u8>),
}
```

#### SqlError

错误类型。

```rust
use sqlrustgo::types::SqlError;

/// 错误类型枚举
pub enum SqlError {
    /// 表不存在
    TableNotFound(String),
    /// 列不存在
    ColumnNotFound(String),
    /// 索引不存在
    IndexNotFound(String),
    /// 语法错误
    SyntaxError(String),
    /// 类型错误
    TypeError(String),
    /// 存储错误
    StorageError(String),
    /// 网络错误
    NetworkError(String),
    /// 认证错误
    AuthError(String),
}
```

#### SqlResult

结果类型别名。

```rust
use sqlrustgo::types::SqlResult;

/// 结果类型
pub type SqlResult<T> = Result<T, SqlError>;
```

---

## 二、事务模块

### 2.1 TransactionManager

事务管理器。

```rust
use sqlrustgo::transaction::TransactionManager;

/// 创建事务管理器
let tm = TransactionManager::new(wal);

/// 开始事务
let tx_id = tm.begin()?;

/// 提交事务
tm.commit(tx_id)?;

/// 回滚事务
tm.rollback(tx_id)?;
```

### 2.2 WriteAheadLog

预写日志。

```rust
use sqlrustgo::transaction::WriteAheadLog;

/// 创建 WAL
let wal = WriteAheadLog::new("/path/to/wal")?;

/// 追加记录
wal.append(&WalRecord::Begin { tx_id: 1 })?;

/// 读取所有记录
let records = wal.read_all()?;

/// 截断日志
wal.truncate()?;
```

---

## 三、认证模块

### 3.1 AuthManager

认证管理器。

```rust
use sqlrustgo::auth::{AuthManager, Role};

/// 创建认证管理器
let auth = AuthManager::new();

/// 注册用户
auth.register("admin", "password", Role::Admin)?;

/// 登录
let session = auth.login("admin", "password")?;

/// 验证会话
let valid = auth.verify(&session.id)?;

/// 登出
auth.logout(&session.id)?;
```

**角色类型**:

| 角色 | 权限 |
|------|------|
| `Admin` | 完全访问权限 |
| `User` | 读写权限 |
| `Readonly` | 只读权限 |

---

## 四、Lexer 模块

### 4.1 Lexer

词法分析器。

```rust
use sqlrustgo::lexer::Lexer;

/// 创建词法分析器
let lexer = Lexer::new("SELECT * FROM users");

/// 获取所有 token
let tokens = lexer.tokenize()?;

/// 迭代 token
for token in lexer {
    println!("{:?}", token);
}
```

### 4.2 Token

词法单元。

```rust
/// Token 类型
pub enum Token {
    /// 关键字
    Keyword(Keyword),
    /// 标识符
    Identifier(String),
    /// 字面量
    Literal(Value),
    /// 运算符
    Operator(Operator),
    /// 分隔符
    Delimiter(Delimiter),
    /// 文件结束
    Eof,
}
```

---

## 五、使用示例

### 5.1 基础查询

```rust
use sqlrustgo::{ExecutionEngine, parse};

fn main() -> SqlResult<()> {
    let mut engine = ExecutionEngine::new();
    
    // 创建表
    engine.execute(parse("CREATE TABLE users (id INT, name TEXT)")?)?;
    
    // 插入数据
    engine.execute(parse("INSERT INTO users VALUES (1, 'Alice')")?)?;
    
    // 查询数据
    let result = engine.execute(parse("SELECT * FROM users")?)?;
    println!("{:?}", result);
    
    Ok(())
}
```

### 5.2 网络服务

```rust
use sqlrustgo::network::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::bind("0.0.0.0:3306")?;
    server.run().await?;
    Ok(())
}
```

### 5.3 事务操作

```rust
use sqlrustgo::{ExecutionEngine, TransactionManager, WriteAheadLog};
use std::sync::Arc;

fn main() -> SqlResult<()> {
    let wal = Arc::new(WriteAheadLog::new("./wal")?);
    let tm = TransactionManager::new(wal);
    let mut engine = ExecutionEngine::new();
    
    let tx_id = tm.begin()?;
    
    engine.execute(parse("INSERT INTO users VALUES (2, 'Bob')")?)?;
    
    tm.commit(tx_id)?;
    
    Ok(())
}
```

---

## 六、错误处理

### 6.1 错误传播

```rust
use sqlrustgo::{SqlResult, SqlError};

fn query_users() -> SqlResult<Vec<User>> {
    let mut engine = ExecutionEngine::new();
    let result = engine.execute(parse("SELECT * FROM users")?)?;
    
    // 转换结果
    result.rows.into_iter()
        .map(|row| User::from_row(row))
        .collect()
}
```

### 6.2 错误匹配

```rust
match engine.execute(plan) {
    Ok(result) => println!("Success: {:?}", result),
    Err(SqlError::TableNotFound(name)) => eprintln!("Table not found: {}", name),
    Err(SqlError::SyntaxError(msg)) => eprintln!("Syntax error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
*最后更新: 2026-03-03*
