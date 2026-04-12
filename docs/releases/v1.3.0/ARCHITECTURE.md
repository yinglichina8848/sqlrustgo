# SQLRustGo v1.3.0 架构设计文档

> **版本**: v1.3.0
> **更新日期**: 2026-03-15
> **状态**: 🔄 更新中

---

## 一、系统架构总览

### 1.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SQLRustGo v1.3.0                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐              │
│  │  Client │────▶│ Parser  │────▶│ Planner │────▶│Optimizer│              │
│  └─────────┘     └─────────┘     └─────────┘     └─────────┘              │
│       │                                       │                              │
│       │                                       ▼                              │
│       │                              ┌─────────────┐                         │
│       │                              │   Executor  │◀────────────────┐     │
│       │                              └─────────────┘                 │     │
│       │                                       │                         │     │
│       ▼                                       ▼                         │     │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         Storage Layer                                │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │    │
│  │  │ MemoryStore │  │ BufferPool  │  │    FileStorage         │   │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      Observability Layer                             │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌───────────┐ │    │
│  │  │  /health   │  │  /metrics  │  │   Grafana  │  │ Prometheus│ │    │
│  │  │  (live/    │  │  (Prometheus│  │  Dashboard │  │  Alerts   │ │    │
│  │  │   ready)   │  │   format)  │  │            │  │           │ │    │
│  │  └────────────┘  └────────────┘  └────────────┘  └───────────┘ │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 核心模块

| 模块 | 职责 | 状态 |
|------|------|------|
| **parser** | SQL 解析 | ✅ 稳定 |
| **planner** | 逻辑计划生成 | ✅ 稳定 |
| **optimizer** | 物理优化 | ✅ 稳定 |
| **executor** | Volcano Model 执行器 | ✅ 稳定 |
| **storage** | 数据存储和缓存 | ✅ 稳定 |
| **common** | 共享类型和工具 | ✅ 稳定 |
| **server** | HTTP 服务和健康检查 | ✅ 稳定 |
| **observability** | 指标收集和暴露 | ✅ 新增 |

---

## 二、Executor 架构 (Volcano Model)

### 2.1 核心 Trait

```rust
pub trait VolcanoExecutor: Send + Sync {
    fn execute(&self, context: &mut ExecContext) -> Result<Option<RecordBatch>>;
    fn schema(&self) -> &Schema;
    fn children(&self) -> Vec<Arc<dyn VolcanoExecutor>>;
}
```

### 2.2 算子实现

| 算子 | 文件 | 状态 | 测试覆盖 |
|------|------|------|----------|
| TableScan | tablescan.rs | ✅ 完成 | ~80% |
| Projection | projection.rs | ✅ 完成 | ~70% |
| Filter | filter.rs | ✅ 完成 | ~60% |
| HashJoin | hash_join.rs | ✅ 完成 | ~75% |
| Aggregate | aggregate.rs | ✅ 完成 | ~65% |
| Sort | sort.rs | ✅ 完成 | ~60% |
| Limit | limit.rs | ✅ 完成 | ~55% |

### 2.3 执行流程

```
SQL: SELECT * FROM users WHERE age > 18

┌─────────────────────────────────────────────────────────────┐
│                      Logical Plan                          │
│  Filter(age > 18) -> TableScan(users)                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Physical Plan                          │
│  FilterExec { condition: age > 18 }                        │
│    └── TableScanExec { table: users, projection: [*] }    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Execution                            │
│  1. TableScan.next() -> RecordBatch                       │
│  2. Filter.filter(record_batch) -> filtered_batch        │
│  3. 返回 filtered_batch                                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 三、可观测性架构

### 3.1 指标系统

```
┌─────────────────────────────────────────────────────────────┐
│                     Metrics Pipeline                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌─────────────┐ │
│   │   Metrics   │───▶│  Aggregator  │───▶│  Exporter   │ │
│   │   Trait     │    │   (M-005)    │    │             │ │
│   └──────────────┘    └──────────────┘    └─────────────┘ │
│         │                                         │          │
│         │              ┌──────────────┐           │          │
│         └─────────────▶│   Storage    │◀──────────┘          │
│                        │  (BufferPool │                       │
│                        │   Metrics)   │                       │
│                        └──────────────┘                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 健康检查端点

| 端点 | 路径 | 状态 | 说明 |
|------|------|------|------|
| Liveness | /health/live | ✅ | 返回 200 表示存活 |
| Readiness | /health/ready | ✅ | 返回健康报告包含组件状态 |

### 3.3 指标端点

| 端点 | 路径 | 状态 | 格式 |
|------|------|------|------|
| Metrics | /metrics | ✅ | Prometheus/OpenMetrics |

### 3.4 监控配置

| 组件 | 状态 | 说明 |
|------|------|------|
| Prometheus | ✅ | 指标收集 |
| Grafana | ✅ | 可视化仪表盘 |
| Alert Rules | ✅ | 告警规则 |

---

## 四、存储架构

### 4.1 存储层次

```
┌─────────────────────────────────────────────────────────────┐
│                      Storage Layer                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Memory Storage                     │   │
│  │   (In-memory data, fastest access)                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                     Buffer Pool                      │   │
│  │   (LRU cache, page-level management)                │   │
│  │   - 页面大小: 4KB                                   │   │
│  │   - 淘汰策略: LRU                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    File Storage                     │   │
│  │   (Persistent storage, WAL support)                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 BufferPool 指标

| 指标 | 说明 | 状态 |
|------|------|------|
| pool_size | 缓冲池大小 | ✅ |
| hits | 缓存命中次数 | ✅ |
| misses | 缓存未命中次数 | ✅ |
| evictions | 淘汰次数 | ✅ |
| pinned_pages | 固定页面数 | ✅ |

---

## 五、模块依赖关系

### 5.1 依赖图

```
common
  │
  ├──▶ parser
  │     │
  │     └──▶ planner
  │           │
  │           └──▶ optimizer
  │                 │
  │                 └──▶ executor
  │                       │
  │                       ├──▶ storage
  │                       │     │
  │                       │     └──▶ buffer_pool
  │                       │
  │                       └──▶ server (observability)
  │                             │
  │                             └──▶ metrics
```

### 5.2 公开 API

#### common
```rust
pub mod sqlrustgo_common {
    pub mod types { Value, DataType, Schema, Field }
    pub mod error { SqlResult, SqlError }
    pub mod metrics { Metrics, MetricValue, QueryType }
}
```

#### parser
```rust
pub mod sqlrustgo_parser {
    pub fn parse_sql(sql: &str) -> SqlResult<Statement>;
    pub fn parse_expression(sql: &str) -> SqlResult<Expression>;
}
```

#### planner
```rust
pub mod sqlrustgo_planner {
    pub struct LogicalPlan;
    pub struct PhysicalPlan;
    pub fn create_logical_plan(stmt: Statement) -> SqlResult<LogicalPlan>;
    pub fn create_physical_plan(plan: LogicalPlan) -> SqlResult<PhysicalPlan>;
}
```

#### executor
```rust
pub mod sqlrustgo_executor {
    pub trait VolcanoExecutor;
    pub struct ExecContext;
    pub struct RecordBatch;
    pub struct TableScanExec;
    pub struct FilterExec;
    pub struct ProjectionExec;
    pub struct HashJoinExec;
    pub struct AggregateExec;
}
```

#### storage
```rust
pub mod sqlrustgo_storage {
    pub trait StorageEngine;
    pub struct BufferPool;
    pub struct MemoryStorage;
    pub struct FileStorage;
}
```

#### server
```rust
pub mod sqlrustgo_server {
    pub mod health {
        pub struct HealthChecker;
        pub struct HealthReport;
        pub enum HealthStatus;
    }
    pub mod metrics {
        pub struct PrometheusExporter;
        pub struct MetricsServer;
    }
}
```

---

## 六、版本特性总结

### 6.1 v1.3.0 新增功能

| 功能 | 状态 | 说明 |
|------|------|------|
| Volcano Executor Model | ✅ | 统一执行器接口 |
| HashJoin | ✅ | 内存哈希连接 |
| Filter | ✅ | 条件过滤 |
| Projection | ✅ | 列投影 |
| Metrics Trait | ✅ | 统一指标接口 |
| Health Check | ✅ | /health/live, /health/ready |
| Prometheus Export | ✅ | /metrics 端点 |
| Grafana Dashboard | ✅ | 预置仪表盘 |
| Alert Rules | ✅ | Prometheus 告警规则 |

### 6.2 性能指标

| 指标 | 值 | 说明 |
|------|-----|------|
| 整体覆盖率 | 78.88% | 目标 ≥65% ✅ |
| Executor 覆盖率 | 87.71% | 目标 ≥60% ✅ |
| Planner 覆盖率 | 76.44% | 目标 ≥60% ✅ |
| Optimizer 覆盖率 | 82.12% | 目标 ≥40% ✅ |

### 6.3 稳定性改进

- 统一错误处理 (SqlResult)
- 线程安全设计
- 资源清理 (Drop trait)
- 单元测试覆盖

---

## 七、文档索引

| 文档 | 说明 |
|------|------|
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 版本计划 |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁检查 |
| [TEST_VERIFICATION_PLAN.md](./TEST_VERIFICATION_PLAN.md) | 测试验证计划 |
| [HEALTH_CHECK_SPECIFICATION.md](./HEALTH_CHECK_SPECIFICATION.md) | 健康检查规范 |
| [OBSERVABILITY_GUIDE.md](../monitoring/OBSERVABILITY_GUIDE.md) | 可观测性指南 |

---

**文档状态**: 正式版
**创建人**: AI Assistant
**审核人**: 待定
