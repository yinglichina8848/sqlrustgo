# SQLRustGo v1.4.0 架构文档

> **版本**: 1.4.0
> **更新日期**: 2026-03-18

---

## 一、架构概述

SQLRustGo 是一个用 Rust 编写的嵌入式 SQL 数据库引擎，v1.4.0 版本专注于**性能优化**，主要包含 CBO 成本优化器和向量化执行基础。

### 1.1 核心特性

| 特性 | 说明 |
|------|------|
| CBO | 基于代价的优化器 |
| 向量化 | SIMD 批量处理基础 |
| Join 算法 | HashJoin, SortMergeJoin, NestedLoopJoin |
| 可观测性 | Prometheus 指标 + Grafana |

### 1.2 技术栈

- **语言**: Rust 2021 Edition
- **测试**: Criterion.rs, cargo test
- **监控**: Prometheus, Grafana

---

## 二、模块架构

```
┌─────────────────────────────────────────────────────────────┐
│                      SQLRustGo v1.4.0                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                 │
│  │ Parser  │───▶│ Planner │───▶│Optimizer│                 │
│  └─────────┘    └─────────┘    └────┬────┘                 │
│                                     │                        │
│                                     ▼                        │
│  ┌─────────┐    ┌─────────┐    ┌────┴────┐                 │
│  │ Storage │◀───│Executor │◀───│  CBO   │                 │
│  └─────────┘    └─────────┘    └─────────┘                 │
│                                                             │
│  ┌─────────┐    ┌─────────┐                               │
│  │ Server  │    │ Monitor │                               │
│  └─────────┘    └─────────┘                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 三、核心模块

### 3.1 Parser (解析器)

负责 SQL 词法分析和语法解析。

**位置**: `crates/parser/`

**主要功能**:
- SQL 词法分析 (Lexer)
- SQL 语法解析 (Parser)
- AST 生成

### 3.2 Planner (计划器)

负责生成逻辑计划和物理计划。

**位置**: `crates/planner/`

**主要功能**:
- 逻辑计划生成
- 物理计划转换
- 执行计划优化

### 3.3 Optimizer (优化器)

v1.4.0 核心模块 - CBO 成本优化器。

**位置**: `crates/optimizer/`

**主要组件**:

| 组件 | 说明 |
|------|------|
| CostModel | 代价模型 |
| StatsCollector | 统计信息收集 |
| JoinReordering | Join 顺序优化 |
| IndexSelect | 索引选择优化 |

### 3.4 Executor (执行器)

负责执行物理计划。

**位置**: `crates/executor/`

**主要算子**:

| 算子 | 说明 |
|------|------|
| TableScanExecutor | 表扫描 |
| FilterExecutor | 过滤 |
| ProjectionExecutor | 投影 |
| HashJoinExecutor | Hash 连接 |
| SortMergeJoinExecutor | 排序合并连接 (新增) |
| NestedLoopJoinExecutor | 嵌套循环连接 (新增) |
| AggregateExecutor | 聚合 |
| SortExecutor | 排序 |

### 3.5 Storage (存储)

负责数据存储和管理。

**位置**: `crates/storage/`

**主要功能**:
- 页式存储
- 缓冲池管理
- B+ 树索引

### 3.6 Server (服务器)

HTTP 服务器和监控端点。

**位置**: `crates/server/`

**主要功能**:
- HTTP API
- /metrics 端点 (新增)
- 健康检查

---

## 四、CBO 成本优化器设计

### 4.1 代价模型

```rust
pub struct SimpleCostModel {
    cpu_cost_per_row: f64,      // CPU 代价
    io_cost_per_page: f64,       // I/O 代价
    network_cost_per_byte: f64,  // 网络代价
}
```

### 4.2 代价估算

| 操作 | 估算方法 |
|------|----------|
| SeqScan | row_count * cpu + page_count * io |
| IndexScan | index_pages * io + data_pages * io + row_count * cpu |
| HashJoin | left_rows * cpu + right_rows * cpu |
| SortMerge | (left + right) * cpu * log(n) |

### 4.3 统计信息

```rust
pub struct TableStats {
    row_count: u64,
    page_count: u64,
}

pub struct ColumnStats {
    null_count: u64,
    n_distinct: u64,
}
```

---

## 五、Join 算法

### 5.1 HashJoin

适用于任意数据分布，内存充足时效率高。

### 5.2 SortMergeJoin

适用于已排序数据或需要排序结果的场景。

### 5.3 NestedLoopJoin

适用于小表驱动大表或 Cross Join 场景。

---

## 六、向量化执行

### 6.1 Vector<T>

SIMD 友好的向量类型，支持批量操作。

### 6.2 RecordBatch

批量行数据，用于减少函数调用开销。

### 6.3 VectorizedExecutor

向量化执行器 trait，支持批量处理。

---

## 七、可观测性

### 7.1 指标系统

- Prometheus 兼容格式
- /metrics HTTP 端点

### 7.2 指标类型

| 类型 | 说明 |
|------|------|
| Counter | 累计值 |
| Gauge | 瞬时值 |
| Histogram | 分布 |

### 7.3 Grafana Dashboard

- 预置仪表盘模板
- 性能监控面板

---

## 八、覆盖率

| 模块 | 覆盖率 |
|------|--------|
| 整体 | 76.25% |
| executor | 60.8% |
| optimizer | 34.2% |
| planner | 82.2% |

---

## 九、相关文档

- [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)
- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [PERFORMANCE_BENCHMARK_REPORT.md](./PERFORMANCE_BENCHMARK_REPORT.md)
- [RELEASE_NOTES.md](./RELEASE_NOTES.md)

---

**最后更新**: 2026-03-18
