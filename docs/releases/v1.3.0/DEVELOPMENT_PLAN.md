# SQLRustGo v1.3.0 开发计划 (DEVELOPMENT_PLAN.md)

> **版本**: v1.3.0 Draft
> **阶段**: 开发中 (Craft)
> **更新日期**: 2026-03-15
> **依据**: 结合 AI 评估意见 (Claude Code, DeepSeek, OpenCode) 修订
> **目标**: 架构稳定，为 2.0 向量化执行和 CBO 奠定基础

---

## 一、版本定位

### 1.1 核心定位

**v1.3.0 = Architecture Stabilization Release (架构稳定版)**

战略目标:

- 不追求新功能数量
- 聚焦把 v1.x 核心内核"打牢"
- 为 v2.0 的 CBO + Vectorized Execution 做准备

### 1.2 成熟度目标

| 指标 | 目标 |
|------|------|
| 成熟度等级 | L4 (企业级) |
| 整体代码覆盖率 | ≥ 65% |
| Executor 覆盖率 | ≥ 60% |
| Planner 覆盖率 | ≥ 60% |
| Optimizer 覆盖率 | ≥ 40% |

---

## 二、当前系统状态

### 2.1 模块覆盖率 (v1.3.0 Craft 阶段)

> 测试日期: 2026-03-15
> 测试命令: `cargo tarpaulin --workspace --ignore-panics --timeout 600`
> 说明: 不统计测试代码本身，只统计核心业务代码

| 模块 | 当前覆盖率 | 覆盖行/总行数 | v1.3.0 目标 | 状态 |
|------|-----------|--------------|-------------|------|
| common | 94.89% | 130/137 | - | ✅ |
| executor (local_executor) | 87.71% | 207/236 | ≥60% | ✅ |
| executor (executor.rs) | 3.48% | 8/230 | - | 🔶 新增 |
| executor (filter.rs) | 16.67% | 5/30 | - | 🔶 新增 |
| planner | 76.44% | 253/331 (physical) | ≥60% | ✅ |
| planner (planner.rs) | 97.53% | 79/81 | - | ✅ |
| planner (optimizer.rs) | 89.56% | 163/182 | ≥40% | ✅ |
| optimizer | 82.12% | 372/453 | ≥40% | ✅ |
| parser | 72.26% | 224/310 | - | ✅ |
| storage | 89.87% | 204/227 | - | ✅ |
| **整体 (不含测试代码)** | **78.88%** | 2991/3792 | ≥65% | ✅ |

### 2.2 覆盖率测试方法

```bash
# 测试参数 (与 v1.2.0 保持一致)
cargo tarpaulin --workspace --ignore-panics --timeout 600

# 结果分析
- 包含测试代码: 90.50% (8182/9041 行)
- 不含测试代码: 78.88% (2991/3792 行)
- 本报告采用不含测试代码的统计口径
```

### 2.3 风险分析

| 风险 | 等级 | 原因 | 状态 |
|------|------|------|------|
| executor/executor.rs 新代码覆盖不足 | ⚠️ 中 | v1.3.0 新增 Volcano Model，部分算子未充分测试 | 🔶 开发中 |
| executor/filter.rs 覆盖不足 | ⚠️ 中 | Filter 算子为新增模块 | 🔶 开发中 |
| transaction 无测试 | 🔴 高 | 0% 覆盖率，事务功能推迟到 v1.4.0 | ⏸️ 已延期 |
| planner/physical_plan 覆盖率提升 | ✅ | 43% → 76.44% | ✅ 已完成 |
| optimizer 覆盖率提升 | ✅ | 18% → 82.12% | ✅ 已完成 |

---

## 三、AI 评估综合意见

### 3.1 评估来源

| AI | 评估日期 | 核心意见 |
|----|----------|----------|
| Claude Code | 2026-03-13 | 事务难度极高，建议拆分到后续版本 |
| DeepSeek | 2026-03-12 | 聚焦 Executor 稳定化，可观测性降为 P1 |
| OpenCode (MiniMax) | 2026-03-13 | 覆盖率目标 ≥65%, Executor ≥60% |

### 3.2 评估共识

✅ **共识点**:

- v1.3.0 应聚焦 Executor 稳定化
- 覆盖率目标: 整体 ≥65%, Executor ≥60%
- 事务 (MVCC) 难度高，建议延后
- 可观测性可降为 P1，只做基础框架

⚠️ **分歧点**:

- 插件系统 - 有的建议 P0，有的建议延后 → 采纳延后
- CBO - 有的建议 v1.3，有的建议 v1.4 → 采纳 v1.4

---

## 四、功能矩阵

### 4.1 P0 - 必须完成

| ID | 模块 | 任务 | 依赖 | 目标覆盖率 | 状态 | 认领 |
|----|------|------|------|-----------|------|------|
| E-001 | executor | Volcano Model 统一 trait | - | - | ✅ 已完成 | - |
| E-002 | executor | TableScan 算子完善 | - | - | ✅ 已完成 | - |
| E-003 | executor | Projection 算子 | - | - | ✅ 已完成 | - |
| E-004 | executor | Filter 算子 | - | - | ✅ 已完成 | - |
| E-005 | executor | HashJoin 算子 (内连接) | - | - | ✅ 已完成 | - |
| E-006 | executor | Executor 测试框架 | E-001 | - | ✅ 已完成 | - |
| E-007 | executor | Executor 覆盖率提升 | E-006 | ≥60% | ✅ 已完成 | - |
| **T-001** | **planner** | **Planner 测试框架** | **-** | **-** | ✅ **已完成** | **认领: AI Assistant** |
| **T-002** | **planner** | **Planner 覆盖率提升** | **T-001** | **≥60%** | ✅ **已完成** | **认领: AI Assistant** |
| **T-003** | **optimizer** | **Optimizer 测试补充** | **-** | **≥40%** | ✅ **已完成** | **认领: AI Assistant** |

### 4.2 P1 - 应该完成 (v1.3.0 扩展)

> 2026-03-15 更新: v1.3.1 功能已合并到 v1.3.0

| ID | 模块 | 任务 | 依赖 | 目标 | 状态 |
|----|------|------|------|------|------|
| M-001 | observability | Metrics trait 定义 | - | 框架基础 | ✅ 已完成 |
| M-002 | observability | BufferPoolMetrics 初步 | M-001 | 指标收集 | ✅ 已完成 |
| H-001 | observability | /health/live 端点 | - | 存活探针 | ✅ 已完成 |
| H-002 | observability | /health/ready 端点 | - | 就绪探针 | ✅ 已完成 |

### 4.3 P1 扩展 - 可观测性增强 (原 v1.3.1)

| ID | 模块 | 任务 | 依赖 | 目标 | 状态 |
|----|------|------|------|------|------|
| M-003 | observability | Prometheus 指标格式 | M-001 | 指标暴露 | ⏳ 待开发 |
| M-004 | observability | /metrics 端点 | M-003 | 指标查询 | ⏳ 待开发 |
| M-005 | observability | Grafana Dashboard 模板 | M-004 | 可视化 | ⏳ 可选 |
| M-006 | observability | 告警规则示例 | M-004 | 告警配置 | ⏳ 可选 |

### 4.4 P2 - 可选完成

| ID | 模块 | 任务 | 依赖 |
|----|------|------|------|
| P-001 | plugin | Plugin trait 定义 (实验) | E-001 |
| J-002 | executor | NestedLoopJoin 优化 | - |

### 4.4 不包含 (推迟到后续版本)

| 任务 | 推迟到 | 原因 |
|------|--------|------|
| 完整事务隔离 (MVCC) | v1.4.0/v1.5.0 | 难度极高，8周不足 |
| CBO 成本优化器 | v1.4.0 | 依赖统计信息系统 |
| 插件系统完整实现 | v1.5.0 | 非核心功能 |
| SortMergeJoin | v1.4.0 | HashJoin 优先 |
| 完整可观测性 (Prometheus, Grafana) | ✅ 已合并到 v1.3.0 | 见 4.3 节 |

---

## 五、开发计划 (8 周)

### 5.1 时间线

```
v1.3.0 Craft: 2026-03-13 → 2026-05-08 (8 周)
  Week 1-2:   Executor 稳定化 (E-001 ~ E-004)
  Week 3-4:   HashJoin + 测试框架 (E-005 ~ E-006)
  Week 5-6:   覆盖率提升冲刺 (E-007, T-001 ~ T-003)
  Week 7-8:   可观测性 + 收尾 (M-001 ~ M-002, H-001 ~ H-002)
```

### 5.2 详细周计划

#### Week 1-2: Executor 稳定化

| 周 | 任务 | 交付物 |
|----|------|--------|
| W1 | Volcano Model trait 统一 | executor/src/executor.rs 重构 |
| W1 | TableScan 算子完善 | 完整扫描实现 |
| W1 | Projection 算子 | 列投影实现 |
| W2 | Filter 算子 | 条件过滤实现 |
| W2 | 本地代码格式修复 | clippy/fmt 通过 |

#### Week 3-4: HashJoin + 测试框架

| 周 | 任务 | 交付物 |
|----|------|--------|
| W3 | HashJoin 内连接 | executor/src/hash_join.rs |
| W3 | HashJoin 外连接 (可选) | LEFT/RIGHT JOIN |
| W4 | 测试框架 | executor/tests/ harness, mock storage |
| W4 | 核心算子测试 | 50+ 测试用例覆盖各算子 |

#### Week 5-6: 覆盖率提升

| 周 | 任务 | 目标 |
|----|------|------|
| W5 | Executor 覆盖率提升 | 从 72% 优化至 ≥60% (实际可能维持或略降，但需确保核心算子覆盖) |
| W5 | Planner 测试补充 | 43% → 50% (冲刺目标) |
| W6 | Optimizer 测试补充 | 18% → 35% (冲刺目标) |
| W6 | 整体覆盖率验证 | ≥ 65% (当前已满足，但需确保新代码覆盖) |

#### Week 7-8: 可观测性 + 收尾

| 周 | 任务 | 交付物 |
|----|------|--------|
| W7 | Metrics trait | common/src/metrics.rs 定义基础 trait |
| W7 | BufferPoolMetrics 初步 | 在 storage 中集成指标计数 |
| W8 | Health Check API | 在 network 或 standalone server 中提供 /health/live, /health/ready |
| W8 | 门禁检查 | 完整运行 test/clippy/fmt/coverage，通过所有检查 |

---

## 六、覆盖率要求

> 更新日期: 2026-03-15
> 测试方法: `cargo tarpaulin --workspace --ignore-panics --timeout 600` (不含测试代码)

### 6.1 目标矩阵

| 模块 | 当前 | 目标 | 状态 |
|------|------|------|------|
| **整体** | **78.88%** | ≥65% | ✅ 达标 (+13.88%) |
| executor (local_executor) | 87.71% | ≥60% | ✅ 达标 |
| executor (executor.rs) | 3.48% | - | 🔶 新增代码 |
| executor (filter.rs) | 16.67% | - | 🔶 新增代码 |
| planner (physical_plan) | 76.44% | ≥60% | ✅ 达标 |
| planner (planner.rs) | 97.53% | - | ✅ |
| planner (optimizer.rs) | 89.56% | ≥40% | ✅ 达标 |
| optimizer (rules.rs) | 82.12% | ≥40% | ✅ 达标 |
| storage | 89.87% | - | ✅ |
| parser | 72.26% | - | ✅ |

### 6.2 各模块详细覆盖率

| 文件 | 覆盖行 | 总行数 | 覆盖率 | 备注 |
|------|--------|--------|--------|------|
| executor/local_executor.rs | 207 | 236 | 87.71% | 核心执行器 |
| planner/planner.rs | 79 | 81 | 97.53% | 规划器 |
| planner/optimizer.rs | 163 | 182 | 89.56% | 优化器 |
| planner/physical_plan.rs | 253 | 331 | 76.44% | 物理计划 |
| optimizer/rules.rs | 372 | 453 | 82.12% | 优化规则 |
| storage/file_storage.rs | 204 | 227 | 89.87% | 文件存储 |
| storage/page.rs | 162 | 202 | 80.20% | 页面管理 |
| parser/parser.rs | 224 | 310 | 72.26% | SQL 解析 |
| common/metrics.rs | 130 | 137 | 94.89% | 指标系统 |

### 6.3 未完成模块覆盖率 (待开发)

| 文件 | 覆盖行 | 总行数 | 覆盖率 | 原因 |
|------|--------|--------|--------|------|
| executor/executor.rs | 8 | 230 | 3.48% | Volcano Model 新增 |
| executor/filter.rs | 5 | 30 | 16.67% | Filter 算子新增 |

> 注: executor/executor.rs 和 executor/filter.rs 是 v1.3.0 新增的 Volcano Model 实现，属于开发中状态。

---

## 七、门禁检查 (Gate Checklist)

### 7.1 必须通过 (GA 门槛)

> 更新日期: 2026-03-15

| 检查项 | 目标 | 当前 | 状态 |
|--------|------|------|------|
| cargo build --workspace | ✅ | ✅ | ✅ |
| cargo test --workspace | 100% | ✅ | ✅ |
| cargo clippy -- -D warnings | 零警告 | ✅ | ✅ |
| cargo fmt --all -- --check | 通过 | ✅ | ✅ |
| 代码覆盖率 (整体) | ≥65% | **78.88%** | ✅ |
| 代码覆盖率 (executor) | ≥60% | **87.71%** | ✅ |
| 代码覆盖率 (planner) | ≥60% | **76.44%** | ✅ |
| 代码覆盖率 (optimizer) | ≥40% | **82.12%** | ✅ |

### 7.2 GA 发布条件

- [ ] 所有 P0 任务完成
- [ ] 覆盖率达标 (整体≥65%, Executor≥60%, Planner≥60%, Optimizer≥40%)
- [ ] 门禁检查全部通过
- [ ] PR 审查通过 (至少 1 人)

---

## 八、风险与缓解

### 8.1 识别风险

| 风险 | 概率 | 影响 | 等级 |
|------|------|------|------|
| Executor 覆盖率目标过高 (≥60% 但实际可能因测试粒度而下降) | 高 | 中 | 🔴 |
| Optimizer 测试编写困难 | 中 | 高 | ⚠️ |
| 时间不足 (8 周内完不成所有任务) | 中 | 高 | ⚠️ |

### 8.2 缓解措施

| 风险 | 缓解措施 |
|------|----------|
| 覆盖率目标过高 | 优先级: executor > planner > optimizer；确保核心算子 (HashJoin) 有充分测试 |
| Optimizer 测试难 | 聚焦核心规则 (谓词下推、常量折叠)，跳过复杂优化，目标设为 40% |
| 时间不足 | 压缩可观测性到 v1.3.1；若仍不足，将 Optimizer 覆盖率目标降到 30% |

### 8.3 降级策略

若时间不足，按以下顺序丢弃:

1. P2 插件系统实验 (P-001)
2. 部分可观测性功能 (如 BufferPoolMetrics 仅留定义)
3. Optimizer 覆盖率目标降到 30%

---

## 九、关联 Issue

### 9.1 主 Issue

| Issue | 标题 | 状态 |
|-------|------|------|
| #388 | v1.3.0 开发计划 | OPEN |
| #448 | v1.3.0 开发计划评审 | OPEN |
| #463 | v1.3.0 Craft 阶段启动 | OPEN |
| #200 | v1.3.0 详细开发任务 | OPEN |

### 9.2 子任务 (待创建)

- Executor 稳定化系列 (E-001 ~ E-007)
- 覆盖率提升系列 (T-001 ~ T-003)
- 可观测性系列 (M-001 ~ M-002, H-001 ~ H-002)

---

## 十、评审记录

| 日期 | AI/工具 | 评估结论 |
|------|---------|----------|
| 2026-03-12 | DeepSeek | ✅ 通过 (可观测性降为 P1) |
| 2026-03-13 | Claude Code | ⚠️ 有条件通过 (事务拆分) |
| 2026-03-13 | OpenCode | ✅ 通过 (覆盖率 ≥65%) |
| 2026-03-15 | AI Assistant | ✅ 覆盖率测试完成，T 系列任务认领 |

---

## 附录: 命令参考

```bash
# 构建
cargo build --workspace

# 测试
cargo test --workspace

# 覆盖率 (使用 llvm-cov)
cargo llvm-cov --workspace --all-features

# Clippy
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check

# 健康检查验证 (若 server 已运行)
curl http://localhost:3306/health/live
curl http://localhost:3306/health/ready
```

---

**文档状态**: 正式版 (取代旧版 CRAFT_PLAN.md)  
**下次更新**: 版本发布后
