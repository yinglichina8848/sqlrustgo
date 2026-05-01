# v2.6.0 Release Notes

> **版本**: v2.6.0 GA
> **发布日期**: 2026-04-21
> **代号**: Production Ready
> **状态**: ✅ GA 已发布

---

## 版本概述

v2.6.0 是 SQLRustGo 迈向 **生产就绪 (Production Ready)** 的关键版本。本版本重点实现 SQL-92 语法完整支持，整合所有已实现但未集成的功能。

### 目标

- **SQL-92 完整支持**: 聚合函数、JOIN、GROUP BY、HAVING、DELETE
- **功能集成**: 索引扫描、CBO、存储过程、触发器
- **覆盖率提升**: 49% → 70%

---

## 新增功能

### SQL 语法扩展

| 功能 | PR | 说明 |
|------|-----|------|
| 聚合函数 | #1545 | COUNT, SUM, AVG, MIN, MAX |
| JOIN 语法 | #1545 | INNER, LEFT, RIGHT, CROSS JOIN |
| GROUP BY | #1545 | 分组查询支持 |
| HAVING 子句 | #1567 | 聚合过滤支持 |
| DELETE 语句 | #1557 | 删除语句支持 |
| 外键约束 | #1436, #1567 | 完整外键支持 |
| 分区表支持 | #1683 | Partition table support |
| CREATE INDEX | #1689 | 索引创建支持 |

### API 增强

| 功能 | PR | 说明 |
|------|-----|------|
| ExecutionEngine | #1566 | 高级 SQL 执行 API |
| MySQL 基准测试 | #1684 | 多数据库基准对比 |

### 测试改进

| 功能 | PR | 说明 |
|------|-----|------|
| 集成测试修复 | #1561 | 恢复集成测试 |
| 覆盖率提升 | #1559, #1564 | 新增测试用例 |
| SQL Corpus | - | 100% 通过率 (59/59) |
| TPC-H 基准 | #1687 | 修复 API 兼容性问题 |
| Sysbench | #1689 | 修复 CREATE INDEX 解析 |

### 代码质量

| 功能 | PR | 说明 |
|------|-----|------|
| Clippy 零警告 | #1570 | 所有 clippy 警告已修复 |

---

## GA 门禁测试结果

### 门禁状态: ✅ 全部通过 (9/9)

| 检查项 | 阈值 | 实际结果 | 状态 |
|--------|------|----------|------|
| L0 冒烟 (Build/Format/Clippy) | 100% | 3/3 | ✅ |
| L1 模块测试 | 100% | 12/12 | ✅ |
| L2 集成测试 | 100% | 67/67 | ✅ |
| SQL Corpus | ≥95% | 100% | ✅ |
| 覆盖率 | ≥70% | 71.02% | ✅ |
| TPC-H SF1 基准 | 通过 | ✅ | ✅ |
| Sysbench QPS | ≥1000 | ~3000 TPS | ✅ |
| 备份恢复 | 通过 | ✅ | ✅ |
| 崩溃恢复 | 通过 | ✅ | ✅ |

### SQL Corpus 测试

```
=== Summary ===
Total: 59 cases, 59 passed, 0 failed
Pass rate: 100.0%
```

### 编译检查

| 检查项 | 状态 |
|--------|------|
| Debug 编译 | ✅ 通过 |
| Release 编译 | ✅ 通过 |
| Clippy | ✅ 通过 (零警告) |
| 格式化 | ✅ 通过 |

### 覆盖率

| Crate | 覆盖率 | 状态 |
|-------|--------|------|
| sqlrustgo-parser | 60.08% | ⚠️ |
| sqlrustgo-planner | 92.23% | ✅ |
| sqlrustgo-executor | 43.45% | ⚠️ |
| sqlrustgo-storage | 83.07% | ✅ |
| sqlrustgo-transaction | 89.09% | ✅ |
| sqlrustgo-optimizer | 80.16% | ✅ |
| **总计** | **71.02%** | **✅** |

---

## 已知问题

### 已解决

1. ✅ **TPC-H 基准**: API 兼容性问题已修复 (PR #1687)
2. ✅ **Sysbench**: CREATE INDEX 解析已修复 (PR #1689)
3. ✅ **MVCC SI**: 快照隔离已实现 (PR #1607)

### 待改进 (v2.6.1)

1. **覆盖率**: executor 覆盖率 43.45%，目标 60%+
2. **FULL OUTER JOIN**: 完全外部连接 (设计中)
3. **MVCC SSI**: 可串行化快照隔离 (规划中)

---

## 升级指南

### 从 v2.5.0 升级

本版本主要包含语法增强和 bug 修复，向后兼容。

**变更**:
- 新增 `ExecutionEngine` API
- HAVING 子句现在可以正常使用
- DELETE 语句现在可以正常使用
- 分区表支持
- CREATE INDEX 支持

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 发布里程碑

| 阶段 | 日期 | 状态 |
|------|------|------|
| Alpha | 2026-04-21 | ✅ 已完成 |
| Beta | 2026-04-21 | ✅ 已完成 |
| RC | 2026-04-21 | ✅ 已完成 |
| **GA** | **2026-04-21** | **✅ 已发布** |

---

## 贡献者

感谢所有贡献者的支持!

---

## 相关链接

- [版本计划](./VERSION_PLAN.md)
- [门禁检查清单](./RELEASE_GATE_CHECKLIST.md)
- [测试计划](./TEST_PLAN.md)
- [RC 测试报告](./report/RC_TEST_REPORT.md)
- [功能集成状态](./INTEGRATION_STATUS.md)
- [性能目标](./PERFORMANCE_TARGETS.md)
- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [GitHub Releases](https://github.com/minzuuniversity/sqlrustgo/releases)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-21*
*GA 发布版本: commit 53d20f80*
