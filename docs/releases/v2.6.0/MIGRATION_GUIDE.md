# 迁移指南

> **版本**: v2.6.0
> **适用版本**: 从 v2.5.0 升级

---

## 1. 概述

本文档提供从 v2.5.0 迁移到 v2.6.0 的完整指南。

---

## 2. 重大变更

### 2.1 API 变更

#### ExecutionEngine API (新增)

v2.6.0 引入了高级 SQL 执行 API:

```rust
// v2.6.0 新增
use sqlrustgo::ExecutionEngine;

// 创建执行引擎
let engine = ExecutionEngine::new(storage);

// 执行 SQL
let result = engine.execute("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]);
```

#### 存储过程表达式

存储过程现在支持更多表达式类型:

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

## 3. SQL 语法变更

### 3.1 新增功能

#### HAVING 子句

```sql
-- v2.5.0: 不支持 HAVING
-- v2.6.0: 支持
SELECT department, AVG(salary) as avg_salary
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

#### DELETE 语句

```sql
-- v2.5.0: 部分支持
-- v2.6.0: 完整支持

-- 基本删除
DELETE FROM users WHERE id = 1;

-- 带子查询
DELETE FROM orders
WHERE user_id IN (
    SELECT id FROM users WHERE status = 'inactive'
);
```

#### 外键约束

```sql
-- v2.6.0 完整支持外键
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

## 4. 数据迁移

### 4.1 存储格式

v2.6.0 与 v2.5.0 数据格式兼容，无需特殊迁移步骤。

### 4.2 索引重建

升级后建议重建索引:

```sql
REINDEX DATABASE;
```

---

## 5. 配置变更

### 5.1 构建要求

v2.6.0 需要以下依赖:

- Rust 1.85+
- Cargo (随 Rust 安装)

### 5.2 特性标志

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 全特性构建
cargo build --all-features
```

---

## 6. 回滚

如果升级后遇到问题，可以回滚到 v2.5.0:

```bash
# 切换到 v2.5.0 标签
git checkout v2.5.0

# 重新构建
cargo build --release
```

---

## 7. 常见问题

### Q1: 编译失败怎么办?

确保 Rust 版本 >= 1.85:

```bash
rustc --version
```

### Q2: 存储过程聚合表达式返回 NULL?

这是预期行为，存储过程中的聚合表达式暂时返回 NULL。

### Q3: FULL OUTER JOIN 不支持?

是的，v2.6.0 暂不支持 FULL OUTER JOIN，将在后续版本中支持。

---

## 8. 获取帮助

- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [文档首页](./README.md)
- [版本计划](./VERSION_PLAN.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
