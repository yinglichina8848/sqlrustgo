# SQLRustGo v2.9.0 用户手册

> **版本**: v2.9.0
> **发布日期**: 2026-05-05
> **阶段**: RC

---

## 目录

1. [快速开始](#快速开始)
2. [SQL 操作](#sql-操作)
3. [事务管理](#事务管理)
4. [性能调优](#性能调优)
5. [安全功能](#安全功能)
6. [运维指南](#运维指南)

---

## 快速开始

### 启动服务

```bash
cargo run --release --bin sqlrustgo
```

服务默认监听 `localhost:5432`。

### 连接数据库

```bash
mysql -h localhost -P 5432 -u root
```

### 执行第一个查询

```sql
SELECT 'Hello, SQLRustGo!' AS greeting;
```

---

## SQL 操作

### 创建数据库和表

```sql
CREATE DATABASE myapp;
USE myapp;

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 插入数据

```sql
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users (id, name, email) VALUES (2, 'Bob', 'bob@example.com');
```

### 查询数据

```sql
-- 简单查询
SELECT * FROM users WHERE id = 1;

-- 聚合查询
SELECT COUNT(*) AS user_count FROM users;

-- 分组查询
SELECT name, COUNT(*) AS cnt FROM users GROUP BY name;
```

### 更新和删除

```sql
UPDATE users SET email = 'new@example.com' WHERE id = 1;
DELETE FROM users WHERE id = 2;
```

---

## 事务管理

### 开启事务

```sql
BEGIN;

INSERT INTO users (id, name, email) VALUES (3, 'Charlie', 'charlie@example.com');
UPDATE users SET email = 'updated@example.com' WHERE id = 1;

COMMIT;
```

### 回滚

```sql
BEGIN;
INSERT INTO users (id, name, email) VALUES (4, 'Dave', 'dave@example.com');
ROLLBACK;
```

### 隔离级别

```sql
-- 设置隔离级别
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
BEGIN;
-- 执行操作
COMMIT;
```

---

## CTE 和窗口函数

### WITH (CTE)

```sql
WITH active_users AS (
    SELECT * FROM users WHERE id > 0
)
SELECT * FROM active_users WHERE name LIKE 'A%';
```

### 窗口函数

```sql
SELECT
    name,
    email,
    ROW_NUMBER() OVER (ORDER BY id) AS row_num,
    RANK() OVER (ORDER BY id) AS rank
FROM users;
```

---

## JSON 操作

```sql
-- 创建带 JSON 的表
CREATE TABLE articles (
    id INTEGER PRIMARY KEY,
    title TEXT,
    metadata JSON
);

-- 插入 JSON
INSERT INTO articles (id, title, metadata) VALUES (
    1,
    'Getting Started',
    '{"author": "Alice", "tags": ["rust", "database"]}'
);

-- 提取 JSON 字段
SELECT
    title,
    json_extract(metadata, '$.author') AS author,
    json_extract(metadata, '$.tags[0]') AS first_tag
FROM articles;
```

---

## 性能调优

### 创建索引

```sql
-- 创建索引
CREATE INDEX idx_users_email ON users(email);

-- 创建唯一索引
CREATE UNIQUE INDEX idx_users_email_unique ON users(email);
```

### 查询分析

```sql
EXPLAIN SELECT * FROM users WHERE email = 'alice@example.com';
```

### 配置缓冲池

```toml
# sqlrustgo.toml
[performance]
buffer_pool_size_mb = 2048
max_connections = 256
```

---

## 安全功能

### 权限管理

```sql
-- 创建用户
CREATE USER 'app'@'%' IDENTIFIED BY 'password';

-- 授予权限
GRANT SELECT, INSERT, UPDATE ON myapp.* TO 'app'@'%';

-- 撤销权限
REVOKE DELETE ON myapp.* FROM 'app'@'%';
```

### 审计日志

审计日志默认开启，记录所有操作到 `logs/audit/`。

```sql
-- 查看审计日志（管理员）
-- 日志文件位于 logs/audit/audit.log
```

---

## 运维指南

### 备份

```bash
# 创建备份
cargo run --bin sqlrustgo -- backup create --path /backup/myapp.db

# 恢复备份
cargo run --bin sqlrustgo -- backup restore --path /backup/myapp.db
```

### 健康检查

```bash
curl http://localhost:5432/health
```

### 监控指标

```bash
curl http://localhost:5432/metrics
```

返回包括：
- 活跃连接数
- 查询 QPS
- 缓存命中率
- 事务数

---

## 相关文档

- [QUICK_START.md](./QUICK_START.md)
- [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)
- [API_REFERENCE.md](./API_REFERENCE.md)
- [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md)

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
