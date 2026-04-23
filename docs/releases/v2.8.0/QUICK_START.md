# 快速开始

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure

---

## 1. 安装

### 1.1 前置要求

- Rust 1.85+
- Cargo (随 Rust 安装)

### 1.2 安装步骤

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.8.0 分支
git checkout develop/v2.8.0

# 构建
cargo build --all-features --release
```

### 1.3 验证安装

```bash
# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --release --bin sqlrustgo
```

---

## 2. 连接方式

SQLRustGo 支持多种连接方式，兼容 MySQL 5.7 协议。

### 2.1 MySQL CLI 连接 (推荐)

```bash
# 终端1: 启动 MySQL 协议服务器
cargo run --release --bin sqlrustgo-mysql-server --host 127.0.0.1 --port 3306

# 终端2: 使用 mysql 客户端连接
mysql -h 127.0.0.1 -P 3306 -u root
```

### 2.2 ODBC 连接 (Windows/Linux)

```bash
# 配置 ODBC 数据源后，使用标准 MySQL ODBC 驱动连接
# 连接字符串: Driver={MySQL ODBC 8.0 Driver};Server=127.0.0.1;Port=3306;Database=default
```

### 2.3 JDBC 连接 (Java)

```java
// 使用 MySQL Connector/J 连接
String url = "jdbc:mysql://127.0.0.1:3306/default";
Connection conn = DriverManager.getConnection(url, "root", "");
```

详细连接方式请参考 [客户端连接指南](./CLIENT_CONNECTION.md)。

---

## 3. REST API

### 3.1 启动 REST API 服务器

```bash
# 启动 HTTP 服务器 (端口 8080)
cargo run --release --bin sqlrustgo-server
```

### 3.2 API 端点

| 端点 | 说明 |
|------|------|
| `GET /health` | 健康检查 |
| `GET /metrics` | Prometheus 指标 |

详细 API 文档请参考 [REST API 参考](./API_REFERENCE.md)。

---

## 4. 基础 SQL 操作

### 4.1 创建表

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    email VARCHAR(255)
);
```

### 4.2 插入数据

```sql
INSERT INTO users (id, name, email) VALUES
(1, 'Alice', 'alice@example.com'),
(2, 'Bob', 'bob@example.com');
```

### 4.3 查询

```sql
SELECT * FROM users;
SELECT name, email FROM users WHERE id = 1;
```

### 4.4 聚合查询

```sql
SELECT COUNT(*) FROM users;
SELECT AVG(age) FROM users GROUP BY department;
```

### 4.5 JOIN

```sql
-- 内连接
SELECT u.name, o.amount
FROM users u
INNER JOIN orders o ON u.id = o.user_id;

-- 左连接
SELECT u.name, o.amount
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- 右连接
SELECT u.name, o.amount
FROM users u
RIGHT JOIN orders o ON u.id = o.user_id;

-- FULL OUTER JOIN (v2.8.0 新增)
SELECT u.name, o.amount
FROM users u
FULL OUTER JOIN orders o ON u.id = o.user_id;
```

---

## 5. 高级 SQL 特性

### 5.1 TRUNCATE TABLE (v2.8.0)

```sql
-- 清空表数据（比 DELETE 更快）
TRUNCATE TABLE users;

-- 等同于 DELETE 但重置自增计数器
TRUNCATE TABLE orders;
```

### 5.2 REPLACE INTO (v2.8.0)

```sql
-- 如果唯一键冲突，替换现有行
REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com');
```

### 5.3 窗口函数 (v2.8.0)

```sql
-- ROW_NUMBER
SELECT name, department,
       ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;

-- RANK
SELECT name, score, RANK() OVER (ORDER BY score DESC) as rank
FROM leaderboard;
```

---

## 6. 分区表 (v2.8.0 分布式能力)

### 6.1 Range 分区

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

### 6.2 List 分区

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

### 6.3 Hash 分区

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

## 7. 向量检索 (HNSW / IVF-PQ)

### 7.1 创建向量表

```sql
-- 创建带向量列的表
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    title VARCHAR(200),
    content TEXT,
    embedding VECTOR(768)  -- 768维向量 (BERT embeddings)
);
```

### 7.2 HNSW 向量检索

```sql
-- 基于 HNSW 索引的向量相似度搜索
SELECT id, title, content,
       VECTOR_DISTANCE(embedding, '[0.1, 0.2, ...]') AS distance
FROM documents
WHERE VECTOR_SEARCH(
    embedding,
    '[0.1, 0.2, ...]',
    'hnsw',
    distance_type => 'cosine',
    limit => 10
)
ORDER BY distance;
```

### 7.3 IVF-PQ 向量检索

```sql
-- 基于 IVF-PQ 索引的向量搜索 (适合大规模数据)
SELECT id, title, content
FROM documents
WHERE VECTOR_SEARCH(
    embedding,
    '[0.1, 0.2, ...]',
    'ivfpq',
    nlist => 100,
    nprobe => 10,
    limit => 20
);
```

---

## 8. 安全特性 (v2.8.0)

### 8.1 审计日志

```sql
-- 审计日志自动记录关键操作
-- 查看审计日志
SELECT * FROM audit_log WHERE action = 'DELETE' ORDER BY timestamp DESC LIMIT 10;
```

### 8.2 列级权限 (v2.8.0)

```sql
-- 限制用户只能查看特定列
GRANT SELECT(salary, department) ON employees TO 'analyst'@'localhost';
REVOKE SELECT(password) ON users FROM 'app'@'localhost';
```

---

## 9. 性能基准

### 9.1 SQL Corpus

```
=== Summary ===
Total: 59 cases, 59 passed, 0 failed
Pass rate: 100.0%
```

### 9.2 向量检索性能

| 索引类型 | 召回率 | 延迟 (p99) | 吞吐量 |
|----------|--------|------------|--------|
| HNSW | 95-99% | < 50ms | > 1000 QPS |
| IVF-PQ | 85-90% | < 30ms | > 2000 QPS |

### 9.3 SIMD 加速 (v2.8.0 性能优化)

| 特性 | 加速比 |
|------|--------|
| SIMD 向量化 | ≥ 2x |
| Hash Join 并行化 | ≥ 1.5x |

---

## 10. 文档

| 文档 | 说明 |
|------|------|
| [README.md](./README.md) | 文档索引 |
| [CLIENT_CONNECTION.md](./CLIENT_CONNECTION.md) | 客户端连接指南 |
| [API_REFERENCE.md](./API_REFERENCE.md) | REST API 参考 |
| [USER_MANUAL.md](./USER_MANUAL.md) | 用户手册 |
| [SECURITY_HARDENING.md](./SECURITY_HARDENING.md) | 安全加固指南 |
| [ERROR_MESSAGES.md](./ERROR_MESSAGES.md) | 错误消息参考 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-23*
