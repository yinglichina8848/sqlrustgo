# SQLRustGo v3.0.0 快速开始

> **版本**: v3.0.0
> **发布日期**: 2026-05-07
> **阶段**: GA (General Availability)
> **预计完成时间**: 5 分钟

---

## 元数据

| 字段 | 值 |
|------|-----|
| 工作目录 | /Users/liying/workspace/dev/yinglichina163/sqlrustgo |
| GitHub 身份 | openclaw |
| AI 工具 | OpenCode (Sisyphus Agent) |
| 当前版本 | v3.0.0 (GA) |
| 工作分支 | develop/v3.0.0 |
| 时间段 | 2026-05-10 |

---

## 一、5 分钟入门

### 1.1 构建项目

```bash
# 克隆并构建
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo
git checkout develop/v3.0.0

# 构建 (约 5-10 分钟)
cargo build --all-features --release
```

### 1.2 启动 REPL

```bash
# 启动交互式 SQL REPL
cargo run --bin sqlrustgo

# 输出:
# SQLRustGo v3.0.0
# Type "help" for more information.
#
# sqlrustgo> _
```

### 1.3 运行第一条 SQL

```sql
-- 创建数据库
CREATE DATABASE test;

-- 使用数据库
USE test;

-- 创建表
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(100)
);

-- 插入数据
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com');

-- 查询数据
SELECT * FROM users;

-- 输出:
# +------+-------+------------------+
# | id   | name  | email            |
# +------+-------+------------------+
# | 1    | Alice | alice@example.com|
# | 2    | Bob   | bob@example.com  |
# +------+-------+------------------+
# 2 rows in set (0.001s)
```

---

## 二、基本操作

### 2.1 CRUD 操作

```sql
-- 更新数据
UPDATE users SET email = 'alice_new@example.com' WHERE id = 1;

-- 删除数据
DELETE FROM users WHERE id = 2;

-- 验证
SELECT * FROM users;
```

### 2.2 聚合查询

```sql
-- 创建订单表
CREATE TABLE orders (
    id INT PRIMARY KEY,
    user_id INT,
    amount DECIMAL(10,2),
    status VARCHAR(20)
);

-- 插入测试数据
INSERT INTO orders VALUES (1, 1, 99.99, 'completed');
INSERT INTO orders VALUES (2, 1, 149.99, 'completed');
INSERT INTO orders VALUES (3, 2, 29.99, 'pending');

-- 聚合查询
SELECT user_id, COUNT(*) as order_count, SUM(amount) as total
FROM orders
GROUP BY user_id
HAVING SUM(amount) > 50;

-- 输出:
# +---------+--------------+---------+
# | user_id | order_count | total   |
# +---------+--------------+---------+
# | 1       | 2            | 249.98  |
# +---------+--------------+---------+
```

### 2.3 JOIN 查询

```sql
-- 联表查询
SELECT u.name, o.amount, o.status
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE o.status = 'completed';
```

---

## 三、窗口函数

```sql
-- 使用窗口函数
SELECT
    name,
    amount,
    ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY amount DESC) as row_num,
    RANK() OVER (ORDER BY amount DESC) as rank,
    DENSE_RANK() OVER (ORDER BY amount DESC) as dense_rank
FROM users u
JOIN orders o ON u.id = o.user_id;
```

---

## 四、CTE (WITH 子句)

```sql
-- 使用 CTE
WITH active_users AS (
    SELECT * FROM users WHERE id IN (SELECT DISTINCT user_id FROM orders)
)
SELECT name, COUNT(*) as order_count
FROM active_users
JOIN orders ON active_users.id = orders.user_id
GROUP BY name;

-- 递归 CTE
WITH RECURSIVE cnt(x) AS (
    SELECT 1
    UNION ALL
    SELECT x + 1 FROM cnt WHERE x < 10
)
SELECT x FROM cnt;
```

---

## 五、事务操作

```sql
-- 开始事务
BEGIN;

-- 执行操作
INSERT INTO users VALUES (3, 'Charlie', 'charlie@example.com');
UPDATE users SET email = 'alice@test.com' WHERE id = 1;

-- 提交
COMMIT;

-- 或者回滚
-- ROLLBACK;
```

---

## 六、性能测试

### 6.1 运行简单基准测试

```bash
# 运行 QPS 基准测试
cargo bench --package sqlrustgo-bench -- simple_select

# 运行 TPC-H 查询
bash scripts/gate/check_tpch.sh sf=0.1
```

### 6.2 性能目标

| 操作 | 目标 QPS | v3.0.0 实际 |
|------|----------|--------------|
| 简单 SELECT | 10,000+ | ~400,000 |
| UPDATE | 10,000+ | ~43,000 |
| DELETE | 10,000+ | ~65,000 |

---

## 七、示例 SQL

### 7.1 完整示例

```sql
-- 创建电商数据库
CREATE DATABASE ecommerce;
USE ecommerce;

-- 创建表
CREATE TABLE customers (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    price DECIMAL(10,2)
);

CREATE TABLE orders (
    id INT PRIMARY KEY,
    customer_id INT,
    product_id INT,
    quantity INT,
    total DECIMAL(10,2)
);

-- 插入测试数据
INSERT INTO customers VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie');
INSERT INTO products VALUES (1, 'Widget', 9.99), (2, 'Gadget', 19.99);
INSERT INTO orders VALUES (1, 1, 1, 2, 19.98), (2, 1, 2, 1, 19.99);

-- 分析查询
SELECT
    c.name,
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id
GROUP BY c.id, c.name
ORDER BY total_spent DESC;
```

---

## 八、下一步

### 8.1 学习更多

- [完整文档](./README.md) - 详细的架构和功能说明
- [迁移指南](./MIGRATION_GUIDE.md) - 从 v2.9.0 升级
- [功能矩阵](./FEATURE_MATRIX.md) - 所有支持的功能

### 8.2 高级主题

- [部署指南](./DEPLOYMENT_GUIDE.md) - 生产环境部署
- [性能调优](./PERFORMANCE_TARGETS.md) - 性能优化建议
- [形式化验证](../proof/README.md) - TLA+ 证明

### 8.3 故障排除

| 问题 | 解决方案 |
|------|----------|
| 构建失败 | 运行 `cargo clean && cargo build` |
| 测试失败 | 检查 LLVM 版本 ≥ 15 |
| 性能不佳 | 确认使用 `--release` 构建 |

---

## 九、获取帮助

```bash
# 查看帮助
cargo run --bin sqlrustgo -- --help

# 在 REPL 中
sqlrustgo> help

# 查看状态
sqlrustgo> status
```

---

*快速开始指南 v3.0.0*
*创建者: openclaw*
*审核者: OpenClaw*
*修改者: openclaw*
*修改记录:*
* - 2026-05-10: 补充元数据和尾部信息*
*最后更新: 2026-05-10*
