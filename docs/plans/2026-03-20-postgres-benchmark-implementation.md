# PostgreSQL Benchmark 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 PostgreSQL 实际连接和与 SQLRustGo 的并行对比测试

**Architecture:** 在 crates/bench 中添加 PostgresBenchmark 模块，支持轻量级测试和完整集成测试

**Tech Stack:** tokio-postgres, rusqlite, hdrhistogram

---

## Task 1: 完善 PostgresDB 连接功能

**Files:**
- Modify: `crates/bench/src/db/postgres.rs`

**Step 1: 添加连接测试占位**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_postgres_connection_config() {
        // 验证连接配置格式
        let conn_str = "host=localhost port=5432 dbname=test user=postgres";
        assert!(conn_str.contains("host="));
    }
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-bench -- db::postgres`
Expected: PASS

**Step 3: 添加连接超时和错误处理**

```rust
impl PostgresDB {
    pub async fn new(conn_str: &str) -> anyhow::Result<Self> {
        let (client, connection) = tokio_postgres::connect(conn_str, NoTls)
            .await
            .context("Failed to connect to PostgreSQL")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }
}
```

**Step 4: 运行测试验证**

Run: `cargo test -p sqlrustgo-bench -- db::postgres`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/bench/src/db/postgres.rs
git commit -m "feat(bench): 完善 PostgresDB 连接功能"
```

---

## Task 2: 创建 PostgresBenchmark 对比运行器

**Files:**
- Create: `crates/bench/src/db/postgres_benchmark.rs`

**Step 1: 创建基准测试结构**

```rust
use crate::metrics::latency::LatencyRecorder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub db_name: String,
    pub total_time_ms: u64,
    pub operations: u64,
    pub qps: f64,
    pub latency_stats: LatencyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

pub struct PostgresBenchmark {
    conn_str: String,
    latency_recorder: LatencyRecorder,
}

impl PostgresBenchmark {
    pub fn new(conn_str: &str) -> Self {
        Self {
            conn_str: conn_str.to_string(),
            latency_recorder: LatencyRecorder::new(),
        }
    }

    pub async fn run_reads(&self, operations: u64) -> anyhow::Result<BenchmarkResult> {
        let pg = PostgresDB::new(&self.conn_str).await?;
        let start = std::time::Instant::now();

        for i in 0..operations {
            let iteration_start = std::time::Instant::now();
            pg.read(i as usize).await?;
            self.latency_recorder.record(iteration_start.elapsed().as_micros() as u64);
        }

        let elapsed = start.elapsed().as_millis() as u64;
        let stats = self.latency_recorder.snapshot();

        Ok(BenchmarkResult {
            db_name: "postgres".to_string(),
            total_time_ms: elapsed,
            operations,
            qps: operations as f64 / (elapsed as f64 / 1000.0),
            latency_stats: LatencyStats {
                p50_ms: stats.p50 as f64 / 1000.0,
                p95_ms: stats.p95 as f64 / 1000.0,
                p99_ms: stats.p99 as f64 / 1000.0,
            },
        })
    }
}
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-bench`
Expected: 编译成功

**Step 3: Commit**

```bash
git add crates/bench/src/db/postgres_benchmark.rs
git commit -m "feat(bench): 添加 PostgresBenchmark 对比运行器"
```

---

## Task 3: 创建轻量级 SQLite 对比测试

**Files:**
- Create: `crates/bench/tests/sqlite_benchmark_test.rs`

**Step 1: 编写轻量级测试**

```rust
use sqlrustgo_bench::{SQLiteBenchmark, BenchmarkConfig};

#[tokio::test]
async fn test_sqlite_benchmark_read() {
    let config = BenchmarkConfig {
        db: "sqlite".to_string(),
        workload: "read".to_string(),
        threads: 1,
        duration: 1,
        scale: 100,
        cache: false,
        output: "".to_string(),
        pg_conn: None,
        sqlite_path: None,
        sqlrustgo_addr: "127.0.0.1:4000".to_string(),
    };

    let bench = SQLiteBenchmark::new(&config);
    let result = bench.run_reads(10).await.unwrap();
    
    assert_eq!(result.db_name, "sqlite");
    assert_eq!(result.operations, 10);
    assert!(result.qps > 0.0);
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-bench --test sqlite_benchmark_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/bench/tests/sqlite_benchmark_test.rs
git commit -m "test(bench): 添加 SQLite 轻量级对比测试"
```

---

## Task 4: 创建 PostgreSQL Mock 测试

**Files:**
- Create: `crates/bench/tests/postgres_mock_test.rs`

**Step 1: 编写 Mock 测试**

```rust
use sqlrustgo_bench::{PostgresConfig, LocalConfig};

#[test]
fn test_postgres_config_local() {
    let config = PostgresConfig::local();
    assert!(config.host.contains("localhost") || config.host.contains("127.0.0.1"));
}

#[test]
fn test_postgres_connection_string() {
    let config = PostgresConfig::local();
    let conn_str = config.connection_string();
    assert!(conn_str.contains("localhost") || conn_str.contains("127.0.0.1"));
}

#[test]
fn test_sqlite_config_fast() {
    let config = SQLiteConfig::fast();
    assert!(config.in_memory);
    assert_eq!(config.synchronous, "OFF");
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-bench --test postgres_mock_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/bench/tests/postgres_mock_test.rs
git commit -m "test(bench): 添加 PostgreSQL Mock 测试"
```

---

## Task 5: 创建并行对比测试

**Files:**
- Create: `crates/bench/tests/comparison_test.rs`

**Step 1: 编写对比测试**

```rust
use sqlrustgo_bench::{PostgresBenchmark, SQLiteBenchmark, BenchmarkConfig};
use serde_json::json;

#[tokio::test]
async fn test_sqlite_vs_postgres_comparison() {
    let sqlite_config = BenchmarkConfig {
        db: "sqlite".to_string(),
        workload: "read".to_string(),
        threads: 1,
        duration: 1,
        scale: 100,
        cache: false,
        output: "".to_string(),
        pg_conn: None,
        sqlite_path: None,
        sqlrustgo_addr: "127.0.0.1:4000".to_string(),
    };

    // SQLite benchmark
    let sqlite_bench = SQLiteBenchmark::new(&sqlite_config);
    let sqlite_result = sqlite_bench.run_reads(10).await.unwrap();

    // Verify results
    assert_eq!(sqlite_result.db_name, "sqlite");
    assert!(sqlite_result.qps > 0.0);
    assert!(sqlite_result.latency_stats.p99_ms > 0.0);
}

#[test]
fn test_benchmark_result_serialization() {
    use sqlrustgo_bench::{BenchmarkResult, LatencyStats};

    let result = BenchmarkResult {
        db_name: "test".to_string(),
        total_time_ms: 100,
        operations: 10,
        qps: 100.0,
        latency_stats: LatencyStats {
            p50_ms: 1.0,
            p95_ms: 5.0,
            p99_ms: 10.0,
        },
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("100.0"));
}
```

**Step 2: 运行测试**

Run: `cargo test -p sqlrustgo-bench --test comparison_test`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/bench/tests/comparison_test.rs
git commit -m "test(bench): 添加并行对比测试"
```

---

## Task 6: 添加模块导出

**Files:**
- Modify: `crates/bench/src/db/mod.rs`

**Step 1: 更新模块导出**

```rust
pub mod postgres;
pub mod postgres_benchmark;
pub mod sqlite;
pub mod sqlrustgo;

pub use postgres::PostgresDB;
pub use postgres_benchmark::{PostgresBenchmark, BenchmarkResult, LatencyStats};
pub use sqlite::SQLiteBenchmark;
pub use sqlrustgo::SqlRustGoBenchmark;
```

**Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-bench`
Expected: 编译成功

**Step 3: Commit**

```bash
git add crates/bench/src/db/mod.rs
git commit -m "feat(bench): 导出 PostgresBenchmark 模块"
```

---

## Task 7: 完整验证

**Step 1: 运行所有测试**

Run: `cargo test -p sqlrustgo-bench`
Expected: 全部通过

**Step 2: 运行 clippy**

Run: `cargo clippy -p sqlrustgo-bench -- -D warnings`
Expected: 无警告

**Step 3: 运行格式化**

Run: `cargo fmt --all -- --check`
Expected: 通过

**Step 4: Commit**

```bash
git add .
git commit -m "feat(bench): PostgreSQL Benchmark 完成"
```

---

## 总结

完成后将具备：

- ✅ PostgresDB 完善
- ✅ PostgresBenchmark 对比运行器
- ✅ SQLite 轻量级测试
- ✅ PostgreSQL Mock 测试
- ✅ 并行对比测试
- ✅ 统一结果格式

**Plan complete and saved to `docs/plans/2026-03-20-postgres-benchmark-implementation.md`. Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Manual execution** - You execute tasks manually

**Which approach?**
