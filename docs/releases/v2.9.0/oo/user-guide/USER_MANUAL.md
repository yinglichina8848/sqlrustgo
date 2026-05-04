# SQLRustGo v2.9.0 用户手册

> **版本**: v2.9.0 (RC)
> **更新日期**: 2026-05-05

---

## 1. 快速开始

### 1.1 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release

# 全特性构建
cargo build --all-features
```

### 1.2 运行

```bash
# 启动 REPL
cargo run --release

# 运行测试
cargo test --workspace
```

---

## 2. SQL 语法

### 2.1 基础操作

```sql
-- 创建表
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);

-- 插入数据
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- 查询
SELECT * FROM users;

-- 更新
UPDATE users SET name = 'Bob' WHERE id = 1;

-- 删除
DELETE FROM users WHERE id = 1;
```

### 2.2 聚合查询

```sql
-- 聚合函数
SELECT COUNT(*) FROM orders;
SELECT SUM(amount) FROM orders;
SELECT AVG(price) FROM products;
SELECT MIN(created_at) FROM events;
SELECT MAX(score) FROM games;

-- 分组
SELECT department, AVG(salary) FROM employees GROUP BY department;

-- 分组过滤
SELECT department, AVG(salary) as avg_sal
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

### 2.3 JOIN

```sql
-- 内连接
SELECT u.name, o.order_id
FROM users u
INNER JOIN orders o ON u.id = o.user_id;

-- 左连接
SELECT u.name, o.order_id
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- 右连接
SELECT u.name, o.order_id
FROM users u
RIGHT JOIN orders o ON u.id = o.user_id;
```

---

## 3. 配置

### 3.1 存储配置

```sql
-- 设置存储引擎
SET storage.engine = 'memory';  -- 或 'disk'
```

### 3.2 事务配置

```sql
-- 设置隔离级别
SET transaction.isolation = 'read_committed';  -- 或 'serializable'
```

---

## 4. 索引

### 4.1 创建索引

```sql
-- 创建 B+ 树索引
CREATE INDEX idx_users_email ON users(email);

-- 创建向量索引
CREATE VECTOR INDEX idx_embeddings ON embeddings USING hnsw(dimension=128);
```

### 4.2 索引类型

| 类型 | 说明 |
|------|------|
| BTree | B+ 树索引，适用于范围查询 |
| Hash | 哈希索引，适用于等值查询 |
| Vector | 向量索引，适用于相似性搜索 |

---

## 5. 事务

### 5.1 事务控制

```sql
-- 开始事务
BEGIN;

-- 提交事务
COMMIT;

-- 回滚事务
ROLLBACK;
```

### 5.2 保存点

```sql
-- 创建保存点
SAVEPOINT sp1;

-- 回滚到保存点
ROLLBACK TO SAVEPOINT sp1;
```

---

## 6. 错误处理

### 6.1 错误代码

| 错误代码 | 说明 |
|----------|------|
| E0001 | 语法错误 |
| E0002 | 表不存在 |
| E0003 | 列不存在 |
| E0004 | 约束违反 |
| E0005 | 事务冲突 |

---

## 7. 性能优化

### 7.1 查询优化

- 使用 EXPLAIN 分析查询计划
- 创建适当的索引
- 避免 SELECT *
- 使用 LIMIT 限制结果集

### 7.2 配置优化

```sql
-- 设置查询超时（毫秒）
SET query.timeout = 30000;

-- 设置最大连接数
SET connection.max = 100;
```

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
