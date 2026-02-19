## PR 标题规范检查

<!-- PR 标题必须符合格式: <type>(<scope>): <summary> -->
<!-- 示例: feat(auth): implement basic authentication -->

- [ ] 标题符合 `<type>(<scope>): <summary>` 格式
- [ ] type 是以下之一: feat, fix, perf, refactor, test, chore, docs
- [ ] scope 是以下之一: parser, executor, planner, network, auth, storage, optimizer, ci, docs

## 变更概述

<!-- 描述此 PR 的范围和意图 -->

## 关联 Issue

Closes #

## Beta 阶段检查

<!-- Beta 阶段只允许 fix, perf, refactor 类型 -->

- [ ] 此 PR 类型符合 Beta 阶段收敛策略 (fix/perf/refactor)
- [ ] 此 PR 不包含大型新功能或破坏性改动

## 风险评估

- 变更风险: low / medium / high
- 主要风险点:
- 影响范围:

## 回滚计划

- 回滚策略:
- 数据兼容性说明:

## 验证证据

- [ ] `cargo build --all-features` 通过
- [ ] `cargo test --all-features` 通过
- [ ] `cargo clippy --all-features -- -D warnings` 通过
- [ ] `cargo fmt --check --all` 通过
- [ ] 覆盖率未下降
- [ ] 无新增 unwrap/panic

## 门禁检查清单

- [ ] 目标分支正确 (`feature/v1.0.0-beta`)
- [ ] CI 检查通过
- [ ] 至少 1 位独立审核者批准
- [ ] 所有审核对话已解决
- [ ] P0 问题为零（或已明确记录例外）
- [ ] 必要的文档/changelog 已更新
