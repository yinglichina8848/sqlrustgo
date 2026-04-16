# SQLRustGo v2.5.0 用户手册

**版本**: v2.5.0 (Full Integration + GMP)
**发布日期**: 2026-04-16
**适用用户**: 数据库使用者、应用开发者、运维人员

---

## 一、快速开始

### 1.1 安装

#### 从源码构建

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 切换到 v2.5.0 分支
git checkout develop/v2.5.0

# 构建 (Debug 模式)
cargo build

# 构建 (Release 模式，推荐用于生产)
cargo build --release

# 运行测试
cargo test --workspace
```

#### 使用 Docker

```bash
# 拉取镜像
docker pull minzuuniversity/sqlrustgo:v2.5.0

# 运行 REPL
docker run -it minzuuniversity/sqlrustgo:v2.5.0

# 运行服务器模式
docker run -p 5432:5432 minzuuniversity/sqlrustgo:v2.5.0 --server
```

---

### 1.2 REPL 模式

```bash
# 启动 REPL
cargo run --release

# 在 REPL 中执行 SQL
SQL > CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);
SQL > INSERT INTO users VALUES (1, 'Alice', 30);
SQL > INSERT INTO users VALUES (2, 'Bob', 25);
SQL > SELECT * FROM users WHERE age > 25;
SQL > UPDATE users SET age = 31 WHERE id = 1;
SQL > DELETE FROM users WHERE id = 2;
```

### 1.3 服务器模式

```bash
# 启动 HTTP 服务器 (默认端口 5432)
cargo run --release -- --server

# 启动 MySQL 协议服务器
cargo run --release -- --mysql-server

# 使用自定义端口
cargo run --release -- --server --port 8080
```

---

## 二、事务支持

### 2.1 MVCC 事务

v2.5.0 支持快照隔离 (Snapshot Isolation)：

```sql
-- 开始事务
BEGIN;

-- 执行操作
INSERT INTO orders VALUES (1, '2024-01-01', 100.00);
SELECT * FROM orders WHERE id = 1;

-- 提交事务
COMMIT;
```

#### 事务隔离级别

```sql
-- 设置隔离级别
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
```

#### Savepoint 支持

```sql
SAVEPOINT sp1;
ROLLBACK TO SAVEPOINT sp1;
```

### 2.2 WAL 崩溃恢复

WAL 自动启用，崩溃后自动恢复：

```bash
# 启用 WAL (默认已启用)
--storage.wal.enabled=true

# 查看 WAL 状态
SELECT * FROM system.wal_status;
```

### 2.3 PITR 时间点恢复

```bash
# 恢复到指定时间点
./sqlrustgo --recover-to-timestamp "2024-01-01 12:00:00"

# 恢复到指定表
./sqlrustgo --recover-table users --to-timestamp "2024-01-01 12:00:00"
```

---

## 三、SQL 特性

### 3.1 外键约束

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT
);

CREATE TABLE orders (
    id INTEGER PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    amount DECIMAL(10, 2)
);

-- 测试外键动作
INSERT INTO users VALUES (1, 'Alice');
INSERT INTO orders VALUES (1, 1, 100.00);
DELETE FROM users WHERE id = 1;  -- 自动删除关联订单
```

#### 支持的动作

- `ON DELETE`: CASCADE, SET NULL, RESTRICT
- `ON UPDATE`: CASCADE, SET NULL, RESTRICT

### 3.2 预处理语句

```sql
-- 准备语句
PREPARE stmt AS SELECT * FROM users WHERE age > $1;

-- 执行
EXECUTE stmt USING 25;

-- 删除
DEALLOCATE PREPARE stmt;
```

### 3.3 子查询

```sql
-- EXISTS 子查询
SELECT * FROM users WHERE EXISTS (
    SELECT 1 FROM orders WHERE user_id = users.id
);

-- IN 子查询
SELECT * FROM users WHERE id IN (
    SELECT user_id FROM orders WHERE amount > 100
);

-- ANY/ALL 子查询
SELECT * FROM users WHERE age > ANY (
    SELECT age FROM users WHERE city = 'Beijing'
);

SELECT * FROM users WHERE age > ALL (
    SELECT age FROM users WHERE city = 'Beijing'
);
```

### 3.4 JOIN 增强

```sql
-- SEMI JOIN (仅返回左表)
SELECT * FROM orders LEFT SEMI JOIN users ON orders.user_id = users.id;

-- ANTI JOIN (返回不匹配的左表)
SELECT * FROM orders LEFT ANTI JOIN users ON orders.user_id = users.id;

-- FULL OUTER JOIN
SELECT * FROM orders FULL OUTER JOIN users ON orders.user_id = users.id;
```

### 3.5 窗口函数

```sql
SELECT 
    name,
    department,
    salary,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rn,
    RANK() OVER (ORDER BY salary) as rank,
    SUM(salary) OVER (PARTITION BY department) as dept_total
FROM employees;
```

---

## 四、图引擎

### 4.1 Cypher 查询

```sql
-- 创建图数据
CREATE GRAPH social;

-- 添加节点
CREATE (p:Person {name: 'Alice', age: 30});
CREATE (p:Person {name: 'Bob', age: 25});
CREATE (p:Person {name: 'Charlie', age: 35});

-- 添加边
CREATE (a:Person {name: 'Alice'})-[:KNOWS {since: 2020}]->(b:Person {name: 'Bob'});
CREATE (b:Person {name: 'Bob'})-[:KNOWS {since: 2021}]->(c:Person {name: 'Charlie'});
```

### 4.2 图查询

```sql
-- 查找朋友
MATCH (p:Person {name: 'Alice'})-[:KNOWS]->(friend)
RETURN friend.name, friend.age;

-- 查找朋友的朋友 (2跳)
MATCH (p:Person {name: 'Alice'})-[:KNOWS*2]->(fof)
RETURN DISTINCT fof.name;

-- 带条件的图查询
MATCH (p:Person)-[r:KNOWS]->(friend)
WHERE r.since > 2020
RETURN p.name, friend.name, r.since
ORDER BY r.since
LIMIT 10;
```

### 4.3 遍历算法

| 算法 | 适用场景 | 示例 |
|------|----------|------|
| BFS | 最短路径 | `MATCH (a)-[:KNOWS*1..3]->(b)` |
| DFS | 模式匹配 | 深度遍历搜索 |
| MultiHop | 多跳查询 | `MATCH (a)-[:KNOWS*n]->(b)` |

---

## 五、向量搜索

### 5.1 创建向量表

```sql
-- 创建带向量列的表
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    content TEXT,
    embedding VECTOR(128)  -- 128 维向量
);

-- 插入向量数据
INSERT INTO documents VALUES (
    1, 
    'SQLRustGo is a database',
    '[0.1, 0.2, 0.3, ...]'::VECTOR(128)
);
```

### 5.2 向量索引

```sql
-- 创建 HNSW 索引
CREATE INDEX idx_embedding ON documents USING HNSW (embedding);

-- 创建 IVFPQ 索引 (压缩)
CREATE INDEX idx_embedding_pq ON documents USING IVFPQ (embedding);
```

### 5.3 向量搜索

```sql
-- 相似度搜索
SELECT id, content, 
       vector_distance(embedding, '[0.1, 0.2, ...]'::VECTOR(128)) as distance
FROM documents
ORDER BY distance ASC
LIMIT 10;
```

### 5.4 配置参数

```sql
-- HNSW 参数
CREATE INDEX idx_hnsw ON t USING HNSW (v) WITH (m=16, ef_construction=200, ef_search=50);

-- IVF 参数
CREATE INDEX idx_ivf ON t USING IVF (v) WITH (nlist=100, nprobe=10);

-- IVFPQ 参数
CREATE INDEX idx_pq ON t USING IVFPQ (v) WITH (m_pq=8, k_sub=256, nlist=100);
```

---

## 六、统一查询 API

### 6.1 HTTP API

```bash
# SQL 查询
curl -X POST http://localhost:5432/sql \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT * FROM users"}'

# 向量搜索
curl -X POST http://localhost:5432/vector \
  -H "Content-Type: application/json" \
  -d '{"table": "documents", "column": "embedding", "query": [0.1, 0.2, ...], "k": 10}'

# 图查询
curl -X POST http://localhost:5432/graph \
  -H "Content-Type: application/json" \
  -d '{"query": "MATCH (p:Person)-[:KNOWS]->(f) RETURN f"}'
```

### 6.2 统一查询 (SQL + 向量 + 图)

```sql
-- SQL 条件 + 向量排序
SELECT id, content, 
       vector_distance(embedding, $query_vector) as score
FROM documents
WHERE category = 'tech'
ORDER BY score ASC
LIMIT 10;
```

---

## 七、性能优化

### 7.1 CBO 优化器

自动启用基于成本的优化：

```sql
-- 查看查询��划
EXPLAIN SELECT * FROM orders JOIN users ON orders.user_id = users.id;

-- 强制索引
SELECT * FROM users USE INDEX (idx_name);
```

### 7.2 BloomFilter

自动启用 BloomFilter 优化 IN/AND 查询：

```sql
-- 查看 BloomFilter 使用情况
EXPLAIN SELECT * FROM orders WHERE user_id IN (1, 2, 3, 4, 5);
```

### 7.3 列式存储

```sql
-- 创建列式表
CREATE TABLE logs (...) WITH (storage = 'columnar', compression = 'zstd');
```

---

## 八、基准测试

### 8.1 TPC-H 基准

```bash
# 生成 SF=1 数据
cargo run --release --bin tpch_gen -- --sf 1

# 运行 TPC-H 查询
cargo test --test tpch_sf1_benchmark
```

### 8.2 Sysbench OLTP

```bash
# 运行 OLTP 工作负载
cargo test --test oltp_workload_test -- --workload oltp_read_only
cargo test --test oltp_workload_test -- --workload oltp_read_write
cargo test --test oltp_workload_test -- --workload oltp_write_only
```

---

## 九、配置参考

### 9.1 服务器配置

```toml
[server]
host = "127.0.0.1"
port = 5432
max_connections = 100

[storage]
wal_enabled = true
buffer_pool_size = 1024  # MB

[storage.wal]
sync_mode = "sync"
check_interval = 30  # seconds

[storage.vector]
default_index = "hnsw"
simd_enabled = true
```

### 9.2 连接池配置

```toml
[connection_pool]
max_size = 50
min_idle = 5
max_lifetime = 3600  # seconds
idle_timeout = 600   # seconds
```

---

## 十、故障排除

### 10.1 常见问题

| 问题 | 解决方案 |
|------|----------|
| 编译失败 | 运行 `cargo update` |
| 测试失败 | 检查日志 `cargo test -- --nocapture` |
| 性能问题 | 检查查询计划 `EXPLAIN` |
| 内存问题 | 调整 `buffer_pool_size` |
| WAL 恢复 | 检查 `system.wal_status` |

### 10.2 日志配置

```bash
# 启用调试日志
RUST_LOG=debug cargo run

# 启用跟踪
RUST_LOG=trace cargo run

# 只 error 级别
RUST_LOG=error cargo run
```

### 10.3 系统表

```sql
-- 查看版本
SELECT * FROM system.version;

-- 查看配置
SELECT * FROM system.config;

-- 查看统计
SELECT * FROM system.stats;

-- 查看锁
SELECT * FROM system.locks;
```

---

## 十一、升级指南

### 11.1 从 v1.x 升级

v2.5.0 包含重大变更：

1. **MVCC 必需 WAL**：确保 WAL 目录有足够空间
2. **新配置格式**：使用新的 TOML 配置
3. **新索引语法**：向量索引使用新的 CREATE INDEX 语法

### 11.2 兼容性

| 功能 | v1.x | v2.5 | 说明 |
|------|------|------|------|
| SQL 语法 | ✅ | ✅ | 兼容 |
| API | ✅ | ⚠️ | 需更新 |
| 索引 | ✅ | ⚠️ | 需重建向量索引 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-16*