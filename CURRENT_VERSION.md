# 当前版本状态

alpha/v2.7.0

## 阶段信息

- **阶段**: Alpha (早期开发)
- **开始日期**: 2026-04-22
- **开发分支**: develop/v2.7.0
- **目标**: 下一代 SQLRustGo
- **开发 Issue**: [#1697](https://github.com/minzuuniversity/sqlrustgo/issues/1697)

## 版本概述

v2.7.0 是 SQLRustGo 的下一代版本，专注于性能优化和高级特性。

**核心目标**: 提升性能、增强可扩展性、完善分布式支持。

## 当前执行口径（2026-04-22）

- 当前版本统一为 `alpha/v2.7.0`
- 当前主开发分支为 `develop/v2.7.0`
- 基于 v2.6.0 (RC 阶段)

## v2.7.0 核心任务

### Phase A-C ✅ 已完成 (2026-04-22)

| 阶段 | 功能 | 状态 | PR |
|------|------|------|-----|
| Phase A | T-01 事务/WAL 恢复 | ✅ | - |
| Phase A | T-02 FK/约束稳定化 | ✅ | - |
| Phase A | T-03 备份恢复演练 | ✅ | - |
| Phase B | T-04 qmd-bridge | ✅ | #1713 |
| Phase B | T-05 统一检索API | ✅ | #1714 |
| Phase B | T-06 混合检索重排 | ✅ | #1714 |
| Phase C | T-07 GMP Top10 | ✅ | #1714 |
| Phase C | T-08 审计证据链 | ✅ | #1718 |

### Phase D - RC/GA 冲刺 (当前)

| 功能 | 状态 | Issue |
|------|------|-------|
| 72h 长稳测试 | 🔄 待执行 | #1691 |
| RC1 门禁验收 | 🔄 待执行 | - |
| 发布 Checkpoint | 🔄 规划中 | - |

## 开发时间线

| 版本 | 日期 | 目标 |
|------|------|------|
| v2.7.0-alpha | 2026-04-22 | Phase A-C 完成 ✅ |
| v2.7.0-beta | TBD | Beta 测试 |
| v2.7.0-rc | TBD | RC 候选 |
| v2.7.0-GA | TBD | 正式发布 |

## 相关文档

- [v2.6.0 文档入口](docs/releases/v2.6.0/README.md)
- [v2.6.0 版本计划](docs/releases/v2.6.0/VERSION_PLAN.md)
- [v2.7.0 开发计划](https://github.com/minzuuniversity/sqlrustgo/issues/1697)

## 变更历史

| 版本 | 日期 | 说明 |
|------|------|-------|
| 1.0 | 2026-04-22 | 创建 v2.7.0 开发分支 |
