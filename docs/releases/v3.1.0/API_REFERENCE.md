# v3.1.0 API Reference

> **版本**: v3.1.0
> **发布日期**: 2026-05-14

---

## 一、Rust API

### 1.1 核心 Crate

#### sqlrustgo-executor

```rust
use sqlrustgo::executor::{Executor, ResultSet};

pub struct Executor {
    // 执行 SQL 查询
    pub fn execute(&mut self, sql: &str) -> Result<ResultSet>;
    
    // 预编译语句
    pub fn prepare(&mut self, sql: &str) -> Result<Statement>;
    
    // 事务控制
    pub fn begin(&mut self) -> Result<()>;
    pub fn commit(&mut self) -> Result<()>;
    pub fn rollback(&mut self) -> Result<()>;
}
```

#### sqlrustgo-parser

```rust
use sqlrustgo::parser::{Parser, Statement};

pub struct Parser {
    pub fn parse(&mut self, sql: &str) -> Result<Vec<Statement>>;
    pub fn parse_single(&mut self, sql: &str) -> Result<Statement>;
}
```

#### sqlrustgo-planner

```rust
use sqlrustgo::planner::{Planner, LogicalPlan};

pub struct Planner {
    pub fn plan(&mut self, stmt: Statement) -> Result<LogicalPlan>;
    pub fn optimize(&mut self, plan: LogicalPlan) -> Result<LogicalPlan>;
}
```

#### sqlrustgo-storage

```rust
use sqlrustgo::storage::{Storage, Page, Transaction};

pub trait Storage {
    fn read_page(&self, page_id: u64) -> Result<Page>;
    fn write_page(&mut self, page: Page) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}
```

#### sqlrustgo-transaction

```rust
use sqlrustgo::transaction::{Transaction, IsolationLevel};

pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

pub trait Transaction {
    fn begin(&mut self, level: IsolationLevel) -> Result<()>;
    fn commit(&mut self) -> Result<()>;
    fn rollback(&mut self) -> Result<()>;
}
```

---

## 二、CLI 命令

### 2.1 服务器

```bash
# 启动服务器
sqlrustgo --port 3306 --data-dir /data/sqlrustgo

# 带 TLS 启动
sqlrustgo --port 3306 --ssl-mode=REQUIRED --ssl-cert=/path/to/cert.pem
```

### 2.2 客户端

```bash
# 连接数据库
sqlrustgo-cli --host 127.0.0.1 --port 3306 -u root

# 执行 SQL 文件
sqlrustgo-cli < queries.sql

# 导入 CSV
sqlrustgo-cli --import-csv /path/to/data.csv --table mytable
```

### 2.3 工具

```bash
# 数据恢复
sqlrustgo-recover --data-dir /data/sqlrustgo

# 完整性检查
sqlrustgo-check --data-dir /data/sqlrustgo

# 基准测试
sqlrustgo-bench --bench tpch --sf 1

# 性能分析
sqlrustgo-profile --query "SELECT * FROM users"
```

---

## 三、配置文件

### 3.1 config.toml

```toml
[server]
port = 3306
host = "0.0.0.0"
max_connections = 100
socket_path = "/tmp/sqlrustgo.sock"

[storage]
data_dir = "./data"
page_size = 8192
buffer_pool_size = 1024

[transaction]
isolation_level = "repeatable_read"
wal_mode = "write_ahead"

[logging]
level = "info"
format = "json"
output = "stdout"

[security]
ssl_enabled = false
ssl_cert = "/path/to/cert.pem"
ssl_key = "/path/to/key.pem"
```

---

## 四、环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SQLRUSTGO_PORT` | 3306 | 服务端口 |
| `SQLRUSTGO_DATA_DIR` | ./data | 数据目录 |
| `SQLRUSTGO_LOG_LEVEL` | info | 日志级别 |
| `SQLRUSTGO_MAX_CONNECTIONS` | 100 | 最大连接数 |

---

## 五、错误码

| 错误码 | 说明 |
|--------|------|
| ER_PARSE_ERROR | SQL 解析错误 |
| ER_TABLE_EXISTS | 表已存在 |
| ER_DUP_ENTRY | 重复条目 |
| ER_NO_SUCH_TABLE | 表不存在 |
| ER_LOCK_DEADLOCK | 死锁 |
| ER_LOCK_WAIT_TIMEOUT | 锁等待超时 |

---

## 六、类型映射

### 6.1 SQL 到 Rust

| SQL 类型 | Rust 类型 |
|----------|-----------|
| INT | i32 |
| BIGINT | i64 |
| FLOAT | f32 |
| DOUBLE | f64 |
| VARCHAR | String |
| TEXT | String |
| BLOB | Vec<u8> |
| DATE | chrono::NaiveDate |
| TIMESTAMP | chrono::DateTime |

---

*最后更新: 2026-05-14*
