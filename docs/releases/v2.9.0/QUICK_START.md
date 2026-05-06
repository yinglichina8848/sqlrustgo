# SQLRustGo v2.9.0 快速开始

> **版本**: v2.9.0
> **阶段**: Alpha → Beta 过渡中

## 环境要求

- Rust 1.85+
-Tokio 异步运行时
- Cargo

## 安装构建

```bash
# 克隆仓库
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# 构建（包含所有功能）
cargo build --all-features

# 运行测试
cargo test --all-features
```

## 启动 REPL

```bash
cargo run --bin sqlrustgo
```

示例会话：

```sql
-- 创建数据库
CREATE DATABASE test;

-- 使用数据库
USE test;

-- 创建表
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100)
);

-- 插入数据
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- 查询数据
SELECT * FROM users;

-- 退出
EXIT;
```

## 连接方式

### TCP 连接（MySQL 协议）

```bash
# 默认端口 3306
mysql -h 127.0.0.1 -P 3306 -u root
```

### 配置连接

编辑 `config.toml`：

```toml
[server]
host = "0.0.0.0"
port = 3306

[storage]
type = "file"
path = "./data"
```

## 基本 SQL 操作

### DDL

```sql
-- 创建表
CREATE TABLE t (
    id INT PRIMARY KEY,
    name TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 修改表
ALTER TABLE t ADD COLUMN email TEXT;
ALTER TABLE t DROP COLUMN email;

-- 创建索引
CREATE INDEX idx_name ON t(name);
CREATE UNIQUE INDEX idx_id ON t(id);

-- 删除表/索引
DROP TABLE t;
DROP INDEX idx_name;
```

### DML

```sql
-- 插入
INSERT INTO t (id, name) VALUES (1, 'test');
INSERT INTO t (id, name) VALUES (2, 'test2') ON DUPLICATE KEY UPDATE name = 'updated';

-- 更新
UPDATE t SET name = 'new_name' WHERE id = 1;

-- 删除
DELETE FROM t WHERE id = 1;
```

### 查询

```sql
-- 基础查询
SELECT * FROM t WHERE id > 10;

-- 聚合
SELECT COUNT(*), SUM(amount) FROM orders GROUP BY status;

-- JOIN
SELECT u.name, o.amount
FROM users u
JOIN orders o ON u.id = o.user_id;

-- 子查询
SELECT * FROM t WHERE id IN (SELECT id FROM other WHERE active = true);

-- CTE
WITH active_users AS (
    SELECT * FROM users WHERE status = 'active'
)
SELECT * FROM active_users;

-- 窗口函数
SELECT name, RANK() OVER (ORDER BY score DESC) FROM leaderboard;
```

## 高级特性

### 存储过程

```sql
CREATE PROCEDURE get_user_count()
BEGIN
    SELECT COUNT(*) FROM users;
END;
```

### 触发器

```sql
CREATE TRIGGER before_insert
BEFORE INSERT ON users
FOR EACH ROW
BEGIN
    SET NEW.created_at = CURRENT_TIMESTAMP;
END;
```

## 分布式功能

### 配置复制

```toml
[replication]
type = "semi-sync"
source = "192.168.0.100:3306"
```

### XA 事务

```sql
XA START 'transaction_id';
-- 执行 SQL
XA END 'transaction_id';
XA PREPARE 'transaction_id';
XA COMMIT 'transaction_id';
```

## 性能测试

```bash
# TPC-H 基准
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6 --iterations 3

# Sysbench
cargo run --bin bench-cli -- sysbench oltp --threads 4
```

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| 连接被拒绝 | 检查端口是否开放，防火墙设置 |
| 查询超时 | 增加 `max_execution_time` 配置 |
| 存储空间不足 | 清理 `data/` 目录或扩展磁盘 |

## 相关文档

- [完整文档索引](../../README.md)
- [API 参考](./API_REFERENCE.md)
- [客户端连接](./CLIENT_CONNECTION.md)
- [迁移指南](./MIGRATION_GUIDE.md)