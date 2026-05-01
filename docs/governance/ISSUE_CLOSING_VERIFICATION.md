# Issue 关闭验证流程

> **版本**: v1.0
> **日期**: 2026-04-23
> **目的**: 防止错误关闭未完成的 Issue

---

## 一、核心原则

**禁止在没有充分证据的情况下关闭 Issue。**

Issue 是任务追踪的最小单位。关闭 Issue 意味着任务已完成。错误的关闭会导致：
1. 任务遗漏
2. 进度虚报
3. 团队信任受损

---

## 二、关闭 Issue 的前置条件

### 2.1 必须满足的条件 (ALL)

| # | 条件 | 验证方法 |
|---|------|----------|
| 1 | **PR 已合并** | `gh issue view <id> --json closedByPullRequestsReferences` 返回非空 |
| 2 | **代码已集成** | 代码在 `main` 或 `develop/*` 分支 |
| 3 | **测试已通过** | 相关测试在 CI/本地通过 |
| 4 | **文档已更新** | 必要文档已更新并合并 |

### 2.2 验收标准检查清单

关闭 Issue 前，必须确认：

```bash
# 1. 检查 PR 是否关闭了该 Issue
gh issue view <id> --json closedByPullRequestsReferences

# 如果为空，说明没有 PR 关闭该 Issue，不能手动关闭！
```

---

## 三、Issue 类型与关闭要求

### 3.1 功能开发 Issue

| 阶段 | 要求 |
|------|------|
| 代码开发 | PR 合并到目标分支，测试通过 |
| 文档更新 | 文档 PR 合并，用户可见 |
| 集成测试 | 集成测试通过并合并 |
| 发布 | 版本标签创建，Release PR 合并 |

### 3.2 测试/修复 Issue

| Issue 类型 | 关闭要求 |
|-----------|----------|
| Bug 修复 | PR 合并 + 回归测试通过 |
| 性能优化 | 性能测试结果符合预期 |
| 测试用例 | 测试用例已集成到测试套件 |
| 文档修复 | 文档更新已合并 |

### 3.3 追踪 Issue (Task/Sub-task)

| 状态 | 处理方式 |
|------|----------|
| 有对应 PR | PR 合并后自动关闭 |
| 无对应 PR | **禁止手动关闭**，除非任务取消并说明原因 |
| 部分完成 | 保持 Open，更新任务状态 |

---

## 四、验证流程 (强制执行)

### Step 1: 检查 closedByPullRequestsReferences

```bash
gh issue view <issue_number> --json closedByPullRequestsReferences
```

**判断规则**:
- ✅ 非空 → PR 已关闭该 Issue，可以验证
- ❌ **为空** → **禁止手动关闭**，除非任务取消

### Step 2: 验证 PR 状态

```bash
gh pr view <pr_number> --json state,mergedAt
```

确认:
- `state` = `MERGED`
- `mergedAt` 有值

### Step 3: 验证代码集成

```bash
# 确认代码已合并到目标分支
git log --oneline <target_branch> | grep <commit_hash>
```

### Step 4: 验证测试通过

```bash
# 运行相关测试
cargo test --all-features <test_name>

# 或检查 CI 状态
gh pr view <pr_number> --json checks
```

### Step 5: 验证文档更新 (如适用)

```bash
# 检查文档变更是否合并
gh pr diff <pr_number> --name-only | grep -E "\.md$"
```

---

## 五、AI Agent 关闭 Issue 前的检查清单

当用户要求关闭 Issue 时，Agent **必须** 执行以下检查：

```
Issue 关闭前检查清单
========================

Issue #: ___________

[ ] 1. 执行 `gh issue view <id> --json closedByPullRequestsReferences`
    结果: ___________ (必须非空才能继续)

[ ] 2. 执行 `gh pr view <pr_number> --json state,mergedAt`
    state: ___________
    mergedAt: ___________

[ ] 3. 确认代码已合并到目标分支
    分支: ___________
    Commit: ___________

[ ] 4. 确认测试通过
    测试命令: ___________
    结果: ___________

[ ] 5. 确认文档已更新 (如适用)
    文档变更: ___________

结论: ___________ (可以关闭 / 禁止关闭 - 原因: ___________)
```

---

## 六、错误关闭 Issue 的纠正流程

如果错误关闭了 Issue，必须立即重新打开：

```bash
# 重新打开 Issue
gh issue reopen <issue_number> -c "<原因说明>"
```

**常见错误场景**:
1. `closedByPullRequestsReferences` 为空时手动关闭
2. PR 合并但测试未通过
3. 代码合并到临时分支而非目标分支
4. 文档未合并

---

## 七、禁止的模式

| 禁止行为 | 原因 |
|-----------|------|
| 手动关闭无 PR 的功能 Issue | 无法追踪实际完成状态 |
| 仅因"任务看起来完成"就关闭 | 缺乏客观证据 |
| 批量关闭多个 Issue | 每个 Issue 需要独立验证 |
| 关闭追踪 Issue 但不验证子任务 | 任务状态不一致 |

---

## 八、相关文档

- [RC_TO_GA_GATE_CHECKLIST.md](./RC_TO_GA_GATE_CHECKLIST.md) - 发布门禁清单
- [BRANCH_GOVERNANCE.md](./BRANCH_GOVERNANCE.md) - 分支治理
- [DOCUMENT_REVIEW_WORKFLOW.md](./DOCUMENT_REVIEW_WORKFLOW.md) - 文档审查流程

---

*本文档由 SQLRustGo Team 维护*
*最后更新: 2026-04-23*
