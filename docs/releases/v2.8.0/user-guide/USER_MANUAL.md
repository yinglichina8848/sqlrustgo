# SQLRustGo v2.8.0 用户手册

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **状态**: Alpha
> **更新日期**: 2026-04-23

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

### 2.2 TRUNCATE TABLE (v2.8.0 新增)

```sql
-- 清空表数据，比 DELETE 更快
TRUNCATE TABLE users;

-- 重置自增计数器
TRUNCATE TABLE orders;
```

### 2.3 REPLACE INTO (v2.8.0 新增)

```sql
-- 如果唯一键冲突，替换现有行
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com');
```

### 2.4 聚合查询

```sql
-- 聚合函数
SELECT COUNT(*) FROM orders;
SELECT SUM(amount) FROM orders;
SELECT AVG(price) FROM products;

-- 分组
SELECT department, AVG(salary) FROM employees GROUP BY department;

-- 分组过滤
SELECT department, AVG(salary) as avg_sal
FROM employees
GROUP BY department
HAVING AVG(salary) > 50000;
```

### 2.5 JOIN

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

-- FULL OUTER JOIN (v2.8.0 新增)
SELECT u.name, o.order_id
FROM users u
FULL OUTER JOIN orders o ON u.id = o.user_id;
```

### 2.6 窗口函数 (v2.8.0)

```sql
-- ROW_NUMBER
SELECT name, department,
       ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;

-- RANK
SELECT name, score, RANK() OVER (ORDER BY score DESC) as rank
FROM leaderboard;

-- DENSE_RANK
SELECT name, score, DENSE_RANK() OVER (ORDER BY score DESC) as dense_rank
FROM leaderboard;
```

---

## 3. 分区表 (v2.8.0 分布式能力)

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

### 3.4 Key 分区

```sql
CREATE TABLE user_data (
    id INTEGER,
    name VARCHAR(100)
)
PARTITION BY KEY (id)
PARTITIONS 4;
```

---

## 4. 主从复制 (v2.8.0 分布式能力)

### 4.1 架构

```
Primary (主节点)  ──GTID复制──>  Replica (从节点)
     │                              │
  读写                              只读
```

### 4.2 配置主节点

```sql
-- 在主节点上创建复制用户
CREATE USER 'repl'@'%' IDENTIFIED BY 'repl_password';
GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';
```

### 4.3 连接从节点到主节点

```sql
-- 在从节点上执行
CHANGE MASTER TO
    MASTER_HOST = '主节点IP',
    MASTER_PORT = 3306,
    MASTER_USER = 'repl',
    MASTER_PASSWORD = 'repl_password',
    MASTER_AUTO_POSITION = 1;

START SLAVE;
```

---

## 5. 检索功能

详细用户指南请参考：
- [GMP 用户指南](./GMP_USER_GUIDE.md) - GMP 审计与合规
- [图检索用户指南](./GRAPH_SEARCH_USER_GUIDE.md) - 图引擎与 Cypher
- [向量检索用户指南](./VECTOR_SEARCH_USER_GUIDE.md) - 向量索引与混合检索

### 5.1 全文检索 (lex)

```sql
-- 全文搜索
SELECT * FROM documents
WHERE MATCH(content, 'keywords');
```

### 5.2 向量检索 (vec)

```sql
-- 语义搜索
SELECT * FROM documents
WHERE VECTOR_SEARCH(content, embedding, top_k=10);
```

### 5.3 图检索 (graph)

```sql
-- 关系查询
SELECT * FROM graph
WHERE PATH(start_id, end_id, hops=3);
```

### 5.4 混合检索 (hybrid)

```sql
-- 混合搜索 (RRF)
SELECT * FROM documents
WHERE HYBRID_SEARCH(
    content,
    embedding,
    strategy='RRF',
    weights=[0.3, 0.7]
);
```

---

## 6. 安全特性 (v2.8.0)

### 6.1 审计日志

审计日志自动记录所有关键操作：

```sql
-- 查看审计日志
SELECT * FROM audit_log
WHERE action IN ('CREATE', 'DROP', 'ALTER')
ORDER BY timestamp DESC
LIMIT 100;

-- 按用户过滤
SELECT * FROM audit_log
WHERE user = 'admin'
ORDER BY timestamp DESC;
```

### 6.2 列级权限 (v2.8.0)

```sql
-- 限制用户只能查看特定列
GRANT SELECT(salary, department) ON employees TO 'analyst'@'localhost';

-- 限制用户只能更新特定列
GRANT UPDATE(amount) ON orders TO 'order_app'@'localhost';

-- 撤销权限
REVOKE SELECT(password) ON users FROM 'app'@'localhost';
```

### 6.3 SQL 防火墙

```sql
-- 阻止潜在危险的 SQL 模式
-- 通过配置 SQL 防火墙规则防止 SQL 注入
```

---

## 7. 性能优化 (v2.8.0)

### 7.1 SIMD 向量化加速

```sql
-- SIMD 自动启用，对向量操作进行加速
-- 向量距离计算使用 SIMD 指令集
SELECT id, VECTOR_DISTANCE(embedding, query) AS dist
FROM documents
ORDER BY dist
LIMIT 10;
```

### 7.2 Hash Join 并行化

```sql
-- Hash Join 自动使用多线程
SELECT u.name, o.amount
FROM users u
JOIN orders o ON u.id = o.user_id;
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

[search]
default_mode = "hybrid"

[replication]
enabled = true
role = "primary"  # or "replica"
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

### 9.2 主从复制问题

```sql
-- 检查复制状态
SHOW SLAVE STATUS\G;

-- 检查主节点状态
SHOW MASTER STATUS;

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
| [API 参考](./API_REFERENCE.md) | REST API 文档 |
| [安全加固](./SECURITY_HARDENING.md) | 安全配置指南 |
| [错误消息](./ERROR_MESSAGES.md) | 错误代码参考 |
| [GMP 用户指南](./GMP_USER_GUIDE.md) | 图谱审计查询 |
| [图检索指南](./GRAPH_SEARCH_USER_GUIDE.md) | 图引擎使用 |
| [向量检索指南](./VECTOR_SEARCH_USER_GUIDE.md) | 向量索引使用 |

---

*用户手册 v2.8.0*
*最后更新: 2026-04-23*
