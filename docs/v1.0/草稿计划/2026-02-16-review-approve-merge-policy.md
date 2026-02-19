# Evaluation -> Alpha 审查与合并责任策略

> 日期：2026-02-16  
> 适用阶段：v1.0.0 evaluation -> alpha

## 1. 角色定义

1. Author（作者）
- 提交代码和 PR。
- 不可审批自己的 PR。

2. Reviewer（审查者）
- 给出技术审查意见。
- 至少 1 人，且身份需独立于作者。

3. Approver（批准者）
- 在审查完成后执行 `Approve`。
- 可与 Reviewer 为同一人，但必须不是 Author。

4. Merger（合并者）
- 检查门禁是否达标后执行合并。
- 建议由主控（Codex）或仓库维护者执行。

## 2. Evaluation -> Alpha 的准入规则

1. 必须满足：
- `cargo test --all-features` 通过。
- `cargo clippy --all-features -- -D warnings` 通过。
- 覆盖率达到阶段目标。
- P0 问题为 0。

2. PR 必须满足：
- 目标分支为 `feature/v1.0.0-alpha`。
- 至少 1 个独立审批。
- 所有 review conversation 已 resolved。
- 无合并冲突。

## 3. 当前建议分工（v1.0.0）

1. `Author`
- OpenClaw 管理的执行 Agent（Claude/opencode）在任务分支提交。

2. `Reviewer/Approver`
- 非作者平台的 Agent 身份执行审查与批准。
- 需要和 Author 使用不同账号身份，避免 self-review 限制。

3. `Merger`
- Codex 主控负责最终门禁复核与合并。

## 4. 操作流程

1. Author 发起 `evaluation -> alpha` PR。
2. Reviewer 审查并评论问题。
3. Author 修复并更新 PR。
4. Approver 执行批准。
5. Merger 复核门禁并合并。

## 5. 一票否决项

1. 合并冲突未解决。
2. 自审自批。
3. 必要检查未通过。
4. P0 未清零且无书面豁免。
