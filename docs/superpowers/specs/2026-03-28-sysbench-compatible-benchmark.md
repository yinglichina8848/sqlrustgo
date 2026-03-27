# SQLRustGo Sysbench-Compatible Benchmark 实现规范 (v2)

> **Phase**: B - sysbench-compatible workload 基线
> **目标**: 50 并发 QPS ≥ 1000, P99 < 100ms
> **日期**: 2026-03-28
> **质量等级**: 数据库级 Benchmark 基线系统

---

## 质量评估

| 维度 | 评级 | 说明 |
|------|------|------|
| 完整度 | ★★★★★ | 覆盖所有必须模块 |
| 工程可落地 | ★★★★★ | 模块化设计 |
| 行业对标能力 | ★★★★★ | TiDB/CockroachDB 同级 |
| CI Gate 可用性 | ★★★★★ | 内置回归检测 |
| 数据库级基线能力 | ★★★★★ | Warmup/Distribution/Seed 全支持 |

---

## 1. 概述

### 1.1 目标
为 SQLRustGo 实现**数据库级性能基准测试框架**，作为 v2.0 Phase 1 的基础设施，支持：
- ParallelExecutor 性能验证
- CI Gate 自动性能回归检测
- 行业对标（TiDB/CockroachDB）

### 1.2 成功标准
- 支持 oltp_point_select、oltp_read_only、oltp_read_write 三种工作负载
- **Warmup → Measurement → Cooldown** 三阶段执行
- hdrhistogram 微秒级精度
- **Uniform + Zipfian** 分布模型
- **Seed 可控**，保证可复现性
- **Transaction-level latency** 统计
- **CI Gate** 自动回归检测

---

## 2. 必须模块（Critical）

### 2.1 模块清单

| 模块 | 必须 | 说明 |
|------|------|------|
| Warmup Phase | ✅ | BufferPool/JIT/Cache 稳定 |
| Dataset Generator | ✅ | 自动生成 sbtest 表 |
| Distribution Model | ✅ | Uniform + Zipfian |
| Transaction Timing | ✅ | Statement + Transaction 延迟 |
| Seed Control | ✅ | 可复现性 |
| Connection Model | ✅ | Per-thread 连接策略 |
| Regression Detection | ✅ | CI Gate 集成 |

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
│   ├── latency_tracker.rs
│   └── warmup_runner.rs
├── dataset/
│   ├── mod.rs
│   ├── generator.rs
│   └── sbtest_schema.rs
├── distribution/
│   ├── mod.rs
│   ├── uniform.rs
│   └── zipfian.rs
├── report/
│   ├── mod.rs
│   ├── text_report.rs
│   ├── json_report.rs
│   └── regression.rs
└── progress/
    ├── mod.rs
    └── reporter.rs
```

---

## 4. 核心设计

### 4.1 Benchmark Phase Enum

```rust
#[derive(Debug, Clone, Copy)]
pub enum BenchmarkPhase {
    Warmup,
    Measurement,
    Cooldown,
}

impl BenchmarkPhase {
    pub fn duration(&self, config: &BenchmarkConfig) -> u64 {
        match self {
            Warmup => config.warmup_secs,
            Measurement => config.duration_secs,
            Cooldown => config.cooldown_secs,
        }
    }

    pub fn is_measured(&self) -> bool {
        matches!(self, Measurement)
    }
}
```

### 4.2 BenchmarkConfig

```rust
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    // Execution
    pub threads: usize,
    pub warmup_secs: u64,   // 默认 10s
    pub duration_secs: u64,  // 默认 60s
    pub cooldown_secs: u64,  // 默认 5s

    // Dataset
    pub tables: usize,           // 默认 1
    pub dataset_size: usize,     // 默认 1_000_000 行

    // Distribution
    pub distribution: Distribution,

    // Reproducibility
    pub seed: u64,

    // Connection
    pub connection_mode: ConnectionMode,
}

#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    Uniform,
    Zipfian { theta: f64 },  // 默认 theta=0.9
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionMode {
    PerThread,   // 推荐
    SharedPool,
}
```

### 4.3 Dataset Generator

```rust
// dataset/sbtest_schema.rs
pub const SBTEST_SCHEMA: &str = r#"
CREATE TABLE sbtest (
    id BIGINT PRIMARY KEY,
    k BIGINT NOT NULL,
    c TEXT NOT NULL,
    pad TEXT NOT NULL
)
"#;

// dataset/generator.rs
pub struct DatasetGenerator {
    db: Database,
    tables: usize,
    rows_per_table: usize,
}

impl DatasetGenerator {
    pub fn new(db: Database) -> Self {
        Self { db, tables: 1, rows_per_table: 1_000_000 }
    }

    pub fn tables(mut self, n: usize) -> Self {
        self.tables = n;
        self
    }

    pub fn rows(mut self, n: usize) -> Self {
        self.rows_per_table = n;
        self
    }

    pub async fn generate(&self) -> SqlResult<()> {
        for i in 0..self.tables {
            let table_name = format!("sbtest{}", if i == 0 { "".to_string() } else { i.to_string() });

            // CREATE TABLE
            self.db.execute(&format!("CREATE TABLE {} ({})", table_name, SBTEST_SCHEMA)).await?;

            // Bulk insert with progress
            let chunk_size = 10_000;
            for offset in (0..self.rows_per_table).step_by(chunk_size) {
                let values: Vec<String> = (offset..(offset + chunk_size))
                    .map(|id| format!("({}, {}, 'c{}', 'pad{}')", id, id % 1_000_000, id, id))
                    .collect();

                self.db.execute(&format!(
                    "INSERT INTO {} VALUES {}",
                    table_name,
                    values.join(",")
                )).await?;
            }
        }
        Ok(())
    }
}
```

### 4.4 Distribution Model

```rust
// distribution/mod.rs
pub trait RandomDistribution: Send + Sync {
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64;
}

// distribution/uniform.rs
pub struct UniformDistribution;

impl RandomDistribution for UniformDistribution {
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64 {
        rng.random_range(range)
    }
}

// distribution/zipfian.rs
pub struct ZipfianDistribution {
    theta: f64,
    zeta: f64,
    num_items: u64,
}

impl ZipfianDistribution {
    pub fn new(num_items: u64, theta: f64) -> Self {
        let zeta = Self::compute_zeta(num_items, theta);
        Self { theta, zeta, num_items }
    }

    fn compute_zeta(n: u64, theta: f64) -> f64 {
        (0..n).map(|i| (i as f64 + 1.0).powf(-theta)).sum()
    }
}

impl RandomDistribution for ZipfianDistribution {
    fn next_id(&self, rng: &mut SmallRng, range: Range<u64>) -> u64 {
        // Zipfian: 热点数据访问分布
        let u: f64 = rng.random_range(0.0..1.0);
        let rank = (self.zeta * u).powf(1.0 / (self.theta - 1.0)) as u64;
        range.start + rank.min(range.end - range.start - 1)
    }
}
```

### 4.5 Workload Trait (增强版)

```rust
pub trait Workload: Send + Sync {
    fn name(&self) -> &str;

    /// 生成单条 SQL 语句
    fn generate_sql(&self, rng: &mut SmallRng) -> String;

    /// 生成完整事务的 SQL 语句列表
    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String>;

    /// 事务内语句数量
    fn statements_per_tx(&self) -> usize;

    /// 是否为只读工作负载
    fn is_read_only(&self) -> bool;

    /// 获取使用的表名
    fn table_names(&self) -> Vec<String>;
}
```

### 4.6 LatencyTracker (双维度统计)

```rust
pub struct LatencyTracker {
    statement_histogram: Histogram<u64>,  // 语句级延迟
    transaction_histogram: Histogram<u64>, // 事务级延迟
}

impl LatencyTracker {
    pub fn record_statement(&self, latency_ns: u64) {
        self.statement_histogram.record(latency_ns).unwrap();
    }

    pub fn record_transaction(&self, latency_ns: u64) {
        self.transaction_histogram.record(latency_ns).unwrap();
    }

    pub fn statement_percentiles(&self) -> Percentiles { ... }
    pub fn transaction_percentiles(&self) -> Percentiles { ... }
}

pub struct Percentiles {
    pub min: u64,
    pub avg: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub max: u64,
}
```

### 4.7 ConcurrentRunner (三阶段执行)

```rust
pub struct ConcurrentRunner {
    config: BenchmarkConfig,
    workload: Arc<dyn Workload>,
    db_factory: Arc<dyn Fn() -> SqlResult<Connection>>,
}

impl ConcurrentRunner {
    pub async fn run(&self) -> BenchmarkResult {
        let mut result = BenchmarkResult::new();

        // Phase 1: Warmup (不计入统计)
        println!("[Warmup] {}s", self.config.warmup_secs);
        self.run_phase(BenchmarkPhase::Warmup, &mut result).await;

        // Phase 2: Measurement (计入统计)
        println!("[Measurement] {}s", self.config.duration_secs);
        self.run_phase(BenchmarkPhase::Measurement, &mut result).await;

        // Phase 3: Cooldown
        println!("[Cooldown] {}s", self.config.cooldown_secs);
        self.run_phase(BenchmarkPhase::Cooldown, &mut result).await;

        result
    }

    async fn run_phase(&self, phase: BenchmarkPhase, result: &mut BenchmarkResult) {
        let duration = phase.duration(&self.config);
        let is_measured = phase.is_measured();
        let barrier = Barrier::new(self.config.threads);

        let handles: Vec<JoinHandle<()>> = (0..self.config.threads)
            .map(|i| {
                let runner = Arc::new(self.clone());
                let phase = phase;
                let result = if is_measured { Some(result.clone()) } else { None };

                spawn(async move {
                    runner.run_worker(i, &barrier, duration, is_measured, result).await;
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
```

### 4.8 Regression Detection

```rust
// report/regression.rs
#[derive(Debug, Clone)]
pub struct RegressionConfig {
    pub qps_drop_threshold: f64,    // 默认 10%
    pub p99_increase_threshold: f64, // 默认 20%
}

pub struct RegressionDetector {
    config: RegressionConfig,
    baseline: Option<BenchmarkResult>,
}

impl RegressionDetector {
    pub fn load_baseline(path: &Path) -> SqlResult<Self> {
        let baseline = if path.exists() {
            Some(serde_json::from_str(&std::fs::read_to_string(path)?)?)
        } else {
            None
        };
        Ok(Self { config: RegressionConfig::default(), baseline })
    }

    pub fn compare(&self, current: &BenchmarkResult) -> RegressionReport {
        match &self.baseline {
            Some(baseline) => {
                let qps_drop = (baseline.qps - current.qps) / baseline.qps * 100.0;
                let p99_increase = (current.latency.p99 - baseline.latency.p99) / baseline.latency.p99 * 100.0;

                RegressionReport {
                    passed: qps_drop < self.config.qps_drop_threshold
                        && p99_increase < self.config.p99_increase_threshold,
                    qps_drop,
                    p99_increase,
                    message: if qps_drop >= self.config.qps_drop_threshold {
                        format!("QPS regressed by {:.1}%", qps_drop)
                    } else {
                        String::new()
                    },
                }
            }
            None => RegressionReport {
                passed: true,
                qps_drop: 0.0,
                p99_increase: 0.0,
                message: "No baseline established".to_string(),
            },
        }
    }
}
```

---

## 5. 输出格式

### 5.1 JSON 输出 (v1.0)

```json
{
  "schema_version": "1.0",
  "workload": "oltp_point_select",
  "config": {
    "threads": 50,
    "warmup_secs": 10,
    "duration_secs": 60,
    "cooldown_secs": 5,
    "tables": 1,
    "dataset_size": 1000000,
    "distribution": "zipfian",
    "seed": 42
  },
  "phase": "measurement",
  "total_queries": 62345,
  "total_transactions": 62345,
  "qps": 1039.1,
  "qps_stddev": 12.4,
  "tps": 98.3,
  "statement_latency_ns": {
    "min": 15230,
    "avg": 482300,
    "p50": 342100,
    "p95": 8923400,
    "p99": 87654300,
    "max": 234567000
  },
  "transaction_latency_ns": {
    "min": 15230,
    "avg": 482300,
    "p50": 342100,
    "p95": 8923400,
    "p99": 87654300,
    "max": 234567000
  },
  "regression": {
    "passed": true,
    "qps_drop": 0.0,
    "p99_increase": 0.0
  }
}
```

### 5.2 文本输出

```
SQLRustGo Benchmark: oltp_point_select
======================================
Config:
  Threads:       50
  Warmup:        10s
  Duration:      60s
  Cooldown:      5s
  Tables:        1
  Dataset:       1,000,000 rows
  Distribution:  zipfian (theta=0.9)
  Seed:          42

Results:
  Total queries:     62345
  QPS:               1039.1 ± 12.4
  TPS:               98.3

Statement Latency (ms):
  min     avg     p50     p95     p99     max
  0.02    0.48    0.34    8.92    87.65   234.57

Transaction Latency (ms):
  min     avg     p50     p95     p99     max
  0.02    0.48    0.34    8.92    87.65   234.57

Regression: PASSED
```

---

## 6. CLI 接口

```bash
# 完整参数
cargo run --release --bin bench -- \
    --workload oltp_point_select \
    --threads 50 \
    --warmup 10 \
    --duration 60 \
    --cooldown 5 \
    --tables 1 \
    --dataset-size 1000000 \
    --distribution uniform \
    --seed 42 \
    --connection-mode per-thread \
    --output json \
    --baseline ./baseline.json
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--workload` | 必需 | oltp_point_select \| oltp_read_only \| oltp_read_write |
| `--threads` | 50 | 并发线程数 |
| `--warmup` | 10 | Warmup 秒数 |
| `--duration` | 60 | 测量秒数 |
| `--cooldown` | 5 | Cooldown 秒数 |
| `--tables` | 1 | 表数量 |
| `--dataset-size` | 1000000 | 每表行数 |
| `--distribution` | uniform | uniform \| zipfian |
| `--seed` | 42 | RNG seed |
| `--connection-mode` | per-thread | per-thread \| shared-pool |
| `--output` | text | text \| json |
| `--baseline` | - | 基线文件路径 |

---

## 7. 依赖变更

```toml
# crates/bench/Cargo.toml
[dependencies]
# 新增
hdrhistogram = "7.5"
rand = { version = "0.8", features = ["small_rng"] }
zipf = "0.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 已有
tokio = { version = "1", features = ["full"] }
sqlrustgo = { path = "../sql" }
```

---

## 8. 测试验证

### 8.1 单元测试
- `test_uniform_distribution` - Uniform 分布正确性
- `test_zipfian_distribution` - Zipfian 分布正确性
- `test_sbtest_schema` - 表结构正确性
- `test_workload_sql_generation` - SQL 生成正确性

### 8.2 集成测试
- `test_warmup_phase` - Warmup 不计入统计
- `test_seed_reproducibility` - 相同 seed 结果一致
- `test_regression_detection` - 回归检测正确
- `test_point_select_qps` - QPS ≥ 1000
- `test_p99_latency` - P99 < 100ms
- `test_concurrent_stability` - 50 并发稳定

---

## 9. CI Gate 集成

### 9.1 GitHub Actions Workflow

```yaml
name: Performance Benchmark Gate
on:
  pull_request:
    paths:
      - 'crates/executor/**'
      - 'crates/storage/**'
      - 'crates/optimizer/**'

jobs:
  benchmark:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v4

      - name: Run Benchmark
        run: |
          cargo run --release --bin bench -- \
            --workload oltp_point_select \
            --threads 50 \
            --duration 60 \
            --output json --baseline ./baseline.json > result.json

      - name: Check Regression
        run: |
          if grep '"passed": false' result.json; then
            echo "Performance regression detected!"
            exit 1
          fi

      - name: Update Baseline
        if: github.ref == 'refs/heads/main'
        run: cp result.json baseline.json
```

---

## 10. 后续阶段

### Phase A
- 24h 连续运行
- 进度报告
- 稳定性指标

### Phase C
- PostgreSQL 对比
- SQLite 对比

---

*本文档为实现规范，批准后开始实现。*
