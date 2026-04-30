# SQLRustGo v2.8.0 Rust API 使用示例

> **版本**: v2.8.0
> **更新日期**: 2026-04-30
> **状态**: 新增

---

## 一、核心 Crate 导入

```rust
use sqlrustgo::prelude::*;
use sqlrustgo::parser::{Parser, SelectStmt};
use sqlrustgo::planner::Planner;
use sqlrustgo::optimizer::Optimizer;
use sqlrustgo::executor::{ExecutionEngine, QueryResult};
use sqlrustgo::storage::{BufferPool, PageId};
use sqlrustgo::catalog::Catalog;
use sqlrustgo::types::{SqlResult, Value};
```

---

## 二、Parser API — 解析 SQL

### 2.1 基础解析

```rust
use sqlrustgo::parser::Parser;

fn parse_sql(sql: &str) -> SqlResult<SelectStmt> {
    let mut parser = Parser::new(sql.as_bytes());
    parser.parse_select()
}

// 解析 SELECT
let sql = "SELECT id, name, email FROM users WHERE age > 18";
let stmt = parse_sql(sql)?;
println!("列: {:?}", stmt.columns);
println!("表: {:?}", stmt.from_table);
```

### 2.2 解析 DDL

```rust
use sqlrustgo::parser::Parser;

fn parse_ddl(sql: &str) -> SqlResult<Statement> {
    let mut parser = Parser::new(sql.as_bytes());
    parser.parse_statement()
}

// CREATE TABLE
let sql = r#"
    CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(255) UNIQUE,
        age INTEGER DEFAULT 18,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
"#;
let stmt = parse_ddl(sql)?;

// DROP TABLE
let sql = "DROP TABLE IF EXISTS logs";
let stmt = parse_ddl(sql)?;

// CREATE INDEX
let sql = "CREATE INDEX idx_users_email ON users(email)";
let stmt = parse_ddl(sql)?;
```

### 2.3 解析 DML

```rust
// INSERT
let sql = "INSERT INTO users (name, email, age) VALUES ('Alice', 'alice@example.com', 25)";
let stmt = parse_ddl(sql)?;

// INSERT ... ON DUPLICATE KEY UPDATE (MySQL 兼容)
let sql = "INSERT INTO counters (id, count) VALUES (1, 1) ON DUPLICATE KEY UPDATE count = count + 1";
let stmt = parse_ddl(sql)?;

// REPLACE INTO (v2.8.0)
let sql = "REPLACE INTO users (id, name, email) VALUES (1, 'Alice', 'alice_new@example.com')";
let stmt = parse_ddl(sql)?;

// UPDATE
let sql = "UPDATE users SET age = age + 1 WHERE id = 1";
let stmt = parse_ddl(sql)?;

// DELETE
let sql = "DELETE FROM users WHERE age < 18";
let stmt = parse_ddl(sql)?;

// TRUNCATE (v2.8.0)
let sql = "TRUNCATE TABLE logs";
let stmt = parse_ddl(sql)?;
```

### 2.4 解析复杂 SELECT

```rust
// LIMIT / OFFSET (v2.8.0)
let sql = "SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 20";
let stmt = parse_sql(sql)?;
// stmt.limit = Some(10), stmt.offset = Some(20)

// GROUP BY + HAVING + ORDER BY
let sql = r#"
    SELECT department, COUNT(*) as cnt, AVG(salary) as avg_sal
    FROM employees
    WHERE hire_date > '2024-01-01'
    GROUP BY department
    HAVING COUNT(*) > 5
    ORDER BY avg_sal DESC
"#;
let stmt = parse_sql(sql)?;

// JOIN
let sql = r#"
    SELECT u.name, o.amount, p.name as product
    FROM users u
    INNER JOIN orders o ON u.id = o.user_id
    LEFT JOIN products p ON o.product_id = p.id
    WHERE o.amount > 100
"#;
let stmt = parse_sql(sql)?;

// SUBQUERY
let sql = r#"
    SELECT name, salary
    FROM employees
    WHERE salary > (SELECT AVG(salary) FROM employees)
"#;
let stmt = parse_sql(sql)?;
```

---

## 三、ExecutionEngine API — 执行查询

### 3.1 创建执行引擎

```rust
use sqlrustgo::executor::ExecutionEngine;

// 内存模式（默认）
let engine = ExecutionEngine::new();

// 持久化模式，指定数据目录
let engine = ExecutionEngine::with_data_dir("/var/lib/sqlrustgo")?;

// 创建带缓存池大小
let engine = ExecutionEngine::builder()
    .data_dir("/var/lib/sqlrustgo")
    .buffer_pool_size(1024) // 1024 页
    .build()?;
```

### 3.2 执行 DDL

```rust
use sqlrustgo::executor::ExecutionEngine;

let engine = ExecutionEngine::new();

// CREATE TABLE
let result = engine.execute("CREATE TABLE users (id INTEGER, name TEXT)")?;
println!("创建表，影响行数: {:?}", result.affected_rows());

// CREATE TABLE ... AS SELECT
let result = engine.execute("CREATE TABLE young_users AS SELECT * FROM users WHERE age < 30")?;

// DROP TABLE
let result = engine.execute("DROP TABLE IF EXISTS temp_logs")?;
```

### 3.3 执行 DML

```rust
let engine = ExecutionEngine::new();

// INSERT
let result = engine.execute("INSERT INTO users (name, email, age) VALUES ('Alice', 'alice@example.com', 25)")?;
println!("插入行数: {:?}", result.affected_rows());

// INSERT 多行
let result = engine.execute("INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com'), ('Carol', 'carol@example.com')")?;

// UPDATE
let result = engine.execute("UPDATE users SET age = age + 1 WHERE id = 1")?;
println!("更新行数: {:?}", result.affected_rows());

// DELETE
let result = engine.execute("DELETE FROM users WHERE age < 18")?;
println!("删除行数: {:?}", result.affected_rows());

// TRUNCATE
let result = engine.execute("TRUNCATE TABLE logs")?;
```

### 3.4 执行 SELECT

```rust
let engine = ExecutionEngine::new();

// 基础查询
let result = engine.execute("SELECT id, name, email FROM users WHERE age > 18")?;
for row in result.rows() {
    println!("行: {:?}", row);
}

// 聚合查询
let result = engine.execute("SELECT COUNT(*) as cnt, AVG(age) as avg_age FROM users")?;
let row = result.rows().next().unwrap();
println!("用户数: {:?}, 平均年龄: {:?}", row[0], row[1]);

// GROUP BY
let result = engine.execute("SELECT department, COUNT(*) FROM employees GROUP BY department")?;
for row in result.rows() {
    println!("部门: {:?}, 人数: {:?}", row[0], row[1]);
}

// ORDER BY + LIMIT
let result = engine.execute("SELECT * FROM users ORDER BY created_at DESC LIMIT 10")?;
for row in result.rows() {
    println!("{:?}", row);
}

// OFFSET (v2.8.0)
let result = engine.execute("SELECT * FROM users LIMIT 5 OFFSET 10")?;
```

### 3.5 执行事务

```rust
let engine = ExecutionEngine::new();

// 开始事务
let tx = engine.begin()?;
tx.execute("INSERT INTO accounts (id, balance) VALUES (1, 1000)")?;
tx.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1")?;

// 提交
tx.commit()?;

// 或者回滚
// tx.rollback()?;
```

### 3.6 EXPLAIN 查询计划

```rust
let engine = ExecutionEngine::new();
let plan = engine.explain("SELECT * FROM users WHERE age > 18")?;
println!("查询计划:\n{}", plan);
```

---

## 四、Vector API — 向量检索

### 4.1 HNSW 向量索引

```rust
use sqlrustgo_vector::{HnswIndex, DistanceMetric, VectorIndex};

let mut hnsw = HnswIndex::new(DistanceMetric::Cosine);
hnsw.set_ef(50);  // 搜索时动态列表大小
hnsw.set_m(16);   // 每个节点的邻居数

// 插入向量
let id = 1u64;
let embedding: Vec<f32> = vec![0.1; 768]; // 768维 BERT embedding
hnsw.insert(id, &embedding)?;

// 批量插入
let vectors: Vec<(u64, Vec<f32>)> = (0..1000)
    .map(|i| (i, vec![rand::random(); 768]))
    .collect();
for (id, vec) in vectors {
    hnsw.insert(id, &vec)?;
}

// 构建索引
hnsw.build()?;

# 使用 SIMD 加速搜索 (v2.8.0)
let query: Vec<f32> = vec![0.1; 768];
let results = hnsw.search(&query, 10)?; // top-10 最近邻
for (id, distance) in results {
    println!("id: {}, distance: {:.4}", id, distance);
}
```

### 4.2 IVF-PQ 向量索引

```rust
use sqlrustgo_vector::{IvfpqIndex, DistanceMetric};

let mut ivfpq = IvfpqIndex::new(DistanceMetric::Euclidean, 100); // 100 个聚类中心
ivfpq.set_pq_m(8);   // PQ 分组数
ivfpq.set_pq_nbits(8); // 每组量化位数

// 插入和构建
for (id, vec) in vectors {
    ivfpq.insert(id, &vec)?;
}
ivfpq.build()?;

# SIMD 加速搜索
let results = ivfpq.search(&query, 20, 5)?; // top-20, 每聚类探查5个
```

### 4.3 混合检索 (向量 + 关键词)

```rust
use sqlrustgo_vector::{HybridSearcher, SearchOptions};

let searcher = HybridSearcher::new(hnsw_index, text_index);

let results = searcher.search(
    "SELECT * FROM docs WHERE VECTOR_SEARCH(content, '[0.1, ...]', 'hnsw', limit => 10)",
    &query_embedding,
    SearchOptions::default()
        .with_rerank(true)
        .with_hybrid_alpha(0.7), // 0.7*向量 + 0.3*关键词
)?;
```

---

## 五、SIMD 向量化 API (v2.8.0)

### 5.1 显式 SIMD 函数

```rust
use sqlrustgo_vector::simd_explicit::{
    l2_distance_simd,
    cosine_distance_simd,
    dot_product_simd,
    batch_dot_product_simd,
    batch_l2_distance_simd,
    batch_cosine_distance_simd,
    detect_simd_lanes,
};

# 检测支持的 SIMD 能力
let lanes = detect_simd_lanes();
println!("SIMD lanes: {}", lanes); // AVX2=8, AVX-512=16, 或 fallback=1

# 单向量距离计算
let a: &[f32] = &[0.1; 128];
let b: &[f32] = &[0.2; 128];

let l2 = l2_distance_simd(a, b);
let cos = cosine_distance_simd(a, b);
let dot = dot_product_simd(a, b);

# 批量计算 (AVX2 加速)
let vectors: Vec<Vec<f32>> = (0..1000).map(|_| vec![rand::random(); 128]).collect();
let distances = batch_l2_distance_simd(a, &vectors);
# 返回 Vec<f32>，每向量与 query 的 L2 距离
```

### 5.2 性能基准对比

```rust
use sqlrustgo_vector::simd_explicit::{l2_distance_simd, l2_distance_scalar};
use std::time::Instant;

let a: Vec<f32> = (0..4096).map(|_| rand::random()).collect();
let b: Vec<f32> = (0..4096).map(|_| rand::random()).collect();
let iterations = 10000;

// Scalar 版本（ fallback）
let start = Instant::now();
for _ in 0..iterations {
    let _ = l2_distance_scalar(&a, &b);
}
let scalar_ms = start.elapsed().as_secs_f64() * 1000.0;

// SIMD 版本
let start = Instant::now();
for _ in 0..iterations {
    let _ = l2_distance_simd(&a, &b);
}
let simd_ms = start.elapsed().as_secs_f64() * 1000.0;

let speedup = scalar_ms / simd_ms;
println!("SIMD speedup: {:.2f}x", speedup); // 目标 >= 3x
```

---

## 六、存储引擎 API

### 6.1 BufferPool

```rust
use sqlrustgo_storage::buffer_pool::{BufferPool, PageId, Page};
use sqlrustgo_storage::disk::DiskManager;

let disk = DiskManager::open("data.db")?;
let mut pool = BufferPool::new(1024, disk); // 1024 页

// 读取页
let page_id = PageId(1);
let page = pool.get(page_id)?;
println!("页 {} 内容: {:?}", page_id, &page.data[..32]);

// 标记脏页
pool.mark_dirty(page_id);

// 刷新所有脏页
pool.flush_all()?;
```

### 6.2 WAL (Write-Ahead Log)

```rust
use sqlrustgo_transaction::wal::{WalManager, WalEntry, WalType};
use std::fs::File;

let wal = WalManager::open("sqlrustgo.wal")?;

// 写入 WAL 条目
let entry = WalEntry::new(WalType::Insert, b"table_name", b"row_data");
wal.append(&entry)?;

// 检查点
wal.checkpoint()?;

// 从 WAL 恢复
let entries = wal.recover()?;
for entry in entries {
    println!("恢复: {:?}", entry);
}
```

---

## 七、网络 API (MySQL 协议)

### 7.1 启动 MySQL 协议服务器

```rust
use sqlrustgo_network::mysql::{MySqlServer, ServerConfig};

let config = ServerConfig {
    addr: "127.0.0.1:3306",
    ..Default::default()
};

let server = MySqlServer::new(config, engine);
server.listen()?;
```

### 7.2 使用 MySQL 客户端连接

```bash
# 终端 1: 启动服务器
cargo run --bin sqlrustgo-mysql-server

# 终端 2: 连接
mysql -h 127.0.0.1 -P 3306 -u root
```

---

## 八、TriBool 三值逻辑 API (v2.8.0)

```rust
use sqlrustgo_types::tribool::TriBool;

// TriBool: True, False, Unknown (对应 SQL: TRUE, FALSE, NULL)
let a = TriBool::True;
let b = TriBool::False;
let c = TriBool::Unknown; // NULL

// AND 运算
assert_eq!(a.and(b), TriBool::False);
assert_eq!(a.and(c), TriBool::Unknown);
assert_eq!(c.and(c), TriBool::Unknown);

// OR 运算
assert_eq!(a.or(b), TriBool::True);
assert_eq!(a.or(c), TriBool::True);
assert_eq!(c.or(c), TriBool::Unknown);

// NOT 运算
assert_eq!(c.not(), TriBool::Unknown);

// NULL = NULL (SQL 标准: UNKNOWN, 不 MATCH)
let null: TriBool = TriBool::Unknown;
assert_eq!(null, TriBool::Unknown); // UNKNOWN
```

---

## 九、列级权限 API (v2.8.0)

```rust
use sqlrustgo_catalog::{Catalog, Privilege, ColumnPrivilege};

let mut catalog = Catalog::new();

// 授予列级 SELECT 权限
catalog.grant_column_privilege(
    "analyst_user",
    "employees",
    "salary",
    Privilege::Select,
)?;

// 授予多列权限
catalog.grant_column_privileges(
    "analyst_user",
    "employees",
    &["salary", "department", "title"],
    Privilege::Select,
)?;

// 检查列权限（执行时自动过滤未授权列）
let allowed = catalog.filter_columns("analyst_user", "employees", &["id", "name", "salary", "password"])?;
// 返回: ["id", "name", "salary"] — password 被过滤
```

---

## 十、常用命令速查

```bash
# 构建
cargo build --all-features --release

# 测试
cargo test --all-features

# 格式化
cargo fmt --all

# Clippy 检查
cargo clippy --all-features -- -D warnings

# 文档
cargo doc --all-features --no-deps

# Benchmarks
cargo bench --all-features

# 运行 REPL
cargo run --release --bin sqlrustgo
```

---

## 版本历史

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-04-30 | 1.0 | 新增 Rust API 使用示例，匹配 v2.8.0 功能集 |
