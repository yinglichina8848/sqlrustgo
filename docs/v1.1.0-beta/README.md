# v1.1.0-Beta 文档索引

## 阶段概述

- **版本**: v1.1.0-beta
- **开始时间**: 2026-02-20
- **目标**: 聚合函数实现 + 错误处理改进 + 测试覆盖率提升

## 文档列表

### 用户文档
给运行 sqlrustgo 可执行程序的用户

| 文档 | 描述 | 状态 |
|------|------|------|
| [用户指南](./user/README.md) | REPL 使用、SQL 支持、服务器模式 | ✅ |

### 开发文档
给开发者使用，包含开发环境、项目结构、代码规范

| 文档 | 描述 | 状态 |
|------|------|------|
| [开发指南](./developer/README.md) | 开发环境搭建、代码规范、调试 | ✅ |

### 测试文档
教使用者如何对 sqlrustgo 进行功能和性能测试

| 文档 | 描述 | 状态 |
|------|------|------|
| [测试指南](./testing/README.md) | 单元测试、集成测试、性能测试 | ✅ |

### 流程文档

| 文档 | 描述 | 状态 |
|------|------|------|
| [任务看板](./task-board.md) | Beta 阶段任务追踪 | ✅ |
| [PR 证据链](./pr-evidence.md) | PR 审核与风险摘要 | ✅ |
| [阶段日报模板](./daily-template.md) | 课堂用日报模板 | ✅ |
| [执行手册(学生版)](./handbook-student.md) | 学生可复现步骤 | ✅ |
| [执行手册(助教版)](./handbook-ta.md) | PR 证据链示例 | ✅ |

## 门禁检查

| 检查项 | 命令 | 要求 |
|--------|------|------|
| 编译 | `cargo build --all-features` | 通过 |
| 测试 | `cargo test --all-features` | 全部通过 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --check` | 通过 |
| 覆盖率 | `cargo tarpaulin` | ≥ 80% |

## 相关链接

- [Issue #18 - Phase 2 任务](https://github.com/minzuuniversity/sqlrustgo/issues/18)
- [Phase 1 文档](../v1.0.0-alpha)
- [版本演进计划](../plans/2026-02-16-version-evolution-plan.md)
