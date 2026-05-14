# v3.1.0 用户手册

> **版本**: v3.1.0
> **发布日期**: 2026-05-14
> **状态**: GA Ready

---

## 一、快速开始

### 1.1 安装

```bash
# 从源码编译
cargo build --release --bin sqlrustgo

# 或使用 Docker
docker pull sqlrustgo:v3.1.0
```

### 1.2 启动服务

```bash
# 启动 MySQL 兼容服务器
./target/release/sqlrustgo --port 3306 --data-dir /data/sqlrustgo

# 使用 Docker
docker run -p 3306:3306 sqlrustgo:v3.1.0
```

### 1.3 连接

```bash
mysql -h 127.0.0.1 -P 3306 -u root
```

---

## 二、SQL 参考

### 2.1 DDL 操作

#### 创建数据库

```sql
CREATE DATABASE IF NOT EXISTS mydb;
USE mydb;
```

#### 创建表

```sql
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 修改表

```sql
ALTER TABLE users ADD COLUMN age INT;
ALTER TABLE users MODIFY COLUMN name VARCHAR(200);
ALTER TABLE users DROP COLUMN age;
```

#### 索引

```sql
CREATE INDEX idx_email ON users(email);
CREATE UNIQUE INDEX idx_name ON users(name);
DROP INDEX idx_email ON users;
```

### 2.2 DML 操作

#### 插入

```sql
INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
INSERT INTO users VALUES (1, 'Bob', 'bob@example.com', NULL);
INSERT INTO users (name, email) VALUES 
    ('Charlie', 'charlie@example.com'),
    ('Diana', 'diana@example.com');
```

#### 更新

```sql
UPDATE users SET email = 'new@example.com' WHERE id = 1;
UPDATE users SET age = 30 WHERE name LIKE 'A%';
```

#### 删除

```sql
DELETE FROM users WHERE id = 1;
DELETE FROM users WHERE created_at < '2026-01-01';
```

#### 替换

```sql
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice.new@example.com');
```

### 2.3 DQL 操作

#### 查询

```sql
SELECT * FROM users;
SELECT id, name, email FROM users WHERE id > 10;
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users;
```

#### 聚合

```sql
SELECT department, COUNT(*) as cnt 
FROM employees 
GROUP BY department 
HAVING cnt > 5;
```

#### JOIN

```sql
SELECT u.name, o.order_id 
FROM users u 
INNER JOIN orders o ON u.id = o.user_id;

SELECT u.name, o.order_id 
FROM users u 
LEFT JOIN orders o ON u.id = o.user_id;
```

#### 子查询

```sql
SELECT * FROM users 
WHERE id IN (SELECT user_id FROM orders WHERE total > 100);
```

### 2.4 事务

```sql
START TRANSACTION;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
COMMIT;
```

### 2.5 SAVEPOINT

```sql
START TRANSACTION;
INSERT INTO orders (user_id, total) VALUES (1, 100);
SAVEPOINT sp1;
INSERT INTO orders (user_id, total) VALUES (1, 200);
ROLLBACK TO SAVEPOINT sp1;
COMMIT;
```

---

## 三、配置选项

### 3.1 命令行选项

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `--port` | 3306 | 监听端口 |
| `--host` | 0.0.0.0 | 监听地址 |
| `--data-dir` | ./data | 数据目录 |
| `--log-level` | info | 日志级别 |
| `--max-connections` | 100 | 最大连接数 |

### 3.2 环境变量

| 变量 | 说明 |
|------|------|
| `SQLRUSTGO_DATA_DIR` | 数据目录 |
| `SQLRUSTGO_LOG_LEVEL` | 日志级别 |
| `SQLRUSTGO_PORT` | 端口 |

---

## 四、性能调优

### 4.1 连接池

```sql
-- 查看连接池状态
SHOW STATUS LIKE 'Connection_pool%';

-- 配置连接池大小
SET GLOBAL max_connections = 200;
```

### 4.2 缓存

```sql
-- 查看查询缓存
SHOW STATUS LIKE 'Qcache%';

-- 清空缓存
RESET QUERY CACHE;
```

### 4.3 EXPLAIN

```sql
EXPLAIN SELECT * FROM users WHERE email = 'test@example.com';
EXPLAIN ANALYZE SELECT * FROM orders WHERE user_id = 1;
```

---

## 五、故障排除

### 5.1 连接问题

```bash
# 检查端口占用
lsof -i :3306

# 检查防火墙
sudo firewall-cmd --list-ports
```

### 5.2 性能问题

```sql
-- 慢查询日志
SHOW VARIABLES LIKE 'slow_query_log%';

-- 查看进程列表
SHOW PROCESSLIST;
KILL <process_id>;
```

### 5.3 数据恢复

```bash
# 从 WAL 恢复
sqlrustgo-recover --data-dir /data/sqlrustgo

# 检查数据完整性
sqlrustgo-check --data-dir /data/sqlrustgo
```

---

## 六、参考

### 6.1 系统表

| 表 | 说明 |
|------|------|
| information_schema.schemata | 数据库列表 |
| information_schema.tables | 表列表 |
| information_schema.columns | 列信息 |
| performance_schema.events_statements_summary | 语句统计 |

### 6.1 相关文档

- [README](./README.md) - 项目概述
- [QUICK_START](./QUICK_START.md) - 快速开始
- [API_REFERENCE](./API_REFERENCE.md) - API 参考

---

*最后更新: 2026-05-14*
