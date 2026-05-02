# v2.9.0 发布说明

> **版本**: v2.9.0
> **发布日期**: 2026-05-xx
> **阶段**: Alpha

## 版本概述

v2.9.0 是**企业级韧性**版本，聚焦于分布式架构完成和生产就绪特性。

## 主要新功能

### 分布式架构 (D-01 ~ D-04)

- **D-01 Semi-sync 复制**: 确保数据在主从之间同步
- **D-02 MTS 并行复制**: 提升从库复制性能
- **D-03 Multi-source 复制**: 支持多主源场景
- **D-04 XA 事务**: 两阶段提交支持

### SQL 兼容性提升 (C-01 ~ C-06)

- **CTE/WITH**: 通用表表达式和递归查询
- **窗口函数**: ROW_NUMBER, RANK, DENSE_RANK
- **CASE/WHEN**: 完整条件表达式
- **JSON 操作**: JSON 提取和路径查询

### MySQL 5.7 命令补全

- CREATE/DROP TABLE IF [NOT] EXISTS
- INSERT ON DUPLICATE KEY UPDATE
- ALTER TABLE DROP/MODIFY COLUMN
- CREATE/DROP VIEW
- SHOW DATABASES / SHOW CREATE TABLE

## 重大变更

无重大破坏性变更。

## 升级指南

从 v2.8.0 升级无需特殊步骤：

```bash
cargo update
cargo build --all
```

## 性能目标

| 指标 | 目标 | 状态 |
|------|------|------|
| QPS | ≥10,000 | ⏳ 进行中 |
| 延迟 P99 | <10ms | ⏳ 待测 |

## 已知问题

- E-08 性能优化目标 (≥10K QPS) 尚未达成，将在后续版本继续优化

## 相关链接

- [版本计划](./VERSION_PLAN.md)
- [测试计划](./TEST_PLAN.md)
- [门禁清单](./RELEASE_GATE_CHECKLIST.md)