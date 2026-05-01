# SQLRustGo v2.6.0 用户手册

> **版本**: alpha/v2.6.0
> **更新日期**: 2026-04-18

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
-- INNER JOIN
SELECT u.name, o.amount
FROM users u
INNER JOIN orders o ON u.id = o.user_id;

-- LEFT JOIN
SELECT u.name, o.amount
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- RIGHT JOIN
SELECT u.name, o.amount
FROM users u
RIGHT JOIN orders o ON u.id = o.user_id;

-- CROSS JOIN
SELECT * FROM users CROSS JOIN roles;
```

### 2.4 外键约束

```sql
CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    total DECIMAL(10,2)
);

-- 自引用外键
CREATE TABLE employees (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    manager_id INTEGER REFERENCES employees(id)
);
```

---

## 3. API 使用

### 3.1 ExecutionEngine

```rust
use sqlrustgo::ExecutionEngine;

// 创建执行引擎
let engine = ExecutionEngine::new(storage);

// 执行查询
let result = engine.execute("SELECT * FROM users", vec![])?;
```

---

## 4. 配置

### 4.1 存储引擎

```rust
// 使用内存存储
let storage = MemoryStorage::new();

// 使用文件存储
let storage = FileStorage::new("data/");

// 使用列式存储
let storage = ColumnarStorage::new();
```

---

## 5. 限制

### 5.1 已知限制

1. **FULL OUTER JOIN**: 尚未支持
2. **MVCC SSI**: 仅支持快照隔离 (SI)
3. **存储过程**: 部分支持

---

## 6. 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [文档首页](../../README.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
