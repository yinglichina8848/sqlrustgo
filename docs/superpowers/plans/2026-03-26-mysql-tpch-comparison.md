# MySQL TPC-H 对比测试实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 SQLRustGo 添加 MySQL TPC-H 性能对比测试，完善 benchmark 体系，支持 SQLRustGo vs MySQL vs PostgreSQL vs SQLite 横向对比。

**Architecture:**
- 创建 `benches/mysql_config.rs` MySQL 配置模块（参考 `postgres_config.rs` 和 `sqlite_config.rs`）
- 在 `crates/bench/examples/tpch_compare.rs` 添加 MySQL 支持
- 在 `benches/tpch_comprehensive.rs` 添加 MySQL benchmark runner
- 添加 MySQL 专用的 TPC-H 查询适配器（处理语法差异）

**Tech Stack:** mysql crate, rusqlite, tokio-postgres, criterion

---

## 文件结构

```
benches/
├── mysql_config.rs        # 新建: MySQL 配置模块
├── postgres_config.rs     # 已存在
├── sqlite_config.rs       # 已存在
└── ...

crates/bench/
├── Cargo.toml             # 修改: 添加 mysql crate
└── examples/
    └── tpch_compare.rs    # 修改: 添加 MySQL benchmark
```

---

## Task 1: 创建 MySQL 配置文件

**Files:**
- Create: `benches/mysql_config.rs`
- Test: `benches/mysql_config.rs` (embedded tests)

- [ ] **Step 1: 创建 MySQL 配置模块**

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MySqlConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub connection_timeout: Duration,
}

impl Default for MySqlConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            dbname: "tpch".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
            connection_timeout: Duration::from_secs(30),
        }
    }
}

impl MySqlConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        )
    }

    pub fn local() -> Self {
        Self::default()
    }

    pub fn docker() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MySqlConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
    }

    #[test]
    fn test_connection_string() {
        let config = MySqlConfig::default();
        let conn_str = config.connection_string();
        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("localhost"));
    }
}
```

- [ ] **Step 2: 运行测试验证**

Run: `cargo test --package sqlrustgo-bench mysql_config -- --nocapture`
Expected: PASS (2 tests)

- [ ] **Step 3: Commit**

```bash
git add benches/mysql_config.rs
git commit -m "feat(bench): add MySQL configuration module"
```

---

## Task 2: 添加 mysql crate 依赖

**Files:**
- Modify: `crates/bench/Cargo.toml`

- [ ] **Step 1: 添加 mysql crate 依赖**

在 `crates/bench/Cargo.toml` 的 `[dependencies]` section 添加:

```toml
mysql = "25"
```

- [ ] **Step 2: 验证编译**

Run: `cargo check --package sqlrustgo-bench`
Expected: 成功编译无错误

- [ ] **Step 3: Commit**

```bash
git add crates/bench/Cargo.toml
git commit -m "chore(bench): add mysql crate dependency"
```

---

## Task 3: 在 tpch_compare.rs 添加 MySQL benchmark runner

**Files:**
- Modify: `crates/bench/examples/tpch_compare.rs:296-321`
- Modify: `crates/bench/examples/tpch_compare.rs:367-396`

- [ ] **Step 1: 添加 run_mysql_benchmarks 函数**

在 `fn run_sqlite_benchmarks()` 后添加:

```rust
fn run_mysql_benchmarks() -> Option<SystemResult> {
    let config = MySqlConfig::docker();
    let conn_str = config.connection_string();

    // Try to connect, return None if MySQL not available
    let pool = match mysql::Pool::new(conn_str.as_str()) {
        Ok(p) => p,
        Err(e) => {
            println!("  [Note] MySQL not available: {}", e);
            return None;
        }
    };

    let queries = vec![
        ("Q1", "SELECT l_returnflag, SUM(l_quantity) as sum_qty FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag"),
        ("Q3", "SELECT o_orderkey, o_orderdate, o_totalprice FROM orders WHERE o_orderdate > 88000"),
        ("Q6", "SELECT SUM(l_extendedprice * (1 - l_discount)) as revenue FROM lineitem WHERE l_quantity > 20"),
    ];

    let mut system_result = SystemResult::new("MySQL".to_string());

    for (name, sql) in queries {
        let mut latencies = LatencyStats::new();

        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let _ = pool.query(sql);
            latencies.record(start.elapsed().as_micros() as u64);
        }

        system_result.queries.push(QueryResult {
            name: name.to_string(),
            avg_latency_ms: latencies.avg() / 1000.0,
            p50_ms: latencies.p50() / 1000,
            p95_ms: latencies.p95() / 1000,
            p99_ms: latencies.p99() / 1000,
            min_ms: latencies.min().map(|v| v / 1000),
            max_ms: latencies.max().map(|v| v / 1000),
            iterations: ITERATIONS,
        });
    }

    Some(system_result)
}
```

- [ ] **Step 2: 在 main() 中调用 run_mysql_benchmarks**

在 `fn main()` 的 "// Run SQLite benchmarks" 后添加:

```rust
// Run MySQL benchmarks (if available)
println!("Running MySQL benchmarks...");
result.mysql = run_mysql_benchmarks();
```

在 `struct ComparisonResult` 添加:

```rust
pub mysql: Option<SystemResult>,
```

- [ ] **Step 3: 在 print_summary 中添加 MySQL 输出**

在 `fn print_summary()` 中添加:

```rust
if let Some(ref mysql) = result.mysql {
    self.print_system_results("MySQL", mysql);
}
```

- [ ] **Step 4: 验证编译**

Run: `cargo check --example tpch_compare --package sqlrustgo-bench`
Expected: 成功编译无错误

- [ ] **Step 5: Commit**

```bash
git add crates/bench/examples/tpch_compare.rs
git commit -m "feat(bench): add MySQL benchmark runner to tpch_compare"
```

---

## Task 4: 在 tpch_comprehensive.rs 添加 MySQL 支持

**Files:**
- Modify: `benches/tpch_comprehensive.rs`

- [ ] **Step 1: 添加 MySQL 依赖导入**

在文件顶部添加:

```rust
use mysql::Pool;
```

- [ ] **Step 2: 添加 MySQL benchmark 方法**

在 `TpchBenchmark` impl 块中添加:

```rust
fn run_mysql_benchmark(&self, queries: &[(&str, &str)]) -> f64 {
    let config = MySqlConfig::docker();
    let pool = match Pool::new(config.connection_string().as_str()) {
        Ok(p) => p,
        Err(_) => return 0.0, // MySQL not available
    };

    let mut total_ms = 0.0;
    for (_, sql) in queries {
        let start = Instant::now();
        let _ = pool.query(sql);
        total_ms += start.elapsed().as_secs_f64() * 1000.0;
    }
    total_ms
}
```

- [ ] **Step 3: 修改 run() 方法以包含 MySQL**

在 `run()` 方法中添加 MySQL 执行:

```rust
let mysql_ms = self.run_mysql_benchmark(&queries);
let avg_mysql_per_query = mysql_ms / queries.len() as f64;
```

- [ ] **Step 4: 验证编译**

Run: `cargo check --package sqlrustgo-bench`
Expected: 成功编译无错误

- [ ] **Step 5: Commit**

```bash
git add benches/tpch_comprehensive.rs
git commit -m "feat(bench): add MySQL support to tpch_comprehensive"
```

---

## Task 5: 创建集成测试验证 MySQL TPC-H

**Files:**
- Create: `tests/integration/mysql_tpch_test.rs`

- [ ] **Step 1: 创建集成测试**

```rust
//! MySQL TPC-H Integration Tests
//!
//! Tests that TPC-H queries execute correctly on MySQL.

#[cfg(test)]
mod tests {
    use mysql::Pool;

    fn get_mysql_pool() -> Option<Pool> {
        let config = mysql::Opts::new()
            .set_user(Some("root"))
            .set_pass(Some(""))
            .set_ip_or_hostname(Some("127.0.0.1"))
            .set_tcp_port(3306);
        Pool::new(config).ok()
    }

    #[test]
    fn test_mysql_connection() {
        if let Some(pool) = get_mysql_pool() {
            let result = pool.query("SELECT 1");
            assert!(result.is_ok());
        } else {
            println!("MySQL not available, skipping test");
        }
    }

    #[test]
    fn test_mysql_tpch_q1() {
        let pool = match get_mysql_pool() {
            Some(p) => p,
            None => return,
        };

        // Create test table
        pool.query("CREATE TABLE IF NOT EXISTS test_q1 (
            id INT,
            quantity INT,
            price DECIMAL(10,2)
        )").ok();

        // Insert test data
        pool.query("INSERT INTO test_q1 VALUES (1, 10, 100.00)").ok();

        // Execute Q1-like aggregation
        let result = pool.query(
            "SELECT SUM(quantity) as total_qty FROM test_q1"
        );

        assert!(result.is_ok());

        // Cleanup
        pool.query("DROP TABLE test_q1").ok();
    }
}
```

- [ ] **Step 2: 添加测试到 Cargo.toml**

在 `[dev-dependencies]` 添加:

```toml
mysql = "25"
```

- [ ] **Step 3: 运行测试**

Run: `cargo test --test mysql_tpch_test -- --nocapture`
Expected: PASS (tests pass or skip if MySQL unavailable)

- [ ] **Step 4: Commit**

```bash
git add tests/integration/mysql_tpch_test.rs
git add Cargo.toml  # if modified
git commit -m "test: add MySQL TPC-H integration tests"
```

---

## Task 6: 更新文档

**Files:**
- Modify: `docs/benchmark/benchmark_report.md`

- [ ] **Step 1: 更新 benchmark 文档**

在文档中添加 MySQL 基准测试结果占位符:

```markdown
### 7. TPC-H Multi-Database Comparison

| System | Q1 (ms) | Q3 (ms) | Q6 (ms) | Q10 (ms) |
|--------|---------|---------|---------|----------|
| SQLRustGo | TBD | TBD | TBD | TBD |
| MySQL | TBD | TBD | TBD | TBD |
| PostgreSQL | TBD | TBD | TBD | TBD |
| SQLite | TBD | TBD | TBD | TBD |

## Running MySQL Benchmarks

```bash
# Start MySQL container
docker run --name mysql-tpch -e MYSQL_ROOT_PASSWORD= -p 3306:3306 -d mysql:8

# Run TPC-H comparison
cargo run --example tpch_compare --package sqlrustgo-bench
```
```

- [ ] **Step 2: Commit**

```bash
git add docs/benchmark/benchmark_report.md
git commit -m "docs: update benchmark report with MySQL comparison section"
```

---

## 验证清单

- [ ] `benches/mysql_config.rs` 创建完成
- [ ] MySQL crate 依赖添加到 `crates/bench/Cargo.toml`
- [ ] `tpch_compare.rs` 支持 MySQL benchmark
- [ ] `tpch_comprehensive.rs` 支持 MySQL benchmark
- [ ] 集成测试 `tests/integration/mysql_tpch_test.rs` 创建
- [ ] 文档更新完成
- [ ] `cargo test --all-features` 通过
- [ ] `cargo clippy --all-features -- -D warnings` 无错误

---

## Issue 关联

- GitHub Issue: #836
- 父 Issue: #790 (v1.9.0 开发总控)
