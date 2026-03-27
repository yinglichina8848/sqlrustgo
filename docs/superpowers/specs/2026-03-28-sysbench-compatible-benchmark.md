# SQLRustGo Sysbench-Compatible Benchmark 实现规范

> **Phase**: B - sysbench-compatible workload 基线
> **目标**: 50 并发 QPS ≥ 1000, P99 < 100ms
> **日期**: 2026-03-28

---

## 1. 概述

### 1.1 目标
为 SQLRustGo 实现 sysbench-compatible 性能基准测试框架，作为 v2.0 Phase 1 的基础设施，支持后续 ParallelExecutor 性能验证。

### 1.2 成功标准
- 支持 oltp_point_select、oltp_read_only、oltp_read_write 三种工作负载
- 输出标准化的 QPS、TPS、延迟百分位（P50/P95/P99）
- hdrhistogram 精度支持微秒级
- 与数据库行业标准（TiDB/CockroachDB）一致

---

## 2. 实现范围

### 2.1 必须实现的工作负载

| 工作负载 | SQL 类型 | 用途 |
|---------|---------|------|
| `oltp_point_select` | `SELECT c FROM sbtest WHERE id=?` | 主键点查延迟、索引效率 |
| `oltp_read_only` | 多表 SELECT 混合 | 全表扫描、索引、投影 |
| `oltp_read_write` | BEGIN/SELECT/UPDATE/INSERT/DELETE/COMMIT | 事务引擎、锁管理、WAL |

### 2.2 核心指标输出

```yaml
threads: 50
duration: 60s
queries: 1000000
qps: 1234
tps: 98
latency(ms):
  min: 0.1
  avg: 2.3
  p50: 1.8
  p95: 45.2
  p99: 87.6
  max: 234.5
```

---

## 3. 目录结构

```
crates/bench/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   └── lib.rs
├── workload/
│   ├── mod.rs
│   ├── oltp_point_select.rs
│   ├── oltp_read_only.rs
│   └── oltp_read_write.rs
├── runner/
│   ├── mod.rs
│   ├── concurrent_runner.rs
│   └── latency_tracker.rs
└── report/
    ├── mod.rs
    ├── text_report.rs
    └── json_report.rs
```

### 3.1 新增模块职责

| 模块 | 职责 |
|------|------|
| `workload/` | 定义三类 OLTP 工作负载的 SQL 模式 |
| `runner/` | 并发执行器 + hdrhistogram 延迟追踪 |
| `report/` | 文本和 JSON 格式输出 |

---

## 4. 详细设计

### 4.1 Workload Trait

```rust
/// Workload trait - 所有 OLTP 工作负载必须实现
pub trait Workload: Send + Sync {
    /// 工作负载名称
    fn name(&self) -> &str;

    /// 生成单次操作的 SQL 语句
    fn generate_sql(&self, rng: &mut SmallRng) -> String;

    /// 返回预期的事务语句数量（用于 TPS 计算）
    fn statements_per_tx(&self) -> usize;

    /// 是否为只读工作负载
    fn is_read_only(&self) -> bool;
}
```

### 4.2 oltp_point_select

```rust
pub struct OlTPPointSelect {
    table_name: String,
    id_range: Range<u64>,
}

impl Workload for OlTPPointSelect {
    fn name(&self) -> &str { "oltp_point_select" }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.random_range(self.id_range.clone());
        format!("SELECT c FROM {} WHERE id={}", self.table_name, id)
    }

    fn statements_per_tx(&self) -> usize { 1 }
    fn is_read_only(&self) -> bool { true }
}
```

### 4.3 oltp_read_only

```rust
/// 10 个 SELECT 混合：点查 + 范围查 + 聚合
pub struct OlTPReadOnly {
    tables: Vec<String>,
}

impl Workload for OlTPReadOnly {
    fn name(&self) -> &str { "oltp_read_only" }

    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        // 随机选择一种查询模式
        match rng.random_u32() % 4 {
            0 => format!("SELECT * FROM {} WHERE id={}", table, id),  // Point select
            1 => format!("SELECT * FROM {} WHERE id BETWEEN {} AND {}", table, l, r), // Range
            2 => format!("SELECT COUNT(*) FROM {}", table), // Aggregation
            _ => format!("SELECT c FROM {} ORDER BY id LIMIT 10", table), // Sort + limit
        }
    }

    fn statements_per_tx(&self) -> usize { 10 }
    fn is_read_only(&self) -> bool { true }
}
```

### 4.4 oltp_read_write

```rust
/// 完整事务：BEGIN → SELECT → UPDATE → INSERT → DELETE → COMMIT
pub struct OlTPReadWrite {
    tables: Vec<String>,
}

impl Workload for OlTPReadWrite {
    fn name(&self) -> &str { "oltp_read_write" }

    fn generate_transaction_sql(&self, rng: &mut SmallRng) -> Vec<String> {
        vec![
            "BEGIN".to_string(),
            format!("SELECT c FROM {} WHERE id={}", table, id),
            format!("UPDATE {} SET c='updated' WHERE id={}", table, id),
            format!("INSERT INTO {} (id, c) VALUES ({}, 'new')", table, new_id),
            "COMMIT".to_string(),
        ]
    }

    fn statements_per_tx(&self) -> usize { 5 }
    fn is_read_only(&self) -> bool { false }
}
```

### 4.5 ConcurrentRunner

```rust
pub struct ConcurrentRunner {
    num_threads: usize,
    duration_secs: u64,
    workload: Arc<dyn Workload>,
    connection_factory: Arc<dyn Fn() -> SqlResult<Connection>>,
}

impl ConcurrentRunner {
    pub fn run(&self) -> BenchmarkResult {
        // 1. 创建线程池
        // 2. 每个线程独立连接 + SmallRng
        // 3. 并发执行 + hdrhistogram 记录延迟
        // 4. 汇总结果
    }
}
```

### 4.6 LatencyTracker (hdrhistogram)

使用 `hdrhistogram` crate：

```rust
use hdrhistogram::{Histogram, LatencyHistogram};

pub struct LatencyTracker {
    histogram: Histogram<u64>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        // 1 微秒最小精度，最大 1 秒
        Self {
            histogram: Histogram::new(1, 1_000_000_000, 3).unwrap(),
        }
    }

    pub fn record(&self, latency_ns: u64) {
        self.histogram.record(latency_ns).unwrap();
    }

    pub fn percentiles(&self) -> Percentiles {
        Percentiles {
            p50: self.histogram.value_at_quantile(0.50),
            p95: self.histogram.value_at_quantile(0.95),
            p99: self.histogram.value_at_quantile(0.99),
        }
    }
}
```

---

## 5. 测试验证

### 5.1 单元测试
- `test_oltp_point_select_sql_generation` - 验证 SQL 生成正确性
- `test_oltp_read_only_transaction` - 验证只读事务
- `test_oltp_read_write_transaction` - 验证读写事务

### 5.2 集成测试
- `test_point_select_qps` - 验证 QPS ≥ 1000
- `test_read_write_latency` - 验证 P99 < 100ms
- `test_concurrent_stability` - 50 并发稳定性

---

## 6. CLI 接口

```bash
# 运行 point select 测试
cargo run --release --bin bench -- \
    --workload oltp_point_select \
    --threads 50 \
    --duration 60 \
    --table sbtest \
    --output json

# 运行 read write 测试
cargo run --release --bin bench -- \
    --workload oltp_read_write \
    --threads 50 \
    --duration 60 \
    --table sbtest \
    --output text
```

---

## 7. 输出格式

### 7.1 JSON 输出

```json
{
  "workload": "oltp_point_select",
  "threads": 50,
  "duration_sec": 60,
  "total_queries": 62345,
  "qps": 1039,
  "latency_ns": {
    "min": 15230,
    "avg": 482300,
    "p50": 342100,
    "p95": 8923400,
    "p99": 87654300,
    "max": 234567000
  }
}
```

### 7.2 文本输出

```
SQLRustGo Benchmark: oltp_point_select
======================================
Threads:             50
Duration:            60s
Total queries:       62345
QPS:                 1039

Latency percentiles (ms):
  min     avg     p50     p95     p99     max
  0.02    0.48    0.34    8.92    87.65   234.57
```

---

## 8. 后续阶段

### Phase A（下一步）
- 24h 连续运行支持
- 进度报告
- 稳定性验证

### Phase C（最后）
- PostgreSQL 对比运行器
- SQLite 对比运行器

---

## 9. 依赖变更

```toml
# crates/bench/Cargo.toml
[dependencies]
# 新增
hdrhistogram = "7.5"
rand = { version = "0.8", features = ["small_rng"] }

# 已有
tokio = { version = "1", features = ["full"] }
sqlrustgo = { path = "../sql" }
```

---

*本文档为实现规范，批准后开始实现。*
