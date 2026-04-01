# SQLRustGo v2.1.0 用户手册

**版本**: v2.1.0
**更新日期**: 2026-04-02

---

## 一、安装与启动

### 1.1 安装

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 编译release版本
cargo build --release

# 安装
cargo install --path . --release
```

### 1.2 启动服务器

```bash
# 启动SQLRustGo服务器
cargo run --release --bin sqlrustgo-server

# 或使用安装后的命令
sqlrustgo-server --host 0.0.0.0 --port 5432
```

### 1.3 连接客户端

```bash
# 启动REPL客户端
cargo run --bin sqlrustgo

# 或使用CLI
sqlrustgo --host localhost --port 5432
```

---

## 二、SQL 基本操作

### 2.1 数据库操作

```sql
-- 创建数据库
CREATE DATABASE mydb;

-- 使用数据库
USE mydb;

-- 删除数据库
DROP DATABASE mydb;
```

### 2.2 表操作

```sql
-- 创建表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    created_at DATE
);

-- 查看表结构
DESCRIBE users;

-- 删除表
DROP TABLE users;
```

### 2.3 插入数据

```sql
-- 单行插入
INSERT INTO users (id, name, email, created_at) 
VALUES (1, '张三', 'zhang@example.com', DATE '2026-01-01');

-- 批量插入
INSERT INTO users (id, name, email) VALUES 
    (2, '李四', 'li@example.com'),
    (3, '王五', 'wang@example.com');
```

### 2.4 查询数据

```sql
-- 基本查询
SELECT * FROM users;

-- 条件查询
SELECT name, email FROM users WHERE id > 1;

-- BETWEEN 查询
SELECT * FROM users WHERE id BETWEEN 1 AND 10;

-- IN 查询
SELECT * FROM users WHERE name IN ('张三', '李四', '王五');

-- 日期查询
SELECT * FROM users WHERE created_at = DATE '2026-01-01';
```

### 2.5 更新和删除

```sql
-- 更新数据
UPDATE users SET email = 'new@example.com' WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE id = 3;
```

---

## 三、数据类型

### 3.1 基本数据类型

| 类型 | 说明 | 示例 |
|------|------|------|
| INTEGER | 32位整数 | 42 |
| BIGINT | 64位整数 | 9223372036854775807 |
| VARCHAR(n) | 可变长度字符串 | 'hello' |
| CHAR(n) | 固定长度字符串 | 'hello' |
| DATE | 日期 | DATE '2026-01-01' |
| TIMESTAMP | 时间戳 | TIMESTAMP '2026-01-01 12:00:00' |
| FLOAT | 32位浮点 | 3.14 |
| DOUBLE | 64位浮点 | 3.14159265359 |
| BOOLEAN | 布尔值 | TRUE/FALSE |

### 3.2 UUID 类型 (新增)

```sql
-- 创建带UUID的表
CREATE TABLE sessions (
    session_id UUID PRIMARY KEY,
    user_id INTEGER,
    created_at TIMESTAMP
);

-- 插入UUID
INSERT INTO sessions VALUES 
    ('550e8400-e29b-41d4-a716-446655440000', 1, NOW());
```

### 3.3 ARRAY 类型 (新增)

```sql
-- 创建带数组的表
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    values INTEGER ARRAY
);

-- 插入数组
INSERT INTO tags VALUES (1, 'numbers', ARRAY[1, 2, 3]);
```

### 3.4 ENUM 类型 (新增)

```sql
-- 创建枚举类型
CREATE TYPE status AS ENUM ('pending', 'active', 'closed');

-- 创建带枚举的表
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    status status
);

-- 插入枚举值
INSERT INTO orders VALUES (1, 'pending');
INSERT INTO orders VALUES (2, 'active');
```

---

## 四、工具使用

### 4.1 Physical Backup (物理备份)

```bash
# 创建全量备份
sqlrustgo-tools physical-backup backup \
    --dir /data/backup \
    --data-dir /data/db \
    --wal-dir /data/wal

# 创建增量备份
sqlrustgo-tools physical-backup backup \
    --dir /data/backup \
    --data-dir /data/db \
    --wal-dir /data/wal \
    --incremental

# 列出所有备份
sqlrustgo-tools physical-backup list --dir /data/backup

# 验证备份完整性
sqlrustgo-tools physical-backup verify \
    --dir /data/backup/backup_2026-04-01_12-00-00

# 恢复备份
sqlrustgo-tools physical-backup restore \
    --dir /data/backup/backup_2026-04-01_12-00-00 \
    --target /data/restore
```

### 4.2 备份保留策略 (Prune)

```bash
# 保留最近7个备份
sqlrustgo-tools physical-backup prune \
    --dir /data/backup \
    --keep 7

# 保留最近30天的备份
sqlrustgo-tools physical-backup prune \
    --dir /data/backup \
    --keep-days 30

# 预览删除（不实际删除）
sqlrustgo-tools physical-backup prune \
    --dir /data/backup \
    --keep 7 \
    --dry-run

# 强制删除（跳过确认）
sqlrustgo-tools physical-backup prune \
    --dir /data/backup \
    --keep 7 \
    --force
```

### 4.3 mysqldump 导入导出

```bash
# 导出数据库
sqlrustgo-tools mysqldump --database mydb --out /tmp/mydb.sql

# 导出指定表
sqlrustgo-tools mysqldump --database mydb --table users --out /tmp/users.sql

# 导入数据库
sqlrustgo-tools mysqldump --database mydb --in /tmp/mydb.sql
```

---

## 五、可观测性

### 5.1 健康检查端点

```bash
# 综合健康检查
curl http://localhost:8080/health

# 就绪检查
curl http://localhost:8080/ready

# 响应示例
{
    "status": "healthy",
    "timestamp": "2026-04-02T12:00:00Z",
    "components": {
        "storage": "ok",
        "executor": "ok"
    }
}
```

### 5.2 Prometheus 指标

```bash
# 获取指标
curl http://localhost:8080/metrics

# 指标示例
# HELP sqlrustgo_query_total Total number of queries
# TYPE sqlrustgo_query_total counter
sqlrustgo_query_total{type="select"} 1234
sqlrustgo_query_total{type="insert"} 567
sqlrustgo_query_total{type="update"} 89

# HELP sqlrustgo_query_duration_seconds Query duration
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.1"} 1000
sqlrustgo_query_duration_seconds_bucket{le="1.0"} 1500
```

### 5.3 慢查询日志

慢查询日志默认记录执行时间超过阈值的查询：

```bash
# 查看慢查询日志
cat /var/log/sqlrustgo/slow_query.log

# 日志格式
2026-04-02 12:00:00 [SLOW] query_id=abc123 duration=2.5s 
    SELECT * FROM large_table WHERE ...
```

---

## 六、SQL 防火墙

### 6.1 进程管理

```sql
-- 查看所有进程
SHOW PROCESSLIST;

-- 杀死超时进程
KILL 12345;

-- 杀死指定用户的进程
KILL USER 'zhang@example.com';
```

### 6.2 SQL 防火墙告警

SQL 防火墙会记录异常查询：

```bash
# 查看告警日志
cat /var/log/sqlrustgo/firewall_alerts.log

# 告警格式
2026-04-02 12:00:00 [ALERT] 
    reason="potential_sql_injection" 
    query="SELECT * FROM users WHERE id=1 OR 1=1"
    client="192.168.1.100"
```

---

## 七、窗口函数

### 7.1 基本窗口函数

```sql
-- ROW_NUMBER
SELECT 
    name,
    department,
    salary,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;

-- RANK
SELECT 
    name,
    salary,
    RANK() OVER (ORDER BY salary DESC) as salary_rank
FROM employees;

-- SUM OVER
SELECT 
    name,
    salary,
    SUM(salary) OVER (PARTITION BY department) as dept_total
FROM employees;
```

---

## 八、存储过程

### 8.1 基本语法

```sql
-- 创建存储过程
CREATE PROCEDURE get_user_count(IN dept_id INTEGER, OUT count INTEGER)
AS
BEGIN
    SELECT COUNT(*) INTO count FROM users WHERE department_id = dept_id;
END;

-- 调用存储过程
CALL get_user_count(1, @count);
SELECT @count;
```

### 8.2 控制流

```sql
-- IF 条件
CREATE PROCEDURE check_balance(IN user_id INTEGER)
AS
BEGIN
    DECLARE balance DECIMAL(10,2);
    SELECT account_balance INTO balance FROM accounts WHERE user_id = user_id;
    
    IF balance < 0 THEN
        SELECT '负余额' as status;
    ELSEIF balance = 0 THEN
        SELECT '零余额' as status;
    ELSE
        SELECT '正常' as status;
    END IF;
END;
```

---

## 九、备份与恢复

### 9.1 备份策略建议

| 备份类型 | 频率 | 保留时间 |
|----------|------|----------|
| 全量备份 | 每天 | 7天 |
| 增量备份 | 每小时 | 3天 |
| WAL归档 | 实时 | 1天 |

### 9.2 备份示例

```bash
#!/bin/bash
# 每日全量备份脚本

BACKUP_DIR=/data/backup
DATA_DIR=/data/db
WAL_DIR=/data/wal

# 创建备份
sqlrustgo-tools physical-backup backup \
    --dir $BACKUP_DIR \
    --data-dir $DATA_DIR \
    --wal-dir $WAL_DIR \
    --compression gzip

# 清理旧备份（保留最近7个）
sqlrustgo-tools physical-backup prune \
    --dir $BACKUP_DIR \
    --keep 7 \
    --force

# 验证备份
LATEST=$(ls -t $BACKUP_DIR | head -1)
sqlrustgo-tools physical-backup verify \
    --dir $BACKUP_DIR/$LATEST
```

---

## 十、AgentSQL Extension

### 10.1 概述

AgentSQL 是 SQLRustGo 的 AI Agent 数据库接口，支持自然语言查询、Schema introspection、Agent 记忆等功能。

### 10.2 模块列表

| 模块 | 功能 |
|------|------|
| `schema` | Schema introspection API |
| `stats` | 表统计信息 API |
| `nl2sql` | 自然语言转 SQL |
| `memory` | Agent 上下文记忆 |
| `policy_engine` | RBAC 策略引擎 |
| `column_masking` | 列级数据脱敏 |
| `explain` | 执行计划解释 |
| `optimizer` | SQL 优化建议 |

### 10.3 REST API

```bash
# Schema introspection
GET /agentsql/schema

# SQL 执行
POST /agentsql/query

# 表统计
GET /agentsql/stats

# 自然语言查询
POST /agentsql/nl_query

# 保存记忆
POST /agentsql/memory/save

# 加载记忆
GET /agentsql/memory/load

# 搜索记忆
POST /agentsql/memory/search

# 查询计划解释
POST /agentsql/explain

# SQL 优化建议
POST /agentsql/optimize

# 权限检查
POST /agentsql/policy/check
```

### 10.4 NL2SQL 使用

```rust
use sqlrustgo_agentsql::nl2sql::Nl2SqlService;
use sqlrustgo_agentsql::schema::SchemaService;

let schema = Arc::new(SchemaService::new());
let nl2sql = Nl2SqlService::new(schema);

let result = nl2sql.natural_language_to_sql("show all users where active");
println!("SQL: {}", result.sql);
println!("Confidence: {}", result.confidence);
```

### 10.5 Memory 服务

```rust
use sqlrustgo_agentsql::memory::{MemoryService, SaveMemoryRequest};

let service = MemoryService::new();
let mut service = service;

// 保存记忆
service.save_memory(SaveMemoryRequest {
    content: "User prefers dark mode".to_string(),
    memory_type: Some(MemoryType::Conversation),
    tags: Some(vec!["preference".to_string()]),
    agent_id: Some("agent1".to_string()),
    importance: Some(0.8),
    ..Default::default()
});

// 加载记忆
let response = service.load_memory(LoadMemoryRequest {
    agent_id: Some("agent1".to_string()),
    ..Default::default()
});
```

### 10.6 Policy Engine

```rust
use sqlrustgo_agentsql::policy_engine::{PolicyEngine, PolicyCheckRequest};

let engine = PolicyEngine::new();
let request = PolicyCheckRequest {
    user_id: "user1".to_string(),
    resource: "table:orders".to_string(),
    action: "SELECT".to_string(),
    context: None,
};

let response = engine.check(&request);
if !response.allowed {
    println!("Access denied: {}", response.denial_reason);
}
```

### 10.7 Column Masking

```rust
use sqlrustgo_agentsql::column_masking::{ColumnMasker, MaskingType};

let masker = ColumnMasker::new();

// 邮箱脱敏
let masked_email = masker.mask_value("email", &serde_json::json!("user@example.com"));
assert!(masked_email.as_str().unwrap().contains('*'));

// SSN 脱敏
let masked_ssn = masker.mask_value("ssn", &serde_json::json!("123-45-6789"));
```

### 10.8 测试

```bash
# 运行 AgentSQL 单元测试
cargo test -p sqlrustgo-agentsql --lib

# 运行所有 AgentSQL 测试
cargo test agentsql
```

---

## 十一、配置参考

### 10.1 配置文件位置

- 主配置: `etc/sqlrustgo.toml`
- 日志配置: `etc/logging.toml`

### 10.2 主要配置项

```toml
[server]
host = "0.0.0.0"
port = 5432
workers = 4

[storage]
type = "memory"  # memory, columnar
data_dir = "/data/db"

[observability]
prometheus_enabled = true
slow_query_threshold_ms = 1000

[firewall]
enabled = true
alert_on_suspicious = true
```

---

## 附录 A: 错误代码

| 代码 | 说明 |
|------|------|
| E0001 | 语法错误 |
| E0002 | 表不存在 |
| E0003 | 列不存在 |
| E0004 | 约束违反 |
| E0005 | 权限不足 |
| E0006 | 连接超时 |

---

## 附录 B: 限制

- 最大表名长度: 64 字符
- 最大列数: 1024
- 最大行大小: 64KB
- 最大数组长度: 65535
- 最大字符串长度: 16MB

---

*用户手册 v2.1.0*
