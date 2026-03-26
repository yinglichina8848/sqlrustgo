# SQLRustGo v2.0 用户手册与 SQL 参考

> **版本**: 2.0
> **日期**: 2026-03-26
> **状态**: 规划中

---

## 1. 快速开始

### 1.1 连接数据库

```bash
# 使用 CLI
sqlrustgo-cli -h localhost -P 3306 -u root

# 执行 SQL
sqlrustgo-cli -e "CREATE TABLE users (id INT, name TEXT);"
```

### 1.2 第一个查询

```sql
-- 创建表
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 插入数据
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com');

-- 查询数据
SELECT * FROM users;
SELECT name, email FROM users WHERE id > 1;

-- 更新数据
UPDATE users SET email = 'alice_new@example.com' WHERE name = 'Alice';

-- 删除数据
DELETE FROM users WHERE id = 2;
```

---

## 2. 数据类型

### 2.1 数值类型

| 类型 | 范围 | 存储 |
|------|------|------|
| TINYINT | -128 ~ 127 | 1 字节 |
| SMALLINT | -32768 ~ 32767 | 2 字节 |
| INTEGER / INT | -2^31 ~ 2^31-1 | 4 字节 |
| BIGINT | -2^63 ~ 2^63-1 | 8 字节 |
| FLOAT | ±3.4E38 | 4 字节 |
| DOUBLE | ±1.7E308 | 8 字节 |
| DECIMAL(p,s) | 精确数值 | 可变 |

### 2.2 字符串类型

| 类型 | 最大长度 | 说明 |
|------|----------|------|
| CHAR(n) | 255 | 固定长度 |
| VARCHAR(n) | 65535 | 可变长度 |
| TEXT | 64KB | 大文本 |
| MEDIUMTEXT | 16MB | 中等文本 |
| LONGTEXT | 4GB | 长文本 |

### 2.3 日期时间类型

| 类型 | 范围 | 格式 |
|------|------|------|
| DATE | '1000-01-01' ~ '9999-12-31' | YYYY-MM-DD |
| TIME | '-838:59:59' ~ '838:59:59' | HH:MM:SS |
| DATETIME | '1000-01-01 00:00:00' ~ '9999-12-31 23:59:59' | YYYY-MM-DD HH:MM:SS |
| TIMESTAMP | '1970-01-01 00:00:01' ~ '2038-01-19 03:14:07' | Unix 时间戳 |

### 2.4 其他类型

| 类型 | 说明 |
|------|------|
| BOOLEAN | 布尔值 (TRUE/FALSE) |
| BLOB | 二进制数据 |
| JSON | JSON 文本 |

---

## 3. SQL 语法

### 3.1 DDL - 数据定义

#### 创建表

```sql
CREATE TABLE table_name (
    column_name data_type [constraints],
    ...
    [, PRIMARY KEY (column)]
    [, FOREIGN KEY (column) REFERENCES table(column)]
    [, UNIQUE (column)]
    [, CHECK (condition)]
);

-- 示例
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    product_name VARCHAR(100),
    quantity INTEGER DEFAULT 1,
    price DECIMAL(10,2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

#### 修改表

```sql
-- 添加列
ALTER TABLE table_name ADD column_name data_type;

-- 删除列
ALTER TABLE table_name DROP COLUMN column_name;

-- 修改列
ALTER TABLE table_name MODIFY COLUMN column_name new_data_type;

-- 添加约束
ALTER TABLE table_name ADD CONSTRAINT constraint_name PRIMARY KEY (column);
```

#### 删除表

```sql
DROP TABLE table_name;
DROP TABLE IF EXISTS table_name;
```

#### 索引

```sql
-- 创建索引
CREATE INDEX idx_name ON table_name (column);
CREATE UNIQUE INDEX idx_unique ON table_name (column);

-- 删除索引
DROP INDEX idx_name ON table_name;
```

### 3.2 DML - 数据操作

#### 插入数据

```sql
-- 插入单行
INSERT INTO table_name (col1, col2) VALUES (val1, val2);

-- 插入多行
INSERT INTO table_name (col1, col2) VALUES 
    (val1, val2), 
    (val3, val4);

-- 插入查询结果
INSERT INTO table_name (col1, col2) SELECT col1, col2 FROM other_table;

-- UPSERT
INSERT INTO table_name (id, col1, col2) VALUES (1, val1, val2)
ON DUPLICATE KEY UPDATE col1 = val1, col2 = val2;
```

#### 更新数据

```sql
UPDATE table_name 
SET col1 = value1, col2 = value2
WHERE condition;

-- 示例
UPDATE users SET email = 'new@example.com' WHERE id = 1;
```

#### 删除数据

```sql
DELETE FROM table_name WHERE condition;

-- 示例
DELETE FROM users WHERE created_at < '2025-01-01';
```

### 3.3 DQL - 数据查询

#### 基本查询

```sql
SELECT column1, column2 FROM table_name;
SELECT * FROM table_name;
SELECT DISTINCT column FROM table_name;
```

#### 条件查询

```sql
SELECT * FROM table_name WHERE condition;

-- 条件运算符
WHERE id = 1;
WHERE age > 18;
WHERE name LIKE 'A%';
WHERE id IN (1, 2, 3);
WHERE age BETWEEN 18 AND 30;
WHERE col IS NULL;
WHERE col IS NOT NULL;
```

#### 排序与分页

```sql
-- 排序
SELECT * FROM table_name ORDER BY column [ASC|DESC];
SELECT * FROM table_name ORDER BY col1 ASC, col2 DESC;

-- 分页
SELECT * FROM table_name LIMIT 10;
SELECT * FROM table_name OFFSET 10 LIMIT 10;
SELECT * FROM table_name LIMIT 10 OFFSET 20;
```

#### 聚合查询

```sql
SELECT COUNT(*) FROM table_name;
SELECT SUM(column) FROM table_name;
SELECT AVG(column) FROM table_name;
SELECT MIN(column) FROM table_name;
SELECT MAX(column) FROM table_name;

-- 分组
SELECT column, COUNT(*) FROM table_name GROUP BY column;
SELECT department, AVG(salary) FROM employees GROUP BY department;

-- 分组过滤
SELECT department, AVG(salary) as avg_sal 
FROM employees 
GROUP BY department 
HAVING avg_sal > 5000;
```

#### 连接查询

```sql
-- 内连接
SELECT * FROM a INNER JOIN b ON a.id = b.a_id;
SELECT * FROM a JOIN b ON a.id = b.a_id;

-- 左外连接
SELECT * FROM a LEFT JOIN b ON a.id = b.a_id;

-- 右外连接
SELECT * FROM a RIGHT JOIN b ON a.id = b.a_id;

-- 全外连接
SELECT * FROM a FULL OUTER JOIN b ON a.id = b.a_id;

-- 交叉连接
SELECT * FROM a CROSS JOIN b;
```

#### 子查询

```sql
-- WHERE 子查询
SELECT * FROM table WHERE column IN (SELECT column FROM other_table);

-- FROM 子查询
SELECT * FROM (SELECT * FROM table1) AS subquery;

-- EXISTS
SELECT * FROM table WHERE EXISTS (SELECT 1 FROM other_table WHERE ...);
```

#### 集合操作

```sql
-- 并集
SELECT * FROM table1 UNION SELECT * FROM table2;
SELECT * FROM table1 UNION ALL SELECT * FROM table2;

-- 交集
SELECT * FROM table1 INTERSECT SELECT * FROM table2;

-- 差集
SELECT * FROM table1 EXCEPT SELECT * FROM table2;
```

#### 窗口函数

```sql
-- 行号
SELECT *, ROW_NUMBER() OVER (ORDER BY column) FROM table;

-- 排名
SELECT *, RANK() OVER (PARTITION BY col1 ORDER BY col2) FROM table;
SELECT *, DENSE_RANK() OVER (PARTITION BY col1 ORDER BY col2) FROM table;

-- 累计聚合
SELECT *, SUM(amount) OVER (ORDER BY date) FROM sales;
SELECT *, AVG(salary) OVER (PARTITION BY department ORDER BY hire_date) FROM employees;

-- 首尾值
SELECT *, FIRST_VALUE(col) OVER (ORDER BY date) FROM table;
SELECT *, LAST_VALUE(col) OVER (ORDER BY date) FROM table;
```

---

## 4. 事务控制

### 4.1 事务语句

```sql
-- 开始事务
BEGIN;
START TRANSACTION;

-- 提交事务
COMMIT;

-- 回滚事务
ROLLBACK;

-- 回滚到保存点
ROLLBACK TO SAVEPOINT savepoint_name;

-- 创建保存点
SAVEPOINT savepoint_name;
```

### 4.2 隔离级别

```sql
-- 设置隔离级别
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;

-- 会话级
SET SESSION transaction_isolation = 'READ-COMMITTED';
```

---

## 5. 系统变量

### 5.1 会话变量

```sql
-- 查看变量
SHOW VARIABLES LIKE 'max_connections';
SELECT @@max_connections;

-- 设置变量
SET max_connections = 200;
SET SESSION sql_mode = 'STRICT_TRANS_TABLES';
```

### 5.2 常用变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| max_connections | 100 | 最大连接数 |
| buffer_pool_size | 256MB | Buffer Pool 大小 |
| query_cache_size | 64MB | 查询缓存大小 |
| log_error | - | 错误日志路径 |
| sql_mode | - | SQL 模式 |

---

## 6. 常用函数

### 6.1 字符串函数

| 函数 | 说明 | 示例 |
|------|------|------|
| CONCAT(s1, s2, ...) | 拼接字符串 | CONCAT('Hello', ' ', 'World') |
| SUBSTRING(s, start, len) | 截取字符串 | SUBSTRING('Hello', 2, 3) |
| TRIM(s) | 去除首尾空格 | TRIM(' hello ') |
| UPPER(s) | 转换为大写 | UPPER('hello') |
| LOWER(s) | 转换为小写 | LOWER('HELLO') |
| LENGTH(s) | 返回长度 | LENGTH('hello') |
| REPLACE(s, old, new) | 替换字符串 | REPLACE('a*b', '*', '-') |

### 6.2 数值函数

| 函数 | 说明 | 示例 |
|------|------|------|
| ABS(n) | 绝对值 | ABS(-5) |
| CEIL(n) | 向上取整 | CEIL(3.2) |
| FLOOR(n) | 向下取整 | FLOOR(3.8) |
| ROUND(n, d) | 四舍五入 | ROUND(3.14159, 2) |
| MOD(m, n) | 取模 | MOD(10, 3) |
| POW(n, e) | 幂运算 | POW(2, 3) |

### 6.3 日期函数

| 函数 | 说明 | 示例 |
|------|------|------|
| NOW() | 当前时间 | NOW() |
| CURDATE() | 当前日期 | CURDATE() |
| CURTIME() | 当前时间 | CURTIME() |
| DATE_ADD(d, interval) | 日期加减 | DATE_ADD(NOW(), INTERVAL 1 DAY) |
| DATEDIFF(d1, d2) | 日期差 | DATEDIFF('2026-01-01', '2025-01-01') |
| YEAR(d) | 提取年份 | YEAR('2026-01-01') |
| MONTH(d) | 提取月份 | MONTH('2026-01-01') |
| DAY(d) | 提取日期 | DAY('2026-01-01') |

### 6.4 条件函数

| 函数 | 说明 | 示例 |
|------|------|------|
| IF(cond, t, f) | 条件判断 | IF(age > 18, 'adult', 'minor') |
| IFNULL(v, alt) | NULL 替换 | IFNULL(NULL, 'default') |
| COALESCE(...) | 返回第一个非 NULL | COALESCE(a, b, c) |
| CASE | 条件分支 | CASE WHEN... THEN... END |

---

## 7. EXPLAIN 分析

### 7.1 执行计划

```sql
-- 查看查询计划
EXPLAIN SELECT * FROM users WHERE id = 1;

-- 详细分析
EXPLAIN ANALYZE SELECT * FROM users WHERE id = 1;
```

### 7.2 输出说明

| 字段 | 说明 |
|------|------|
| id | 查询编号 |
| select_type | 查询类型 |
| table | 涉及的表 |
| type | 访问类型 |
| possible_keys | 可用索引 |
| key | 实际使用索引 |
| rows | 预估扫描行数 |
| Extra | 额外信息 |

---

## 8. 限制与差异

### 8.1 与 MySQL 的差异

| 功能 | MySQL | SQLRustGo | 说明 |
|------|-------|-----------|------|
| 存储引擎 | InnoDB, MyISAM | Memory, File | 有限 |
| 字符集 | 多种 | UTF-8 | 主要支持 |
| 窗口函数 | 支持 | 部分支持 | v2.0 完善中 |
| 存储过程 | 支持 | 有限 | v2.0 完善中 |

### 8.2 已知限制

- 最大表数量：受内存限制
- 最大行数：受存储引擎限制
- 复合索引：最大 16 列
- 字符串长度：最大 64KB (TEXT 类型)

---

**文档版本历史**

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-03-26 | 初始版本 |

**状态**: ✅ 规划完成
