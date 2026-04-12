# MySQL → SQLRustGo 对照表

## 概述

本文档提供 MySQL 与 SQLRustGo 之间的语法对照，帮助教师和学生快速上手 SQLRustGo。

---

## 数据类型对照

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `INT` | `Integer` | 整数类型 |
| `BIGINT` | `Integer` (64bit) | 长整数 |
| `VARCHAR(n)` | `Text` | 变长字符串 |
| `CHAR(n)` | `Text` | 定长字符串 |
| `TEXT` | `Text` | 长文本 |
| `DECIMAL(p,s)` | `Float` | 精确小数 |
| `FLOAT` | `Float` | 浮点数 |
| `DOUBLE` | `Float` | 双精度浮点 |
| `BOOLEAN` | `Boolean` | 布尔值 |
| `DATE` | `Date` | 日期 |
| `TIME` | `Time` | 时间 |
| `DATETIME` | `Timestamp` | 日期时间 |
| `TIMESTAMP` | `Timestamp` | 时间戳 |
| `JSON` | `Text` | JSON 字符串 |
| `BLOB` | `Blob` | 二进制数据 |

---

## 关键字对照

### DDL (数据定义语言)

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `CREATE TABLE` | `CREATE TABLE` | ✅ 相同 |
| `DROP TABLE` | `DROP TABLE` | ✅ 相同 |
| `ALTER TABLE` | 部分支持 | 正在完善 |
| `CREATE INDEX` | `CREATE INDEX` | ✅ 相同 |
| `DROP INDEX` | `DROP INDEX` | ✅ 相同 |

### DML (数据操作语言)

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `SELECT` | `SELECT` | ✅ 相同 |
| `INSERT INTO` | `INSERT INTO` | ✅ 相同 |
| `UPDATE ... SET` | `UPDATE ... SET` | ✅ 相同 |
| `DELETE FROM` | `DELETE FROM` | ✅ 相同 |

### 条件与过滤

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `WHERE` | `WHERE` | ✅ 相同 |
| `AND` | `AND` | ✅ 相同 |
| `OR` | `OR` | ✅ 相同 |
| `NOT` | `NOT` | ✅ 相同 |
| `IN (...)` | `IN (...)` | ✅ 相同 |
| `BETWEEN a AND b` | `BETWEEN a AND b` | ✅ 相同 |
| `LIKE '%pattern%'` | `LIKE '%pattern%'` | ✅ 相同 |
| `IS NULL` | `IS NULL` | ✅ 相同 |
| `IS NOT NULL` | `IS NOT NULL` | ✅ 相同 |

### 排序与分页

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `ORDER BY col ASC` | `ORDER BY col ASC` | ✅ 相同 |
| `ORDER BY col DESC` | `ORDER BY col DESC` | ✅ 相同 |
| `LIMIT n` | `LIMIT n` | ✅ 相同 |
| `LIMIT offset, n` | `LIMIT offset, n` | ✅ 相同 |

### 聚合与分组

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `GROUP BY` | `GROUP BY` | ✅ 相同 |
| `HAVING` | `HAVING` | ✅ 相同 |
| `COUNT(*)` | `COUNT(*)` | ✅ 相同 |
| `SUM(col)` | `SUM(col)` | ✅ 相同 |
| `AVG(col)` | `AVG(col)` | ✅ 相同 |
| `MAX(col)` | `MAX(col)` | ✅ 相同 |
| `MIN(col)` | `MIN(col)` | ✅ 相同 |

### 表连接

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `INNER JOIN` | `INNER JOIN` | ✅ 相同 |
| `LEFT JOIN` | `LEFT JOIN` | ✅ 相同 |
| `RIGHT JOIN` | `RIGHT JOIN` | ✅ 相同 |
| `FULL OUTER JOIN` | 不支持 | 使用 UNION |
| `CROSS JOIN` | `CROSS JOIN` | ✅ 相同 |

---

## 函数对照

### 字符串函数

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `CONCAT(a, b)` | `CONCAT(a, b)` | ✅ 相同 |
| `SUBSTRING(s, start, len)` | `SUBSTRING(s, start, len)` | ✅ 相同 |
| `LENGTH(s)` | `LENGTH(s)` | ✅ 相同 |
| `UPPER(s)` | `UPPER(s)` | ✅ 相同 |
| `LOWER(s)` | `LOWER(s)` | ✅ 相同 |
| `TRIM(s)` | `TRIM(s)` | ✅ 相同 |
| `REPLACE(s, old, new)` | `REPLACE(s, old, new)` | ✅ 相同 |

### 日期函数

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `NOW()` | `NOW()` | ✅ 相同 |
| `CURRENT_DATE` | `CURRENT_DATE` | ✅ 相同 |
| `CURRENT_TIME` | `CURRENT_TIME` | ✅ 相同 |
| `DATE_ADD(date, interval)` | 不支持 | 使用 + |
| `DATEDIFF(d1, d2)` | 不支持 | 正在完善 |
| `YEAR(date)` | `YEAR(date)` | ✅ 相同 |
| `MONTH(date)` | `MONTH(date)` | ✅ 相同 |
| `DAY(date)` | `DAY(date)` | ✅ 相同 |

### 数学函数

| MySQL | SQLRustGo | 说明 |
|-------|-----------|------|
| `ABS(n)` | `ABS(n)` | ✅ 相同 |
| `CEIL(n)` | `CEIL(n)` | ✅ 相同 |
| `FLOOR(n)` | `FLOOR(n)` | ✅ 相同 |
| `ROUND(n, d)` | `ROUND(n, d)` | ✅ 相同 |
| `MOD(a, b)` | `MOD(a, b)` | ✅ 相同 |
| `POWER(n, e)` | `POWER(n, e)` | ✅ 相同 |
| `SQRT(n)` | `SQRT(n)` | ✅ 相同 |

---

## MySQL 特有语法（需注意）

### SQLRustGo 不支持的语法

| MySQL 语法 | 说明 | 替代方案 |
|-----------|------|----------|
| `AUTO_INCREMENT` | 自增 | 使用应用层生成 ID |
| `ON DUPLICATE KEY UPDATE` | 冲突更新 | 使用 `REPLACE` 或应用逻辑 |
| `REPLACE INTO` | 替换插入 | 使用 `DELETE` + `INSERT` |
| `INSERT ... ON DUPLICATE` | 条件插入 | 使用 `IF EXISTS` 检查 |
| `DELAYED INSERT` | 延迟插入 | 不支持 |
| `STRAIGHT_JOIN` | 强制顺序 JOIN | 使用 `JOIN` |
| `FOR UPDATE` | 行锁 | 正在完善 |
| `LOCK TABLES` | 表锁 | 不支持 |
| `UNLOCK TABLES` | 解锁表 | 不支持 |
| `SET TRANSACTION` | 事务隔离级别 | 正在完善 |

### 限制支持

| 功能 | 支持程度 |
|------|----------|
| 子查询 | 部分支持 |
| 视图 | 不支持 |
| 存储过程 | 不支持 |
| 触发器 | 不支持 |
| 事务 | 部分支持 |
| 外键约束 | 部分支持 |
| 全文索引 | 不支持 |

---

## 配置差异

### 环境变量

| MySQL 配置 | SQLRustGo 环境变量 | 说明 |
|-----------|-------------------|------|
| 性能模式 | `SQLRUSTGO_BENCHMARK_MODE=1` | 禁用缓存和统计 |
| 教学演示 | `SQLRUSTGO_TEACHING_MODE=1` | 强制 EXPLAIN 输出 |

### SQLRUSTGO_TEACHING_MODE

设置 `SQLRUSTGO_TEACHING_MODE=1` 启用教学模式：

- 禁用查询优化器（展示原始执行计划）
- 强制所有 SELECT 返回 EXPLAIN 输出
- 适合数据库原理教学

```bash
# 启用教学模式
export SQLRUSTGO_TEACHING_MODE=1

# 禁用教学模式
export SQLRUSTGO_TEACHING_MODE=0
```

---

## 使用示例

### 基本查询

```sql
-- MySQL
SELECT id, name, price FROM products WHERE price > 100;

-- SQLRustGo (完全相同)
SELECT id, name, price FROM products WHERE price > 100;
```

### JOIN 查询

```sql
-- MySQL
SELECT o.id, u.name, o.total
FROM orders o
INNER JOIN users u ON o.user_id = u.id;

-- SQLRustGo (完全相同)
SELECT o.id, u.name, o.total
FROM orders o
INNER JOIN users u ON o.user_id = u.id;
```

### 聚合查询

```sql
-- MySQL
SELECT category, COUNT(*) as cnt, AVG(price) as avg_price
FROM products
GROUP BY category
HAVING cnt > 5;

-- SQLRustGo (完全相同)
SELECT category, COUNT(*) as cnt, AVG(price) as avg_price
FROM products
GROUP BY category
HAVING cnt > 5;
```

---

## 教学建议

### 适合 SQLRustGo 的实验

1. ✅ 基础 SELECT 查询
2. ✅ WHERE 条件过滤
3. ✅ ORDER BY 排序
4. ✅ 聚合函数与 GROUP BY
5. ✅ JOIN 多表关联
6. ✅ 子查询
7. ✅ INSERT / UPDATE / DELETE
8. ✅ CREATE TABLE 建表
9. ✅ 事务基础（部分）

### 建议使用 MySQL 的实验

1. ❌ 存储过程
2. ❌ 触发器
3. ❌ 复杂事务控制
4. ❌ 外键约束
5. ❌ 视图
6. ❌ 全文搜索

---

## 错误排查

### 常见错误

| 错误信息 | 原因 | 解决方案 |
|---------|------|----------|
| `Parse error` | 语法错误 | 检查 SQL 语法 |
| `Table not found` | 表不存在 | 检查表名 |
| `Column not found` | 列不存在 | 检查列名 |
| `Type mismatch` | 类型不匹配 | 检查数据类型 |

---

*更新时间: 2024-03*
*SQLRustGo 版本: 1.7.0*
