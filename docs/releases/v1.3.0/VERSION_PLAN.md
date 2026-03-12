# SQLRustGo v1.3.0 版本计划

> **版本**: v1.3
> **制定日期**: 2026-03-12
> **更新依据**: VERSION_ROADMAP.md (v1.1)
> **核心原则**: 聚焦稳定，暂不扩展功能

---

## 一、版本概述

### 1.1 版本目标

| 项目 | 值 |
|------|-----|
| **版本号** | v1.3.0 |
| **阶段** | Draft |
| **成熟度目标** | L4 |
| **核心目标** | 架构稳定 + Expression Engine |
| **预计时间** | 2026-04 - 2026-07 (3-4个月) |

### 1.2 版本原则

> **重要**: v1.3.0 是整个路线图的基础，必须建立 Expression Engine，否则后续算子无法实现。

### 1.3 前置依赖

- ✅ v1.2.0 GA 发布

### 1.4 真实工程量估算

| 模块 | 真实时间 |
|------|----------|
| Expression Engine | 1-2 周 |
| Volcano trait | 1-2 天 |
| TableScan | 2-3 天 |
| Projection | 1 天 |
| Filter | 1-2 天 |
| HashJoin | 3-5 天 |
| 测试框架 | 2-3 天 |
| 覆盖率提升 | 1-2 周 |

**总计**: 约 3-4 周 (而非 3 周!)

---

## 二、开发轨道

### 轨道 A: Expression Engine (P0 - 关键!)

```
Week 1-2:
├── X-001: Expression trait 定义
├── X-002: Literal expression
├── X-003: Column expression
├── X-004: Binary expression (+, -, *, /, =, <, >, etc.)
└── X-005: Expression evaluator

这是 Filter, Projection, HashJoin 的基础!
```

### 轨道 B: Executor 稳定化 (P0)

```
Week 3-4:
├── E-001: Volcano Model trait 统一
├── E-002: TableScan 算子
├── E-003: Projection 算子
└── E-004: Filter 算子 (依赖 Expression Engine!)

Week 5:
├── E-005: HashJoin 算子 (依赖 Expression + 哈希表)
└── E-006: Executor 测试框架
```

### 轨道 C: 覆盖率提升 (P0)

| 模块 | 目标覆盖率 |
|------|-----------|
| 整体 | ≥65% |
| Executor | ≥60% |

### 轨道 D: 可观测性基础 (P1)

```
Week 6:
├── M-001: Metrics trait 定义
├── M-002: BufferPoolMetrics 实现
├── H-001: /health/live 端点
└── H-002: /health/ready 端点

实现方式: 使用现有 HTTP 框架 (hyper/axum) 嵌入到网络层
```

---

## 三、任务清单

### P0 - 必须完成

| ID | 任务 | 预估时间 | 状态 |
|----|------|----------|------|
| E-001 | Volcano Model trait 统一 | 4h | ⏳ |
| E-002 | TableScan 实现 | 8h | ⏳ |
| E-003 | Projection 实现 | 4h | ⏳ |
| E-004 | Filter 实现 | 4h | ⏳ |
| E-005 | HashJoin 实现 | 8h | ⏳ |
| E-006 | 测试框架 | 8h | ⏳ |
| E-007 | 单元测试 | 16h | ⏳ |
| E-008 | 覆盖率提升 | 8h | ⏳ |

### P1 - 应该完成

| ID | 任务 | 预估时间 | 状态 |
|----|------|----------|------|
| M-001 | Metrics trait | 4h | ⏳ |
| M-002 | BufferPoolMetrics | 4h | ⏳ |
| H-001 | /health/live | 2h | ⏳ |
| H-002 | /health/ready | 2h | ⏳ |

---

## 四、推迟的任务

以下任务不在 v1.3.0 范围，推迟到后续版本：

| 任务 | 推迟到 | 原因 |
|------|--------|------|
| 插件系统 (P-001 ~ P-005) | v1.5.0 | 稳定性优先 |
| CBO 完善 (C-001 ~ C-005) | v1.4.0/v1.5.0 | 依赖统计信息 |
| 完整事务隔离 (T-001 ~ T-004) | v1.5.0 | 复杂度高 |
| Join 算法演进 (J-001 ~ J-004) | v1.4.0 | HashJoin 已实现 |
| 性能基准框架 | v1.4.0 | 依赖核心稳定 |

---

## 五、里程碑

```
v1.3.0-draft    ──────────────────────────────────────────────►
    │ (2026-04-01)
    │ 目标: 架构设计完成
    │
v1.3.0-alpha   ──────────────────────────────────────────────►
    │ (2026-04-15)
    │ 目标: 核心算子可用
    │
v1.3.0-beta    ──────────────────────────────────────────────►
    │ (2026-05-15)
    │ 目标: 测试完成，覆盖率达标
    │
v1.3.0-rc      ──────────────────────────────────────────────►
    │ (2026-06-01)
    │ 目标: 稳定性验证
    │
v1.3.0 GA      ──────────────────────────────────────────────
    │ (2026-06-15)
    │
```

| 阶段 | 目标日期 | 门禁 |
|------|----------|------|
| Draft | 2026-04-15 | 架构设计完成 |
| Alpha | 2026-05-01 | 核心算子可用 |
| Beta | 2026-05-15 | 覆盖率 ≥65% |
| RC | 2026-06-01 | 测试 100% 通过 |
| GA | 2026-06-15 | 发布审批 |

---

## 六、验收标准

### 6.1 功能验收

- [ ] Volcano Model 统一 trait
- [ ] TableScan 可用
- [ ] Projection 可用
- [ ] Filter 可用
- [ ] HashJoin 可用

### 6.2 覆盖率验收

| 模块 | 目标 | 测量方法 |
|------|------|----------|
| 整体 | ≥65% | tarpaulin |
| Executor | ≥60% | tarpaulin |
| Planner | ≥60% | tarpaulin |

### 6.3 可观测性验收

- [ ] /health/live 返回 200
- [ ] /health/ready 返回 200
- [ ] BufferPool 指标收集

### 6.4 质量验收

- [ ] Clippy 零警告
- [ ] cargo fmt 通过
- [ ] cargo test 通过

---

## 七、技术设计

### 7.1 Executor Trait

```rust
pub trait Executor: Send {
    fn open(&mut self);
    fn next(&mut self) -> Option<Record>;
    fn close(&mut self);
}
```

### 7.2 Metrics Trait

```rust
pub trait Metrics: Send + Sync {
    fn name(&self) -> &str;
    fn collect(&self) -> Vec<MetricSample>;
}
```

### 7.3 健康检查端点

实现位置: `crates/server/src/health.rs`

使用方式: 嵌入到现有 HTTP 服务器

---

## 八、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 算子实现复杂度 | 中 | 分步实现，先简单后复杂 |
| 覆盖率目标挑战 | 中 | 预留测试时间 |
| 可观测性架构冲突 | 低 | 使用独立模块 |

---

## 九、关联文档

| 文档 | 说明 |
|------|------|
| [VERSION_ROADMAP.md](../VERSION_ROADMAP.md) | 版本演进总览 |
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发详细计划 |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁清单 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-05 | 初始版本 |
| 1.1 | 2026-03-12 | 对齐 VERSION_ROADMAP，推迟插件/CBO/事务 |

---

*制定日期: 2026-03-12*
*更新依据: DeepSeek 评估意见*
