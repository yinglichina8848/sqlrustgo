# SQLRustGo TPC-H Benchmark 用户指南

> **版本**: v1.6.1
> **更新日期**: 2026-03-20

---

## 一、概述

本文档介绍如何使用 SQLRustGo v1.6.1 提供的 TPC-H Benchmark 工具进行性能测试和对比。

### 1.1 功能特性

- **TPC-H 查询测试**: Q1、Q3、Q6、Q10 标准查询
- **延迟统计**: P50/P95/P99 延迟分布
- **多系统对比**: SQLRustGo vs PostgreSQL vs SQLite
- **JSON 输出**: 标准化结果格式
- **可复现性**: 固定种子、确定性数据

### 1.2 支持的查询

| 查询 | 描述 | 复杂度 |
|------|------|--------|
| Q1 | 定价汇总报告 | 聚合 |
| Q3 | 运输优先级查询 | JOIN + 聚合 |
| Q6 | 收入增长查询 | 过滤 + 聚合 |
| Q10 | 客户收入查询 | 多表 JOIN |

---

## 二、安装

### 2.1 前提条件

```bash
# Rust 1.75+
rustc --version

# 依赖
cargo --version
```

### 2.2 构建 Benchmark 工具

```bash
# 克隆仓库
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建发布版本
cargo build --release

# 构建 bench-cli 工具
cargo build --release -p sqlrustgo-bench-cli
```

### 2.3 验证安装

```bash
# 查看 bench-cli 帮助
cargo run --release -p sqlrustgo-bench-cli -- --help
```

---

## 三、快速开始

### 3.1 运行 TPC-H 查询

```bash
# 使用 bench-cli 运行 TPC-H 查询
cargo run --release -p sqlrustgo-bench-cli -- tpch \
    --scale 1 \
    --iterations 10 \
    --queries Q1,Q3,Q6,Q10
```

### 3.2 运行 OLTP 负载

```bash
# 运行 OLTP 负载测试
cargo run --release -p sqlrustgo-bench-cli -- oltp \
    --threads 4 \
    --duration 60 \
    --workload read
```

### 3.3 运行对比测试

```bash
# 运行 SQLRustGo vs SQLite 对比
cargo run --release --example tpch_compare

# 保存结果到 JSON
cargo run --release --example tpch_compare -- --output results.json
```

---

## 四、Benchmark CLI 详解

### 4.1 TPC-H 命令

```bash
cargo run --release -p sqlrustgo-bench-cli -- tpch [OPTIONS]
```

**参数说明**:

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--scale` | 1 | 数据规模因子 |
| `--iterations` | 3 | 迭代次数 |
| `--queries` | Q1,Q3,Q6,Q10 | 查询列表 |
| `--output` | - | JSON 输出文件 |

**示例**:

```bash
# 运行单个查询
cargo run --release -p sqlrustgo-bench-cli -- tpch --queries Q1

# 运行多个查询
cargo run --release -p sqlrustgo-bench-cli -- tpch --queries Q1,Q3,Q6

# 指定迭代次数
cargo run --release -p sqlrustgo-bench-cli -- tpch --iterations 100

# 保存结果
cargo run --release -p sqlrustgo-bench-cli -- tpch --output results.json
```

### 4.2 OLTP 命令

```bash
cargo run --release -p sqlrustgo-bench-cli -- oltp [OPTIONS]
```

**参数说明**:

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--threads` | 1 | 并发线程数 |
| `--duration` | 60 | 运行时长(秒) |
| `--workload` | read | 负载类型(read/write/mixed) |
| `--output` | - | JSON 输出文件 |

**示例**:

```bash
# 4 线程读负载，60 秒
cargo run --release -p sqlrustgo-bench-cli -- oltp --threads 4 --duration 60

# 混合负载
cargo run --release -p sqlrustgo-bench-cli -- oltp --workload mixed
```

### 4.3 自定义命令

```bash
cargo run --release -p sqlrustgo-bench-cli -- custom \
    --file queries.sql \
    --iterations 10 \
    --parallel 4
```

---

## 五、Criterion Benchmark

### 5.1 运行标准 Benchmark

```bash
# 运行所有 TPC-H benchmark
cargo bench --bench tpch_bench

# 运行特定查询
cargo bench --bench tpch_bench -- Q1

# 查看 HTML 报告
open target/criterion/tpch_q1/report/index.html
```

### 5.2 自定义 Benchmark

```bash
# 运行特定查询的详细报告
cargo bench --bench tpch_bench -- --noplot

# 运行带输出的 benchmark
cargo bench --bench tpch_bench -- Q1 --iterations 1000
```

---

## 六、对比测试

### 6.1 SQLRustGo vs SQLite

```bash
# 构建对比工具
cargo build --release --example tpch_compare

# 运行对比
cargo run --release --example tpch_compare
```

**预期输出**:

```
=== TPC-H Benchmark Comparison ===
Scale Factor: 1.0
Iterations: 10
Timestamp: 2026-03-20T10:00:00Z

=== SQLRustGo ===
Query      Avg(ms)     P50(ms)     P95(ms)     P99(ms)
--------------------------------------------------------------
Q1            12.34          12          15          18
Q3             8.45           8          11          14
Q6             5.23           5           7           9

=== SQLite ===
Query      Avg(ms)     P50(ms)     P95(ms)     P99(ms)
--------------------------------------------------------------
Q1            15.67          15          19          23
Q3            12.34          12          16          20
Q6             8.45           8          11          14
```

### 6.2 SQLRustGo vs PostgreSQL

> **注意**: 需要 PostgreSQL 15+ 安装并运行

```bash
# 设置 PostgreSQL 连接
export PGHOST=localhost
export PGPORT=5432
export PGUSER=postgres
export PGPASSWORD=your_password

# 运行 PostgreSQL 对比
cargo run --release --example pg_compare
```

### 6.3 结果分析

对比结果包含以下指标:

| 指标 | 说明 |
|------|------|
| Avg(ms) | 平均延迟 |
| P50(ms) | 中位数延迟 |
| P95(ms) | 95% 分位延迟 |
| P99(ms) | 99% 分位延迟 |
| Min(ms) | 最小延迟 |
| Max(ms) | 最大延迟 |

---

## 七、结果解读

### 7.1 延迟指标

| 指标 | 含义 | 优化目标 |
|------|------|----------|
| P50 | 50% 请求的延迟 | 核心性能 |
| P95 | 95% 请求的延迟 | 尾部延迟 |
| P99 | 99% 请求的延迟 | 极端情况 |

### 7.2 性能判断

**合理范围 (Embedded 模式)**:

| 查询 | 预期延迟 |
|------|----------|
| Q1 | 5-20ms |
| Q3 | 10-30ms |
| Q6 | 3-15ms |
| Q10 | 20-50ms |

**异常情况**:

- P99 > 10x P50: 存在锁竞争或 GC 暂停
- 延迟波动 > 50%: 系统不稳定
- 对比 PostgreSQL 慢 10x+: 需要优化

### 7.3 性能瓶颈定位

根据延迟分布判断瓶颈:

| 现象 | 可能原因 | 优化方向 |
|------|----------|----------|
| P99 远高于 P50 | 锁竞争 | 减少锁粒度 |
| 延迟随并发增长 | 串行瓶颈 | 并行化 |
| 内存使用增长 | 内存泄漏 | 检查内存管理 |
| CPU 100% | CPU 密集 | 向量化/ JIT |

---

## 八、配置

### 8.1 配置文件

创建 `benchmark.yaml`:

```yaml
# Benchmark 配置示例
mode: benchmark
scale: 1.0

# 查询配置
queries:
  - name: Q1
    sql: "SELECT ..."
    iterations: 10

# 系统配置
system:
  query_cache: false
  wal: true
  buffer_pool_size: 1024
```

### 8.2 环境变量

```bash
# 启用详细日志
RUST_LOG=debug cargo run ...

# 禁用 Query Cache
SQLRUSTGO_QUERY_CACHE=0 cargo bench ...

# 指定数据目录
SQLRUSTGO_DATA_DIR=/tmp/benchdata cargo run ...
```

---

## 九、故障排除

### 9.1 常见问题

**Q: Benchmark 执行失败**

A: 检查:
1. 内存是否充足 (至少 4GB)
2. 数据规模是否过大
3. 是否存在编译错误

**Q: 性能数据异常**

A: 
1. 确保 Query Cache 关闭 (`query_cache: false`)
2. 使用足够迭代次数 (≥10)
3. 预热后再测量

**Q: PostgreSQL 连接失败**

A:
1. 检查 PostgreSQL 服务是否运行
2. 验证连接参数
3. 创建测试数据库

### 9.2 调试模式

```bash
# 启用追踪
RUST_LOG=trace cargo run --example tpch_compare

# 生成火焰图
cargo flamegraph --example tpch_compare

# 内存分析
cargo bench --bench tpch_bench -- --profile-time=5
```

---

## 十、输出格式

### 10.1 JSON 格式

```json
{
  "timestamp": "2026-03-20T10:00:00Z",
  "version": "1.6.1",
  "workload": "tpch",
  "scale_factor": 1.0,
  "queries": [
    {
      "name": "Q1",
      "avg_latency_ms": 12.34,
      "p50_ms": 12,
      "p95_ms": 15,
      "p99_ms": 18,
      "min_ms": 8,
      "max_ms": 25,
      "iterations": 10
    }
  ],
  "comparison": {
    "system": "PostgreSQL",
    "queries": [...]
  }
}
```

### 10.2 CSV 格式

```bash
# 转换为 CSV
cat results.json | jq -r '.queries[] | [.name, .avg_latency_ms, .p50_ms, .p95_ms, .p99_ms] | @csv'
```

---

## 十一、延伸阅读

- [性能报告模板](./BENCHMARK_REPORT_TEMPLATE.md)
- [性能分析工具链](./PERFORMANCE_ANALYSIS_TOOLCHAIN.md)
- [发布说明](./RELEASE_NOTES.md)

---

*文档版本: v1.6.1*
*更新日期: 2026-03-20*
