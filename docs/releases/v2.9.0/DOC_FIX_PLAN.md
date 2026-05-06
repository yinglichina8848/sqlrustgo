# SQLRustGo v2.9.0 文档整改计划

> **基于**: DeepSeek 文档审核意见 (2026-05-05)
> **状态**: 规划中
> **优先级**: P0 (阻塞RC) / P1 (RC期间) / P2 (v2.10.0+)

---

## 一、审核意见摘要

### 核心问题分类

| 类别 | 问题数 | 优先级 |
|------|--------|--------|
| 数据不一致 (TPC-H/TLS/QPS) | 5 | P0 |
| 设计文档与实现脱节 (Optimizer) | 2 | P0 |
| 阶段标记过时 | 4 | P0 |
| SSOT 缺失 | 1 | P1 |
| Proof 数量矛盾 | 1 | P1 |
| 覆盖率报告冗余 | 1 | P1 |
| 混沌测试未闭环 | 1 | P2 |

---

## 二、P0 必须修复项

### P0-1: README.md — TPC-H 状态过时

**问题**: README 仍称 "9/22 (41%)"，而 RC_STATUS 已明确 "22/22 全部可运行"

**文件**: `docs/releases/v2.9.0/README.md`

**修正**:
- Line 26: `9/22 (41%)` → `22/22 (100%)`
- Line 286: `9/22 (41%)` → `22/22 (100%)`
- 补充说明: "22/22 Parser+Executor 可运行，3/22 结果与 SQLite 完全一致"

**验证**: `grep -n "9/22" README.md` 应返回空

---

### P0-2: SECURITY_ANALYSIS.md — TLS 虚假声明

**问题**: 安全分析声称 "TLS 支持传输层加密"，但实际未实现 TLS

**文件**: `docs/releases/v2.9.0/SECURITY_ANALYSIS.md`

**修正**:
- Line 48: `TLS 支持 | 传输层加密` → `TLS 支持 | ❌ 未实现`
- 补充说明: TLS 为 P0 缺失，功能矩阵中标注为"未实现"

**验证**: `grep -n "TLS" SECURITY_ANALYSIS.md` 应显示 "未实现"

---

### P0-3: SQL92_COMPLIANCE.md — TPC-H 混淆可运行与正确

**问题**: 声称 "TPC-H 22/22 (100%)"，未区分 "可运行" vs "结果正确"

**文件**: `docs/releases/v2.9.0/oo/reports/SQL92_COMPLIANCE.md`

**修正**:
- Line 140: "TPC-H 可运行查询: 22/22 (100%)" → "TPC-H 可运行查询: 22/22 (100%) / 结果正确: 3/22 (13.6%)"
- Line 222: 表格中 TPC-H 列需拆分
- Line 231: 说明需明确区分

**验证**: 文档中应同时体现 "可运行" 和 "结果正确" 两个指标

---

### P0-4: PERFORMANCE_ANALYSIS.md — QPS 口径混乱

**问题**: 提到 sysbench ~2,000，但未提及 JOIN QPS 12,617，混淆不同测试类型

**文件**: `docs/releases/v2.9.0/oo/reports/PERFORMANCE_ANALYSIS.md`

**修正**:
- Line 90: 补充 QPS 口径说明
  - sysbench point_select: ~2,200 QPS
  - TPC-H JOIN Q7: 12,617 QPS
  - UPDATE/DELETE: ~950/~206 (远低于 10k 目标)
- Line 101: P99 延迟标注测试条件

**验证**: 文档中应有明确的 QPS 分类表

---

### P0-5: OPTIMIZER_DESIGN.md — 缺少实现状态标注

**问题**: 设计文档描述完整 CBO，但代码关键规则均为 TODO stub

**文件**: `docs/releases/v2.9.0/oo/modules/optimizer/OPTIMIZER_DESIGN.md`

**修正**:
- 在文档开头增加 "实现状态表"
- CBO 规则标注: `PredicatePushdown: ❌ TODO` / `ProjectionPruning: ❌ TODO` / `ConstantFolding: ❌ TODO`
- 架构图中 CBO 模块改为虚线标注 "设计目标，待实现"

**验证**: `grep -n "TODO\|stub\|规划中" OPTIMIZER_DESIGN.md` 应有明确标注

---

### P0-6: 版本阶段标记过时

**问题**: VERSION_PLAN.md, TEST_PLAN.md, PERFORMANCE_TARGETS.md 仍标注 Alpha

**文件**:
- `docs/releases/v2.9.0/VERSION_PLAN.md`
- `docs/releases/v2.9.0/TEST_PLAN.md`
- `docs/releases/v2.9.0/PERFORMANCE_TARGETS.md`

**修正**: 将阶段标记从 "Alpha" 更新为 "RC (v2.9.0-rc.1)"

---

## 三、P1 改进项

### P1-1: 建立单一事实来源 (SSOT)

**文件**: `docs/releases/v2.9.0/RC_STATUS_20260505.md`

在文档开头增加 "权威数据源 (SSOT)" 表格:

```
| 指标 | 数值 | 来源 |
|------|------|------|
| 总覆盖率 | 84.18% (lines) | COVERAGE_RC_REPORT.md |
| Executor 覆盖率 | 71.08% | COVERAGE_RC_REPORT.md |
| SQL Corpus | 92.6% (430/464) | TEST_REPORT.md |
| TPC-H 可运行 | 22/22 | RC_STATUS.md |
| TPC-H 结果正确 | 3/22 | RC_STATUS.md |
| Proof 数量 | 18 (Proof-018 不存在) | PROOF_COVERAGE.md |
| sysbench QPS | ~2,200 (point_select) | E08_QPS_BENCHMARK_REPORT.md |
| JOIN QPS | 12,617 (Q7) | E08_QPS_BENCHMARK_REPORT.md |
```

---

### P1-2: Proof-018 矛盾

**问题**: PROOF_COVERAGE.md 说 "不存在"，RC_GATE_REPORT 说 "18/18"

**文件**:
- `docs/releases/v2.9.0/PROOF_COVERAGE.md`
- `docs/releases/v2.9.0/RC_GATE_REPORT.md`

**修正**:
- PROOF_COVERAGE.md: Proof-018 标注为 "不存在，跳过"
- RC_GATE_REPORT.md: 修正为 "17/18 verified + 1 skipped (Proof-018 不存在)"

---

### P1-3: TRANSACTION_DESIGN — SERIALIZABLE 标注

**问题**: 设计文档描述四种隔离级别，但 v2.9.0 仅支持 RC 和 SI

**文件**: `docs/releases/v2.9.0/oo/modules/transaction/TRANSACTION_DESIGN.md`

**修正**:
- Line 50 表格: SERIALIZABLE 列添加标注 "❌ 未实现 (v2.10.0 规划)"
- 增加 v2.9.0 支持的隔离级别列表: RC, SI

---

### P1-4: 覆盖率报告合并

**问题**: 多个覆盖率报告 (COVERAGE_REPORT, COVERAGE_RC_REPORT, COVERAGE_IMPROVEMENT_REPORT) 存在冗余

**方案**: 以 `COVERAGE_RC_REPORT.md` 为权威来源，其他文件添加 "已归档" 标记

---

## 四、P2 完善项

### P2-1: 混沌测试失败项根因

**问题**: TEST_REPORT 提到 2 个集成测试失败、2 个 E2E 失败，但未闭环

**文件**: `docs/releases/v2.9.0/TEST_REPORT.md`

**补充**: 在文档末尾增加 "已知限制" 小节，记录失败项及状态

---

## 五、执行清单

| # | 任务 | 状态 | 验证命令 |
|---|------|------|----------|
| P0-1 | README.md TPC-H 修正 | ⬜ | `grep "9/22" README.md` |
| P0-2 | SECURITY_ANALYSIS.md TLS 修正 | ⬜ | `grep "TLS.*未实现" SECURITY_ANALYSIS.md` |
| P0-3 | SQL92_COMPLIANCE.md TPC-H 修正 | ⬜ | `grep "结果正确" SQL92_COMPLIANCE.md` |
| P0-4 | PERFORMANCE_ANALYSIS.md QPS 口径 | ⬜ | `grep "point_select\|JOIN" PERFORMANCE_ANALYSIS.md` |
| P0-5 | OPTIMIZER_DESIGN.md 实现标注 | ⬜ | `grep "TODO.*CBO\|stub" OPTIMIZER_DESIGN.md` |
| P0-6 | 版本阶段标记更新 | ⬜ | `grep "Alpha\|RC" VERSION_PLAN.md` |
| P1-1 | RC_STATUS SSOT 补充 | ⬜ | `grep "SSOT\|权威" RC_STATUS_20260505.md` |
| P1-2 | Proof-018 矛盾修正 | ⬜ | `grep "018" PROOF_COVERAGE.md` |
| P1-3 | TRANSACTION_DESIGN SERIALIZABLE 标注 | ⬜ | `grep "SERIALIZABLE.*未实现" TRANSACTION_DESIGN.md` |
| P2-1 | 混沌测试根因记录 | ⬜ | |

---

## 六、审核意见来源

1. **门禁文档审核** (全面性 90/100, 正确性 80/100, 合理性 85/100)
2. **OO文档审核** (全面性 85/100, 正确性 70/100, 合理性 80/100)
3. **整合评估** (综合: 全面性 87/100, 正确性 75/100, 合理性 83/100)
