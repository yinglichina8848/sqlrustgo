# SQLRustGo v1.6.1 Release Notes

> **版本**: v1.6.1
> **发布日期**: 2026-03-20
> **代号**: Benchmark 修复版
> **当前阶段**: **Alpha** (测试阶段)

---

## 一、版本概述

### 1.1 发布类型

| 项目 | 值 |
|------|------|
| 版本号 | v1.6.1 |
| 发布类型 | Benchmark 修复版 |
| 目标分支 | release/v1.6.1 |
| 开发分支 | develop/v1.6.1 |
| 前置版本 | v1.6.0 (GA) |

### 1.2 核心特性

v1.6.1 专注于修复 v1.6.0 性能数据可信性问题，核心目标：

- **Benchmark 系统重构**: 统一 CLI + Workload
- **性能可信性修复**: 关闭 Query Cache，统一对比基准
- **专业 Metrics**: P50/P95/P99 延迟统计
- **PostgreSQL 对比**: 引入行业对标

---

## 二、版本定位

### 2.1 v1.6.0 问题总结

| 问题 | 影响 | 修复方案 |
|------|------|----------|
| MemoryStorage vs SQLite 磁盘 | 对比不公平 | 引入 PostgreSQL 对比 |
| Query Cache 开启 | 性能偏高 | Benchmark 模式默认关闭 |
| 无延迟统计 | 缺少专业指标 | P50/P95/P99 实现 |
| 无 JSON 输出 | 报告格式不统一 | 统一 JSON 格式 |

### 2.2 架构升级

```
v1.5 (L3+ 持久化) → v1.6 (L3+ 事务隔离) → v1.6.1 (可信 Benchmark) → v2.0 (分布式)
```

---

## 三、功能变更

### 3.1 新增功能

#### Benchmark 系统重构 (P0)

| 功能 | 说明 | PR |
|------|------|-----|
| Benchmark Runner CLI | 统一命令行工具 | #692 |
| OLTP Workload | YCSB-like 负载 | #692 |
| TPC-H 基准 | Q1/Q3/Q6/Q10 | #692 |

#### 可信性修复 (P0)

| 功能 | 说明 | PR |
|------|------|-----|
| Query Cache 关闭 | Benchmark 模式禁用 | #695 |
| PostgreSQL 对比 | 行业对标 | #695 |
| 统一 SQLite 配置 | 公平对比 | #695 |

#### Metrics 系统 (P1)

| 功能 | 说明 | PR |
|------|------|-----|
| P50/P95/P99 延迟 | 专业延迟统计 | #691 |
| JSON 输出 | 统一报告格式 | #691 |

#### 环境标准化 (P1)

| 功能 | 说明 | PR |
|------|------|-----|
| 配置模板 | YAML 配置 | #694 |
| 数据规模校验 | 数据验证 | #694 |

### 3.2 功能状态

| 功能 | 状态 | Alpha | Beta | RC | GA |
|------|------|-------|------|-----|-----|
| Benchmark CLI | ✅ | ✅ | - | - | - |
| OLTP Workload | ✅ | ✅ | - | - | - |
| TPC-H 基准 | ✅ | ✅ | - | - | - |
| Query Cache 关闭 | ✅ | ✅ | - | - | - |
| PostgreSQL 对比 | ✅ | ✅ | - | - | - |
| P50/P95/P99 | ✅ | ✅ | - | - | - |
| JSON 输出 | ✅ | ✅ | - | - | - |
| 配置模板 | ✅ | ✅ | - | - | - |
| Benchmark CI | ⏳ | - | - | ⏳ | ⏳ |

---

## 四、架构设计

### 4.1 三层测试模式

```
┌─────────────────────────────────────────────────────────────────┐
│                    v1.6.1 Benchmark 架构                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │
│  │  Embedded   │    │    TCP      │    │ PostgreSQL  │          │
│  │   模式      │    │    模式     │    │    对比     │          │
│  └─────────────┘    └─────────────┘    └─────────────┘          │
│         │                  │                  │                   │
│         ▼                  ▼                  ▼                   │
│  ┌─────────────────────────────────────────────────────────┐      │
│  │              Metrics Collector                            │      │
│  │         P50 / P95 / P99 / TPS / JSON                    │      │
│  └─────────────────────────────────────────────────────────┘      │
│                           │                                        │
│                           ▼                                        │
│  ┌─────────────────────────────────────────────────────────┐      │
│  │              Benchmark Report                             │      │
│  │         可复现 / 可对比 / 可解释                           │      │
│  └─────────────────────────────────────────────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 预期现象

```
Embedded >> TCP ≈ PostgreSQL
```

如果 Embedded vs PostgreSQL 还是 3000x 差距 → Benchmark 有 bug

---

## 五、测试结果

### 5.1 测试统计

| 测试类型 | 数量 | 状态 |
|----------|------|------|
| Benchmark | 45 | ✅ |
| Metrics | 12 | ✅ |
| 配置 | 8 | ✅ |
| **总计** | **65+** | **✅** |

### 5.2 覆盖率目标

| 阶段 | 目标覆盖率 | 当前 |
|------|-----------|------|
| Alpha | ≥50% | ⏳ |
| Beta | ≥65% | - |
| RC | ≥75% | - |
| GA | ≥80% | - |

---

## 六、已知问题

### 6.1 延期至后续版本

| 功能 | 说明 |
|------|------|
| Benchmark CI | 轻量版 CI 集成待完成 |
| Serializable | 串行化隔离级别 |
| SIMD 优化 | 向量化加速 |

### 6.2 v1.6.2 改进计划

| 改进项 | 说明 |
|--------|------|
| Benchmark CI | 自动性能回归检测 |
| 覆盖率提升 | 目标 80%+ |
| 性能优化 | 基于瓶颈分析 |

---

## 七、升级指南

### 7.1 从 v1.6.0 升级

v1.6.1 完全向后兼容 v1.6.0，所有现有功能和 API 保持不变。

### 7.2 Benchmark 模式使用

```rust
// 配置文件 benchmark.yaml
mode: benchmark
query_cache: disabled
warmup: 3
iterations: 100

// 运行 Benchmark
cargo run --release --bin benchmark-cli -- --config benchmark.yaml
```

### 7.3 新增 API

```rust
// 延迟统计
let stats = LatencyStats::new();
stats.add(duration);
let p50 = stats.percentile(0.5);
let p95 = stats.percentile(0.95);
let p99 = stats.percentile(0.99);

// JSON 输出
let result = BenchmarkResult {
    tps: 100000,
    latency: stats,
    ..
};
println!("{}", serde_json::to_string_pretty(&result).unwrap());
```

---

## 八、文档索引

| 文档 | 说明 |
|------|------|
| [RELEASE_PLAN.md](./RELEASE_PLAN.md) | 发布计划 |
| [GOALS_AND_PLANNING.md](./GOALS_AND_PLANNING.md) | 版本规划 |
| [ANNOUNCEMENT.md](./ANNOUNCEMENT.md) | 开发公告 |
| [PERFORMANCE_ANALYSIS_TOOLCHAIN.md](./PERFORMANCE_ANALYSIS_TOOLCHAIN.md) | 性能工具链 |
| [BENCHMARK_REPORT_TEMPLATE.md](./BENCHMARK_REPORT_TEMPLATE.md) | 报告模板 |
| [BENCHMARK_REFERENCE_CODE.md](./BENCHMARK_REFERENCE_CODE.md) | 参考代码 |

---

## 九、正确表述声明

### ✅ 可以说

- "架构达到工业级标准"
- "支持完整事务系统"
- "建立可信 Benchmark 体系"
- "P99 延迟 < 5ms"

### ❌ 禁止说

- "比 PostgreSQL 快 3000 倍"
- "TPS 达到 43 万"
- "无死锁"
- "性能远超 SQLite"

---

## 十、发布里程碑

| 日期 | 里程碑 |
|------|---------|
| 2026-03-20 | Craft 规划 |
| 2026-03-20 | **Alpha 发布** 🔥 |
| TBD | Beta 发布 |
| TBD | RC 发布 |
| TBD | GA 正式发布 |

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-20*
*版本: Alpha*
