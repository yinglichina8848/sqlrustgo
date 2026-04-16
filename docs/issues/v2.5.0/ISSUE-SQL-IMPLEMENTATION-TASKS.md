# Issue #XXXX: SQL 语法支持不完整 - 约束与 DML 实现任务

**严重程度**: 高
**类型**: 功能实现
**模块**: parser / executor / catalog
**状态**: 待处理
**创建日期**: 2026-04-16
**Epic**: EPIC-FEATURE_COMPLETION

---

## 概述

SQLRustGo 目前存在 SQL 语法支持不完整的问题，主要集中在：

1. **外键/约束验证缺失** - 数据完整性无法保证
2. **DML 操作不完整** - INSERT SELECT 等常用语法不支持
3. **Parser 语法支持不完整** - CTE、窗口函数、触发器等

---

## 一、INSERT SELECT 实现 (P0)

**任务**: 实现 `INSERT ... SELECT` 语法

**SQL 示例**:
```sql
INSERT INTO users_backup SELECT * FROM users WHERE created_at > '2024-01-01';
```

**实现位置**: `crates/executor/src/`

**验收标准**:
- [ ] `INSERT INTO table SELECT ...` 可以正确执行
- [ ] 子查询中的聚合函数正常工作
- [ ] `INSERT ... SELECT ... WHERE` 条件正确应用
- [ ] 添加测试文件 `sql_corpus/DML/INSERT/insert_select.sql`

---

## 二、外键约束验证实现 (P0)

**任务**: 实现外键约束的运行时验证

**需要实现的功能**:

### 2.1 INSERT 时外键验证

```sql
CREATE TABLE users (id INTEGER PRIMARY KEY);
CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id));

-- 应该成功
INSERT INTO users VALUES (1);
INSERT INTO orders VALUES (1, 1);

-- 应该失败 (user_id = 999 在 users 表中不存在)
INSERT INTO orders VALUES (2, 999);
-- ERROR: Foreign key constraint violation
```

### 2.2 UPDATE 时外键验证

```sql
-- 应该失败
UPDATE orders SET user_id = 999 WHERE id = 1;
-- ERROR: Foreign key constraint violation
```

### 2.3 DELETE 级联操作

```sql
-- ON DELETE CASCADE
CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE CASCADE);
DELETE FROM users WHERE id = 1;  -- 自动删除 orders 中 user_id = 1 的记录

-- ON DELETE SET NULL
CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE SET NULL);
DELETE FROM users WHERE id = 1;  -- 自动将 orders 中 user_id 设为 NULL

-- ON DELETE RESTRICT
CREATE TABLE orders (id INTEGER, user_id INTEGER REFERENCES users(id) ON DELETE RESTRICT);
DELETE FROM users WHERE id = 1;  -- 应该失败，有子记录存在
```

**实现位置**:
- `crates/executor/src/insert.rs` - INSERT 时验证
- `crates/executor/src/update.rs` - UPDATE 时验证
- `crates/executor/src/delete.rs` - DELETE 级联处理
- `crates/storage/src/engine.rs` - 存储层外键检查 API

**验收标准**:
- [ ] INSERT 时验证外键引用存在
- [ ] UPDATE 时验证新外键值有效
- [ ] DELETE CASCADE 自动删除子记录
- [ ] DELETE SET NULL 自动设置子记录外键为 NULL
- [ ] DELETE RESTRICT 阻止有子记录的父记录删除
- [ ] ON UPDATE CASCADE/SET NULL/RESTRICT 同理
- [ ] 添加测试文件 `sql_corpus/DDL/FOREIGN_KEY/validation_insert.sql`
- [ ] 添加测试文件 `sql_corpus/DDL/FOREIGN_KEY/validation_update.sql`
- [ ] 添加测试文件 `sql_corpus/DDL/FOREIGN_KEY/validation_delete_cascade.sql`

---

## 三、主键/唯一约束验证 (P1)

**任务**: 实现主键和唯一键的运行时验证

```sql
CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT UNIQUE);

-- 应该成功
INSERT INTO users VALUES (1, 'a@test.com');

-- 应该失败 (重复主键)
INSERT INTO users VALUES (1, 'b@test.com');
-- ERROR: Primary key constraint violation

-- 应该失败 (重复唯一键)
INSERT INTO users VALUES (2, 'a@test.com');
-- ERROR: Unique constraint violation
```

**实现位置**: `crates/executor/src/insert.rs`, `crates/executor/src/update.rs`

**验收标准**:
- [ ] INSERT 时检测重复主键
- [ ] INSERT 时检测重复唯一键
- [ ] UPDATE 时检测重复主键/唯一键
- [ ] 正确区分 NULL 值 (多个 NULL 在 UNIQUE 索引中是允许的)

---

## 四、CHECK 约束验证 (P1)

**任务**: 实现 CHECK 约束的运行时验证

```sql
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  total INTEGER CHECK (total >= 0),
  status TEXT CHECK (status IN ('pending', 'completed', 'cancelled'))
);

-- 应该失败
INSERT INTO orders VALUES (1, -100, 'pending');
-- ERROR: Check constraint violation: total >= 0

INSERT INTO orders VALUES (2, 100, 'invalid_status');
-- ERROR: Check constraint violation: status IN ('pending', 'completed', 'cancelled')
```

**实现位置**: `crates/executor/src/insert.rs`, `crates/executor/src/update.rs`

**验收标准**:
- [ ] INSERT 时执行 CHECK 约束
- [ ] UPDATE 时执行 CHECK 约束
- [ ] 支持 AND/OR 组合条件
- [ ] 支持 IS NULL / IS NOT NULL 条件

---

## 五、INSERT ON DUPLICATE KEY (UPSERT) (P1)

**任务**: 实现 MySQL 风格的 UPSERT 语法

```sql
-- MySQL 语法
INSERT INTO users (id, name) VALUES (1, 'Alice')
ON DUPLICATE KEY UPDATE name = 'Alice Updated';

-- 或者
INSERT INTO users (id, name) VALUES (1, 'Alice')
ON DUPLICATE KEY UPDATE name = VALUES(name);
```

**实现位置**: `crates/executor/src/insert.rs`

**验收标准**:
- [ ] `ON DUPLICATE KEY UPDATE` 语法正确解析
- [ ] 键冲突时执行 UPDATE
- [ ] 无冲突时执行 INSERT
- [ ] `VALUES(col_name)` 函数正确返回新值

---

## 六、CTE (WITH) 子句实现 (P1)

**任务**: 实现 Common Table Expression

```sql
-- 简单 CTE
WITH active_users AS (
  SELECT id, name FROM users WHERE status = 'active'
)
SELECT * FROM active_users;

-- CTE 与聚合
WITH user_stats AS (
  SELECT user_id, COUNT(*) as order_count
  FROM orders GROUP BY user_id
)
SELECT u.name, us.order_count
FROM users u
JOIN user_stats us ON u.id = us.user_id;

-- 递归 CTE
WITH RECURSIVE cte AS (
  SELECT 1 AS n
  UNION ALL
  SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;
```

**实现位置**: `crates/parser/src/parser.rs`, `crates/planner/src/`

**验收标准**:
- [ ] 非递归 WITH 语法正确解析和执行
- [ ] WITH 子句中可以定义多个 CTE
- [ ] CTE 引用正确
- [ ] 添加测试文件 `sql_corpus/DML/SELECT/cte_basic.sql`

---

## 七、复杂子查询支持 (P1)

**任务**: 修复子查询在非 SELECT 上下文的支持

```sql
-- 标量子查询在 SELECT 中
SELECT name, (SELECT COUNT(*) FROM orders WHERE user_id = users.id) as order_count
FROM users;

-- ALL/ANY 子查询
SELECT * FROM users WHERE id = ALL (SELECT id FROM blocked_users);
SELECT * FROM users WHERE id = ANY (SELECT id FROM admins);
```

**当前错误**: `Expected SELECT in subquery` / `Expected RParen, got Identifier("IS")`

**实现位置**: `crates/parser/src/parser.rs`

**验收标准**:
- [ ] 标量子查询在 SELECT 列表中正常工作
- [ ] `ALL` 子查询正确求值
- [ ] `ANY` 子查询正确求值

---

## 八、ALTER TABLE 扩展 (P1)

**任务**: 支持更多 ALTER TABLE 语法

```sql
-- ADD PRIMARY KEY
ALTER TABLE users ADD PRIMARY KEY (id);

-- DROP PRIMARY KEY
ALTER TABLE users DROP PRIMARY KEY;

-- ADD FOREIGN KEY
ALTER TABLE orders ADD FOREIGN KEY (user_id) REFERENCES users(id);

-- ADD UNIQUE
ALTER TABLE users ADD UNIQUE (email);

-- ALTER COLUMN
ALTER TABLE users ALTER COLUMN email SET DEFAULT 'unknown';
ALTER TABLE users ALTER COLUMN email DROP DEFAULT;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;

-- CHANGE COLUMN
ALTER TABLE users CHANGE COLUMN old_name new_name VARCHAR(100);
```

**实现位置**: `crates/parser/src/parser.rs`, `crates/executor/src/`

**验收标准**:
- [ ] `ALTER TABLE ADD/DROP PRIMARY KEY` 正确执行
- [ ] `ALTER TABLE ADD/DROP FOREIGN KEY` 正确执行
- [ ] `ALTER TABLE ADD/DROP UNIQUE` 正确执行
- [ ] `ALTER TABLE ALTER COLUMN SET/DROP DEFAULT` 正确执行
- [ ] `ALTER TABLE ALTER COLUMN SET/DROP NOT NULL` 正确执行

---

## 九、UNION/INTERSECT/EXCEPT 完善 (P2)

**任务**: 完善集合操作

```sql
-- 当前可能支持
SELECT id FROM users UNION SELECT id FROM admins;

-- 需要确保完整支持
SELECT id FROM users UNION ALL SELECT id FROM admins;
SELECT id FROM users INTERSECT SELECT id FROM admins;
SELECT id FROM users EXCEPT SELECT id FROM admins;
SELECT id FROM users UNION SELECT id FROM admins ORDER BY id DESC LIMIT 10;
```

**验收标准**:
- [ ] UNION / UNION ALL 正确执行
- [ ] INTERSECT 正确执行
- [ ] EXCEPT 正确执行
- [ ] 与 ORDER BY / LIMIT 组合正确工作

---

## 十、窗口函数框架 (P2)

**任务**: 实现窗口函数框架

```sql
SELECT id, name, ROW_NUMBER() OVER (ORDER BY id) as row_num FROM users;
SELECT id, name, RANK() OVER (PARTITION BY dept ORDER BY salary DESC) FROM employees;
SELECT id, LAG(value) OVER (ORDER BY id) as prev_value FROM table;
SELECT id, SUM(amount) OVER (PARTITION BY user_id ORDER BY date ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) FROM orders;
```

**实现位置**: `crates/parser/src/`, `crates/executor/src/`, `crates/planner/src/`

**验收标准**:
- [ ] `ROW_NUMBER()` / `RANK()` / `DENSE_RANK()` 正确执行
- [ ] `LAG()` / `LEAD()` / `FIRST_VALUE()` / `LAST_VALUE()` 正确执行
- [ ] `PARTITION BY` 正确分区
- [ ] `ORDER BY` 在窗口上下文中正确排序
- [ ] 帧定义 (ROWS/RANGE BETWEEN) 正确处理

---

## 十一、触发器框架 (P2)

**任务**: 完成触发器执行框架

```sql
CREATE TRIGGER update_timestamp AFTER UPDATE ON users
BEGIN
  UPDATE users SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER before_insert BEFORE INSERT ON users
WHEN NEW.name IS NOT NULL
BEGIN
  -- validation logic
END;
```

**实现位置**: `crates/parser/src/`, `crates/executor/src/trigger.rs`

**验收标准**:
- [ ] `CREATE TRIGGER` 正确解析
- [ ] `BEFORE/AFTER INSERT/UPDATE/DELETE` 触发器正确执行
- [ ] `NEW/OLD` 引用正确
- [ ] `WHEN` 条件正确过滤
- [ ] `INSTEAD OF` 触发器支持 (视图)

---

## 十二、事务隔离级别 (P2)

**任务**: 实现 SET TRANSACTION 语法

```sql
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
```

**验收标准**:
- [ ] 解析器识别 SET TRANSACTION 语法
- [ ] 隔离级别正确影响并发行为
- [ ] 与 MVCC 正确集成

---

## 开发优先级总结

| 优先级 | 任务 | 预估工作量 |
|--------|------|------------|
| **P0** | INSERT SELECT | 2 天 |
| **P0** | 外键约束验证 | 5 天 |
| **P0** | 子查询修复 | 2 天 |
| **P1** | 主键/唯一约束 | 2 天 |
| **P1** | CHECK 约束 | 2 天 |
| **P1** | UPSERT (ON DUPLICATE KEY) | 2 天 |
| **P1** | CTE 实现 | 4 天 |
| **P1** | ALTER TABLE 扩展 | 3 天 |
| **P2** | UNION/INTERSECT/EXCEPT | 1 天 |
| **P2** | 窗口函数框架 | 5 天 |
| **P2** | 触发器框架 | 5 天 |
| **P2** | 事务隔离级别 | 2 天 |

**总预估工作量**: 约 35 天

---

## 测试文件清单

实现每个功能时，需要同步添加 SKIP 标记移除 + 新测试文件：

| 功能 | 测试文件 | 状态 |
|------|----------|------|
| INSERT SELECT | `sql_corpus/DML/INSERT/insert_select.sql` | 待创建 |
| 外键 INSERT 验证 | `sql_corpus/DDL/FOREIGN_KEY/validation_insert.sql` | 待创建 |
| 外键 DELETE CASCADE | `sql_corpus/DDL/FOREIGN_KEY/validation_delete_cascade.sql` | 待创建 |
| 外键 DELETE SET NULL | `sql_corpus/DDL/FOREIGN_KEY/validation_delete_setnull.sql` | 待创建 |
| 主键/唯一约束 | `sql_corpus/DDL/CONSTRAINT/primary_unique_constraint.sql` | 待创建 |
| CHECK 约束 | `sql_corpus/DDL/CONSTRAINT/check_constraint.sql` | 待创建 |
| UPSERT | `sql_corpus/DML/INSERT/upsert_operations.sql` | 待创建 |
| CTE | `sql_corpus/DML/SELECT/cte_basic.sql` | 待创建 |
| ALTER TABLE | `sql_corpus/DDL/ALTER_TABLE/alter_table_extensions.sql` | 待创建 |

**注意**: 实现功能后，移除对应 SKIP 标记并确保测试通过。

---

## 相关 Issue

- Issue #1379: 外键约束验证功能实现
- Epic: EPIC-FEATURE_COMPLETION
- Issue: SQL Syntax Support Matrix (ISSUE-SQL-SYNTAX-SUPPORT-MATRIX.md)

---

**Issue 创建**: 2026-04-16
**Epic**: EPIC-FEATURE_COMPLETION
**状态**: 🟡 待处理