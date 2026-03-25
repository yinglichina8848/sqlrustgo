# v1.9.0 Changelog

> **Version**: v1.9.0
> **Status**: In Development
> **Branch**: `develop/v1.9.0`

---

## v1.9.0 (开发中)

### 新增功能

#### 数据完整性 & SQL 扩展
- 索引支持优化 (B+ Tree / Hash)
- 外键约束实现
- 事务增强 (SAVEPOINT / ROLLBACK TO SAVEPOINT)

#### SQL 特性增强
- 子查询支持增强 (Scalar, IN, EXISTS, ANY, ALL) - ✅ 已完成
- JOIN 优化 (INNER / LEFT / RIGHT / CROSS)
- 窗口函数基础支持 (PARTITION BY, AVG/SUM/ROW_NUMBER)
- 物化视图支持 (ISSUE #906) - ✅ 已完成

#### 性能优化
- 批量 INSERT 优化 (ISSUE #904) - ✅ 已完成
- 查询缓存
- 连接池实现

#### 运维与安全
- 数据备份导出 (CSV/JSON/SQL) - ✅ 已完成 (ISSUE #911)
- 数据恢复功能 - ✅ 已完成 (ISSUE #912)
- 崩溃恢复测试增强 - ✅ 已完成 (ISSUE #913)
- 生产场景测试 - ✅ 已完成 (ISSUE #914)
- 日志与监控 (SQL日志/错误日志/执行计划)
- 用户权限管理 (READ/WRITE)

### 改进
- 开发流程文档
- 教学示例脚本

---

## PR 合并记录

| PR # | 标题 | 合并日期 | 状态 |
|------|------|----------|------|
| #805 | feat(storage): 数据备份导出功能 (ISSUE #911) | 2026-03-25 | ✅ |
| #806 | feat(storage): 数据恢复功能 (ISSUE #912) | 2026-03-25 | ✅ |
| #807 | feat(test): 崩溃恢复测试增强 (ISSUE #913) | 2026-03-25 | ✅ |
| #808 | feat(planner): 子查询增强 (ISSUE #907) | 2026-03-25 | ✅ |
| #809 | feat(test): 生产场景测试 (ISSUE #914) | 2026-03-25 | ✅ |
| #906 | feat: 物化视图支持 | 2026-03-26 | ✅ |
| #904 | 批量 INSERT 优化 | 2026-03-25 | ✅ |

---

## v1.8.0 (2026-03-24)

- SQL-92 合规性增强
- 性能优化
- 错误处理改进

---

**Last Updated**: 2026-03-26
