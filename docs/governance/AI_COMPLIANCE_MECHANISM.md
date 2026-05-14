# SQLRustGo AI 合规执行机制

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **用途**: 定义 AI Agent 在 SQLRustGo 项目中的合规行为规范、执行约束和违规处理机制
> **SSOT**: 本文是 AI 合规执行的唯一权威来源

> **关联文档**:
> - `AI_COLLABORATION.md` — 人机协作模式定义
> - `GOVERNANCE_STANDARD.md` — 治理标准总纲
> - `GOVERNANCE_INDEX.md` — 文档导航索引
> - `GATE_PHASES_AND_TRACKING.md` — 分阶段门禁追踪

---

## 一、概述

### 1.1 目的

本文档定义 AI Agent 在 SQLRustGo 项目中执行治理相关任务时的合规行为规范，确保：

1. AI 正确使用 governance 模版文档
2. AI 执行门禁检查时遵循规范流程
3. AI 的 Issue 追踪行为符合闭环要求
4. AI 的版本延续操作可追溯
5. AI 不会绕过必须人工审批的操作

### 1.2 适用范围

所有 AI Agent（包括但不限于）：

| AI Agent | 标识 | 职责 |
|----------|------|------|
| **Hermes Agent** | hermes-* | 自主工程 Agent，规则执行 |
| **AI Developer** | ai-dev-* | 代码实现，测试编写 |
| **AI Maintainer** | ai-maint-* | 代码审查，问题诊断 |

### 1.3 合规要求总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AI 合规要求三大支柱                             │
├─────────────────────────────────────────────────────────────────────┤
│  1. 模版强制使用   │ 所有 governance 文档必须基于模版创建            │
│  2. 流程规范遵循   │ 执行门禁检查必须遵循 GATE_PHASES_AND_TRACKING  │
│  3. 权限边界遵守   │ 禁止执行需要人工审批的操作                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 二、模版强制使用机制

### 2.1 必须使用模版的任务

| 任务类型 | 触发关键词 | 必须使用的模版 | 输出路径 |
|----------|------------|----------------|----------|
| 版本开发计划 | "开发计划"、"版本计划"、"v{X}.{Y}.{Z} 开发" | `DEVELOPMENT_PLAN_TEMPLATE.md` | `docs/releases/v{VERSION}/DEVELOPMENT_PLAN.md` |
| 测试计划 | "测试计划"、"测试策略"、"测试用例" | `TEST_PLAN_TEMPLATE.md` | `docs/releases/v{VERSION}/{PHASE}_TEST_PLAN.md` |
| 门禁检查 | "门禁检查"、"Gate"、"门禁报告"、"Alpha/Beta/RC/GA" | `GATE_CHECKLIST_TEMPLATE.md` | `docs/releases/v{VERSION}/{PHASE}_GATE_CHECKLIST.md` |
| Issue 追踪 | "创建 Issue"、"关闭 Issue"、"Issue 追踪" | `gate_lifecycle_tracking.md` 规范 | — |
| 版本延续 | "版本延续"、"延续任务" | 在 `DEVELOPMENT_PLAN.md` §6 建立映射 | — |

### 2.2 模版使用规则

```text
❌ 禁止: 自由格式创建 governance 文档
❌ 禁止: 部分使用模版（如跳过某些章节）
❌ 禁止: 修改模版章节结构
❌ 禁止: 使用过期的模版版本

✅ 必须: 基于模版创建所有 governance 文档
✅ 必须: 包含模版要求的所有章节
✅ 必须: 使用模版指定的文件路径
✅ 必须: 替换所有占位符 {VERSION}、{PHASE} 等
```

### 2.3 模版加载流程

```
步骤 1: 识别任务类型
    │
    ├── 版本开发计划 → DEVELOPMENT_PLAN_TEMPLATE.md
    ├── 测试计划 → TEST_PLAN_TEMPLATE.md
    ├── 门禁检查 → GATE_CHECKLIST_TEMPLATE.md
    └── 其他 → 查阅 GOVERNANCE_INDEX.md

步骤 2: 验证模版存在且为最新
    │
    └── ls docs/governance/{模版名}.md

步骤 3: 按模版执行任务
    │
    ├── 严格遵循章节结构
    ├── 替换所有占位符
    └── 记录证据

步骤 4: 验证输出
    │
    ├── 检查文件路径正确
    ├── 检查章节完整
    └── 检查证据格式正确
```

---

## 三、门禁检查合规执行

### 3.1 门禁执行流程（AI 版）

```text
┌─────────────────────────────────────────────────────────────────────┐
│                    AI 执行门禁检查流程                                 │
├─────────────────────────────────────────────────────────────────────┤
│ 步骤 1: 任务识别                                                     │
│ [ ] 检测到门禁相关关键词                                             │
│ [ ] 查阅 GOVERNANCE_INDEX.md 确定执行流程                            │
│ [ ] 确定门禁阶段 (A/B/R/G)                                          │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 步骤 2: 加载模版                                                     │
│ [ ] 加载 GATE_CHECKLIST_TEMPLATE.md                                 │
│ [ ] 确认检查脚本路径: scripts/gate/check_{phase}_v{VERSION}.sh     │
│ [ ] 确认版本目录: docs/releases/v{VERSION}/                         │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 步骤 3: Pre-Gate 自检                                               │
│ [ ] 确认分支正确 (develop/v{VERSION})                               │
│ [ ] 确认代码已提交                                                   │
│ [ ] 确认环境准备就绪                                                 │
│ [ ] 确认数据准备完成                                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 步骤 4: 执行门禁脚本                                                 │
│ [ ] 执行: bash scripts/gate/check_{phase}_v{VERSION}.sh            │
│ [ ] 记录所有检查结果 (PASS/FAIL/SKIP)                                │
│ [ ] 提取失败项的 ID、命令、输出                                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ 步骤 5: 结果处理                                                     │
│                                                                         │
│ FAIL → [ ] 创建 Issue (如需修复)                                      │
│       → [ ] 评估是否可延续到下版本                                    │
│       → [ ] 在 DEVELOPMENT_PLAN.md §6 建立映射                       │
│                                                                         │
│ PASS → [ ] 记录证据                                                   │
│       → [ ] 生成报告                                                   │
│       → [ ] 更新 milestone                                            │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 AI 执行门禁检查时的禁止行为

```text
❌ 禁止: 在门禁 FAIL 时跳过 Issue 创建直接报告"PASS"
❌ 禁止: 在门禁 FAIL 时修改检查脚本使检查通过
❌ 禁止: 跳过任何检查项不执行
❌ 禁止: 修改检查阈值使本应 FAIL 的项变为 PASS
❌ 禁止: 在证据中造假（如修改命令输出）
❌ 禁止: 跳过豁免申请直接忽略无法满足的检查项
```

### 3.3 证据格式规范

所有门禁检查必须记录以下格式的证据：

```json
{
  "gate": "{PHASE}-GATE-v{VERSION}",
  "commit": "{sha}",
  "date": "{YYYY-MM-DD HH:mm:ss}",
  "executor": "{AI_AGENT_ID}",
  "status": "PASS|FAIL",
  "summary": {
    "total": {N},
    "passed": {N},
    "failed": {N},
    "skipped": {N}
  },
  "evidence": {
    "{GATE_ITEM}": {
      "command": "{执行的命令}",
      "exit_code": {N},
      "output_summary": "{输出摘要}",
      "threshold": "{通过标准}",
      "pass": true|false
    }
  },
  "blockers": [
    {
      "item": "{GATE_ITEM}",
      "issue": "#{issue_number}",
      "pr": "#{pr_number}",
      "status": "{FIXED|IN_PROGRESS|EXEMPTED}"
    }
  ]
}
```

---

## 四、Issue 追踪合规

### 4.1 Issue 创建合规

AI 在创建 Issue 时必须：

```text
✅ 必须: 关联到具体的门禁检查项
✅ 必须: 包含检查命令和失败输出
✅ 必须: 设置正确的 milestone
✅ 必须: 添加正确的 labels
✅ 必须: 包含验收条件
```

**Issue 创建标准格式**：

```markdown
### Issue 标题
[{GATE_ITEM}] {简短描述}

### Issue 内容
- 来源门禁: `check_{phase}_v{VERSION}.sh`
- 检查项: {GATE_ITEM} (如 B9, R10, B-S1 等)
- 检查命令: `{实际执行的命令}`
- 失败输出:
```
{paste output}
```

### 根因分析
{analysis}

### 影响范围
- 阻塞: {PHASE}-Gate
- 影响: {other impacts}

### 验收条件
- [ ] {condition 1}
- [ ] {condition 2}

### 追踪信息
- milestone: v{VERSION}-{phase}
- labels: source/gate-{phase}, type/{type}
```

### 4.2 Issue 关闭合规

**AI 禁止在没有 PR 证据的情况下关闭 Issue。**

关闭前必须验证：

```bash
# Step 1: 检查是否有 PR 关闭该 Issue
gh issue view {id} --json closedByPullRequestsReferences
# 结果非空 → 可以关闭
# 结果为空 → 禁止关闭

# Step 2: 验证 PR 已合并
gh pr view <pr_number} --json state,mergedAt
# state = MERGED 且 mergedAt 有值 → 可以关闭
# 否则 → 禁止关闭

# Step 3: 验证测试通过
cargo test -p sqlrustgo-{package} --test {test_name}
# 必须测试通过

# Step 4: 验证门禁重新检查 PASS
bash scripts/gate/check_{phase}_v{VERSION}.sh
# 必须 PASS
```

### 4.3 版本延续合规

当任务需要延续到下版本时，AI 必须：

```text
✅ 必须: 在 DEVELOPMENT_PLAN.md §6 建立映射
✅ 必须: 原 Issue 已关闭或有豁免记录
✅ 必须: 新版本 DEVELOPMENT_PLAN.md 引用原 Issue
✅ 必须: 验收条件清晰定义
```

**版本延续格式**：

```markdown
## v{NEXT_VERSION} 延续任务（来自 v{CURRENT_VERSION} 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | v{NEXT_VERSION} 目标 | 验收条件 | 优先级 |
|----------|----------|------------|---------------------|----------|--------|
| #{issue} | {任务描述} | {当前状态} | {目标} | {验收条件} | {P0/P1} |
```

---

## 五、权限边界

### 5.1 AI 禁止执行的操作

以下操作需要 Human Architect 审批，AI 禁止执行：

```text
❌ 禁止: 合并 PR 到 main 或 develop 分支
❌ 禁止: 创建或删除 Git Tag
❌ 禁止: 发布 Release
❌ 禁止: 修改分支保护规则
❌ 禁止: 删除 GitHub/Gitea Issue
❌ 禁止: 修改仓库设置
❌ 禁止: 批准门禁豁免
❌ 禁止: 发布版本公告
```

### 5.2 AI 需要先创建 Issue 才能执行的操作

```text
⚠️  以下操作需要先创建 Issue 并获得批准后才能执行:

需要先创建 Issue:
├── 代码架构变更 (需先创建 Issue 讨论)
├── 新增依赖包 (需先创建 Issue 说明)
├── 修改 CI 配置 (需先创建 Issue 记录)
└── 发布非标准版本 (需先创建 Issue 审批)
```

### 5.3 AI 可以自主执行的操作

```text
✅ 可以: 创建 Issue
✅ 可以: 创建 PR（需关联 Issue）
✅ 可以: 执行门禁检查
✅ 可以: 运行测试和构建
✅ 可以: 更新文档
✅ 可以: 提交代码到非保护分支
✅ 可以: 评论 Issue/PR
```

---

## 六、合规检查清单

### 6.1 AI 执行治理任务前检查

```markdown
## AI 合规自查清单 — 执行前

### 任务识别
- [ ] 确认任务类型（查阅 GOVERNANCE_INDEX.md）
- [ ] 确认需要使用的模版文件
- [ ] 确认模版版本为最新

### 模版检查
- [ ] 模版文件存在: `docs/governance/{模版名}.md`
- [ ] 准备基于模版创建文档

### 环境检查（门禁任务）
- [ ] 在正确分支执行
- [ ] 确认脚本路径正确
```

### 6.2 AI 执行治理任务中检查

```markdown
## AI 合规自查清单 — 执行中

### 模版遵循
- [ ] 严格按模版章节结构
- [ ] 未跳过任何章节
- [ ] 所有占位符已替换

### 证据记录
- [ ] 命令已记录
- [ ] 输出已记录
- [ ] 阈值已记录
- [ ] 结果判定已记录

### 禁止行为检查
- [ ] 未跳过检查项
- [ ] 未修改阈值
- [ ] 未伪造证据
```

### 6.3 AI 执行治理任务后检查

```markdown
## AI 合规自查清单 — 执行后

### 输出验证
- [ ] 文档已创建在正确路径
- [ ] 模版要求的所有章节已包含
- [ ] 证据格式符合规范

### Issue 处理（FAIL 时）
- [ ] Issue 已创建
- [ ] Issue 包含所有必需字段
- [ ] 评估是否需要版本延续

### 通知（如需要）
- [ ] Human Architect 已通知（如有阻塞性问题）
```

---

## 七、违规处理

### 7.1 违规分类

| 违规类型 | 严重程度 | 处理方式 |
|----------|----------|----------|
| 使用非模版创建文档 | 高 | 要求重做，记录到治理审计 |
| 门禁 FAIL 但报告 PASS | 严重 | 立即纠正，发布更正报告 |
| 无 PR 证据关闭 Issue | 严重 | 重新打开 Issue，追究责任 |
| 跳过检查项 | 高 | 要求补充检查，记录缺失 |
| 伪造证据 | 严重 | 立即纠正，审计历史记录 |
| 越权执行操作 | 严重 | 回滚操作，通知 Human Architect |

### 7.2 违规报告

当发现 AI 违规行为时，应：

```markdown
## AI 违规报告

### 基本信息
- AI Agent ID: {id}
- 发现时间: {datetime}
- 发现人: {human_or_agent_id}

### 违规详情
- 违规类型: {type}
- 涉及文档/操作: {path_or_operation}
- 违规描述: {description}

### 影响评估
- 对质量的影响: {impact}
- 对追溯性的影响: {impact}

### 处理措施
- [ ] {measure 1}
- [ ] {measure 2}

### 预防措施
- [ ] {prevention 1}
```

---

## 八、合规培训

### 8.1 新 AI Agent 上岗前培训

新加入的 AI Agent 必须：

1. 阅读并理解本文档
2. 阅读 `AI_COLLABORATION.md`
3. 阅读 `GOVERNANCE_INDEX.md`
4. 通过模拟门禁检查测试
5. 获得 Human Architect 的初始授权

### 8.2 持续合规

```text
定期（每版本周期）:
├── AI Agent 回顾上版本合规表现
├── Human Architect 评估 AI 行为
├── 更新合规规范（如有必要）
└── 记录改进建议
```

---

## 九、相关文档

| 文档 | 作用 | 路径 |
|------|------|------|
| AI_COLLABORATION.md | 人机协作模式定义 | `docs/governance/AI_COLLABORATION.md` |
| GOVERNANCE_INDEX.md | 文档导航索引 | `docs/governance/GOVERNANCE_INDEX.md` |
| GOVERNANCE_STANDARD.md | 治理标准总纲 | `docs/governance/GOVERNANCE_STANDARD.md` |
| GATE_PHASES_AND_TRACKING.md | 分阶段门禁追踪 | `docs/governance/GATE_PHASES_AND_TRACKING.md` |
| gate_lifecycle_tracking.md | Issue 追踪闭环 | `docs/governance/gate_lifecycle_tracking.md` |

---

## 十、变更历史

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本，建立 AI 合规执行机制 | hermes-z6g4 |

---

## 附录 A: 模版速查表

| 任务 | 模版 | 路径 |
|------|------|------|
| 版本开发计划 | DEVELOPMENT_PLAN_TEMPLATE.md | `docs/governance/DEVELOPMENT_PLAN_TEMPLATE.md` |
| 测试计划 | TEST_PLAN_TEMPLATE.md | `docs/governance/TEST_PLAN_TEMPLATE.md` |
| 门禁检查清单 | GATE_CHECKLIST_TEMPLATE.md | `docs/governance/GATE_CHECKLIST_TEMPLATE.md` |
| 版本门禁清单 | {PHASE}_GATE_CHECKLIST.md | `docs/releases/v{VERSION}/{PHASE}_GATE_CHECKLIST.md` |

## 附录 B: 禁止行为快速参考

```text
❌ 禁止: 自由格式创建 governance 文档
❌ 禁止: 跳过 Issue 创建直接报告 PASS
❌ 禁止: 修改检查阈值使 FAIL 变 PASS
❌ 禁止: 伪造证据或修改命令输出
❌ 禁止: 无 PR 证据关闭 Issue
❌ 禁止: 合并 PR 到保护分支
❌ 禁止: 发布 Release 或创建 Tag
❌ 禁止: 修改分支保护规则
❌ 禁止: 批准门禁豁免
```

## 附录 C: 合规命令速查

```bash
# 查看模版
cat docs/governance/DEVELOPMENT_PLAN_TEMPLATE.md
cat docs/governance/TEST_PLAN_TEMPLATE.md
cat docs/governance/GATE_CHECKLIST_TEMPLATE.md

# 验证分支
git branch --show-current

# 检查门禁脚本
ls scripts/gate/check_*.sh

# 执行门禁检查
bash scripts/gate/check_{phase}_v{VERSION}.sh

# 检查 Issue 状态
gh issue list --milestone "v{VERSION}-{phase}"

# 验证 Issue 关闭条件
gh issue view {id} --json closedByPullRequestsReferences
```

---

*本文档是 SQLRustGo AI 合规执行的唯一权威来源。所有 AI Agent 在执行治理相关任务时必须遵循本文档的规定。*

*维护人: hermes-z6g4*
*最后更新: 2026-05-14*
