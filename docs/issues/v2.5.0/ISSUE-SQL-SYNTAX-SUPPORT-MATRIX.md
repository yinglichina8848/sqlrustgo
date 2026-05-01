# Issue #XXXX: SQLRustGo 不支持的 SQL 语法清单

**严重程度**: 高
**类型**: 功能缺失
**模块**: parser / executor / catalog
**状态**: 待处理
**创建日期**: 2026-04-16

---

## 概述

本文档列出 SQLRustGo 目前不支持的 SQL 语法，用于指导开发工作和确保测试充分性。这些功能分为以下几类：

1. **Parser 不支持** - SQL 语法解析层面不支持
2. **Executor 未实现** - 语法可以解析但执行逻辑缺失
3. **约束验证缺失** - 外键/主键/唯一约束验证未实现
4. **DML 限制** - INSERT/UPDATE/DELETE 操作不完整

---

## 一、Parser 不支持的 SQL 语法

### 1.1 CTE (Common Table Expression)

**状态**: ❌ 未实现
**文件**: `sql_corpus/DML/SELECT/cte_operations.sql`, `cte_advanced.sql`

```sql
-- 不支持
WITH active_users AS (
  SELECT id, name FROM users WHERE id <= 5
)
SELECT * FROM active_users;

WITH RECURSIVE cte AS (
  SELECT 1 AS n
  UNION ALL
  SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;
```

**Parser 错误**: `Expected SELECT after WITH clause`

---

### 1.2 窗口函数 (Window Functions)

**状态**: ❌ 未实现
**文件**: `sql_corpus/EXPRESSIONS/window_functions_advanced.sql`

```sql
-- 不支持
SELECT id, name, ROW_NUMBER() OVER (ORDER BY id) as row_num FROM users;
SELECT id, name, RANK() OVER (PARTITION BY dept ORDER BY salary) FROM employees;
SELECT id, LAG(value) OVER (ORDER BY id) FROM table;
SELECT id, FIRST_VALUE(col) OVER (ORDER BY id ROWS UNBOUNDED PRECEDING) FROM table;
```

**Parser 错误**: `Expected FROM or column name` 或 `Expected expression`

---

### 1.3 触发器 (CREATE TRIGGER)

**状态**: ❌ 仅框架，无执行
**文件**: `sql_corpus/DDL/TRIGGER/trigger_operations.sql`

```sql
-- Parser 支持解析，但 Executor 未实现
CREATE TRIGGER update_timestamp AFTER UPDATE ON users
BEGIN
  UPDATE users SET email = 'triggered_' || email WHERE id = NEW.id;
END;

CREATE TRIGGER before_insert BEFORE INSERT ON users
WHEN NEW.name IS NOT NULL
BEGIN
  -- trigger body
END;

-- INSTEAD OF 触发器 (用于视图)
CREATE TRIGGER instead_of_update INSTEAD OF UPDATE ON user_view
BEGIN
  UPDATE base_table SET ... WHERE id = OLD.id;
END;
```

**问题**: Parser 有 `parse_create_trigger` 入口，但执行框架未实现

---

### 1.4 存储过程 (Stored Procedures)

**状态**: ⚠️ 部分支持
**文件**: `crates/executor/src/stored_proc.rs`

```sql
-- 支持解析但未连接执行路径
CREATE PROCEDURE get_users()
BEGIN
  SELECT * FROM users;
END;

CALL get_users();
```

**问题**: 解析器和执行框架存在，但未连接调用链

---

### 1.5 ALTER TABLE 扩展语法

**状态**: ❌ 部分不支持
**文件**: `sql_corpus/DDL/ALTER_TABLE/column_operations.sql`, `alter_table.sql`

```sql
-- 不支持
ALTER TABLE users ADD COLUMN age INTEGER DEFAULT 0;  -- Parser 支持
ALTER TABLE users ADD PRIMARY KEY (id);  -- 不支持 "ADD PRIMARY KEY"
ALTER TABLE users DROP PRIMARY KEY;  -- 不支持
ALTER TABLE users ADD UNIQUE (email);  -- 不支持
ALTER TABLE users ADD FOREIGN KEY (parent_id) REFERENCES users(id);  -- 不支持 "ADD FOREIGN KEY"
ALTER TABLE users ALTER COLUMN age SET DEFAULT 10;  -- 不支持 "ALTER COLUMN ... SET"
ALTER TABLE users ALTER COLUMN age DROP DEFAULT;  -- 不支持
ALTER TABLE users ALTER COLUMN age SET NOT NULL;  -- 不支持
ALTER TABLE users ALTER COLUMN age DROP NOT NULL;  -- 不支持
ALTER TABLE users CHANGE COLUMN old_name new_name INTEGER;  -- 不支持
```

**Parser 错误**: `Expected ADD, DROP or MODIFY` / `Expected Column, got Primary`

---

### 1.6 DROP TABLE IF EXISTS / IF EXISTS 语法

**状态**: ⚠️ 部分不支持
**文件**: `sql_corpus/DDL/DROP_TABLE/drop_table.sql`

```sql
-- 不支持
DROP TABLE IF EXISTS users;
DROP INDEX IF EXISTS idx_name;
DROP TRIGGER IF EXISTS trigger_name;
```

**Parser 错误**: `Expected table name`

---

### 1.7 INSERT ... ON DUPLICATE KEY (MySQL)

**状态**: ❌ 未实现
**文件**: `sql_corpus/DML/INSERT/upsert_operations.sql`

```sql
-- 不支持 (MySQL 语法)
INSERT INTO users (id, name) VALUES (1, 'Alice')
ON DUPLICATE KEY UPDATE name = 'Alice Updated';

-- 不支持 (PostgreSQL 语法)
INSERT INTO users (id, name) VALUES (1, 'Alice')
ON CONFLICT (id) DO UPDATE SET name = 'Alice Updated';
```

---

### 1.8 INSERT ... SELECT

**状态**: ❌ 未实现
**文件**: `sql_corpus/DML/INSERT/insert_tests.sql`

```sql
-- 不支持
INSERT INTO users_backup SELECT * FROM users WHERE created_at > '2024-01-01';
```

**Parser 错误**: `Expected VALUES`

---

### 1.9 子查询在非 SELECT 上下文

**状态**: ⚠️ 部分不支持
**文件**: `sql_corpus/DML/SELECT/subquery_corner_cases.sql`

```sql
-- 不支持
SELECT COALESCE((SELECT MAX(id) FROM users), 0);  -- 子查询作为标量表达式

SELECT * FROM users WHERE id = ALL (SELECT id FROM admins);

SELECT * FROM users WHERE id = ANY (SELECT id FROM moderators);
```

**Parser 错误**: `Expected SELECT in subquery` / `Expected RParen, got Identifier("IS")`

---

### 1.10 CASE / WHEN 表达式

**状态**: ⚠️ 部分支持
**文件**: `sql_corpus/EXPRESSIONS/case_expressions.sql`

```sql
-- 支持简单 CASE
SELECT CASE status WHEN 1 THEN 'active' WHEN 2 THEN 'inactive' ELSE 'unknown' END FROM users;

-- 不支持搜索 CASE
SELECT CASE WHEN id > 10 THEN 'high' WHEN id > 5 THEN 'medium' ELSE 'low' END FROM users;
SELECT CASE WHEN x IS NULL THEN 'N/A' ELSE x END FROM table;  -- NULL 条件
```

---

### 1.11 FULL OUTER JOIN

**状态**: ⚠️ Parser 支持但执行不支持
**文件**: `sql_corpus/DML/SELECT/outer_join.sql`

```sql
-- Parser 可以解析，但执行返回错误
SELECT u.name, o.order_id
FROM users u
FULL OUTER JOIN orders o ON u.id = o.user_id;
```

---

### 1.12 LIMIT / OFFSET 在复杂上下文

**状态**: ⚠️ 部分不支持
**文件**: `sql_corpus/DML/SELECT/limit_offset.sql`

```sql
-- 支持
SELECT * FROM users LIMIT 10;

-- 不支持
SELECT DISTINCT id FROM users LIMIT 10;  -- DISTINCT + LIMIT
SELECT * FROM users ORDER BY id LIMIT 10 OFFSET 5;  -- 复杂 OFFSET
```

---

### 1.13 PRAGMA 语句

**状态**: ❌ 未实现
**文件**: `sql_corpus/SPECIAL/full_text_search.sql`

```sql
-- 不支持
PRAGMA table_info('users');
PRAGMA index_list('users');
PRAGMA foreign_key_list('orders');
PRAGMA cache_size = 1000;
```

**Parser 错误**: `Expected TABLE, VIEW, TRIGGER, or PROCEDURE after CREATE`

---

### 1.14 TEMPORARY TABLE

**状态**: ❌ Parser 不识别
**文件**: `sql_corpus/DDL/TEMPORARY/temp_table_operations.sql`

```sql
-- 不支持
CREATE TEMPORARY TABLE temp_users (id INTEGER PRIMARY KEY, name TEXT);
CREATE TEMP TABLE temp_data AS SELECT * FROM users;
```

---

## 二、Executor 未实现的 SQL 功能

### 2.1 外键约束验证

**状态**: ❌ 仅数据结构，未实现验证
**问题**: `table_foreign_keys: None` 占位符遍布代码库

```rust
// crates/executor/src/harness.rs
table_foreign_keys: None,  // 只是占位，未实现
```

**需要实现**:
- [ ] INSERT 时验证外键引用
- [ ] UPDATE 时验证外键引用
- [ ] DELETE CASCADE 级联删除
- [ ] DELETE SET NULL 设置为 NULL
- [ ] DELETE RESTRICT 阻止删除
- [ ] UPDATE CASCADE 级联更新
- [ ] UPDATE SET NULL / RESTRICT

---

### 2.2 主键/唯一约束验证

**状态**: ❌ 未实现
**文件**: `sql_corpus/DML/DELETE/delete_tests.sql`

```sql
-- 需要验证
INSERT INTO users (id, email) VALUES (1, 'a@test.com');
INSERT INTO users (id, email) VALUES (1, 'b@test.com');  -- 应失败: 重复主键

INSERT INTO users (id, email) VALUES (2, 'a@test.com');
INSERT INTO users (id, email) VALUES (3, 'a@test.com');  -- 应失败: 重复唯一键 email
```

---

### 2.3 CHECK 约束

**状态**: ❌ 未实现
**文件**: `sql_corpus/DDL/CONSTRAINT/constraint_operations.sql`

```sql
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  total INTEGER CHECK (total >= 0),
  status TEXT CHECK (status IN ('pending', 'completed', 'cancelled'))
);
```

---

### 2.4 唯一约束 (UNIQUE)

**状态**: ❌ 未实现
**文件**: `sql_corpus/DDL/CONSTRAINT/constraint_operations.sql`

```sql
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE
);
```

---

### 2.5 约束命名 (CONSTRAINT name)

**状态**: ⚠️ Parser 支持但执行忽略
**文件**: `sql_corpus/DDL/CONSTRAINT/constraint_operations.sql`

```sql
-- Parser 解析但不验证
CREATE TABLE orders (
  id INTEGER,
  CONSTRAINT pk_orders PRIMARY KEY (id),
  CONSTRAINT fk_orders_users FOREIGN KEY (user_id) REFERENCES users(id),
  CONSTRAINT uq_email UNIQUE (email),
  CONSTRAINT chk_total CHECK (total >= 0)
);
```

---

## 三、DML 操作限制

### 3.1 DELETE 限制

```sql
-- 支持
DELETE FROM users WHERE id = 1;

-- 不支持
DELETE FROM users WHERE id IN (SELECT id FROM inactive_users);
DELETE FROM users WHERE id = ALL (SELECT id FROM blocked_users);
```

---

### 3.2 UPDATE 限制

```sql
-- 支持
UPDATE users SET email = 'new@test.com' WHERE id = 1;

-- 不支持
UPDATE users SET email = (SELECT email FROM backup WHERE backup.id = users.id);
UPDATE users SET id = id + 1 WHERE status = 'active';
```

---

## 四、其他缺失功能

### 4.1 索引提示 (Index Hints)

**状态**: ⚠️ Parser 解析但执行忽略
**文件**: `sql_corpus/DDL/INDEX/index_operations.sql`

```sql
-- Parser 支持但执行忽略
SELECT * FROM users USE INDEX (PRIMARY) WHERE id = 1;
SELECT * FROM users IGNORE INDEX (idx_name) WHERE id = 1;
SELECT * FROM users FORCE INDEX (idx_email) WHERE email LIKE '%test%';
```

---

### 4.2 EXPLAIN 输出格式

**状态**: ⚠️ 基本支持
**文件**: `sql_corpus/DEBUG/explain_queries.sql`

```sql
-- 支持
EXPLAIN SELECT * FROM users WHERE id = 1;

-- 不支持
EXPLAIN FORMAT=JSON SELECT * FROM users;
EXPLAIN ANALYZE SELECT * FROM users;  -- PostgreSQL 语法
```

---

### 4.3 事务隔离级别

**状态**: ⚠️ 部分支持
**文件**: `sql_corpus/TRANSACTION/transaction_isolation.sql`

```sql
-- 不支持
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
```

**Parser 错误**: `Unexpected token: Set`

---

### 4.4 SAVEPOINT / ROLLBACK TO

**状态**: ⚠️ Parser 支持但执行不完整
**文件**: `sql_corpus/TRANSACTION/transaction_advanced.sql`

```sql
-- 部分支持
BEGIN;
SAVEPOINT sp1;
INSERT INTO users VALUES (1, 'Alice');
ROLLBACK TO SAVEPOINT sp1;
-- ROLLBACK TO 后数据应该保留，但当前可能未实现
```

---

### 4.5 序列 (SEQUENCE)

**状态**: ❌ 未实现

```sql
-- 不支持
CREATE SEQUENCE user_id_seq START WITH 1 INCREMENT BY 1;
SELECT NEXTVAL('user_id_seq');
SELECT CURRVAL('user_id_seq');
```

---

### 4.6 AUTOINCREMENT / SERIAL

**状态**: ⚠️ 仅部分支持

```sql
-- 支持
CREATE TABLE users (id INTEGER PRIMARY KEY AUTO_INCREMENT);

-- 不支持
CREATE TABLE orders (id SERIAL PRIMARY KEY);  -- PostgreSQL 语法
```

---

### 4.7 物化视图 (Materialized View)

**状态**: ❌ 未实现

```sql
-- 不支持
CREATE MATERIALIZED VIEW active_users AS
SELECT * FROM users WHERE status = 'active';

REFRESH MATERIALIZED VIEW active_users;
```

---

## 五、测试覆盖要求

### 5.1 必须添加的测试文件

| 功能 | 测试文件 | 优先级 |
|------|----------|--------|
| CTE | `sql_corpus/DML/SELECT/cte_basic.sql` | P0 |
| 窗口函数 | `sql_corpus/DML/SELECT/window_basic.sql` | P0 |
| 外键验证 | `sql_corpus/DDL/FOREIGN_KEY/validation_tests.sql` | P0 |
| 触发器 | `sql_corpus/DDL/TRIGGER/basic_trigger.sql` | P1 |
| CHECK 约束 | `sql_corpus/DDL/CONSTRAINT/check_constraint.sql` | P1 |
| INSERT SELECT | `sql_corpus/DML/INSERT/insert_select.sql` | P1 |

### 5.2 当前跳过文件清单 (72个)

```
DDL/ALTER_TABLE/alter_table.sql (SKIP)
DDL/ALTER_TABLE/column_operations.sql (SKIP)
DDL/CONSTRAINT/constraint_operations.sql (SKIP)
DDL/CREATE_TABLE/create_table.sql (SKIP - DROP IF EXISTS)
DDL/DROP_TABLE/drop_table.sql (SKIP - IF EXISTS)
DDL/INDEX/index_operations.sql (SKIP - INDEX HINTS)
DDL/METADATA/schema_metadata.sql (SKIP - PRAGMA)
DDL/MISC/misc_operations.sql (SKIP)
DDL/TEMPORARY/temp_table_operations.sql (SKIP - TEMP TABLE)
DDL/TRIGGER/trigger_operations.sql (SKIP - TRIGGER 未实现)
DDL/VIEW/view_operations.sql (SKIP)
DEBUG/explain_queries.sql (SKIP - EXPLAIN ANALYZE)
DEBUG/query_optimization.sql (SKIP)
DML/DELETE/delete_complex.sql (SKIP - 子查询)
DML/DELETE/delete_variations.sql (SKIP)
DML/INSERT/batch_operations.sql (SKIP)
DML/INSERT/insert_tests.sql (SKIP - INSERT SELECT)
DML/INSERT/insert_variations.sql (SKIP)
DML/INSERT/upsert_operations.sql (SKIP - ON DUPLICATE KEY)
DML/SELECT/advanced_patterns.sql (SKIP)
DML/SELECT/aggregate_functions.sql (SKIP)
DML/SELECT/aggregate_patterns.sql (SKIP)
DML/SELECT/alias_operations.sql (SKIP)
DML/SELECT/comparison_operators.sql (SKIP)
DML/SELECT/complex_combinations.sql (SKIP)
DML/SELECT/complex_where.sql (SKIP)
DML/SELECT/correlated_subquery.sql (SKIP)
DML/SELECT/cte_advanced.sql (SKIP - CTE)
DML/SELECT/cte_operations.sql (SKIP - CTE)
DML/SELECT/distinct_all.sql (SKIP)
DML/SELECT/group_by_having.sql (SKIP)
DML/SELECT/inline_views.sql (SKIP)
DML/SELECT/join_combinations.sql (SKIP)
DML/SELECT/join_corner_cases.sql (SKIP)
DML/SELECT/order_by.sql (SKIP - DISTINCT + ORDER BY)
DML/SELECT/outer_join.sql (SKIP - FULL OUTER JOIN)
DML/SELECT/pattern_matching.sql (SKIP)
DML/SELECT/range_operations.sql (SKIP)
DML/SELECT/self_join.sql (SKIP)
DML/SELECT/set_operations_advanced.sql (SKIP)
DML/SELECT/subquery_corner_cases.sql (SKIP - 子查询)
DML/SELECT/subqueries.sql (SKIP - 子查询)
DML/SELECT/union_operations.sql (SKIP)
DML/SELECT/where_conditions.sql (SKIP)
DML/UPDATE/update_complex.sql (SKIP)
DML/UPDATE/update_variations.sql (SKIP)
EXPRESSIONS/case_expressions.sql (SKIP)
EXPRESSIONS/conditional_expressions.sql (SKIP)
EXPRESSIONS/data_types.sql (SKIP)
EXPRESSIONS/datetime_functions.sql (SKIP)
EXPRESSIONS/datetime_operations.sql (SKIP)
EXPRESSIONS/expression_evaluation.sql (SKIP)
EXPRESSIONS/expression_tests.sql (SKIP)
EXPRESSIONS/json_functions.sql (SKIP)
EXPRESSIONS/logical_operators.sql (SKIP)
EXPRESSIONS/math_functions.sql (SKIP)
EXPRESSIONS/numeric_operations.sql (SKIP)
EXPRESSIONS/special_functions.sql (SKIP)
EXPRESSIONS/string_operations.sql (SKIP)
EXPRESSIONS/type_conversion.sql (SKIP)
EXPRESSIONS/window_functions.sql (SKIP)
EXPRESSIONS/window_functions_advanced.sql (SKIP)
SPECIAL/full_text_search.sql (SKIP - FTS/PRAGMA)
SPECIAL/null_semantics_advanced.sql (SKIP)
TRANSACTION/transaction_advanced.sql (SKIP - SET TRANSACTION)
TRANSACTION/transaction_isolation.sql (SKIP - 隔离级别)
```

---

## 六、优先级排序

### P0 - 必须实现 (影响核心功能)

| 功能 | 原因 | 预估工作量 |
|------|------|------------|
| INSERT SELECT | 常用 SQL 语法 | 2 天 |
| CTE (WITH) | 复杂查询必需 | 3 天 |
| 外键验证 | 数据完整性核心 | 5 天 |
| 子查询在 WHERE | 常用模式 | 2 天 |
| CHECK 约束 | 数据验证 | 1 天 |

### P1 - 重要 (影响用户体验)

| 功能 | 原因 | 预估工作量 |
|------|------|------------|
| 窗口函数 | 分析查询必需 | 4 天 |
| 触发器 | 业务逻辑常用 | 5 天 |
| ALTER TABLE ADD FK | 常用 DDL | 2 天 |
| UNIQUE 约束验证 | 数据唯一性 | 1 天 |
| ON CONFLICT (UPSERT) | 常用模式 | 2 天 |

### P2 - 增强 (可选)

| 功能 | 原因 | 预估工作量 |
|------|------|------------|
| FULL OUTER JOIN | 现有 JOIN 扩展 | 2 天 |
| 事务隔离级别 | 标准 SQL | 2 天 |
| TEMPORARY TABLE | 临时数据 | 2 天 |
| 存储过程 | 复杂业务逻辑 | 5 天 |
| 序列 (SEQUENCE) | AUTO_INCREMENT 替代 | 2 天 |

---

## 七、相关 Issue

- Issue #1379: 外键约束验证功能实现
- Epic: EPIC-FEATURE_COMPLETION

---

**Issue 创建**: 2026-04-16
**最后更新**: 2026-04-16
**状态**: 🟡 进行中