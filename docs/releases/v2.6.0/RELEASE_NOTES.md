# v2.6.0 Release Notes

> **版本**: alpha/v2.6.0
> **发布日期**: TBD (Alpha)
> **代号**: Production Ready

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

### API 增强

| 功能 | PR | 说明 |
|------|-----|------|
| ExecutionEngine | #1566 | 高级 SQL 执行 API |

### 测试改进

| 功能 | PR | 说明 |
|------|-----|------|
| 集成测试修复 | #1561 | 恢复集成测试 |
| 覆盖率提升 | #1559, #1564 | 新增测试用例 |
| SQL Corpus | - | 100% 通过率 (59/59) |

### 代码质量

| 功能 | PR | 说明 |
|------|-----|------|
| Clippy 零警告 | #1570 | 所有 clippy 警告已修复 |

---

## 测试结果

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
| Release 编译 | ⏳ 待测 |
| Clippy | ✅ 通过 |
| 格式化 | ✅ 通过 |

---

## 已知问题

### 待完成功能

1. **MVCC SSI**: 可串行化快照隔离 (待开发)
2. **CBO 优化**: 统计信息集成
3. **FULL OUTER JOIN**: 完全外部连接
4. **覆盖率**: 目标 70%，当前待测

---

## 升级指南

### 从 v2.5.0 升级

本版本主要包含语法增强和 bug 修复，向后兼容。

**变更**:
- 新增 `ExecutionEngine` API
- HAVING 子句现在可以正常使用
- DELETE 语句现在可以正常使用

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 路线图

| 阶段 | 日期 | 目标 |
|------|------|------|
| Alpha | 2026-04-21 | P0 功能开发完成 |
| Beta | 2026-04-28 | P1 功能开发完成 |
| RC | 2026-05-05 | 候选发布 |
| GA | 2026-05-12 | 生产就绪 |

---

## 贡献者

感谢所有贡献者的支持!

---

## 相关链接

- [版本计划](./VERSION_PLAN.md)
- [门禁检查清单](./RELEASE_GATE_CHECKLIST.md)
- [测试计划](./TEST_PLAN.md)
- [功能集成状态](./INTEGRATION_STATUS.md)
- [性能目标](./PERFORMANCE_TARGETS.md)
- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [GitHub Releases](https://github.com/minzuuniversity/sqlrustgo/releases)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-18*
