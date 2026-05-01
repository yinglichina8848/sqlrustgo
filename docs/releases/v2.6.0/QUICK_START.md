# 快速开始

> **版本**: alpha/v2.6.0

---

## 1. 安装

### 1.1 前置要求

- Rust 1.85+
- Cargo (随 Rust 安装)

### 1.2 安装步骤

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建
cargo build --release
```

---

## 2. 快速使用

### 2.1 启动 REPL

```bash
cargo run --release
```

### 2.2 运行测试

```bash
# 所有测试
cargo test --workspace

# SQL Corpus 测试
cargo test -p sqlrustgo-sql-corpus
```

---

## 3. 基础 SQL 操作

### 3.1 创建表

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);
```

### 3.2 插入数据

```sql
INSERT INTO users (id, name, email) VALUES 
(1, 'Alice', 'alice@example.com'),
(2, 'Bob', 'bob@example.com');
```

### 3.3 查询

```sql
SELECT * FROM users;
SELECT name, email FROM users WHERE id = 1;
```

### 3.4 聚合查询

```sql
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users GROUP BY department;
```

### 3.5 JOIN

```sql
SELECT u.name, o.amount
FROM users u
INNER JOIN orders o ON u.id = o.user_id;
```

---

## 4. 文档

| 文档 | 说明 |
|------|------|
| [README.md](./README.md) | 文档索引 |
| [USER_MANUAL.md](./oo/user-guide/USER_MANUAL.md) | 用户手册 |
| [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md) | 升级指南 |

---

## 5. 性能基准

### 5.1 SQL Corpus

```
=== Summary ===
Total: 59 cases, 59 passed, 0 failed
Pass rate: 100.0%
```

### 5.2 目标性能

| 场景 | 目标 |
|------|------|
| 点查 | 75,000 TPS |
| TPC-H Q1 | < 200ms |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
