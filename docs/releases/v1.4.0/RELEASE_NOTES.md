# SQLRustGo v1.4.0 发行说明

> **版本**: v1.4.0
> **代号**: CBO & Vectorization Ready
> **发布日期**: 2026-03-16
> **状态**: ✅ 正式发布
> **版本类型**: 🚀 性能增强版 - L5 性能优化级

---

## 版本亮点

v1.4.0 是**性能增强版本**，核心目标是实现 CBO (基于代价的优化器) 和向量化执行基础。

### 🎯 核心成就

- **📈 CBO 基础**: 代价模型 + 统计信息集成
- **⚡ 向量化执行**: SIMD 基础设施 + 批量处理
- **🔗 Join 算法增强**: SortMergeJoin + NestedLoopJoin
- **📊 可观测性**: Prometheus 格式 + /metrics 端点 + Grafana Dashboard
- **✅ 质量门禁**: 编译、测试、clippy 全通过

---

## 架构变更说明

### 架构演进

```
v1.3.0 架构                    v1.4.0 架构 (性能增强)
┌─────────────────────┐       ┌─────────────────────┐
│ HashJoin Only      │       │ HashJoin           │
│ (单一连接算法)     │  ──→  │ + SortMergeJoin    │
│                    │       │ + NestedLoopJoin    │
└─────────────────────┘       └─────────────────────┘
┌─────────────────────┐       ┌─────────────────────┐
│ 简化 CBO           │       │ 代价模型 + 统计信息 │
│ (无成本估算)       │  ──→  │ + 索引选择优化     │
└─────────────────────┘       └─────────────────────┘
┌─────────────────────┐       ┌─────────────────────┐
│ 无向量              │       │ SIMD 基础设施     │
│ (行式执行)         │  ──→  │ + 批量处理        │
└─────────────────────┘       └─────────────────────┘
┌─────────────────────┐       ┌─────────────────────┐
│ /health 端点       │       │ + /metrics 端点    │
│ (基础监控)         │  ──→  │ + Prometheus 格式  │
└─────────────────────┘       │ + Grafana Dashboard│
                              └─────────────────────┘
```

### 模块变更

| 模块 | v1.3.0 | v1.4.0 | 变化类型 |
|------|--------|--------|----------|
| Executor | Volcano Trait | + SortMergeJoin + NestedLoopJoin | 扩展 |
| Optimizer | 规则稳定 | CBO 代价模型 + 统计集成 | 新增 |
| 向量化 | 无 | SIMD 基础设施 | 新增 |
| 可观测性 | Health | + Metrics + Prometheus | 增强 |
| 测试覆盖 | 81.26% | 82%+ | 提升 |

---

## 新增功能

### CBO 优化器 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
| 代价模型基础 | CostModel 结构定义 | ✅ 已完成 |
| 统计信息集成 | stats.rs 与 CBO 集成 | ✅ 已完成 |
| 索引选择优化 | IndexSelect 规则 | ✅ 已完成 |
| Join 顺序优化 | JoinReordering 规则 | ✅ 已存在 |

### Join 算法增强 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
| SortMergeJoin 算子 | 排序合并连接 | ✅ 已完成 |
| SortMergeJoin 测试 | 单元+集成测试 | ✅ 已完成 |
| NestedLoopJoin 算子 | 嵌套循环连接 | ✅ 已完成 |
| Cross Join 支持 | 笛卡尔积 | ✅ 已完成 |
| Outer Join 支持 | Left/Right/Full | ✅ 已完成 |

### 向量化基础 (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
| SIMD 基础设施 | vectorization.rs | ✅ 已完成 |
| 批量处理 | BatchIterator trait | ✅ 已完成 |

### 可观测性增强 (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
| Prometheus 格式 | Metrics 导出 | ✅ 已完成 |
| /metrics 端点 | HTTP 暴露 | ✅ 已完成 |
| Grafana Dashboard | 监控模板 | ✅ 已完成 |

### 性能基准 (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
| TPC-H 基准测试 | 性能基线 | ✅ 已完成 |
| CBO 基准 | 优化效果验证 | ✅ 已完成 |
| v1.4.0 基准报告 | 性能对比 | ✅ 已完成 |

---

## 依赖变更

### 新增依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| actix-web | 4 | HTTP 服务器 |
| actix-rt | 2 | 异步运行时 |

### 内部模块

| 模块 | 变更 |
|------|------|
| sqlrustgo-executor | + vectorization.rs |
| sqlrustgo-optimizer | + cost.rs, 规则增强 |
| sqlrustgo-server | + http_server.rs, metrics_endpoint.rs |

---

## 性能提升

### 预期改进

| 场景 | 预期提升 |
|------|----------|
| 大表 Join | SortMergeJoin 减少内存 |
| 复杂查询 | CBO 代价优化 20-30% |
| 批量处理 | 向量化提升 10-20% |
| 监控集成 | Prometheus 标准化 |

---

## 迁移指南

### 从 v1.3.0 升级

v1.4.0 保持向后兼容，无需代码修改。

### 新增配置

```toml
[dependencies]
sqlrustgo-server = "1.4"
actix-web = "4"
```

### API 使用

```rust
// /metrics 端点
use sqlrustgo_server::metrics_endpoint::{configure_metrics_scope, MetricsRegistry};

// CBO 代价模型
use sqlrustgo_optimizer::cost::CostModel;
```

---

## 已知限制

1. **向量化**: SIMD 基础设施已完成，实际性能提升需后续优化
2. **CBO**: 代价模型为基础版本，复杂查询优化需完善
3. **基准测试**: TPC-H 基准为简化版本

---

## 路线图

### v1.4.x 后续版本

- [ ] 完整向量化执行引擎
- [ ] 代价模型精细化
- [ ] 更多 Join 优化规则

---

## 贡献者

感谢以下贡献者参与 v1.4.0 开发：

- Claude Code (AI Assistant)
- SQLRustGo 团队

---

## 下载

```bash
cargo add sqlrustgo --version 1.4.0
```

或访问 GitHub Releases: https://github.com/minzuuniversity/sqlrustgo/releases

---

**文档版本**: 1.0
**最后更新**: 2026-03-16
