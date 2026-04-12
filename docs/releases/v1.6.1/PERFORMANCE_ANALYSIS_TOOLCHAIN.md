# SQLRustGo v1.6.1 性能分析工具链

> **版本**: v1.6.1
> **目标**: 建立可信性能分析体系
> **创建日期**: 2026-03-20

---

## 一、工具链整体架构

```
                 ┌────────────────────┐
                 │ Benchmark Runner   │
                 └────────┬───────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
   Metrics JSON     Flamegraph         System Metrics
 (TPS / P99 / WAL)   (CPU Hotspots)    (CPU / IO / MEM)
        │                 │                 │
        └──────────┬──────┴──────┬──────────┘
                   ▼             ▼
           Analysis Engine   Regression CI
                   │
                   ▼
             Report (Markdown)
```

---

## 二、工具链组件

### 2.1 Benchmark Metrics (已实现)

扩展输出格式:

```json
{
  "tps": 120000,
  "latency": {
    "p50": 200,
    "p95": 800,
    "p99": 3200,
    "max": 15000
  },
  "lock": {
    "wait_time_ratio": 0.25,
    "conflicts": 150
  },
  "wal": {
    "flush_time_ratio": 0.35,
    "bytes_written": 1024000
  },
  "executor": {
    "cpu_time_ratio": 0.60,
    "scan_count": 50000,
    "join_count": 10000
  },
  "buffer": {
    "hit_rate": 0.92,
    "miss_count": 800
  }
}
```

### 2.2 Flamegraph (火焰图)

#### 安装

```bash
# macOS
cargo install flamegraph

# Linux
sudo apt install linux-tools-common linux-tools-generic
cargo install flamegraph
```

#### 使用

```bash
cargo flamegraph --bin sqlrustgo-bench -- --threads 10
```

#### 解读

| 现象 | 说明 |
|------|------|
| executor 占大头 | 执行器慢 |
| lock_manager 很宽 | 锁竞争 |
| wal_write 很宽 | WAL 瓶颈 |

### 2.3 Tracing (Rust 内核级)

#### 依赖

```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### 初始化

```rust
use tracing_subscriber;
tracing_subscriber::fmt::init();
```

#### 关键埋点

| 模块 | 函数 | 说明 |
|------|------|------|
| Executor | execute | 执行耗时 |
| Lock | acquire | 锁等待 |
| WAL | write | 写入延迟 |
| BufferPool | fetch | 缓存命中 |

```rust
use tracing::instrument;

#[instrument(skip_all, fields(tps, latency))]
fn execute_query(&self, sql: &str) -> Result<()> {
    // executor code
}

#[instrument]
fn acquire_lock(&self, key: &str) -> Result<()> {
    // lock code
}
```

### 2.4 Perf (Linux 系统级)

```bash
# 记录性能数据
perf record -g ./target/release/sqlrustgo-bench

# 查看报告
perf report
```

能分析:
- cache miss
- CPU cycles
- syscall

### 2.5 eBPF (高级可选)

工具: bcc / bpftrace

用途:
- IO 延迟分析
- syscall 追踪

---

## 三、自动分析引擎

### 3.1 分析逻辑

```rust
pub fn analyze(metrics: &BenchmarkMetrics) -> Vec<String> {
    let mut warnings = Vec::new();

    // 锁瓶颈检测
    if metrics.lock.wait_time_ratio > 0.2 {
        warnings.push("⚠️ Lock contention detected (>20%)".to_string());
    }

    // WAL 瓶颈检测
    if metrics.wal.flush_time_ratio > 0.3 {
        warnings.push("⚠️ WAL bottleneck detected (>30%)".to_string());
    }

    // 延迟尾部检测
    if metrics.latency.p99 > 5000 {
        warnings.push("⚠️ High tail latency (>5ms)".to_string());
    }

    // 缓存命中率
    if metrics.buffer.hit_rate < 0.9 {
        warnings.push("⚠️ Low buffer hit rate (<90%)".to_string());
    }

    warnings
}
```

### 3.2 输出示例

```
=== PERFORMANCE ANALYSIS ===

⚠️ Lock contention detected (wait_ratio: 25%)
⚠️ WAL bottleneck detected (flush_ratio: 35%)

Analysis:
- 锁等待是主要瓶颈，建议优化锁粒度
- WAL 写入影响较大，建议启用 group commit

Recommendation:
1. 优先优化 Lock Manager
2. 考虑批量 WAL 写入
```

---

## 四、CI 性能回归检测

### 4.1 GitHub Actions

```yaml
name: Benchmark Regression

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Benchmark
        run: |
          cargo run --release -- \
            --threads 10 \
            --duration 60 \
            --output json > result.json

      - name: Compare with Baseline
        run: |
          python scripts/compare_bench.py result.json baseline.json

      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: result.json
```

### 4.2 对比脚本

```python
#!/usr/bin/env python3
import json
import sys

def compare(new_path, base_path):
    new = json.load(open(new_path))
    base = json.load(open(base_path))

    errors = []

    # TPS 回归检测 (允许 10% 下降)
    tps_threshold = base["tps"] * 0.9
    if new["tps"] < tps_threshold:
        errors.append(f"❌ TPS regression: {new['tps']} < {tps_threshold}")

    # P99 延迟回归 (允许 20% 上升)
    p99_threshold = base["latency"]["p99"] * 1.2
    if new["latency"]["p99"] > p99_threshold:
        errors.append(f"❌ P99 latency regression: {new['latency']['p99']} > {p99_threshold}")

    if errors:
        for e in errors:
            print(e)
        sys.exit(1)
    else:
        print("✅ No regression detected")
        sys.exit(0)

if __name__ == "__main__":
    compare(sys.argv[1], sys.argv[2])
```

### 4.3 基准数据管理

```
bench/
├── baseline/
│   ├── ol tp_baseline.json
│   └── tpch_baseline.json
├── scripts/
│   └── compare.py
└── README.md
```

---

## 五、完整性能分析流程

### 5.1 执行顺序

| 阶段 | 时间 | 任务 |
|------|------|------|
| Day 1 | 2h | Metrics JSON + P99 |
| Day 1 | 2h | 自动分析引擎 |
| Day 2 | 4h | Flamegraph 集成 |
| Day 2 | 4h | Tracing 埋点 |
| Day 3 | 4h | CI 回归检测 |

### 5.2 分析流程

```
1. 运行 Benchmark
   ↓
2. 收集 Metrics JSON
   ↓
3. 运行 Flamegraph (可选)
   ↓
4. 自动分析瓶颈
   ↓
5. 生成报告
   ↓
6. CI 对比基准 (发布时)
```

---

## 六、关键指标阈值

| 指标 | 阈值 | 状态 | 说明 |
|------|------|------|------|
| lock.wait_ratio | >20% | ⚠️ | 锁瓶颈 |
| wal.flush_ratio | >30% | ⚠️ | WAL 瓶颈 |
| latency.p99 | >5ms | ⚠️ | 高延迟 |
| buffer.hit_rate | <90% | ⚠️ | 缓存问题 |
| tps | 下降 >10% | ❌ | 回归 |

---

## 七、交付清单

### 7.1 代码

- [x] Benchmark Runner CLI
- [x] Metrics 收集 (JSON)
- [x] 自动分析引擎 (文档已完成，代码见 analyze 函数)
- [x] Tracing 埋点 (tracing crate 已集成)
- [ ] CI 回归检测 (待开发 EPIC-05)

### 7.2 文档

- [x] 性能分析指南 (本文档)
- [x] 瓶颈定位手册 (本文档第 2-3 节)
- [x] CI 配置说明 (本文档第 4 节)

### 7.3 基线数据

- [x] OLTP 基线 (crates/bench)
- [x] TPC-H 基线 (crates/bench)

---

## 八、预期能力

### 8.1 定位能力

- [x] 精确定位锁瓶颈 (metrics.lock.wait_time_ratio)
- [x] 精确定位 WAL 瓶颈 (metrics.wal.flush_time_ratio)
- [x] 精确定位 Executor 瓶颈 (metrics.executor.cpu_time_ratio)

### 8.2 解释能力

- [x] P99 高因为锁等待 (lock.wait_time_ratio)
- [x] TPS 瓶颈在 WAL (wal.flush_time_ratio)
- [x] CPU 在 executor (executor.cpu_time_ratio)

### 8.3 防护能力

- [ ] CI 自动检测 TPS 回归 (待开发 EPIC-05)
- [ ] CI 自动检测延迟回归 (待开发 EPIC-05)

---

## 九、总结

### 工具链价值

| 能力 | 状态 |
| 能测性能 | ✅ 已实现 |
| 能定位瓶颈 | ✅ 已实现 |
| 能解释原因 | ✅ 已实现 |
| 防性能回退 | ⏳ CI 待开发 |

### 最终目标

```
能写系统 → 能证明对 → 能解释表现 → 防性能退化
```

---

*本文档为性能分析工具链设计*
*创建日期: 2026-03-20*
