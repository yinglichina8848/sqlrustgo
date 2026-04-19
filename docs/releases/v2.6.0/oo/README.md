# SQLRustGo v2.6.0 OO 架构文档

> **版本**: alpha/v2.6.0
> **更新日期**: 2026-04-18

---

## 概述

本文档目录包含 SQLRustGo v2.6.0 的面向对象架构设计文档、模块设计、报告和用户指南。

---

## 目录结构

```
oo/
├── architecture/          # 架构设计文档
│   └── ARCHITECTURE_V2.6.md
├── modules/             # 模块设计文档
│   ├── sql-parser/
│   ├── executor/
│   └── storage/
├── reports/             # 分析报告
│   ├── SQL92_COMPLIANCE.md
│   └── PERFORMANCE_ANALYSIS.md
└── user-guide/          # 用户指南
    └── USER_MANUAL.md
```

---

## 文档索引

### 架构文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE_V2.6.md](./architecture/ARCHITECTURE_V2.6.md) | v2.6.0 整体架构设计 |

### 模块文档

| 模块 | 文档 | 说明 |
|------|------|------|
| SQL Parser | - | SQL-92 语法解析 |
| Executor | - | 执行器设计 |
| Storage | - | 存储引擎设计 |

### 报告

| 文档 | 说明 |
|------|------|
| [SQL92_COMPLIANCE.md](./reports/SQL92_COMPLIANCE.md) | SQL-92 合规性报告 |
| [PERFORMANCE_ANALYSIS.md](./reports/PERFORMANCE_ANALYSIS.md) | 性能分析报告 |

### 用户指南

| 文档 | 说明 |
|------|------|
| [USER_MANUAL.md](./user-guide/USER_MANUAL.md) | 用户手册 |

---

## 版本特性

v2.6.0 重点实现 SQL-92 完整支持：

- 聚合函数 (COUNT, SUM, AVG, MIN, MAX)
- JOIN 语法 (INNER, LEFT, RIGHT, CROSS)
- GROUP BY 分组查询
- HAVING 子句
- DELETE 语句
- 外键约束

---

## 相关文档

- [../VERSION_PLAN.md](../VERSION_PLAN.md) - 版本计划
- [../RELEASE_NOTES.md](../RELEASE_NOTES.md) - 发布说明
- [../FEATURE_MATRIX.md](../FEATURE_MATRIX.md) - 功能矩阵

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
