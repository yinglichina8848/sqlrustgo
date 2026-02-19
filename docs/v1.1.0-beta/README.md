# v1.1.0-Beta 文档索引

## 阶段概述

- **版本**: v1.1.0-beta
- **开始时间**: 2026-02-20
- **目标**: 聚合函数实现 + 错误处理改进 + 测试覆盖率提升

## 文档列表

| 文档 | 描述 | 状态 |
|------|------|------|
| [任务看板](./task-board.md) | Beta 阶段任务追踪 | ✅ |
| [PR 证据链](./pr-evidence.md) | PR 审核与风险摘要 | ✅ |
| [阶段日报模板](./daily-template.md) | 课堂用日报模板 | ✅ |
| [执行手册(学生版)](./handbook-student.md) | 学生可复现步骤 | ⏳ |
| [执行手册(助教版)](./handbook-ta.md) | PR 证据链示例 | ⏳ |

## 相关链接

- [Issue #18 - Phase 2 任务](https://github.com/minzuuniversity/sqlrustgo/issues/18)
- [Phase 1 文档](../v1.0.0-alpha)
- [版本演进计划](../plans/2026-02-16-version-evolution-plan.md)

## 门禁检查

| 检查项 | 命令 | 要求 |
|--------|------|------|
| 编译 | `cargo build --all-features` | 通过 |
| 测试 | `cargo test --all-features` | 全部通过 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --check` | 通过 |
| 覆盖率 | `cargo tarpaulin` | ≥ 80% |
