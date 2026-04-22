# 快速开始

> **版本**: alpha/v2.7.0

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

# 切换到 v2.7.0 分支
git checkout develop/v2.7.0

# 构建
cargo build --release
```

### 1.3 验证安装

```bash
# 运行测试
cargo test --workspace

# 启动 REPL
cargo run --release
```

---

## 2. 连接方式

SQLRustGo 支持多种连接方式，兼容 MySQL 5.7 协议。

### 2.1 MySQL CLI 连接 (推荐)

```bash
# 终端1: 启动 MySQL 协议服务器
sqlrustgo-mysql-server --host 127.0.0.1 --port 3306

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
SELECT u.name, o.amount
FROM users u
INNER JOIN orders o ON u.id = o.user_id;
```

---

## 5. 向量检索 (HNSW / IVF-PQ)

### 5.1 创建向量表

```sql
-- 创建带向量列的表
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    title VARCHAR(200),
    content TEXT,
    embedding VECTOR(768)  -- 768维向量 (BERT embeddings)
);
```

### 5.2 插入向量数据

```sql
INSERT INTO documents (id, title, content, embedding) VALUES 
(1, 'Rust编程', 'Rust是一种安全的系统编程语言', '[0.1, 0.2, ...]'),
(2, 'Go语言', 'Go是Google开发的编译型语言', '[0.3, 0.4, ...]');
```

### 5.3 HNSW 向量检索

```sql
-- 基于 HNSW 索引的向量相似度搜索
SELECT id, title, content, 
       VECTOR_DISTANCE(embedding, '[0.1, 0.2, ...]') AS distance
FROM documents
WHERE VECTOR_SEARCH(
    embedding, 
    '[0.1, 0.2, ...]',
    'hnsw',           -- 使用 HNSW 索引
    distance_type => 'cosine',
    limit => 10
)
ORDER BY distance;
```

### 5.4 IVF-PQ 向量检索

```sql
-- 基于 IVF-PQ 索引的向量搜索 (适合大规模数据)
SELECT id, title, content
FROM documents
WHERE VECTOR_SEARCH(
    embedding, 
    '[0.1, 0.2, ...]',
    'ivfpq',          -- 使用 IVF-PQ 索引
    nlist => 100,     -- 倒排列表数量
    nprobe => 10,     -- 查询时探查的簇数
    limit => 20
);
```

### 5.5 混合距离搜索

```sql
-- 多距离度量支持
SELECT id, title
FROM documents
WHERE VECTOR_SEARCH(
    embedding,
    '[0.1, 0.2, ...]',
    'hnsw',
    distance_type => 'l2',  -- L2距离
    limit => 5
);
```

---

## 6. GMP 图谱查询 (Top10 场景)

> GMP (Graph Memory Processor) 是 SQLRustGo 的图谱处理引擎

### 6.1 启用图谱

```sql
-- 创建好友关系表
CREATE TABLE friendships (
    user_id BIGINT,
    friend_id BIGINT,
    created_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (friend_id) REFERENCES users(id)
);

-- 启用图谱
ALTER TABLE friendships ADD GRAPH (user_id, friend_id);
```

### 6.2 场景1: 社交网络好友推荐 (二度人脉)

```sql
-- 查找用户A的可能认识的人
GRAPH MATCH (me)-[:friendship]->(friend)-[:friendship]->(suggestion)
WHERE me.id = @current_user_id
  AND suggestion.id != @current_user_id
  AND suggestion.id NOT IN (
    SELECT friend_id FROM friendships WHERE user_id = @current_user_id
  )
RETURN suggestion.name, COUNT(*) AS common_friends
ORDER BY common_friends DESC
LIMIT 10;
```

### 6.3 场景2: 知识图谱多跳查询

```sql
-- 三跳查询: 老师的学生的学习单位
GRAPH MATCH (university)-[:located_in]->(city)<-[:located_in]-(org)<-[:works_for]-(student)<-[:teaches]-(professor)
WHERE university.name = '清华大学' AND professor.name = @professor_name
RETURN DISTINCT org.name AS company, COUNT(DISTINCT student) AS num_students
ORDER BY num_students DESC;
```

### 6.4 场景3: 欺诈检测 (异常模式)

```sql
-- 检测资金快速转移模式 (黑钱洗白)
GRAPH MATCH (source)-[:transaction*1..3]->(middleman)-[:transaction]->(destination)
WHERE source.account_type = 'personal'
  AND destination.account_type = 'corporate'
  AND ALL(r IN relationships WHERE r.amount > 10000)
RETURN source.name, destination.name, COUNT(*) AS hop_count
ORDER BY hop_count DESC;
```

### 6.5 场景4: 推荐系统 (协同过滤)

```sql
-- 查找相似用户喜欢的物品
GRAPH MATCH (me)-[:action{purchase}]->(item)<-[:action{purchase}]-(similar_user)-[:action{purchase}]->(recommended)
WHERE me.id = @current_user_id
  AND recommended.id NOT IN (
    SELECT item_id FROM user_item_actions WHERE user_id = @current_user_id AND action_type = 'purchase'
  )
RETURN recommended.name, COUNT(DISTINCT similar_user) AS popularity
ORDER BY popularity DESC
LIMIT 10;
```

### 6.6 场景5: 供应链追踪

```sql
-- 查找原材料到最终产品的所有路径
GRAPH MATCH path = (raw_material)-[:supply_edge*1..5]->(final_product)
WHERE raw_material.node_type = 'supplier'
  AND final_product.node_type = 'retailer'
  AND final_product.name = @product_name
RETURN [n IN nodes(path) | n.name] AS supply_chain,
       REDUCE(cost = 0, r IN relationships(path) | cost + r.cost) AS total_cost
ORDER BY total_cost ASC;
```

---

## 7. 混合搜索 (qmd-bridge)

> qmd-bridge 是 SQLRustGo 与 QMD (Query Memory Database) 的双向数据桥梁

### 7.1 同步数据到 QMD

```sql
-- 将向量数据同步到 QMD
SYNC TO QMD FROM documents WHERE condition;

-- 检查同步状态
SELECT * FROM qmd_sync_status();
```

### 7.2 纯向量检索

```sql
SELECT * FROM users
WHERE VECTOR_SEARCH(embedding, query_vector, 'hnsw', limit => 10);
```

### 7.3 图谱检索

```sql
SELECT * FROM users
WHERE GRAPH_MATCH(pattern, 'MATCH (a)-[r]->(b) WHERE a.age > 30');
```

### 7.4 混合检索 (向量 + 图谱 + 全文)

```sql
-- 混合搜索: 结合向量相似度、图谱关系、关键词
SELECT * FROM documents
WHERE HYBRID_SEARCH(
    embedding => query_vector,           -- 向量搜索
    graph_pattern => 'MATCH (a)-[r]->(b)', -- 图谱模式
    text_query => 'Rust 编程',           -- 关键词搜索
    weights => [0.4, 0.3, 0.3],          -- 权重分配
    limit => 10
);
```

### 7.5 检索 API (Rust)

```rust
use sqlrustgo::qmd_bridge::{QmdBridge, HybridQuery};

// 混合检索
let query = HybridQuery {
    vector: Some(query_vector),
    graph_pattern: Some("MATCH (a)-[r]->(b)".to_string()),
    text_query: Some("Rust".to_string()),
    filters: vec![],
    limit: 10,
};

let result = qmd.search_from_qmd(&query)?;
```

---

## 8. 性能基准

### 8.1 SQL Corpus

```
=== Summary ===
Total: 59 cases, 59 passed, 0 failed
Pass rate: 100.0%
```

### 8.2 向量检索性能

| 索引类型 | 召回率 | 延迟 (p99) | 吞吐量 |
|----------|--------|------------|--------|
| HNSW | 95-99% | < 50ms | > 1000 QPS |
| IVF-PQ | 85-90% | < 30ms | > 2000 QPS |

### 8.3 GMP 图谱性能

| 场景 | 数据规模 | 查询延迟目标 | 吞吐量目标 |
|------|----------|--------------|------------|
| 社交网络 | 100万用户, 1000万关系 | < 100ms | > 1000 QPS |
| 知识图谱 | 100万实体, 500万关系 | < 200ms | > 500 QPS |
| 欺诈检测 | 10万账户, 100万交易 | < 50ms | > 5000 QPS |

---

## 9. 文档

| 文档 | 说明 |
|------|------|
| [README.md](./README.md) | 文档索引 |
| [INSTALL.md](./INSTALL.md) | 安装指南 |
| [CLIENT_CONNECTION.md](./CLIENT_CONNECTION.md) | 客户端连接指南 |
| [API_REFERENCE.md](./API_REFERENCE.md) | REST API 参考 |
| [gmp-top10-scenarios.md](./gmp-top10-scenarios.md) | GMP Top10 场景详解 |
| [qmd-bridge-design.md](./qmd-bridge-design.md) | qmd-bridge 设计文档 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-22*
