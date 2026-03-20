# Epic-02 Benchmark 可信性修复实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 建立可信 Benchmark 体系，解决 Query Cache 和数据库对比的可信性问题

**Architecture:** 
- B-04: SessionConfig 增加 benchmark_mode 标志，环境变量触发，Cache 层统一判断
- B-06: 统一 SQLite 配置模块
- B-05: 扩展 TPCH Runner 支持 PostgreSQL 对比

**Tech Stack:** Rust, rusqlite, tokio-postgres, criterion

---

## B-04: Benchmark 模式实现

### 任务 1: 创建 SessionConfig 模块

**Files:**
- Create: `crates/executor/src/session_config.rs`

**Step 1: 写入测试**

```rust
// crates/executor/tests/session_config_test.rs
#[test]
fn test_benchmark_mode_from_env() {
    std::env::set_var("SQLRUSTGO_BENCHMARK_MODE", "1");
    let config = SessionConfig::default();
    assert!(config.benchmark_mode);
    std::env::remove_var("SQLRUSTGO_BENCHMARK_MODE");
}
```

**Step 2: 运行测试确认失败**

```bash
cd /Users/liying/workspace/dev/heartopen/sqlrustgo && cargo test session_config_test
```

**Step 3: 实现代码**

```rust
// crates/executor/src/session_config.rs
use std::env;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub benchmark_mode: bool,
    pub cache_enabled: bool,
    pub stats_enabled: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        let benchmark_mode = env::var("SQLRUSTGO_BENCHMARK_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);
        
        Self {
            benchmark_mode,
            cache_enabled: !benchmark_mode,
            stats_enabled: !benchmark_mode,
        }
    }
}
```

**Step 4: 运行测试确认通过**

```bash
cargo test session_config_test
```

**Step 5: 提交**

```bash
git add crates/executor/src/session_config.rs crates/executor/tests/session_config_test.rs
git commit -m "feat: add SessionConfig with benchmark_mode support"
```

---

### 任务 2: 修改 QueryCache 集成 benchmark_mode

**Files:**
- Modify: `crates/executor/src/query_cache.rs:25-50`

**Step 1: 修改 QueryCache.get() 方法**

在 `get()` 方法开头添加：

```rust
pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult> {
    // B-04: Check benchmark_mode - skip cache entirely
    if self.config.benchmark_mode {
        return None;
    }
    // ... existing code
}
```

**Step 2: 修改 QueryCache.put() 方法**

在 `put()` 方法开头添加同样检查

**Step 3: 添加 benchmark_mode 到 QueryCacheConfig**

修改 `query_cache_config.rs`:

```rust
pub struct QueryCacheConfig {
    pub benchmark_mode: bool,  // 新增
    // ... existing fields
}
```

**Step 4: 运行现有测试确保没有破坏**

```bash
cargo test query_cache
```

**Step 5: 提交**

```bash
git add crates/executor/src/query_cache.rs crates/executor/src/query_cache_config.rs
git commit -m "feat: integrate benchmark_mode in QueryCache to disable cache"
```

---

## B-06: SQLite 统一配置

### 任务 3: 创建 SQLiteConfig 模块

**Files:**
- Create: `benches/sqlite_config.rs`

**Step 1: 写入代码**

```rust
// benches/sqlite_config.rs
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct SQLiteConfig {
    pub in_memory: bool,
    pub cache_size: i32,      // 页数
    pub page_size: u32,       // 字节
    pub wal_mode: bool,
    pub synchronous: String,   // OFF/NORMAL/FULL
}

impl Default for SQLiteConfig {
    fn default() -> Self {
        Self {
            in_memory: true,
            cache_size: 2000,      // 8MB
            page_size: 4096,
            wal_mode: false,
            synchronous: "OFF",   // 最快
        }
    }
}

impl SQLiteConfig {
    pub fn to_connection(&self) -> rusqlite::Result<Connection> {
        let conn = if self.in_memory {
            Connection::open_in_memory()
        } else {
            Connection::open("benchmark.db")
        }?;
        
        conn.execute_batch(&format!(
            "PRAGMA cache_size = {};", self.cache_size
        ))?;
        conn.execute_batch(&format!(
            "PRAGMA page_size = {};", self.page_size
        ))?;
        if self.wal_mode {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }
        conn.execute_batch(&format!(
            "PRAGMA synchronous = {};", self.synchronous
        ))?;
        
        Ok(conn)
    }
}
```

**Step 2: 验证编译**

```bash
cd /Users/liying/workspace/dev/heartopen/sqlrustgo && cargo check -p sqlrustgo
```

**Step 3: 提交**

```bash
git add benches/sqlite_config.rs
git commit -m "feat: add unified SQLiteConfig for benchmarks"
```

---

### 任务 4: 替换现有 TPCH 中的 SQLite 代码

**Files:**
- Modify: `benches/tpch_comprehensive.rs:250-280`

**Step 1: 替换 SQLite 连接代码**

将现有的 `rusqlite::Connection::open_in_memory()` 替换为使用 `SQLiteConfig`

**Step 2: 验证编译**

```bash
cargo check -p sqlrustgo
```

**Step 3: 提交**

```bash
git add benches/tpch_comprehensive.rs
git commit -m "refactor: use unified SQLiteConfig in TPCH benchmark"
```

---

## B-05: PostgreSQL 对比支持

### 任务 5: 创建 PostgreSQL 配置模块

**Files:**
- Create: `benches/postgres_config.rs`

**Step 1: 写入代码**

```rust
// benches/postgres_config.rs
use tokio_postgres::{NoTls, Client};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            dbname: "tpch".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }
}

impl PostgresConfig {
    pub async fn connect(&self) -> Result<Client, Box<dyn std::error::Error>> {
        let conn_string = format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.dbname, self.user, self.password
        );
        let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(client)
    }
}
```

**Step 2: 添加依赖到 Cargo.toml**

```toml
# benches/Cargo.toml
tokio-postgres = "0.7"
```

**Step 3: 验证编译**

```bash
cargo check -p sqlrustgo
```

**Step 4: 提交**

```bash
git add benches/postgres_config.rs benches/Cargo.toml
git commit -m "feat: add PostgresConfig for benchmark comparisons"
```

---

### 任务 6: 扩展 TPCH Runner 支持 PostgreSQL

**Files:**
- Modify: `benches/tpch_comprehensive.rs`

**Step 1: 添加 Backend 枚举**

```rust
#[derive(Debug, Clone)]
pub enum BenchmarkBackend {
    SQLRustGo,
    SQLite,
    PostgreSQL,
}
```

**Step 2: 添加 PostgreSQL 执行方法**

在 TpchBenchmark 结构中添加：

```rust
impl TpchBenchmark {
    pub async fn run_postgresql(&self, config: &PostgresConfig) -> Result<f64, Box<dyn std::error::Error>> {
        let client = config.connect().await?;
        
        // Create table if not exists
        client.execute(
            "CREATE TABLE IF NOT EXISTS lineitem (...)
             -- TPCH schema",
            &[]
        ).await?;
        
        let start = std::time::Instant::now();
        
        // Execute query
        for query in &self.queries {
            let _ = client.query(query.0, &[]).await?;
        }
        
        Ok(start.elapsed().as_millis() as f64)
    }
}
```

**Step 3: 修改 run_benchmark 方法支持选择 backend**

```rust
pub fn run_benchmark(&self, backend: BenchmarkBackend) -> BenchmarkResult {
    match backend {
        BenchmarkBackend::SQLRustGo => self.run_sqlrustgo(),
        BenchmarkBackend::SQLite => self.run_sqlite(),
        BenchmarkBackend::PostgreSQL => // 需要 async runtime
            todo!("Use tokio runtime for PostgreSQL"),
    }
}
```

**Step 4: 验证编译**

```bash
cargo check -p sqlrustgo
```

**Step 5: 提交**

```bash
git add benches/tpch_comprehensive.rs
git commit -m "feat: add PostgreSQL backend support to TPCH benchmark"
```

---

## 验证任务

### 任务 7: 完整功能测试

**Step 1: 运行所有测试**

```bash
cd /Users/liying/workspace/dev/heartopen/sqlrustgo
cargo test --workspace
```

**Step 2: 运行 benchmark 验证**

```bash
cargo run --bin tpch_benchmark
```

**Step 3: 验证 benchmark_mode 生效**

```bash
SQLRUSTGO_BENCHMARK_MODE=1 cargo run --bin tpch_benchmark
# 检查日志输出，确认 cache 被跳过
```

**Step 4: 提交**

```bash
git add .
git commit -m "test: verify Epic-02 implementations"
```

---

## 总结

| 任务 | 描述 | 提交数 |
|------|------|--------|
| 1 | SessionConfig 模块 | 1 |
| 2 | QueryCache 集成 | 1 |
| 3 | SQLiteConfig 模块 | 1 |
| 4 | TPCH 替换 SQLite | 1 |
| 5 | PostgresConfig 模块 | 1 |
| 6 | TPCH 支持 PostgreSQL | 1 |
| 7 | 完整测试验证 | 1 |

**Total: 7 commits**
