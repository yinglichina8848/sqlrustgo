# v1.7.0 Issues Index

> **版本**: v1.7.0
> **代号**: MySQL 教学替代版
> **发布日期**: 2026-04-25
> **创建日期**: 2026-03-21
> **最后更新**: 2026-03-21

---

## 版本定位

**v1.7.0 = 原 v1.7 + v1.8 + v1.9**

合并版目标：一站式替代 MySQL 用于教学

---

## Epic 总览

| Epic | 名称 | 来源 | 状态 |
|------|------|------|------|
| Epic-01 | SQL 补完 | 原 v1.7 | ✅ 已完成 |
| Epic-02 | 可观测性 | 原 v1.7 | ✅ 已完成 |
| Epic-03 | Benchmark 完善 | 原 v1.7 | ✅ 已完成 |
| Epic-04 | 错误系统 | 原 v1.7 | ✅ 已完成 |
| Epic-05 | 约束与外键 | 原 v1.8 | 🔄 开发中 |
| Epic-06 | MySQL 兼容语法 | 原 v1.8 | 🔄 开发中 |
| Epic-07 | CLI 工具完善 | 原 v1.8 | 🔄 开发中 |
| Epic-08 | 稳定性强化 | 原 v1.9 | 📋 待开发 |
| Epic-09 | 覆盖率提升 | 原 v1.9 | 📋 待开发 |
| Epic-10 | 教学支持 | 原 v1.9 | 📋 待开发 |

---

## Issue 列表

### Epic-01: SQL 补完 ✅

| Issue | 标题 | 状态 |
|-------|------|------|
| SQL-01 | UNION / UNION ALL 支持 | ✅ 已完成 |
| SQL-02 | INTERSECT / EXCEPT | ✅ 已完成 |
| SQL-03 | VIEW 创建和查询 | ✅ 已完成 |
| SQL-04 | BOOLEAN 类型补齐 | ✅ 已完成 |
| SQL-05 | BLOB 类型支持 | ✅ 已完成 |

### Epic-02: 可观测性 ✅

| Issue | 标题 | 状态 |
|-------|------|------|
| OBS-01 | EXPLAIN 执行计划 | ✅ 已完成 |
| OBS-02 | EXPLAIN ANALYZE | ✅ 已完成 |
| OBS-03 | 算子级 Profiling | 🔄 开发中 |
| OBS-04 | 格式化输出 | ✅ 已完成 |

### Epic-03: Benchmark 完善 ✅

| Issue | 标题 | 状态 |
|-------|------|------|
| BEN-01 | Benchmark CLI 完善 | ✅ 已完成 |
| BEN-02 | JSON 输出格式 | ✅ 已完成 |
| BEN-03 | PostgreSQL 对比测试 | ✅ 已完成 |

### Epic-04: 错误系统 ✅

| Issue | 标题 | 状态 |
|-------|------|------|
| ERR-01 | Unknown column 错误 | ✅ 已完成 |
| ERR-02 | Table not found 错误 | ✅ 已完成 |
| ERR-03 | Duplicate key 错误 | ✅ 已完成 |

### Epic-05: 约束与外键 🔄

| Issue | 标题 | 状态 |
|-------|------|------|
| FK-01 | FOREIGN KEY 约束实现 | 📋 待开发 |
| FK-02 | 约束检查 | 📋 待开发 |

### Epic-06: MySQL 兼容语法 🔄

| Issue | 标题 | 状态 |
|-------|------|------|
| SYN-01 | AUTO_INCREMENT 支持 | 📋 待开发 |
| SYN-02 | LIMIT offset, count | 📋 待开发 |
| SYN-03 | SHOW TABLES / SHOW COLUMNS | 📋 待开发 |
| SYN-04 | DESCRIBE table_name | 📋 待开发 |
| SYN-05 | 常用函数 NOW(), COUNT() 等 | 📋 待开发 |

### Epic-07: CLI 工具完善 🔄

| Issue | 标题 | 状态 |
|-------|------|------|
| CLI-01 | .tables 命令 | 📋 待开发 |
| CLI-02 | .schema table_name | 📋 待开发 |
| CLI-03 | .indexes table_name | 📋 待开发 |
| CLI-04 | MySQL 协议支持 | 📋 可选 |

### Epic-08: 稳定性强化 📋

| Issue | 标题 | 状态 |
|-------|------|------|
| STA-01 | WAL 恢复强化 | 📋 待开发 |
| STA-02 | Crash 安全机制 | 📋 待开发 |
| STA-03 | 长时间运行测试 | 📋 待开发 |

### Epic-09: 覆盖率提升 📋

| Issue | 标题 | 状态 |
|-------|------|------|
| COV-01 | 覆盖率提升至 ≥85% | 📋 待开发 |

### Epic-10: 教学支持 📋

| Issue | 标题 | 状态 |
|-------|------|------|
| EDU-01 | SQLRUSTGO_TEACHING_MODE | 📋 待开发 |
| EDU-02 | 12 个标准实验 | 📋 待开发 |
| EDU-03 | MySQL → SQLRustGo 对照表 | 📋 待开发 |

---

## 里程碑

```
Week 1-2 (已完成)      Week 3-4 (开发中)      Week 5-6 (待开发)
├── Epic-01~04 ✅       ├── Epic-05~07 🔄       ├── Epic-08~10 📋
└── PR #727 合并        └── CLI + MySQL 语法    └── 稳定性 + 教学
```

---

## 相关文档

| 文档 | 说明 |
|------|------|
| [EPIC-01-SQL补完.md](./EPIC-01-SQL补完.md) | SQL 补完详细设计 |
| [EPIC-02-可观测性.md](./EPIC-02-可观测性.md) | 可观测性详细设计 |
| [EPIC-03-Benchmark完善.md](./EPIC-03-Benchmark完善.md) | Benchmark 详细设计 |
| [EPIC-04-错误系统.md](./EPIC-04-错误系统.md) | 错误系统详细设计 |
| [../2026-03-21-v170-issue-spec.md](../2026-03-21-v170-issue-spec.md) | 完整 Issue 规格 |
| [../2026-03-21-v170-release-plan.md](../2026-03-21-v170-release-plan.md) | 发布计划 |
| [../../VERSION_ROADMAP.md](../../VERSION_ROADMAP.md) | 版本路线图 (v7.0) |

---

## 版本路线

```
v1.6 ✅  →  v1.7 (合并版)  →  v2.0
  Benchmark     MySQL教学替代    高性能分析
                ⭐⭐⭐
```

---

**文档状态**: 草稿
**最后更新**: 2026-03-21
