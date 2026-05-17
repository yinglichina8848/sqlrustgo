# v3.3.0 Release

> **状态**: Alpha
> **起点**: develop/v3.2.0 (commit `539f2957`) — GA release point
> **分支**: `alpha/v3.3.0` / `develop/v3.3.0`
> **Milestone**: v3.3.0 (id=13)
> **同步完成**: 2026-05-17 19:50 UTC

---

## 一、版本目标

解决 v3.2.0 GA 遗留问题：

| Issue | 遗留项 | 优先级 | 目标阶段 |
|-------|--------|--------|----------|
| #1196/#1197 | executor 覆盖率 70.7% < 85% | 🔴 P0 | Alpha |
| #1201 | MySQL Protocol 握手失败 | 🔴 P0 | Alpha |
| #1198 | TPC-H SF=1 数据缺失 | 🟡 P1 | Alpha |
| #1198 | 72h 稳定性测试未完成 | 🟡 P2 | Beta/RC |
| #1202 | Coverage 测量数据矛盾 | 🟡 P2 | Alpha |

### 豁免复审

所有 EX-v320-xxx 豁免记录需在 v3.3.0 Alpha 前复审。

---

## 二、Milestone Issue 列表

<!-- DO NOT EDIT MANUALLY - Auto-synced from Gitea API -->
<!-- Issue #1196, #1197, #1198, #1201, #1202 → milestone v3.3.0 -->

| # | 标题 | 优先级 | 状态 |
|---|------|--------|------|
| #1196 | executor 覆盖率不达标（stored_proc.rs） | P0 | Open |
| #1197 | executor 模块拆分重构 | P0 | Open |
| #1198 | TPC-H SF=1 数据缺失 + 72h 稳定性测试 | P1/P2 | Open |
| #1201 | MySQL Protocol 握手失败分析 | P0 | Open |
| #1202 | Coverage 测量数据矛盾根因分析 | P1 | Open |

---

## 三、Alpha 阶段任务

### 3.1 P0 阻塞项（必须 Alpha 内解决）

#### executor 覆盖率（#1196/#1197）

- [ ] 模块拆分：`stored_proc.rs` → `expression.rs` + `cursor.rs` + `handler.rs` + `cte.rs`
- [ ] 增量测试覆盖：目标 +14.3%（约 1,294 行）
- [ ] 重测覆盖率 ≥85%

#### MySQL Protocol（#1201）

- [ ] 抓包分析握手失败原因
- [ ] 修复握手状态机/auth plugin
- [ ] 端到端连接验证

### 3.2 P1/P2 项

#### Coverage 测量矛盾（#1202）

- [ ] 统一测量工具：`cargo llvm-cov`
- [ ] 明确测量范围（全部 crate vs 核心 crate）
- [ ] SSOT 文档建立

#### TPC-H SF=1（#1198）

- [ ] 在 Z6G4 服务器生成 SF=1 数据
- [ ] 执行 `bash scripts/gate/check_tpch.sh --sf1`
- [ ] 22/22 查询通过

---

## 四、版本信息

| 属性 | 值 |
|------|-----|
| 起点 commit | `1b8eeec3` |
| 前置版本 | v3.2.0 GA |
| 分支策略 | `alpha/v3.3.0` → `beta/v3.3.0` → `release/v3.3.0` |
| 豁免依据 | `docs/governance/GATE_EXEMPTIONS.md` v1.1 |
