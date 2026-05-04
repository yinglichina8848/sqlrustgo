# v2.9.0 发布说明

> **版本**: v2.9.0
> **发布日期**: 2026-05-05
> **阶段**: RC

---

## 版本概述

v2.9.0 是**企业级韧性**版本，聚焦于分布式架构完成和生产就绪特性。

---

## 新增功能

### 分布式架构 (D-01 ~ D-04)

- **D-01 Semi-sync 复制**: 确保数据在主从之间同步
- **D-02 MTS 并行复制**: 提升从库复制性能
- **D-03 Multi-source 复制**: 支持多主源场景
- **D-04 XA 事务**: 两阶段提交支持

### SQL 兼容性提升 (C-01 ~ C-06)

- **CTE/WITH**: 通用表表达式和递归查询
- **窗口函数**: ROW_NUMBER, RANK, DENSE_RANK, PARTITION BY
- **CASE/WHEN**: 完整条件表达式
- **JSON 操作**: JSON 提取和路径查询

### MySQL 5.7 命令补全

- CREATE/DROP TABLE IF [NOT] EXISTS
- INSERT ON DUPLICATE KEY UPDATE
- ALTER TABLE DROP/MODIFY COLUMN
- CREATE VIEW / DROP VIEW
- SHOW DATABASES / SHOW CREATE TABLE

### 形式化验证 (Phase S)

- Proof Registry v2 系统
- TLA+ / Dafny / Formulog 三工具验证
- Proof Coverage 报告

---

## 重大变更

无重大破坏性变更。

---

## 性能状态

| 指标 | 目标 | 当前 | 状态 |
|------|------|------|------|
| SQL Corpus | ≥85% | 92.6% | ✅ |
| TPC-H SF=0.1 | 18/22 | 13/22 | ⚠️ v2.10.0 |
| sysbench QPS | ≥10,000 | ~2,000 | ⚠️ v2.10.0 |

---

## 升级指南

从 v2.8.0 升级无需特殊步骤：

```bash
cargo update
cargo build --all
```

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)。

---

## 已知问题

| 问题 | 严重程度 | 解决方式 |
|------|----------|----------|
| TPC-H 18/22 未达成 | 低 | v2.10.0 继续优化 |
| sysbench QPS < 10K | 低 | v2.10.0 优化 |

---

## 相关链接

- [版本计划](./VERSION_PLAN.md)
- [测试计划](./TEST_PLAN.md)
- [门禁清单](./RELEASE_GATE_CHECKLIST.md)
- [CHANGELOG](./CHANGELOG.md)

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
