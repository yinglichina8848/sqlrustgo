# [Epic-10] 教学支持

## 概述

提供教学支持功能，让 SQLRustGo 可正式替代 MySQL 用于课程教学。

**优先级**: P0
**来源**: 原 v1.9

---

## Issues

### EDU-01: SQLRUSTGO_TEACHING_MODE 教学模式

**优先级**: P0

**描述**: 提供教学模式，让数据库行为更透明

**Acceptance Criteria**:
- [ ] `SQLRUSTGO_TEACHING_MODE=1` 启用教学模式
- [ ] 教学模式下禁用优化器（便于教学）
- [ ] 教学模式下强制 EXPLAIN 输出
- [ ] 更详细的执行日志

---

### EDU-02: 12 个标准实验

**优先级**: P1

**描述**: 提供 12 个标准数据库实验

**Acceptance Criteria**:
- [ ] 实验 1: 数据库安装与基本 SQL
- [ ] 实验 2: SELECT 与条件查询
- [ ] 实验 3: 多表查询与 JOIN
- [ ] 实验 4: 索引与性能优化
- [ ] 实验 5: 数据库设计与 E-R 图
- [ ] 实验 6: 规范化与反规范化
- [ ] 实验 7: 存储过程基础
- [ ] 实验 8: 触发器
- [ ] 实验 9: 事务与并发
- [ ] 实验 10: 备份与恢复
- [ ] 实验 11: 权限管理
- [ ] 实验 12: 综合项目

---

### EDU-03: MySQL → SQLRustGo 对照表

**优先级**: P1

**描述**: 提供 MySQL 与 SQLRustGo 命令/语法对照表

**Acceptance Criteria**:
- [ ] 常用命令对照
- [ ] 语法差异说明
- [ ] 注意事项提醒

---

## 教学模式效果

```bash
export SQLRUSTGO_TEACHING_MODE=1
sqlrustgo

# 输出示例
[TEACHING] 教学模式已启用
[TEACHING] 优化器已禁用
[TEACHING] 所有查询将显示 EXPLAIN
```

---

## 关键文件

| 文件 | 用途 |
|------|------|
| `crates/server/src/teaching.rs` | 教学模式实现 |
| `docs/tutorials/` | 教学文档 |

---

**关联 Issue**: EDU-01, EDU-02, EDU-03
