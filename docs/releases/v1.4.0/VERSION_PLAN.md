# SQLRustGo v1.4.0 版本计划 (VERSION_PLAN.md)

> **版本**: v1.4.0 Draft
> **主题**: 查询优化与执行增强
> **发布日期**: 预计 2026-05-10
> **更新日期**: 2026-03-15

---

## 一、版本主题

**查询优化与执行增强**

v1.4.0 专注于提升查询性能，通过 CBO 成本优化器和 SortMergeJoin 算子提供更智能的执行计划。

---

## 二、轨道规划

### 2.1 轨道 A: 查询优化 (必须)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| CBO 成本模型 | 基于代价的查询优化器框架 | P0 | ⏳ |
| 统计信息集成 | 从 ANALYZE 获取表统计信息 | P0 | ⏳ |
| Join 顺序优化 | 基于成本的 Join 重排 | P0 | ⏳ |
| 索引选择 | 智能选择索引 vs 全表扫描 | P0 | ⏳ |

### 2.2 轨道 B: 执行增强 (必须)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| SortMergeJoin | 新的连接算法 (替代 HashJoin) | P0 | ⏳ |
| NestedLoopJoin | 支持 Cross Join 和外连接 | P1 | ⏳ |

### 2.3 轨道 C: 可观测性增强 (重要)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| Prometheus 格式 | OpenMetrics 兼容指标格式 | P1 | ✅ |
| /metrics 端点 | HTTP 暴露指标 | P1 | ✅ |
| Grafana Dashboard | 基础仪表盘模板 | P2 | ✅ |

### 2.4 轨道 D: 基准测试 (可选)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| TPC-H 基准 | 性能基线测试 | P1 | ⏳ |
| 性能对比报告 | v1.3 vs v1.4 性能对比 | P2 | ⏳ |

---

## 三、功能详情

### 3.1 CBO 成本优化器

#### 3.1.1 成本模型

- **seq_page_cost**: 顺序扫描页面成本 (默认 1.0)
- **random_page_cost**: 随机扫描页面成本 (默认 4.0)
- **cpu_tuple_cost**: 行处理 CPU 成本 (默认 0.01)
- **cpu_index_tuple_cost**: 索引元组 CPU 成本 (默认 0.005)

#### 3.1.2 优化规则

1. **Join 重排**: 使用动态规划选择最优 Join 顺序
2. **索引选择**: 基于成本选择是否使用索引
3. **扫描方式**: SeqScan vs IndexScan vs IndexOnlyScan

### 3.2 SortMergeJoin

- **输入**: 两个已排序的关系
- **算法**: 双指针归并
- **适用场景**: 大数据集、有序输入、外部排序

### 3.3 Prometheus 指标

```
# HELP sqlrustgo_queries_total Total number of queries
# TYPE sqlrustgo_queries_total counter
sqlrustgo_queries_total{type="select"} 1234

# HELP sqlrustgo_query_duration_seconds Query duration in seconds
# TYPE sqlrustgo_query_duration_seconds histogram
sqlrustgo_query_duration_seconds_bucket{le="0.005"} 100
```

---

## 四、API 变更

### 4.1 新增公开 API

```rust
// optimizer
pub mod optimizer {
    pub struct CostModel;
    pub struct CostEstimator;
    pub struct PhysicalPlanner;
}

// executor  
pub mod executor {
    pub struct SortMergeJoinExecutor;
    pub struct NestedLoopJoinExecutor;
}

// observability
pub mod observability {
    pub struct PrometheusExporter;
    pub struct MetricsServer;
}
```

### 4.2 配置变更

```toml
[optimizer]
enable_cbo = true
default_plan_mode = "auto"  # auto, rule_based, cost_based

[costs]
seq_page_cost = 1.0
random_page_cost = 4.0
cpu_tuple_cost = 0.01

[metrics]
enabled = true
endpoint = "/metrics"
format = "prometheus"
```

---

## 五、性能目标

| 指标 | v1.3.0 基线 | v1.4.0 目标 | 说明 |
|------|-------------|-------------|------|
| TPC-H Q1 | 1.0x | 1.2x | 成本优化效果 |
| Join 重排 | N/A | 启用 | 多表 Join 优化 |
| 指标采集开销 | <1ms | <1ms | Prometheus 无明显开销 |

---

## 六、测试计划

### 6.1 单元测试

- CostModel 单元测试
- SortMergeJoin 边界测试
- 统计信息集成测试

### 6.2 集成测试

- CBO 端到端测试
- 多表 Join 优化测试
- Prometheus 指标集成测试

### 6.3 性能测试

- TPC-H 基准测试 (至少 Q1-Q6)
- 内存使用 profiling
- 延迟分布测试

---

## 七、文档更新

### 7.1 用户文档

- CBO 使用指南
- 性能调优手册
- Prometheus 集成指南

### 7.2 开发者文档

- CBO 架构设计
- 成本模型参数说明
- 基准测试运行指南

---

## 八、发布检查清单

- [ ] cargo build --workspace
- [ ] cargo test --workspace
- [ ] cargo clippy -- -D warnings
- [ ] 覆盖率 ≥80%
- [ ] TPC-H 基准测试通过
- [ ] API 文档更新
- [ ] CHANGELOG.md 更新

---

## 九、预计时间线

| 阶段 | 周 | 日期 | 主要任务 |
|------|-----|------|----------|
| 规划 | 1 | 03/15-03/21 | CBO 框架设计 |
| CBO | 2-3 | 03/22-04/04 | CBO 核心实现 |
| Join | 4 | 04/05-04/11 | SortMergeJoin |
| 监控 | 5 | 04/12-04/18 | Prometheus 集成 |
| 测试 | 6 | 04/19-04/25 | 测试和修复 |
| 优化 | 7 | 04/26-05/02 | 性能调优 |
| 发布 | 8 | 05/03-05/10 | 文档和发布 |

---

## 十、风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| CBO 实现复杂度 | 高 | 高 | 分阶段交付 |
| SortMergeJoin 性能不达预期 | 中 | 中 | 保持 HashJoin 作为备选 |
| 测试数据不足 | 中 | 高 | 提前准备 TPC-H 数据 |

---

**文档状态**: 草稿  
**创建人**: AI Assistant  
**审核人**: 待定
