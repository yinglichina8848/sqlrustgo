# SQLRustGo v2.9.0 用户手册

> **版本**: v2.9.0
> **代号**: Enterprise Resilience
> **状态**: RC (v2.9.0-rc.1)
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
cargo test --all-features
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

### 2.2 DDL 增强 (v2.9.0 新增)

```sql
-- IF NOT EXISTS 支持
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100)
);

-- IF EXISTS 支持
DROP TABLE IF EXISTS users;

-- ALTER TABLE DROP/MODIFY COLUMN
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users MODIFY COLUMN name VARCHAR(200);

-- CREATE/DROP VIEW
CREATE VIEW active_users AS
    SELECT * FROM users WHERE status = 'active';

DROP VIEW active_users;

-- CREATE UNIQUE INDEX
CREATE UNIQUE INDEX idx_email ON users(email);

-- DROP INDEX IF EXISTS
DROP INDEX IF EXISTS idx_email;

-- SHOW DATABASES
SHOW DATABASES;

-- SHOW CREATE TABLE
SHOW CREATE TABLE users;
```

### 2.3 INSERT 增强 (v2.9.0 新增)

```sql
-- INSERT ON DUPLICATE KEY UPDATE
INSERT INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com')
ON DUPLICATE KEY UPDATE name = 'Alice Updated';

-- TRUNCATE TABLE
TRUNCATE TABLE users;

-- REPLACE INTO
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com');
```

### 2.4 聚合查询

```sql
-- 聚合函数
SELECT COUNT(*) FROM orders;
SELECT SUM(amount) FROM orders;
SELECT AVG(price) FROM products;

-- COUNT(DISTINCT) (v2.9.0 新增, PR #256)
SELECT COUNT(DISTINCT status) FROM orders;
SELECT COUNT(DISTINCT customer_id) FROM orders WHERE amount > 100;

-- 分组
SELECT department, AVG(salary) FROM employees GROUP BY department;

-- 分组过滤
SELECT department, AVG(salary) as avg_sal
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

### 2.5 CASE/WHEN (v2.9.0 新增)

```sql
-- 简单 CASE 表达式
SELECT
    name,
    CASE status
        WHEN 'active' THEN '启用'
        WHEN 'inactive' THEN '禁用'
        ELSE '未知'
    END as status_text
FROM users;

-- 搜索 CASE 表达式
SELECT
    name,
    CASE
        WHEN salary < 5000 THEN '低收入'
        WHEN salary >= 5000 AND salary < 10000 THEN '中等收入'
        WHEN salary >= 10000 THEN '高收入'
        ELSE '未知'
    END as income_level
FROM employees;
```

### 2.6 JOIN

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

-- FULL OUTER JOIN
SELECT u.name, o.order_id
FROM users u
FULL OUTER JOIN orders o ON u.id = o.user_id;
```

### 2.7 窗口函数 (v2.9.0 增强)

```sql
-- ROW_NUMBER with PARTITION BY (v2.9.0 新增)
SELECT
    department,
    name,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;

-- RANK
SELECT name, score, RANK() OVER (ORDER BY score DESC) as rank
FROM leaderboard;

-- DENSE_RANK
SELECT name, score, DENSE_RANK() OVER (ORDER BY score DESC) as dense_rank
FROM leaderboard;
```

### 2.8 CTE/WITH (v2.9.0 新增)

```sql
-- 简单 CTE
WITH regional_sales AS (
    SELECT region, SUM(amount) as total
    FROM orders
    GROUP BY region
)
SELECT * FROM regional_sales WHERE total > 10000;

-- 递归 CTE (v2.9.0 新增)
WITH RECURSIVE cte AS (
    SELECT 1 AS n
    UNION ALL
    SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;

-- 递归 CTE 用于组织结构
WITH RECURSIVE org_chain AS (
    SELECT id, name, manager_id, 1 as level
    FROM employees WHERE manager_id IS NULL
    UNION ALL
    SELECT e.id, e.name, e.manager_id, oc.level + 1
    FROM employees e
    JOIN org_chain oc ON e.manager_id = oc.id
)
SELECT * FROM org_chain;
```

### 2.9 JSON 操作 (v2.9.0 新增)

```sql
-- JSON 提取
SELECT JSON_EXTRACT(data, '$.name') FROM configs;

-- JSON 路径 (简写)
SELECT data->>'$.name' FROM configs;

-- JSON 设置
SELECT JSON_SET(data, '$.name', 'new_name') FROM configs;
```

---

## 3. 分区表

### 3.1 Range 分区

```sql
CREATE TABLE sales (
    id INTEGER,
    sale_date DATE,
    amount DECIMAL(10,2)
)
PARTITION BY RANGE (YEAR(sale_date)) (
    PARTITION p2024 VALUES LESS THAN (2025),
    PARTITION p2025 VALUES LESS THAN (2026),
    PARTITION p2026 VALUES LESS THAN MAXVALUE
);
```

### 3.2 List 分区

```sql
CREATE TABLE orders (
    id INTEGER,
    region VARCHAR(50),
    amount DECIMAL(10,2)
)
PARTITION BY LIST (region) (
    PARTITION p_north VALUES IN ('Beijing', 'Shanghai'),
    PARTITION p_south VALUES IN ('Guangzhou', 'Shenzhen'),
    PARTITION p_other VALUES IN (DEFAULT)
);
```

### 3.3 Hash 分区

```sql
CREATE TABLE transactions (
    id INTEGER,
    user_id INTEGER,
    amount DECIMAL(10,2)
)
PARTITION BY HASH (user_id)
PARTITIONS 8;
```

---

## 4. 分布式功能 (v2.9.0)

### 4.1 分布式架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                    SQLRustGo v2.9.0                             │
│                   Enterprise Resilience                         │
├─────────────────────────────────────────────────────────────────┤
│  D-01 Semi-sync    │  D-02 MTS          │  D-03 Multi-source    │
│  半同步复制        │  多线程并行复制    │  多主源复制            │
├─────────────────────────────────────────────────────────────────┤
│                      D-04 XA 事务                              │
│                  两阶段提交分布式事务                            │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Semi-sync 复制

半同步复制确保数据在主从之间同步后才返回成功。

```sql
-- 查看 Semi-sync 状态
SHOW VARIABLES LIKE 'semi_sync%';

-- 启用 Semi-sync
SET GLOBAL semi_sync_enabled = ON;
```

**配置：**
```toml
[replication]
type = "semi-sync"
source = "192.168.0.100:3306"
ack_timeout = 10  # 秒
```

### 4.3 MTS (Multi-Threaded Slave) 并行复制

```sql
-- 查看 MTS 状态
SHOW SLAVE STATUS;

-- 配置 MTS workers
SET GLOBAL mts_workers = 4;
```

**启动 MTS 从节点：**
```bash
sqlrustgo-server --role replica --port 3307 \
  --source 127.0.0.1:3306 \
  --mts-workers 4
```

### 4.4 Multi-source 复制

连接多个主源进行复制：

```sql
-- 添加复制通道
CHANGE MASTER TO
    MASTER_HOST = '192.168.0.101',
    MASTER_PORT = 3306,
    MASTER_USER = 'repl'
    FOR CHANNEL 'source_1';

CHANGE MASTER TO
    MASTER_HOST = '192.168.0.102',
    MASTER_PORT = 3306,
    MASTER_USER = 'repl'
    FOR CHANNEL 'source_2';

-- 启动所有通道
START SLAVE;
```

**启动多源复制从节点：**
```bash
sqlrustgo-server --role replica --port 3308 \
  --source-list 192.168.0.101:3306,192.168.0.102:3306
```

### 4.5 XA 事务

两阶段提交分布式事务支持：

```sql
-- 启动 XA 事务
XA START 'transaction_id';

-- 执行 SQL
INSERT INTO orders (id, amount) VALUES (1, 100);

-- 结束 XA 事务
XA END 'transaction_id';

-- 准备提交
XA PREPARE 'transaction_id';

-- 提交 XA 事务
XA COMMIT 'transaction_id';

-- 回滚 XA 事务
XA ROLLBACK 'transaction_id';

-- 查看 XA 事务状态
XA RECOVER;
```

**Java/JDBC XA 示例：**

```java
import javax.sql.XAConnection;
import javax.transaction.xa.*;

public class SqlrustgoXA {
    public static void main(String[] args) throws Exception {
        javax.sql.DataSource ds = new com.mysql.cj.jdbc.MysqlXADataSource();
        ((com.mysql.cj.jdbc.MysqlXADataSource) ds).setUrl("jdbc:mysql://127.0.0.1:3306/default");

        XAConnection xaConn = ds.getXAConnection();
        XAResource xaRes = xaConn.getXAResource();

        Xid xid = new MyXid(1, new byte[]{0x01}, new byte[]{0x02});
        xaRes.start(xid, XAResource.TMNOFLAGS);

        // 执行 SQL
        // ...

        xaRes.end(xid, XAResource.TMSUCCESS);
        xaRes.prepare(xid);
        xaRes.commit(xid, false);

        xaConn.close();
    }
}
```

---

## 5. 存储过程与触发器

### 5.1 存储过程

```sql
CREATE PROCEDURE get_user_count()
BEGIN
    SELECT COUNT(*) FROM users;
END;

-- 调用存储过程
CALL get_user_count();
```

### 5.2 触发器

```sql
CREATE TRIGGER before_insert
BEFORE INSERT ON users
FOR EACH ROW
BEGIN
    SET NEW.created_at = CURRENT_TIMESTAMP;
END;
```

---

## 6. 安全特性

### 6.1 审计日志

```sql
-- 查看审计日志
SELECT * FROM audit_log
WHERE action IN ('CREATE', 'DROP', 'ALTER')
ORDER BY timestamp DESC
LIMIT 100;
```

### 6.2 GRANT/REVOKE (v2.9.0)

```sql
-- 授予权限
GRANT SELECT, INSERT ON database.* TO 'user'@'localhost';
GRANT UPDATE(amount) ON orders TO 'order_app'@'localhost';

-- 撤销权限
REVOKE INSERT ON database.* FROM 'user'@'localhost';

-- 创建角色
CREATE ROLE analyst, developer;

-- 授予角色
GRANT analyst TO 'analyst_user'@'localhost';
```

### 6.3 角色管理 (v2.9.0)

```sql
-- 创建角色
CREATE ROLE data_analyst;

-- 授予角色权限
GRANT SELECT ON sales.* TO data_analyst;
GRANT UPDATE ON inventory TO data_analyst;

-- 分配角色给用户
GRANT data_analyst TO 'user'@'localhost';

-- 查看角色
SHOW ROLES;
```

---

## 7. 性能优化

### 7.1 TPC-H 基准测试

```bash
# 运行 TPC-H 基准
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6 --iterations 3

# 指定规模因子
cargo run --bin bench-cli -- tpch bench --queries Q1-Q22 --iterations 3 --sf 0.1
```

### 7.2 Sysbench OLTP

```bash
# 运行 Sysbench OLTP 测试
cargo run --bin bench-cli -- sysbench oltp --threads 4

# 自定义测试
cargo run --bin bench-cli -- sysbench oltp --threads 8 --time 60
```

### 7.3 索引优化

```sql
-- 创建索引
CREATE INDEX idx_name ON users(name);

-- 创建复合索引
CREATE INDEX idx_dept_salary ON employees(department, salary);

-- 查看查询计划
EXPLAIN SELECT * FROM users WHERE name = 'Alice';
```

---

## 8. 配置

### 8.1 配置文件

```toml
# sqlrustgo.toml
[database]
path = "./data"

[wal]
enabled = true
checkpoint_interval = 300

[server]
host = "0.0.0.0"
port = 3306

[storage]
type = "file"
path = "./data"

[replication]
type = "semi-sync"
source = "192.168.0.100:3306"
ack_timeout = 10

[transaction]
xa_enabled = true
xa_timeout = 60
```

### 8.2 命令行参数

```bash
# 启动服务器
sqlrustgo-server --host 0.0.0.0 --port 3306

# 启动主节点
sqlrustgo-server --role primary --port 3306

# 启动从节点 (Semi-sync)
sqlrustgo-server --role replica --port 3307 --source 127.0.0.1:3306

# 启动从节点 (MTS)
sqlrustgo-server --role replica --port 3307 --source 127.0.0.1:3306 --mts-workers 4
```

---

## 9. 故障排查

### 9.1 常见问题

| 问题 | 解决方案 |
|------|----------|
| 启动失败 | 检查端口是否被占用 |
| 性能下降 | 运行 `VACUUM` 清理 |
| 索引失效 | 重建索引 `REINDEX` |
| 复制延迟 | 检查网络连接和主节点负载 |
| XA 事务失败 | 检查 `XA RECOVER` 状态 |

### 9.2 分布式问题

```sql
-- 检查 Semi-sync 状态
SHOW VARIABLES LIKE 'semi_sync%';

-- 检查 MTS 状态
SHOW SLAVE STATUS;

-- 检查 XA 事务状态
XA RECOVER;

-- 重启复制
STOP SLAVE;
START SLAVE;
```

---

## 10. 相关文档

| 文档 | 说明 |
|------|------|
| [快速开始](./QUICK_START.md) | 快速入门指南 |
| [客户端连接](./CLIENT_CONNECTION.md) | 连接方式详解 |
| [迁移指南](./MIGRATION_GUIDE.md) | 从 v2.8.0 升级 |
| [分布式设计](./DISTRIBUTED_DESIGN.md) | 分布式架构详解 |
| [API 参考](./API_REFERENCE.md) | REST API 文档 |
| [安全加固](./SECURITY_HARDENING.md) | 安全配置指南 |

---

*用户手册 v2.9.0*
*最后更新: 2026-05-05*
