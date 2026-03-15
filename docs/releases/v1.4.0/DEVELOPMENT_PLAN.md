# SQLRustGo v1.4.0 开发计划 (DEVELOPMENT_PLAN.md)

> **版本**: v1.4.0 Draft
> **阶段**: 开发中 (Craft)
> **更新日期**: 2026-03-15
> **目标**: 查询优化与执行增强，同时完成 Prometheus 可观测性

---

## 一、版本概述

### 1.1 主题

**查询优化与执行增强**

### 1.2 核心目标

v1.4.0 聚焦于查询优化和执行器增强，主要包括：
1. CBO (Cost-Based Optimization) 成本优化器基础实现
2. SortMergeJoin 算子实现
3. 统计信息系统集成
4. 可观测性增强：Prometheus 指标暴露

### 1.3 预计周期

8 周 (2026-03-15 ~ 2026-05-10)

---

## 二、前置依赖

### 2.1 来自 v1.3.0

| 功能 | 状态 | 说明 |
|------|------|------|
| Volcano Executor trait | ✅ | 统一的执行器接口 |
| HashJoin 算子 | ✅ | 内连接实现 |
| StatsCollector | ✅ | 统计信息收集器 |
| Metrics trait | ✅ | 指标抽象 |
| /health 端点 | ✅ | 健康检查 |

### 2.2 外部依赖

- **统计信息系统**: v1.2.0 已实现 ANALYZE TABLE，需集成到优化器
- **测试基准**: 需准备 TPC-H 或自定义基准测试数据

---

## 三、功能列表

### 3.1 P0 - 必须完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| CBO-01 | optimizer | CBO 成本模型基础 | StatsCollector | 代价估算框架 |
| CBO-02 | optimizer | 统计信息集成 | CBO-01 | 从 v1.2 统计信息系统获取数据 |
| CBO-03 | optimizer | Join 顺序优化 | CBO-01 | 基于成本的 Join 重排 |
| CBO-04 | optimizer | 索引选择优化 | CBO-01 | 使用索引 vs 全表扫描 |
| SMJ-01 | executor | SortMergeJoin 算子 | HashJoin | 替代 HashJoin 的新连接算法 |
| SMJ-02 | executor | SortMergeJoin 测试 | SMJ-01 | 单元测试和集成测试 |

### 3.2 P1 - 应该完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| M-003 | observability | Prometheus 指标格式 | Metrics trait | OpenMetrics 兼容格式 | ✅ 已完成 |
| M-004 | observability | /metrics 端点 | M-003 | HTTP 端点暴露指标 | ✅ 已完成 |
| M-005 | observability | Grafana Dashboard 模板 | M-004 | 基础仪表盘 JSON | ✅ 已完成 |
| NLJ-01 | executor | NestedLoopJoin 优化 | - | 支持 Cross Join 和外连接 |
| PB-01 | benchmark | TPC-H 基准测试 | - | 性能基线数据 |

### 3.3 P2 - 可选完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| CBO-05 | optimizer | 代价估算测试 | CBO-04 | 验证优化效果 |
| PB-02 | benchmark | 性能对比报告 | PB-01 | v1.3 vs v1.4 性能对比 |

---

## 四、模块设计

### 4.1 CBO 成本优化器

#### 4.1.1 成本模型

```rust
pub struct CostModel {
    seq_page_cost: f64,
    random_page_cost: f64,
    cpu_tuple_cost: f64,
    cpu_index_tuple_cost: f64,
}

impl CostModel {
    pub fn estimate_scan(&self, table: &TableStats) -> Cost;
    pub fn estimate_index_scan(&self, index: &IndexStats) -> Cost;
    pub fn estimate_join(&self, left: Cost, right: Cost, join_type: JoinType) -> Cost;
}
```

#### 4.1.2 统计信息

```rust
pub struct TableStats {
    row_count: u64,
    page_count: u64,
    column_stats: HashMap<String, ColumnStats>,
}

pub struct ColumnStats {
    null_count: u64,
    n_distinct: u64,
    histogram: Vec<HistogramBin>,
}
```

### 4.2 SortMergeJoin 实现

```rust
pub struct SortMergeJoinExecutor {
    left: Arc<dyn VolcanoExecutor>,
    right: Arc<dyn VolcanoExecutor>,
    join_keys: Vec<Expression>,
    join_type: JoinType,
}

impl VolcanoExecutor for SortMergeJoinExecutor {
    fn execute(&self, context: &mut ExecContext) -> Result<Option<RecordBatch>>;
}
```

### 4.3 Prometheus 指标

```rust
pub struct PrometheusMetricsExporter {
    metrics: Arc<MetricsRegistry>,
}

impl MetricsExporter for PrometheusMetricsExporter {
    fn export(&self) -> Result<String>;
}
```

---

## 五、覆盖率要求

### 5.1 目标矩阵

| 模块 | 当前 | 目标 | 优先级 |
|------|------|------|--------|
| 整体 | 78.88% | ≥80% | P0 |
| optimizer | 82.12% | ≥85% | P0 |
| executor | 87.71% | ≥90% | P1 |
| planner | 76.44% | ≥80% | P1 |

### 5.2 新增测试估算

| 模块 | 需新增测试 |
|------|-----------|
| CBO 成本模型 | 50+ |
| SortMergeJoin | 30+ |
| Prometheus Exporter | 20+ |
| NestedLoopJoin | 20+ |

---

## 六、门禁检查 (Gate Checklist)

### 6.1 构建门禁

| 检查项 | 目标 |
|--------|------|
| cargo build --workspace | ✅ |
| cargo test --workspace | 100% |
| cargo clippy -- -D warnings | 零警告 |
| cargo fmt --all -- --check | 通过 |

### 6.2 覆盖率门禁

| 模块 | 目标 |
|------|------|
| 整体 | ≥80% |
| optimizer | ≥85% |
| executor | ≥90% |

### 6.3 功能门禁

| ID | 检查项 | 说明 |
|----|--------|------|
| CBO-01 | CBO 成本模型 | 基础框架可用 |
| CBO-03 | Join 顺序优化 | 基于成本重排 |
| SMJ-01 | SortMergeJoin | 算子可用 |
| M-003 | Prometheus 格式 | OpenMetrics 兼容 |
| M-004 | /metrics 端点 | HTTP 暴露 |

---

## 七、风险与缓解

### 7.1 技术风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| CBO 复杂度高 | 🔴 高 | 分阶段实现，先基础后优化 |
| 统计信息不准 | ⚠️ 中 | 提供手动 ANALYZE 触发机制 |
| SortMergeJoin 性能 | ⚠️ 中 | 与 HashJoin 性能对比测试 |

### 7.2 进度风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 工作量超预期 | ⚠️ 中 | P2 功能可推迟到 v1.4.x |
| 测试数据不足 | 🔴 高 | 提前准备 TPC-H 数据集 |

---

## 八、评审记录

| 日期 | AI/工具 | 评估结论 |
|------|---------|----------|
| 2026-03-15 | AI Assistant | 初始版本，基于 v1.3.0 规划 |

---

## 九、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-15 | 初始版本 |

---

**文档状态**: 草稿  
**创建人**: AI Assistant  
**审核人**: 待定
