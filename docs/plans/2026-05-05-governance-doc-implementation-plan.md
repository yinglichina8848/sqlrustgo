# 治理文档整改实现计划

> **日期**: 2026-05-05
> **版本**: v1.0
> **维护人**: hermes-z6g4
> **分支**: `docs/v2.9.0-doc-fix-20260505`
> **审查状态**: 外部 AI 审查意见已整合（DeepSeek #321 + 增强方案）

---

## 一、背景

基于 DeepSeek 对 `docs/governance/` 的审查（Issue #321）及增强方案分析，当前治理文档存在三大根因问题：

| 根因 | 证据 | 影响 |
|------|------|------|
| **tarpaulin → llvm-cov 未同步** | gate_spec.md R5 使用 `cargo tarpaulin`，但 v2.9.0 CI 已切 `cargo-llvm-cov` | 规范与 CI 执行脱节 |
| **模块级覆盖率阈值缺失** | gate_spec.md 只有整体阈值，B-Gate 84.18% 但 executor=71.08% 无明确判定依据 | 改进无指引，文档与实际判定不一致 |
| **规范碎片化无 SSOT** | gate_spec.md、RELEASE_POLICY.md、AI_COLLABORATION.md 均描述门禁，相互引用但不统一 | 无法形成唯一事实源 |

---

## 二、Open Questions（需人工决策）

| # | 问题 | 选项 |
|---|------|------|
| Q1 | R9 性能回归检查如何处理？ | (A) 延期至 v2.10.0，建立豁免记录 / (B) 建立临时基准 |
| Q2 | G-Gate 85% 阈值是否设宽限条件？ | (A) 设"RC 宽限"关键模块达标即可 / (B) 维持 85% 长期目标 |
| Q3 | 是否追溯补充现有 18 个 proof files 的 tool_output 字段？ | (A) 追溯补充 / (B) 仅要求新增 proof 必须包含 |

---

## 三、任务分解

### P0 — GA 前必须完成（阻断性问题）

#### P0.1: gate_spec.md v1.2

**目标**: 确立 gate_spec.md 为唯一门禁权威定义，修正所有工具命令，补充模块阈值矩阵和证据格式。

**变更清单**:

| # | 变更内容 | 具体修改 |
|---|----------|----------|
| 1 | 工具链更新 | 全文 `cargo tarpaulin` → `cargo llvm-cov`，参数调整为 `--all-features --lcov --output-path lcov.info` |
| 2 | R5 命令更新 | `cargo tarpaulin --workspace --all-features` → `cargo llvm-cov --all-features --lcov --output-path lcov.info` |
| 3 | 覆盖率阈值矩阵 | 补充 R-Gate 模块级阈值（当前缺失），A/B/R/G 四级全部对齐 v2.9.0 实际 |
| 4 | R7 扩展 | 从仅"死链检查"扩展为 R7a(死链) + R7b(必选文档) + R7c(版本一致性) + R7d(代码-文档状态) |
| 5 | R9 性能回归 | 明确: 无基准时标注"延期至 v2.10.0"，关联 GATE_EXEMPTIONS.md |
| 6 | R10 形式化证明 | 要求 JSON proof 文件必须包含 `tool_output` 字段 |
| 7 | 证据格式规范化 | 每个 R1-R10 检查项增加证据模板: `{status, command, output_summary, artifact_path, timestamp}` |
| 8 | 门禁状态表移除"当前状态" | 第 7.2 节"当前状态 (v2.9.0)"中 R-Gate="🔄 进行中" 应移除或标注"定期更新" |

**R-Gate 模块级阈值矩阵**:

| 模块 | A-Gate | B-Gate | R-Gate | G-Gate |
|------|--------|--------|--------|--------|
| executor | ≥45% | ≥60% | **≥75%** | ≥80% |
| optimizer | ≥40% | ≥50% | **≥60%** | ≥70% |
| storage | ≥15% | ≥20% | **≥30%** | ≥40% |
| catalog | ≥50% | ≥60% | **≥70%** | ≥75% |
| parser | ≥50% | ≥60% | **≥70%** | ≥80% |
| **整体** | ≥50% | ≥75% | **≥75%** | ≥85% |

*注: R-Gate 列加粗为本次新增（B-Gate 和整体阈值已在 v1.1 中定义）*

---

#### P0.2: GATE_EXEMPTIONS.md（新建）

**目标**: 建立门禁豁免/延期记录的单一事实源。

**内容结构**:
```markdown
# 门禁豁免与延期记录

| 豁免 ID | 版本 | 门禁项 | 类型 | 豁免原因 | 审批人 | 日期 | 关联 Issue |
|---------|------|--------|------|----------|--------|------|------------|
| EX-001 | v2.9.0 | R9 性能回归 | 延期 | 无性能基准基础设施，延期至 v2.10.0 | Tech Lead | 2026-05-05 | #296 |
| EX-002 | v2.9.0 | B-Gate executor 覆盖率 71.08% | 豁免 | 距 75% 差距 3.92%，B-Gate 通过时已记录 | Tech Lead | 2026-05-04 | #319 |
```

---

#### P0.3: VERSION_DOCS_SPEC.md（新建）

**目标**: 统一版本文档的元数据格式、命名规范、最小文档集。

**核心内容**:
- 每个版本文档必须包含: 版本号、更新日期、阶段状态、维护人
- 最小文档集: 与 DOCUMENT_COMPLETENESS_CHECK.md 对齐，分必选/可选
- 命名规范: `VERSION_PLAN.md`, `CHANGELOG.md` 等固定文件名
- 链接规范: 文档间引用使用相对路径，禁止绝对路径

---

### P1 — GA 前完成（质量改进）

#### P1.1: RELEASE_POLICY.md v2.1

**变更**:
- 移除第 2.3 节 R1-R10 重复定义，改为"详见 gate_spec.md"
- 移除第 2.2 节阶段-门禁对应表中的重复阈值，引用 gate_spec.md
- 其他文档（RELEASE_LIFECYCLE.md, AI_COLLABORATION.md）同步清理重复门禁描述

---

#### P1.2: AI_COLLABORATION.md v2.1

**变更**:
- 第 2.3 节 R1-R10 描述改为引用 gate_spec.md
- 门禁流程描述与 gate_spec.md 完全对齐

---

#### P1.3: GATE_CI_CD.md v1.1

**变更**:
- 文档头标注"目标架构 vs 当前实现"：R8, R9, R10 的 CI 实现尚未完全落地，仅为规划目标
- 更新 pipeline 图中 R5 命令为 `cargo llvm-cov`

---

#### P1.4: RELEASE_LIFECYCLE.md 状态清理

**变更**:
- 移除"当前状态"列（该信息应仅存在于 RC_STATUS_*.md 等状态报告）
- 治理文档只保留规范定义，不维护实时状态

---

### P2 — 中期改进（v2.10.0）

| # | 任务 | 关联增强项 |
|---|------|-----------|
| P2.1 | 实施 Conventional Commits + commitlint 自动化 | 增强 3.4 |
| P2.2 | `pre-commit` 钩子: AI 代码审计（unwrap/TODO/unsafe） | 增强 2.2 |
| P2.3 | 性能基准库 `perf_baselines/` 建立 | 增强 4.2 |
| P2.4 | `cargo semver-checks` 破坏性变更检测 | 增强 2.3 |
| P2.5 | `GATE_EXEMPTIONS.md` 中 R9 延期项到期检查 | 增强 1.2 |

---

## 四、执行顺序

```
Step 1: gate_spec.md v1.2  (P0.1) — 核心，必须最先完成
         ↓
Step 2: GATE_EXEMPTIONS.md  (P0.2) — 依赖 gate_spec v1.2 中的 R9 豁免定义
         ↓
Step 3: VERSION_DOCS_SPEC.md  (P0.3) — 新建，可独立推进
         ↓
Step 4: RELEASE_POLICY.md v2.1  (P1.1) — 引用 gate_spec v1.2
         ↓
Step 5: AI_COLLABORATION.md v2.1  (P1.2) — 引用 gate_spec v1.2
         ↓
Step 6: GATE_CI_CD.md v1.1  (P1.3) — 更新命令标注
         ↓
Step 7: RELEASE_LIFECYCLE.md  (P1.4) — 移除"当前状态"
         ↓
Step 8: 提交 PR → CI → Review → Merge
```

---

## 五、成功标准

| 指标 | 目标 |
|------|------|
| gate_spec.md 中 tarpaulin 出现次数 | 0（全文替换为 llvm-cov） |
| 模块级阈值覆盖度 | A/B/R/G × executor/optimizer/storage/catalog/parser = 24 个阈值单元格全部填充 |
| 门禁证据链规范完整性 | R1-R10 每项均有 `{command, output_summary, artifact_path}` 模板 |
| 跨文档门禁描述一致性 | RELEASE_POLICY / AI_COLLABORATION / GATE_CI_CD 均引用 gate_spec.md，无独立门禁定义 |
| GATE_EXEMPTIONS.md 初始记录 | ≥2 条（EX-001 R9 延期 + EX-002 B-Gate executor 豁免） |

---

## 六、交付物清单

| 文件 | 操作 | PR |
|------|------|-----|
| `docs/governance/gate_spec.md` | 修改 | PR 到 `develop/v2.9.0` |
| `docs/governance/GATE_EXEMPTIONS.md` | 新建 | 同上 |
| `docs/governance/VERSION_DOCS_SPEC.md` | 新建 | 同上 |
| `docs/governance/RELEASE_POLICY.md` | 修改 | 同上 |
| `docs/governance/AI_COLLABORATION.md` | 修改 | 同上 |
| `docs/governance/GATE_CI_CD.md` | 修改 | 同上 |
| `docs/governance/RELEASE_LIFECYCLE.md` | 修改 | 同上 |

---

## 七、风险与依赖

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| gate_spec.md 大面积修改可能引入格式错误 | 高 | 修改后运行 `check_docs_links.sh` |
| 多文档同步修改可能遗漏引用 | 中 | 用 `grep` 搜索 `tarpaulin` 确保全部替换 |
| v2.9.0 CI 当前使用 tarpaulin（runner 健康未知） | 低（CI 由其他 Agent 维护） | 整改文档规范，不改 CI 脚本 |

---

*本文档由 hermes-z6g4 维护，整合了 DeepSeek 审查意见（#321）与增强方案分析*
