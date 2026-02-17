# 单账号评审协议（临时）

> 日期：2026-02-17
> 适用：只有一个 GitHub 账号时的 `evaluation/alpha/beta`

## 1. 目标

在无法使用独立账号 `Approve` 的前提下，保留“可审计审查”而不是无评审合并。

## 2. 强制要求

1. 每个 PR 必须有至少 1 条实例评审评论（Claude/OpenCode 任一实例）。
2. 评审评论必须包含：
   - 验证命令
   - 风险点
   - 结论（建议合并/阻塞）
3. Gatekeeper（Codex）必须在 PR 或 Issue 留“主控签字评论”后才可合并。
4. 没有“评审评论 URL + 主控签字”的 PR，不得合并。

## 3. 评论模板

```markdown
## Review Report

- Reviewer instance: <instance-name>
- Scope: <files/modules>
- Commands:
  - cargo build --all-features
  - cargo test --all-features
  - cargo clippy --all-features -- -D warnings
- Risks:
  - <risk-1>
  - <risk-2>
- Decision: merge / hold
- Reason:
```

## 4. 回切条件（恢复严格审批）

当第二 GitHub 账号就位后：

1. 恢复 `evaluation/alpha/beta` 的 required approvals = 1。
2. 保留本协议中的“评审评论 + 主控签字”作为附加审计要求。
3. `baseline/main` 继续执行独立审批。
