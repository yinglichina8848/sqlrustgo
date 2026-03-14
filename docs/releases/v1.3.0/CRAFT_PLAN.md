# v1.3.0 Craft 阶段规划文档

> **版本**: v1.3.0 Draft  
> **阶段**: Craft  
> **更新日期**: 2026-03-13  
> **依据**: Claude Code + DeepSeek + OpenCode (MiniMax-M2.1) 评估意见

---

## 一、版本定位

### 1.1 核心定位

**v1.3.0 = Architecture Stabilization Release (架构稳定版)**

**战略目标**:
- 不追求新功能数量
- 聚焦把 v1.x 核心内核"打牢"
- 为 v2.0 的 CBO + Vectorized Execution 做准备

### 1.2 成熟度目标

| 指标 | 目标 |
|------|------|
| 成熟度等级 | L4 (企业级) |
| 整体覆盖率 | ≥ 65% |
| Executor 覆盖率 | ≥ 60% |
| Planner 覆盖率 | ≥ 60% |
| Optimizer 覆盖率 | ≥ 40% |

---

## 二、当前系统状态

### 2.1 模块覆盖率 (v1.2.0 GA)

| 模块 | 当前覆盖率 | 行数 | 风险等级 |
|------|-----------|------|----------|
| common | 100% | 4/4 | ✅ 安全 |
| catalog | 14% | 1/7 | 🔴 高 |
| types | 38% | 26/68 | ⚠️ 中 |
| parser | 75% | 477/637 | ✅ 安全 |
| storage | 58% | 704/1208 | ⚠️ 可接受 |
| planner | 43% | 535/1238 | ⚠️ 中 |
| optimizer | 18% | 567/3119 | 🔴 高 |
| executor | 72% | 167/231 | ⚠️ 中 |
| transaction | 0% | 0/26 | 🔴 极高 |
| **整体** | **80.30%** | 2433/3030 | ✅ |

### 2.2 风险分析

| 风险 | 等级 | 原因 |
|------|------|------|
| executor 测试不完整 | 🔴 高 | 覆盖率仅 72%，核心算子需完善 |
| optimizer 覆盖率低 | 🔴 高 | 仅 18%，规则未充分测试 |
| transaction 无测试 | 🔴 极高 | 0% 覆盖率 |
| planner 覆盖率不足 | ⚠️ 中 | 43%，需提升到 60% |

---

## 三、AI 评估综合意见

### 3.1 评估来源

| AI | 评估日期 | 核心意见 |
|----|----------|----------|
| Claude Code | 2026-03-13 | 事务难度极高，建议拆分到 v1.3.1 |
| DeepSeek | 2026-03-12 | 聚焦 Executor 稳定化，可观测性降为 P1 |
| OpenCode (MiniMax) | 2026-03-13 | 覆盖率目标 ≥65%，Executor ≥60% |

### 3.2 评估共识

✅ **共识点**:
1. v1.3.0 应聚焦 **Executor 稳定化**
2. 覆盖率目标: 整体 ≥65%, Executor ≥60%
3. 事务 (MVCC) 难度高，建议延后
4. 可观测性可降为 P1

⚠️ **分歧点**:
1. 插件系统 - 有的建议 P0，有的建议延后
2. CBO - 有的建议 v1.3，有的建议 v1.4

---

## 四、功能矩阵

### 4.1 P0 - 必须完成

| ID | 模块 | 任务 | 依赖 | 目标覆盖率 |
|----|------|------|------|-----------|
| E-001 | executor | Volcano Model 统一 trait | - | - |
| E-002 | executor | TableScan 算子完善 | - | - |
| E-003 | executor | Projection 算子 | - | - |
| E-004 | executor | Filter 算子 | - | - |
| E-005 | executor | HashJoin 算子 | - | - |
| E-006 | executor | Executor 测试框架 | E-001 | - |
| E-007 | executor | Executor 覆盖率提升 | E-006 | ≥60% |
| T-001 | planner | Planner 测试框架 | - | - |
| T-002 | planner | Planner 覆盖率提升 | T-001 | ≥60% |
| T-003 | optimizer | Optimizer 测试补充 | - | ≥40% |

### 4.2 P1 - 应该完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| M-001 | observability | Metrics trait 定义 | - | 框架基础 |
| M-002 | observability | BufferPoolMetrics | M-001 | 指标收集 |
| H-001 | observability | /health/live | - | 存活探针 |
| H-002 | observability | /health/ready | - | 就绪探针 |

### 4.3 P2 - 可选完成

| ID | 模块 | 任务 | 依赖 |
|----|------|------|------|
| P-001 | plugin | Plugin trait 定义 | E-001 |
| P-002 | plugin | 插件加载器基础 | P-001 |

### 4.4 不包含 (推迟到后续版本)

| 任务 | 推迟到 | 原因 |
|------|--------|------|
| 完整事务隔离 (MVCC) | v1.4.0/v1.5.0 | 难度极高，8周不足 |
| CBO 成本优化器 | v1.4.0 | 依赖统计信息系统 |
| 插件系统完整实现 | v1.5.0 | 非核心功能 |
| SortMergeJoin | v1.4.0 | HashJoin 优先 |

---

## 五、开发计划 (8 周)

### 5.1 时间线

```
v1.3.0 Craft: 2026-03-13 → 2026-05-08 (8 周)
  Week 1-2:   Executor 稳定化 (E-001 ~ E-004)
  Week 3-4:   Executor 测试框架 + HashJoin (E-005 ~ E-006)
  Week 5-6:   覆盖率提升冲刺 (E-07, T-001 ~ T-003)
  Week 7-8:   可观测性 + 收尾 (M-001 ~ M-002, H-001 ~ H-002)
```

### 5.2 详细周计划

#### Week 1-2: Executor 稳定化

| 周 | 任务 | 交付物 |
|----|------|--------|
| W1 | Volcano Model trait 统一 | `executor/src/executor.rs` 重构 |
| W1 | TableScan 算子完善 | 完整扫描实现 |
| W1 | Projection 算子 | 列投影实现 |
| W2 | Filter 算子 | 条件过滤实现 |
| W2 | 本地代码格式修复 | clippy/fmt 通过 |

#### Week 3-4: HashJoin + 测试框架

| 周 | 任务 | 交付物 |
|----|------|--------|
| W3 | HashJoin 内连接 | `executor/src/hash_join.rs` |
| W3 | HashJoin 外连接 | LEFT/RIGHT/FULL JOIN |
| W4 | 测试框架 | `executor/tests/` harness |
| W4 | 核心算子测试 | 50+ 测试用例 |

#### Week 5-6: 覆盖率提升

| 周 | 任务 | 目标 |
|----|------|------|
| W5 | Executor 覆盖率提升 | 45% → 55% |
| W5 | Planner 测试补充 | 43% → 50% |
| W6 | Optimizer 测试补充 | 18% → 35% |
| W6 | 整体覆盖率验证 | ≥ 65% |

#### Week 7-8: 可观测性 + 收尾

| 周 | 任务 | 交付物 |
|----|------|--------|
| W7 | Metrics trait | `common/src/metrics.rs` |
| W7 | BufferPoolMetrics | 缓冲池指标收集 |
| W8 | Health Check API | `/health/live`, `/health/ready` |
| W8 | 门禁检查 | test/clippy/fmt/coverage |

---

## 六、覆盖率要求

### 6.1 目标矩阵

| 模块 | 当前 | 目标 | 差距 | 优先级 |
|------|------|------|------|--------|
| **整体** | 80.30% | **65%** | ✅ 达标 | - |
| executor | 72% | **60%** | -12% | P0 |
| planner | 43% | **60%** | +17% | P0 |
| optimizer | 18% | **40%** | +22% | P1 |
| transaction | 0% | - | - | 延后 |
| storage | 58% | - | ✅ | - |
| parser | 75% | - | ✅ | - |

### 6.2 测试数量估算

| 模块 | 当前测试 | 目标测试 | 需新增 |
|------|---------|---------|--------|
| executor | 29 | 80+ | 50+ |
| planner | 255 | 350+ | 100+ |
| optimizer | 186 | 250+ | 70+ |
| **合计** | 470 | 680+ | 220+ |

---

## 七、门禁检查 (Gate Checklist)

### 7.1 必须通过 (GA 门槛)

| 检查项 | 目标 | 当前 | 状态 |
|--------|------|------|------|
| cargo build | ✅ | ✅ | ⏳ |
| cargo test | 100% | ✅ | ⏳ |
| cargo clippy | 零警告 | ✅ | ⏳ |
| cargo fmt | 通过 | ✅ | ⏳ |
| 覆盖率 | ≥65% | 80.30% | ⏳ |

### 7.2 GA 发布条件

- [ ] 所有 P0 任务完成
- [ ] 覆盖率达标 (整体≥65%, Executor≥60%, Planner≥60%)
- [ ] 门禁检查全部通过
- [ ] PR 审查通过

---

## 八、风险与缓解

### 8.1 识别风险

| 风险 | 概率 | 影响 | 等级 |
|------|------|------|------|
| Executor 覆盖率目标过高 | 高 | 中 | 🔴 |
| Optimizer 测试编写困难 | 中 | 高 | ⚠️ |
| 时间不足 | 中 | 高 | ⚠️ |

### 8.2 缓解措施

| 风险 | 缓解措施 |
|------|----------|
| 覆盖率目标过高 | 优先级: executor > planner > optimizer |
| Optimizer 测试难 | 聚焦核心规则，跳过复杂优化 |
| 时间不足 | 压缩可观测性到 v1.3.1 |

### 8.3 降级策略

若时间不足，按以下顺序丢弃:
1. P2 插件系统 (P-001, P-002)
2. 部分可观测性功能
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

| 日期 | AI | 评估结论 |
|------|-----|----------|
| 2026-03-12 | DeepSeek | ✅ 通过 (可观测性降为 P1) |
| 2026-03-13 | Claude Code | ⚠️ 有条件通过 (事务拆分) |
| 2026-03-13 | OpenCode | ✅ 通过 (覆盖率 ≥65%) |

---

## 附录: 命令参考

```bash
# 构建
cargo build --workspace

# 测试
cargo test --workspace

# 覆盖率
cargo tarpaulin --workspace --all-features

# Clippy
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all -- --check
```

---

**文档状态**: Draft  
**下次更新**: 评审会议后
