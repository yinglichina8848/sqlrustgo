# v2.6.0 升级指南

> **版本**: v2.6.0
> **适用**: 从 v2.5.0 升级

---

## 概述

本指南帮助用户从 v2.5.0 升级到 v2.6.0。v2.6.0 主要包含语法增强和 bug 修复，向后兼容。

---

## 重大变更

### API 变更

#### 新增 ExecutionEngine API

v2.6.0 引入了高级 SQL 执行 API:

```rust
use sqlrustgo::ExecutionEngine;

// 创建执行引擎
let engine = ExecutionEngine::new(storage);

// 执行 SQL
let result = engine.execute("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]);
```

#### 存储过程表达式增强

存储过程现在支持更多表达式类型，包括 Aggregate 表达式:

```sql
-- 存储过程中使用聚合表达式
CREATE PROCEDURE get_user_count()
BEGIN
    DECLARE count_val INTEGER;
    SELECT COUNT(*) INTO count_val FROM users;
    SELECT count_val AS total;
END;
```

---

## SQL 语法变更

### 新增功能

#### HAVING 子句

现在可以在聚合查询中使用 HAVING 子句:

```sql
SELECT department, AVG(salary) as avg_salary
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

#### DELETE 语句

完整的 DELETE 支持:

```sql
-- 基本删除
DELETE FROM users WHERE id = 1;

-- 带子查询
DELETE FROM orders
WHERE user_id IN (
    SELECT id FROM users WHERE status = 'inactive'
);
```

#### 外键约束

外键约束现在完整支持:

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

## 配置变更

### 构建要求

v2.6.0 需要以下依赖:

- Rust 1.85+
- Cargo (随 Rust 安装)

### 特性标志

推荐构建方式:

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 全特性构建 (包含所有功能)
cargo build --all-features
```

---

## 数据迁移

### 存储格式

v2.6.0 与 v2.5.0 数据格式兼容，无需特殊迁移步骤。

### 索引

如果使用索引，升级后建议重建索引:

```sql
-- 重建所有索引
REINDEX DATABASE;
```

---

## 已知问题

### 待解决

1. **FULL OUTER JOIN**: 完整外部连接尚未支持
2. **MVCC SSI**: 可串行化快照隔离尚未支持
3. **覆盖率**: 目标 70%，当前待测

### 限制

- 存储过程中的聚合表达式返回 NULL
- 触发器集成尚未完成

---

## 回滚

如果升级后遇到问题，可以回滚到 v2.5.0:

```bash
# 切换到 v2.5.0 标签
git checkout v2.5.0

# 重新构建
cargo build --release
```

---

## 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [文档首页](./README.md)
- [版本计划](./VERSION_PLAN.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
