# SQLRustGo 治理标准总纲

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **用途**: SQLRustGo 项目治理的核心规范，所有 AI Agent 和开发者必须遵循的最高层治理文档

> **SSOT 声明**: 本文档是 SQLRustGo 治理体系的最高层规范文档，定义治理原则、AI 合规要求、版本生命周期和文档体系索引。所有其他治理文档必须与本文档保持一致，如有冲突以本文档为准。

---

## 一、治理体系概述

### 1.1 治理体系架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GOVERNANCE_STANDARD.md (本文档)                   │
│                         治理标准总纲 · SSOT 最高层                   │
├─────────────────────────────────────────────────────────────────────┤
│                              │                                       │
│        ┌─────────────────────┼─────────────────────┐                │
│        ▼                     ▼                     ▼                │
│ ┌──────────────┐  ┌──────────────────┐  ┌─────────────────────┐   │
│ │   AI合规     │  │   版本生命周期     │  │    文档体系         │   │
│ │ 执行机制     │  │   管理             │  │    索引             │   │
│ │              │  │                   │  │                     │   │
│ │ 引用:        │  │ 引用:             │  │ 引用:               │   │
│ │ AI_COMPLIANCE│  │ VERSION_LIFECYCLE │  │ GOVERNANCE_INDEX    │   │
│ │ _MECHANISM   │  │ _MANAGEMENT       │  │                     │   │
│ └──────────────┘  └──────────────────┘  └─────────────────────┘   │
│        │                     │                     │               │
│        └─────────────────────┼─────────────────────┘                │
│                              ▼                                       │
│ ┌────────────────────────────────────────────────────────────────┐  │
│ │                    GATE_PHASES_AND_TRACKING.md                  │  │
│ │                    分阶段门禁 + 跨版本追踪                        │  │
│ ├────────────────────────────────────────────────────────────────┤  │
│ │  门禁规范 SSOT: GATE_SPEC_MASTER.md                              │  │
│ │  门禁检查清单模版: GATE_CHECKLIST_TEMPLATE.md                     │  │
│ │  门禁生命周期追踪: gate_lifecycle_tracking.md                     │  │
│ └────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 治理目标

| 目标 | 说明 |
|------|------|
| **质量保证** | 所有发布版本必须通过定义的门禁检查 |
| **可追溯性** | 从需求到代码到发布全链路可追溯 |
| **一致性** | 跨版本、跨阶段开发活动遵循统一规范 |
| **合规性** | AI Agent 行为必须符合治理规范 |
| **持续改进** | 治理体系通过反馈机制持续优化 |

### 1.3 适用范围

- **版本**: v2.9.0 及以上
- **阶段**: Alpha / Beta / RC / GA 全生命周期
- **角色**: Human Architect、AI Developer、AI Maintainer、CI System
- **文档**: 所有 governance 相关文档

---

## 二、核心治理原则

### 2.1 原则清单

```
┌─────────────────────────────────────────────────────────────────────┐
│                        五大核心原则                                   │
├─────────────────────────────────────────────────────────────────────┤
│  1. SSOT 原则     │ 单一权威来源，规范不得重复定义                    │
│  2. 门禁强制原则   │ 所有发布必须通过对应阶段门禁，AI不可绕过            │
│  3. Issue 闭环原则 │ FAIL → Issue → PR → 验证 → 关闭，无例外          │
│  4. 版本延续原则   │ 未完成任务必须映射到下版本，不得丢失               │
│  5. 证据驱动原则   │ 所有检查必须记录证据，证据格式必须符合规范           │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 SSOT 原则详解

| 文档类型 | SSOT 文档 | 引用关系 |
|----------|-----------|----------|
| 门禁规范 | `GATE_SPEC_MASTER.md` | 其他门禁文档必须引用本文档 |
| 版本生命周期 | `VERSION_LIFECYCLE_MANAGEMENT.md` | 引用 `RELEASE_LIFECYCLE.md` |
| AI 合规 | `AI_COMPLIANCE_MECHANISM.md` | 引用 `AI_COLLABORATION.md` |
| 分阶段门禁追踪 | `GATE_PHASES_AND_TRACKING.md` | 引用 `gate_lifecycle_tracking.md` |
| 文档导航 | `GOVERNANCE_INDEX.md` | 索引所有治理文档 |

### 2.3 禁止的模式

```text
❌ 规范冲突: 在非 SSOT 文档中定义与 GATE_SPEC_MASTER.md 不一致的检查项
❌ 门禁绕过: A/B/R/G Gate FAIL 但跳过 Issue 创建直接合并
❌ Issue 丢失: 门禁失败但未创建 Issue，任务未延续到下版本
❌ 证据缺失: 检查通过但未记录命令、输出、阈值等证据
❌ 版本混乱: 未按语义化版本规则发布，Tag 与版本号不一致
❌ AI 越权: AI Agent 执行需要 Human Architect 审批的操作
```

### 2.4 正确的模式

```text
✅ 规范遵循: 检查项引用 GATE_SPEC_MASTER.md，不自行定义
✅ 门禁闭环: Gate FAIL → Issue #N → PR 修复 → 验证 PASS → 关闭 Issue
✅ 任务延续: 当前版本无法修复 → 在 v{NEXT} DEVELOPMENT_PLAN.md 建立映射
✅ 证据完整: {command, exit_code, output_summary, threshold, pass}
✅ 版本规范: v{MAJOR}.{MINOR}.{PATCH}-{phase}{N} → v{MAJOR}.{MINOR}.{PATCH}
✅ 权限分明: AI 执行需审批操作前先创建 Issue，等待 Human Architect 决策
```

---

## 三、AI 合规执行机制

### 3.1 合规要求总览

> **详见**: `AI_COMPLIANCE_MECHANISM.md`

所有 AI Agent 在执行治理相关任务时必须：

1. **任务识别**: 检测任务类型（开发计划/测试计划/门禁检查/Issue追踪）
2. **文档加载**: 根据 `GOVERNANCE_INDEX.md` 的决策树加载正确模版
3. **执行验证**: 按模版要求执行，不得自由发挥
4. **证据记录**: 所有检查必须记录符合规范的证据格式
5. **Issue 闭环**: FAIL 项必须创建 Issue 并追踪到关闭

### 3.2 AI 可执行任务清单

| 任务类型 | 触发关键词 | 必须使用的模版 |
|----------|------------|----------------|
| 版本开发计划 | "开发计划"、"版本计划"、"v{X}.{Y}.{Z} 开发" | `DEVELOPMENT_PLAN_TEMPLATE.md` |
| 测试计划 | "测试计划"、"测试策略"、"测试用例" | `TEST_PLAN_TEMPLATE.md` |
| 门禁检查 | "门禁检查"、"Gate"、"门禁报告"、"Alpha/Beta/RC/GA" | `GATE_CHECKLIST_TEMPLATE.md` |
| Issue 追踪 | "创建 Issue"、"关闭 Issue"、"Issue 追踪" | `gate_lifecycle_tracking.md` |
| 版本延续 | "版本延续"、"延续任务" | 在 `DEVELOPMENT_PLAN.md` §6 建立映射 |

### 3.3 AI 禁止执行的操作

```text
❌ 禁止: 未使用模版创建 governance 文档
❌ 禁止: 在非 SSOT 文档中定义门禁检查项
❌ 禁止: 在没有 PR 证据的情况下关闭 Issue
❌ 禁止: 跳过门禁检查直接合并代码
❌ 禁止: 执行需要 Human Architect 审批的操作（合并 PR、发布版本等）
❌ 禁止: 创建与 GATE_SPEC_MASTER.md 不一致的检查阈值
```

### 3.4 合规检查清单

```markdown
## AI 合规自查清单

### 执行治理任务前
- [ ] 确认任务类型，查阅 GOVERNANCE_INDEX.md
- [ ] 确认使用的模版文件存在
- [ ] 确认模版版本为最新

### 执行中
- [ ] 严格按模版章节结构创建文档
- [ ] 所有占位符 {VERSION}、{PHASE} 等已替换
- [ ] 证据格式符合规范定义

### 执行后
- [ ] 验证文档已创建在正确路径
- [ ] 验证模版要求的所有章节已包含
- [ ] 如有 FAIL 项，确认已创建 Issue
```

---

## 四、版本生命周期管理

### 4.1 四级门禁模型

> **详见**: `VERSION_LIFECYCLE_MANAGEMENT.md`

```
A-Gate ──▶ B-Gate ──▶ R-Gate ──▶ G-Gate
 (α入口)    (β入口)    (RC入口)    (GA入口)
```

| 门禁 | 名称 | 阶段目标 | 覆盖率目标 | 性能目标 |
|------|------|----------|------------|----------|
| A-Gate | Alpha Gate | 开发完成，可运行原型 | ≥50% | 基线建立 |
| B-Gate | Beta Gate | 功能冻结，进入稳定期 | ≥75% | TPC-H SF=0.1 22/22 |
| R-Gate | RC Gate | 发布候选，性能优化完成 | ≥85% | TPC-H SF=1 22/22 + QPS 基线 |
| G-Gate | GA Gate | 正式发布，所有门槛达标 | ≥85% | Point Select ≥10K QPS |

### 4.2 阶段转换规则

```text
阶段转换 = 前置门禁 PASS + 所有 FAIL 项有 Issue/PR + 所有豁免项已审批
```

| 转换 | 前置条件 |
|------|----------|
| → A-Gate | 开发任务完成，代码已提交 |
| A-Gate → B-Gate | A-Gate 全部检查 PASS |
| B-Gate → R-Gate | B-Gate 全部检查 PASS，TPC-H SF=0.1 22/22 |
| R-Gate → G-Gate | R-Gate 全部检查 PASS，性能达标 |
| → GA | G-Gate 全部检查 PASS，所有 Issue 已关闭 |

### 4.3 版本号规则

> **详见**: `RELEASE_POLICY.md`

```text
v{MAJOR}.{MINOR}.{PATCH}-{phase}{N}
         │
         ├── MAJOR: 破坏性变更
         ├── MINOR: 新功能（向后兼容）
         └── PATCH: Bug 修复（向后兼容）
```

Tag 命名规范：
- Alpha: `vX.Y.Z-alpha{N}` → 示例: v3.1.0-alpha1
- Beta: `vX.Y.Z-beta{N}` → 示例: v3.1.0-beta1
- RC: `vX.Y.Z-rc{N}` → 示例: v3.1.0-rc1
- GA: `vX.Y.Z` → 示例: v3.1.0

---

## 五、分阶段门禁与跨版本追踪

### 5.1 门禁检查流程

> **详见**: `GATE_PHASES_AND_TRACKING.md`

```text
┌─────────────────────────────────────────────────────────────┐
│                      开始门禁检查                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 1: 识别门禁阶段 (A/B/R/G)                              │
│ - 检查当前分支和 milestone                                   │
│ - 确定执行的检查脚本                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 2: Pre-Gate 自检                                       │
│ - 代码已提交、已推送                                         │
│ - 环境准备就绪                                               │
│ - 数据准备完成                                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 3: 执行 scripts/gate/check_{phase}_v{VERSION}.sh       │
│ - 记录所有检查结果 (PASS/FAIL/SKIP)                         │
│ - 提取失败项的 ID、命令、输出                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 4: 结果分类                                            │
│ PASS → 记录证据，继续下一步                                  │
│ FAIL → 创建 Issue → 修复 PR → 验证 PASS                      │
│ SKIP → 人工判断 → 需要豁免? → 申请豁免                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 5: 生成检查报告                                         │
│ - 按 GATE_CHECKLIST_TEMPLATE.md 格式                        │
│ - 更新 milestone 状态                                        │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 跨版本追踪机制

```text
v3.0.0 B-Gate FAIL (SQL Corpus 20%)
         │
         ├── Issue #451 创建 (milestone: v3.0.0-beta)
         │
         ├── 修复需要 3 人周以上 → 触发版本延续
         │
         ▼
v3.1.0 DEVELOPMENT_PLAN.md §6 建立映射
┌──────────────────────────────────────┐
│ v3.1.0 延续任务（来自 v3.0.0）      │
├──────────────────────────────────────┤
│ #451 → SQL Operations 语法支持        │
│ 目标: 20% → ≥80%                     │
│ 优先级: P0                            │
└──────────────────────────────────────┘
         │
         ▼
v3.1.0 B-Gate 验证 #451 修复 PASS
         │
         ▼
Issue #451 关闭（需 PR 证据）
```

### 5.3 追踪检查清单

```markdown
## 跨版本追踪检查清单

### 版本发布后
- [ ] 确认所有 OPEN Issue 已分配 milestone
- [ ] 确认无法修复的 Issue 已延续到下版本 DEVELOPMENT_PLAN.md
- [ ] 确认豁免记录已写入 GATE_EXEMPTIONS.md
- [ ] 更新 gate_lifecycle_tracking.md §7 当前版本差距追踪

### 下版本开发开始时
- [ ] 查阅上版本 DEVELOPMENT_PLAN.md §6 延续任务
- [ ] 为每个延续任务创建对应的 Issue（如果原 Issue 已关闭）
- [ ] 确认延续任务的验收条件清晰定义
```

---

## 六、文档体系索引

### 6.1 核心 SSOT 文档

| 文档 | 用途 | 路径 |
|------|------|------|
| **GOVERNANCE_STANDARD.md** | 治理标准总纲（本文档） | `docs/governance/GOVERNANCE_STANDARD.md` |
| **GATE_SPEC_MASTER.md** | 门禁规范 SSOT | `docs/governance/GATE_SPEC_MASTER.md` |
| **VERSION_LIFECYCLE_MANAGEMENT.md** | 版本生命周期管理 | `docs/governance/VERSION_LIFECYCLE_MANAGEMENT.md` |
| **AI_COMPLIANCE_MECHANISM.md** | AI 合规执行机制 | `docs/governance/AI_COMPLIANCE_MECHANISM.md` |
| **GATE_PHASES_AND_TRACKING.md** | 分阶段门禁+跨版本追踪 | `docs/governance/GATE_PHASES_AND_TRACKING.md` |

### 6.2 治理流程文档

| 文档 | 用途 | 路径 |
|------|------|------|
| GOVERNANCE_INDEX.md | 文档导航索引 | `docs/governance/GOVERNANCE_INDEX.md` |
| gate_lifecycle_tracking.md | Issue 追踪闭环 | `docs/governance/gate_lifecycle_tracking.md` |
| RELEASE_LIFECYCLE.md | 四级门禁模型 | `docs/governance/RELEASE_LIFECYCLE.md` |
| RELEASE_POLICY.md | 发布策略 | `docs/governance/RELEASE_POLICY.md` |
| GATE_EXEMPTIONS.md | 门禁豁免记录 | `docs/governance/GATE_EXEMPTIONS.md` |
| governance_self_improvement.md | 治理自我进化 | `docs/governance/governance_self_improvement.md` |

### 6.3 模版文档

| 模版 | 用途 | 路径 |
|------|------|------|
| DEVELOPMENT_PLAN_TEMPLATE.md | 版本开发计划模版 | `docs/governance/DEVELOPMENT_PLAN_TEMPLATE.md` |
| TEST_PLAN_TEMPLATE.md | 测试计划模版 | `docs/governance/TEST_PLAN_TEMPLATE.md` |
| GATE_CHECKLIST_TEMPLATE.md | 门禁检查清单模版 | `docs/governance/GATE_CHECKLIST_TEMPLATE.md` |

### 6.4 版本特定文档命名规范

```text
docs/releases/v{VERSION}/
├── DEVELOPMENT_PLAN.md              # 版本开发计划
├── {PHASE}_GATE_CHECKLIST.md        # {ALPHA/BETA/RC/GA}_GATE_CHECKLIST.md
├── {PHASE}_TEST_PLAN.md             # {ALPHA/BETA/RC}_TEST_PLAN.md
└── GOVERNANCE_AUDIT.md              # 版本治理审计报告

scripts/gate/
├── check_alpha_v{VERSION}.sh        # Alpha 门禁脚本
├── check_beta_v{VERSION}.sh         # Beta 门禁脚本
├── check_rc_v{VERSION}.sh           # RC 门禁脚本
└── check_ga_v{VERSION}.sh           # GA 门禁脚本
```

---

## 七、检查清单

### 7.1 版本开发前检查

```markdown
## 版本开发前检查清单

### 治理文档检查
- [ ] GOVERNANCE_STANDARD.md 已阅读
- [ ] GATE_SPEC_MASTER.md 已参考
- [ ] VERSION_LIFECYCLE_MANAGEMENT.md 已参考
- [ ] AI_COMPLIANCE_MECHANISM.md 已参考（AI Agent）

### 文档创建检查
- [ ] 版本目录 docs/releases/v{VERSION}/ 已创建
- [ ] DEVELOPMENT_PLAN.md 已基于模版创建
- [ ] 测试计划已规划（Alpha/Beta/RC 各阶段）
- [ ] 门禁检查脚本已创建或更新

### 追踪机制检查
- [ ] Milestone v{VERSION}-{phase} 已创建
- [ ] Issue 追踪机制已就绪
- [ ] 上版本未完成任务已梳理
```

### 7.2 阶段门禁前检查

```markdown
## 阶段门禁前检查清单

### 门禁执行检查
- [ ] 在正确分支执行（develop/v{VERSION}）
- [ ] 使用正确的检查脚本
- [ ] Pre-Gate 自检已完成
- [ ] 所有检查结果已记录

### 失败项处理检查
- [ ] 所有 FAIL 项已创建 Issue
- [ ] Issue 已关联 milestone
- [ ] 修复 PR 已创建并关联 Issue
- [ ] PR 已合并或豁免已申请

### 文档更新检查
- [ ] GATE_CHECKLIST 已更新
- [ ] gate_lifecycle_tracking.md 已更新
- [ ] 延续任务已映射到下版本（如适用）
```

### 7.3 发布前最终检查

```markdown
## 发布前最终检查清单

### 版本完整性
- [ ] 所有 milestone 下的 Issue 已关闭
- [ ] 无 OPEN 的 blocker Issue
- [ ] 版本号与 Tag 一致

### 门禁通过
- [ ] A-Gate 已通过（Alpha 阶段）
- [ ] B-Gate 已通过（Beta 阶段）
- [ ] R-Gate 已通过（RC 阶段）
- [ ] G-Gate 已通过（GA 阶段）

### 文档完整性
- [ ] CHANGELOG 已更新
- [ ] Release Notes 已创建
- [ ] 用户文档已更新
- [ ] GOVERNANCE_AUDIT.md 已生成
```

---

## 八、规范同步要求

### 8.1 同步触发条件

| 触发事件 | 必须同步的文档 |
|----------|----------------|
| GATE_SPEC_MASTER.md 新增检查项 | 所有版本 gate 脚本、CHECKLIST 模版 |
| 模版文档变更 | 所有使用该模版的文档 |
| 新版本开发计划创建 | 版本目录下的所有文档 |
| 豁免申请批准 | GATE_EXEMPTIONS.md |
| 门禁失败记录 | gate_lifecycle_tracking.md |
| 治理体系重大变更 | GOVERNANCE_STANDARD.md（本文档） |

### 8.2 同步检查命令

```bash
# 检查规范与脚本是否同步
for gate in $(grep "^|| [A-Z][0-9]" GATE_SPEC_MASTER.md | awk '{print $2}'); do
    if ! grep -q "$gate" scripts/gate/check_*v*.sh; then
        echo "MISSING: $gate not in any gate script"
    fi
done

# 检查模版与文档是否一致
for doc in docs/releases/*/DEVELOPMENT_PLAN.md; do
    if ! head -20 "$doc" | grep -q "DEVELOPMENT_PLAN_TEMPLATE.md"; then
        echo "WARNING: $doc may not be based on template"
    fi
done
```

---

## 九、变更历史

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本，建立治理标准总纲 | hermes-z6g4 |

---

## 十、附录

### 10.1 术语表

| 术语 | 定义 |
|------|------|
| SSOT | Single Source of Truth，唯一权威来源 |
| Gate | 门禁，质量检查点 |
| Issue 闭环 | FAIL → Issue → PR → 验证 → 关闭的完整追踪链 |
| 版本延续 | 将当前版本未完成任务映射到下版本 |
| 证据格式 | 检查结果的标准化记录格式 |
| 豁免 | 因客观原因无法满足的门禁项，经审批后记录 |

### 10.2 快速参考命令

```bash
# 查看当前分支
git branch --show-current

# 查看最新 commit
git log --oneline -1

# 检查门禁脚本
ls scripts/gate/check_*.sh

# 执行 Alpha 门禁
bash scripts/gate/check_alpha_v{VERSION}.sh

# 检查文档链接
bash scripts/gate/check_docs_links.sh

# 查看 Issue 状态
gh issue list --milestone "v{VERSION}-{phase}"
```

---

*本文档是 SQLRustGo 治理体系的最高层规范。所有 AI Agent 和开发者在执行治理相关任务前必须阅读本文档。*

*维护人: hermes-z6g4*
*最后更新: 2026-05-14*
