# v3.0.0 全面治理审查报告

> **版本**: 1.0
> **日期**: 2026-05-08
> **审查依据**: docs/governance/ 全部治理文档
> **目的**: 对 v3.0.0 的文档、测试、发布流程、门禁进行全量审核，输出差距清单和优化建议

---

## 一、文档完整性审查

### 1.1 VERSION_DOCS_SPEC vs v3.0.0 实际对比

> 依据: `docs/governance/VERSION_DOCS_SPEC.md` §四（最小文档集）

#### 必选文档 (Required)

| 文档 | 规范要求 RC 后 | v3.0.0 存在 | 状态 | 问题 |
|------|--------------|-------------|------|------|
| `RELEASE_NOTES.md` | RC 后 | ✅ 存在 | OK | 阶段标注为 v3.0.0-alpha，与当前阶段一致 |
| `CHANGELOG.md` | RC 后 | ✅ 存在 | OK | — |
| `MIGRATION_GUIDE.md` | RC 后（有 breaking changes 时） | ✅ 存在 | OK | — |
| `COVERAGE_REPORT.md` | RC 后 | ❌ **不存在** | **P1** | 缺失覆盖率报告 |
| `SECURITY_ANALYSIS.md` | RC 后 | ❌ **不存在** | **P1** | 缺失安全分析报告 |
| `TEST_PLAN.md` | Alpha 后 | ✅ 存在 | OK | — |
| RC Gate 报告 | GA 前 | ❌ **不存在** | **P1** | 无 R1-R12 全部检查项通过证据 |

#### 可选文档 (Optional)

| 文档 | 规范建议 | v3.0.0 存在 | 状态 | 问题 |
|------|----------|-------------|------|------|
| `PERFORMANCE_TARGETS.md` | 推荐 | ✅ 存在 | OK | — |
| `DEPLOYMENT_GUIDE.md` | 推荐 | ❌ **不存在** | **P2** | 缺失部署指南 |
| `DEVELOPMENT_GUIDE.md` | 推荐 | ❌ **不存在** | **P2** | 缺失开发指南 |

**结论**: v3.0.0 缺失 5 个文档（COVERAGE_REPORT、SECURITY_ANALYSIS、RC Gate 报告、DEPLOYMENT_GUIDE、DEVELOPMENT_GUIDE）。

---

### 1.2 DOCUMENT_COMPLETENESS_CHECK 详细审查

> 依据: `docs/governance/DOCUMENT_COMPLETENESS_CHECK.md` §二

#### 根目录文档检查

| 文档 | 必选 | 存在 | 备注 |
|------|------|------|------|
| README.md | ✅ | ❌ **不存在** | v3.0.0 版本入口缺失 |
| CHANGELOG.md | ✅ | ✅ | — |
| RELEASE_NOTES.md | ✅ | ✅ | — |
| MIGRATION_GUIDE.md | ✅ | ✅ | — |
| DEPLOYMENT_GUIDE.md | ✅ | ❌ **不存在** | — |
| DEVELOPMENT_GUIDE.md | ✅ | ❌ **不存在** | — |
| TEST_PLAN.md | ✅ | ✅ | — |
| TEST_MANUAL.md | ✅ | ❌ **不存在** | — |
| EVALUATION_REPORT.md | ✅ | ❌ **不存在** | — |
| DOCUMENT_AUDIT.md | ✅ | ❌ **不存在** | — |
| FEATURE_MATRIX.md | ✅ | ✅ | — |
| COVERAGE_REPORT.md | ✅ | ❌ **不存在** | — |
| SECURITY_ANALYSIS.md | ✅ | ❌ **不存在** | — |
| PERFORMANCE_TARGETS.md | ✅ | ✅ | — |
| QUICK_START.md | ✅ | ❌ **不存在** | — |
| INSTALL.md | ✅ | ❌ **不存在** | — |
| API_DOCUMENTATION.md | ✅ | ❌ **不存在** | — |

**缺失率**: 12/17 (70.6%) 根目录文档缺失

#### OO 架构文档检查

| 文档 | 必选 | 存在 | 状态 |
|------|------|------|------|
| `oo/README.md` | ✅ | ❌ **不存在** | 目录不存在 |
| `oo/architecture/ARCHITECTURE_V3.0.md` | ✅ | ❌ **不存在** | — |
| `oo/user-guide/USER_MANUAL.md` | ✅ | ❌ **不存在** | — |
| `oo/user-guide/GMP_USER_GUIDE.md` | ✅ | ❌ **不存在** | — |
| `oo/user-guide/GRAPH_SEARCH_USER_GUIDE.md` | ✅ | ❌ **不存在** | — |
| `oo/user-guide/VECTOR_SEARCH_USER_GUIDE.md` | ✅ | ❌ **不存在** | — |
| `oo/reports/PERFORMANCE_ANALYSIS.md` | ✅ | ❌ **不存在** | — |
| `oo/reports/SQL92_COMPLIANCE.md` | ✅ | ❌ **不存在** | — |

**缺失率**: 8/8 (100%) OO 文档缺失

#### 模块设计文档检查

`oo/modules/` 目录整体不存在。以下模块设计文档全部缺失：

mvcc, wal, executor, parser, graph, vector, storage, optimizer, catalog, planner, transaction, server, bench, unified-query — **14 个模块设计文档 0% 存在**。

---

### 1.3 文档元数据审查

> 依据: `docs/governance/VERSION_DOCS_SPEC.md` §二（元数据要求）

检查 v3.0.0 文档头部元数据：

| 文档 | 元数据完整 | 问题 |
|------|-----------|------|
| RELEASENOTES.md | ✅ | 版本 v3.0.0-alpha，日期 2026-05-06 |
| CHANGELOG.md | ✅ | 版本 v3.0.0-alpha，日期 2026-05-06 |
| MIGRATION_GUIDE.md | ✅ | 版本 v3.0.0，日期 2026-05-06 |
| TEST_PLAN.md | ✅ | 版本 v3.0.0-alpha，日期 2026-05-06 |
| DEVELOPMENT_PLAN.md | ✅ | 版本 v3.0.0，日期 2026-05-06 |
| BETA_GATE_MASTER_CONTROL.md | ✅ | 版本 1.0，日期 2026-05-07 |
| FEATURE_MATRIX.md | ✅ | 版本 v3.0.0-alpha，日期 2026-05-06 |

**结论**: 所有文档元数据头部格式正确。

---

### 1.4 版本一致性审查

> 依据: `docs/governance/VERSION_DOCS_SPEC.md` §七

| 检查项 | 命令 | 结果 |
|--------|------|------|
| 文档版本号 = v3.0.0 | `grep -r "版本>: v3.0.0" docs/releases/v3.0.0/` | ✅ 大部分文档已更新 |
| 无遗留旧版本号 | `grep -r "v1.8.0\|v2.7.0" docs/releases/v3.0.0/` | ✅ 无遗留 |
| 日期格式正确 | YYYY-MM-DD | ✅ 所有文档使用正确格式 |

---

## 二、门禁规范一致性审查

### 2.1 gate_spec.md vs gate_spec_v300.md 双版本问题

**问题**: `docs/governance/` 目录同时存在两个门禁规范：

| 文件 | 版本 | 维护人 | 状态 |
|------|------|--------|------|
| `gate_spec.md` | v1.2 (2026-05-05) | hermes-z6g4 | 旧版，面向 v2.9.0 |
| `gate_spec_v300.md` | v1.0 (2026-05-06) | hermes-z6g4 | 新版，面向 v3.0.0 |

**冲突点**:

1. **覆盖率阈值冲突**:
   - `gate_spec.md` R5: ≥75%（RC Gate）
   - `gate_spec_v300.md` R5: ≥85%（RC Gate）

2. **AI_COLLABORATION.md 引用冲突**:
   - AI_COLLABORATION.md §六引用 `gate_spec.md` 的 R1-R10
   - 但 v3.0.0 已迁移到 `gate_spec_v300.md` 的 G1-G15

3. **RELEASE_POLICY.md 引用冲突**:
   - RELEASE_POLICY.md §二.2 表引用 `gate_spec.md` R1-R10
   - 但 v3.0.0 实际使用 `gate_spec_v300.md` 的 R1-R12 + G1-G15

**遗留问题编号**: DOC-GAP-01

---

### 2.2 规范与脚本不一致

> 已在 GA_GATE_AUDIT.md §一.1 定义，此处简略引用

| 不一致项 | 规范定义 | 脚本实现 | 遗留编号 |
|----------|----------|----------|----------|
| G7/G8/G9 QPS | 实际测量 ≥ 阈值 | check_regression.sh（无实际测量） | GA-GAP-02 |
| G11 SQL Corpus | ≥98% | check_ga_v300.sh ≥95% | GA-GAP-03 |
| G12 B-S 稳定性测试 | 应纳入 GA | 未纳入 | GA-GAP-04 |
| G13 MySQL Protocol | 应检查 | 未实现 | GA-GAP-05 |

---

### 2.3 GATE_EXEMPTIONS.md v3.0.0 未覆盖

> 依据: `docs/governance/GATE_EXEMPTIONS.md`

GATE_EXEMPTIONS.md 当前仅记录：
- EX-001: v2.9.0 性能基准先决条件
- EX-002: v2.9.0 executor 覆盖率 71.08% 豁免
- EX-003: v2.9.0 proof tool_output 追溯延期

**问题**: v3.0.0 的 GA-GAP 系列（特别是 GA-GAP-01 R-05 semver 漏洞）未登记在 GATE_EXEMPTIONS.md 中。

**遗留问题编号**: DOC-GAP-02

---

## 三、测试计划审查

### 3.1 TEST_PLAN.md 审查

> 依据: `docs/releases/v3.0.0/TEST_PLAN.md`

**已定义**: Alpha-1 ~ Alpha-4 测试分层
**缺失**: 无 Beta/RC/GA 测试计划

**Beta 阶段测试计划**: 未定义（应包含 B-S1~B-S5 的具体测试用例和阈值）
**RC 阶段测试计划**: 未定义
**GA 阶段测试计划**: 未定义

**遗留问题编号**: DOC-GAP-03

---

### 3.2 ALPHA_INTEGRATION_TESTING_PLAN.md 重复

**问题**: 该文档存在于两个位置：
- `docs/governance/ALPHA_INTEGRATION_TESTING_PLAN.md` (2026-05-06)
- `docs/releases/v3.0.0/ALPHA_INTEGRATION_TESTING_PLAN.md` (2026-05-06)

内容高度相似，可能导致维护混乱（版本不一致时无法判断哪个是权威）。

**遗留问题编号**: DOC-GAP-04

---

## 四、发布流程审查

### 4.1 RC_TO_GA_GATE_CHECKLIST.md 审查

> 依据: `docs/governance/RC_TO_GA_GATE_CHECKLIST.md`

**问题**: 该文档定义的是 v1.0.0 时代的模板，包含大量 ⏳ 占位符（未填入实际检查结果），且针对 v2.7.0+ 要求的所有 OO 文档、模块文档在 v3.0.0 目录中均不存在。

**与 v3.0.0 的实际差距**:
1. 文档完整性检查 → v3.0.0 缺失 70%+ 文档
2. OO 架构文档检查 → v3.0.0 完全缺失
3. 模块设计文档检查 → v3.0.0 完全缺失
4. 用户指南检查 → v3.0.0 完全缺失

**遗留问题编号**: DOC-GAP-05

---

### 4.2 RELEASE_LIFECYCLE.md 审查

> 依据: `docs/governance/RELEASE_LIFECYCLE.md`

**版本状态不一致**:
- RELEASE_LIFECYCLE.md §2.2 Beta Gate 定义测试通过率 ≥90%
- gate_spec_v300.md §四 B2 定义测试通过率 ≥90% ✅ 一致
- 但 BETA_GATE_MASTER_CONTROL.md §二 表格也定义为 ≥90% ✅ 一致

**阶段定义不一致**:
- RELEASE_LIFECYCLE.md §2.2 Beta "Tag 标记版本" 示例使用 alpha 而非 beta
  ```
  alpha/v2.9.0
  v2.9.0-alpha1
  v2.9.0-alpha2
  ```
  这与实际 v3.0.0 使用的 `beta/v3.0.0` / `v3.0.0-beta1` 命名规范不一致。

**遗留问题编号**: DOC-GAP-06

---

### 4.3 RELEASE_POLICY.md 审查

> 依据: `docs/governance/RELEASE_POLICY.md`

**覆盖率目标不一致**:
- RELEASE_POLICY.md §二.1 表: R-Gate 覆盖率目标 ≥75%
- gate_spec_v300.md §五.2 R5: ≥85%

**缺口**: RELEASE_POLICY.md 未同步更新到 v3.0.0 的 85% 目标。

**遗留问题编号**: DOC-GAP-07

---

## 五、CI/CD 流程审查

### 5.1 GATE_CI_CD.md 审查

> 依据: `docs/governance/GATE_CI_CD.md`

**目标架构 vs 当前实现**:

| 目标（文档定义） | 当前实际状态 |
|-----------------|-------------|
| `.github/workflows/r-gate.yml` | Gitea Actions，不使用 .github |
| R1-R7 Core Gates 并行执行 | ✅ 脚本化 |
| R8 SQL / R9 Perf / R10 Proof / G-Gate AV 扩展 | ⚠️ 部分实现 |
| Webhook notifications | ⚠️ 脚本存在但未验证 |
| `actions/checkout@v4` | Gitea Actions 使用不同语法 |

**遗留问题编号**: DOC-GAP-08

---

### 5.2 Nomad / Runner 状态

> 依据: `docs/governance/ALPHA_INTEGRATION_TESTING_PLAN.md` §六

文档记录：
```
HP Z6G4: ready ✅
250 MacMini: ready ✅
```

**实际问题**（需验证）:
- Runner 是否正常接收 Gitea Actions 任务？
- Nomad 双节点是否仍在运行？
- Gitea Actions API 是否正常返回 runner 列表？

**遗留问题编号**: DOC-GAP-09

---

## 六、AI 协作规范审查

### 6.1 AI_COLLABORATION.md 引用过期

> 依据: `docs/governance/AI_COLLABORATION.md`

**过期引用**:
- §六表格引用 `gate_spec.md` R1-R10，应更新为 `gate_spec_v300.md`
- §十相关文档列表未包含 `gate_spec_v300.md`

**遗留问题编号**: DOC-GAP-10

---

## 七、Issue 追踪机制审查

### 7.1 ISSUE_CLOSING_VERIFICATION.md 执行情况

> 依据: `docs/governance/ISSUE_CLOSING_VERIFICATION.md`

**当前执行状态**: 已建立流程，但实际执行不一致：
- Issue #451（SQL operations 20%）: 创建了 Gitea Issue ✅
- 但 Issue #451 未关联 milestone（应绑定 v3.0.0-beta）
- 未验证 `closedByPullRequestsReferences` 非空后再关闭

**遗留问题编号**: DOC-GAP-11

---

### 7.2 gate_lifecycle_tracking.md 状态

> 依据: `docs/governance/gate_lifecycle_tracking.md`

**已建立**:
- §7.1 Beta Gate 失败项追踪（#451）
- §7.2 Alpha 未完成任务延续
- §7.3 v3.1.0 必需完成项

**缺失**:
- GATE_EXEMPTIONS.md 中未登记 v3.0.0 的豁免项（如 GA-GAP-01 R-05）
- Issue #451 未在 gate_lifecycle_tracking.md §7.1 中正式登记（刚创建）

---

## 八、遗留问题总清单

### 8.1 文档缺失（P1-P2）

| 编号 | 文档 | 优先级 | 所属规范 |
|------|------|--------|----------|
| DOC-GAP-D01 | COVERAGE_REPORT.md | **P1** | VERSION_DOCS_SPEC Required |
| DOC-GAP-D02 | SECURITY_ANALYSIS.md | **P1** | VERSION_DOCS_SPEC Required |
| DOC-GAP-D03 | RC Gate 报告 | **P1** | VERSION_DOCS_SPEC Required |
| DOC-GAP-D04 | DEPLOYMENT_GUIDE.md | P2 | VERSION_DOCS_SPEC Optional |
| DOC-GAP-D05 | DEVELOPMENT_GUIDE.md | P2 | VERSION_DOCS_SPEC Optional |
| DOC-GAP-D06 | TEST_MANUAL.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D07 | EVALUATION_REPORT.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D08 | DOCUMENT_AUDIT.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D09 | QUICK_START.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D10 | INSTALL.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D11 | API_DOCUMENTATION.md | P2 | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D12 | README.md | **P1** | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D13 | oo/ 整个目录树 | **P1** | DOCUMENT_COMPLETENESS_CHECK |
| DOC-GAP-D14 | oo/modules/ 14个模块设计文档 | **P1** | DOCUMENT_COMPLETENESS_CHECK |

### 8.2 规范不一致（P1）

| 编号 | 问题 | 涉及文档 | 优先级 |
|------|------|----------|--------|
| DOC-GAP-01 | gate_spec.md 与 gate_spec_v300.md 双版本冲突 | AI_COLLABORATION, RELEASE_POLICY, gate_spec*.md | **P1** |
| DOC-GAP-02 | GATE_EXEMPTIONS.md 未覆盖 v3.0.0 GA-GAP 项 | GATE_EXEMPTIONS.md | **P1** |
| DOC-GAP-03 | 无 Beta/RC/GA 测试计划 | TEST_PLAN.md | **P1** |
| DOC-GAP-04 | ALPHA_INTEGRATION_TESTING_PLAN.md 重复 | docs/governance/ & docs/releases/v3.0.0/ | P2 |
| DOC-GAP-05 | RC_TO_GA_GATE_CHECKLIST.md v1.0.0 模板未适配 v3.0.0 | RC_TO_GA_GATE_CHECKLIST.md | P2 |
| DOC-GAP-06 | RELEASE_LIFECYCLE.md Beta Tag 示例使用 alpha 命名 | RELEASE_LIFECYCLE.md | P2 |
| DOC-GAP-07 | RELEASE_POLICY.md 覆盖率目标 75% vs gate_spec 85% | RELEASE_POLICY.md | **P1** |
| DOC-GAP-08 | GATE_CI_CD.md 目标架构与 Gitea Actions 实际不匹配 | GATE_CI_CD.md | P2 |
| DOC-GAP-09 | Nomad/Runner 状态需验证 | ALPHA_INTEGRATION_TESTING_PLAN.md | P2 |
| DOC-GAP-10 | AI_COLLABORATION.md 引用过期 gate_spec.md | AI_COLLABORATION.md | **P1** |
| DOC-GAP-11 | ISSUE_CLOSING_VERIFICATION.md 执行不一致 | ISSUE_CLOSING_VERIFICATION.md | P2 |

---

## 九、优化建议

### 9.1 文档体系整合

**建议**: 将 `gate_spec.md` 合并到 `gate_spec_v300.md`，消除双版本冲突。

```
gate_spec.md (v1.2, v2.9.0) → 归档为 gate_spec_v2.9.md
gate_spec_v300.md (v1.0, v3.0.0) → 升为 gate_spec.md
```

### 9.2 创建缺失文档

**优先级排序**:
1. **P1 阻塞发布**: README.md, COVERAGE_REPORT.md, SECURITY_ANALYSIS.md, oo/ 目录
2. **P1 阻塞门禁**: RC Gate 报告（check_rc_v300.sh 输出归档）
3. **P2 提升完整性**: 其余 10 个缺失文档

### 9.3 GATE_EXEMPTIONS.md 更新

在 GATE_EXEMPTIONS.md §二添加 v3.0.0 豁免记录：

```markdown
| EX-004 | v3.0.0 | GA-GAP-01 R-05 semver | **豁免申请** | 待审批 | cargo audit 存在已知漏洞，R-05 影响有限 | Tech Lead | 2026-05-08 | #451 | v3.1.0 GA 前 | 需评估实际影响 |
```

### 9.4 规范同步

| 文档 | 需同步内容 |
|------|-----------|
| RELEASE_POLICY.md §二 | R-Gate 覆盖率目标改为 ≥85% |
| AI_COLLABORATION.md §六 | 引用 gate_spec_v300.md 而非 gate_spec.md |
| RELEASE_LIFECYCLE.md §2.2 | Beta Tag 示例改为 beta/v3.0.0 |

### 9.5 测试计划扩展

在 `TEST_PLAN.md` 中新增：
- Beta 阶段测试计划（B-S1~B-S5 详细用例）
- RC 阶段测试计划（R1~R12 验证方案）
- GA 阶段测试计划（G1~G15 验证方案）

---

## 十、v3.1.0 文档任务追踪

以下文档任务需在 v3.1.0 中完成：

| 任务 | 来源 | 优先级 |
|------|------|--------|
| 创建 README.md | DOC-GAP-D12 | P1 |
| 创建 COVERAGE_REPORT.md | DOC-GAP-D01 | P1 |
| 创建 SECURITY_ANALYSIS.md | DOC-GAP-D02 | P1 |
| 创建 oo/ 目录及子文档 | DOC-GAP-D13, D14 | P1 |
| 合并 gate_spec_v300.md 为唯一 gate_spec | DOC-GAP-01 | P1 |
| 更新 GATE_EXEMPTIONS.md v3.0.0 豁免 | DOC-GAP-02 | P1 |
| 同步 RELEASE_POLICY.md 覆盖率目标 | DOC-GAP-07 | P1 |
| 更新 AI_COLLABORATION.md 引用 | DOC-GAP-10 | P1 |
| 创建 RC Gate 报告 | DOC-GAP-D03 | P1 |
| 扩展 TEST_PLAN.md 覆盖 Beta/RC/GA | DOC-GAP-03 | P1 |
| 修复 ALPHA_INTEGRATION_TESTING_PLAN.md 重复 | DOC-GAP-04 | P2 |
| 更新 RC_TO_GA_GATE_CHECKLIST.md | DOC-GAP-05 | P2 |
| 修复 RELEASE_LIFECYCLE.md Beta Tag 示例 | DOC-GAP-06 | P2 |
| 验证 Nomad/Runner 状态 | DOC-GAP-09 | P2 |
| 创建其余 8 个缺失文档 | DOC-GAP-D04~D11 | P2 |

---

## 十一、相关文档索引

| 文档 | 用途 |
|------|------|
| `docs/governance/gate_spec_v300.md` | v3.0.0 门禁规范 SSOT |
| `docs/governance/gate_lifecycle_tracking.md` | 门禁生命周期闭环追踪 |
| `docs/releases/v3.0.0/GA_GATE_AUDIT.md` | GA 门禁差距审查 |
| `docs/governance/VERSION_DOCS_SPEC.md` | 版本文档规范 |
| `docs/governance/DOCUMENT_COMPLETENESS_CHECK.md` | 文档完整性检查 |
| `docs/governance/GATE_EXEMPTIONS.md` | 门禁豁免记录 |
| `docs/governance/RELEASE_POLICY.md` | 发布策略 |
| `docs/governance/RELEASE_LIFECYCLE.md` | 版本生命周期 |
| `docs/governance/RC_TO_GA_GATE_CHECKLIST.md` | RC→GA 关门清单 |
| `docs/governance/AI_COLLABORATION.md` | AI 协作规则 |

---

*本文档由 hermes agent 创建，基于 docs/governance/ 全部治理文档的全量审查。*
*最后更新: 2026-05-08*
