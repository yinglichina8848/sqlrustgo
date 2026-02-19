# SQLRustGo 版本规划路线图

> 最后更新：2026-02-18

---

## v1.0.0-alpha ✅ 已完成

**目标**：基础功能实现

**完成内容**：
- [x] 基础 SQL 解析 (SELECT, INSERT, UPDATE, DELETE)
- [x] B+ Tree 索引
- [x] Volcano 执行引擎
- [x] 事务管理 (BEGIN, COMMIT, ROLLBACK)
- [x] WAL 日志
- [x] MySQL 协议基础

**标签**：`v1.0.0-alpha.1`

---

## v1.0.0-beta ✅ 已完成

**目标**：稳定性验证 + 功能增强

**任务列表**：
- [x] 聚合函数 (COUNT/SUM/AVG/MIN/MAX)
- [x] 错误处理改进 (替换 unwrap)
- [x] 测试覆盖率提升 (76% → 90%)
- [ ] 客户端/服务器分离架构 (推迟到 v2.0)
- [x] 基准测试建立

**Issue 列表**：
- #18 Phase 2: v1.1.0-beta 功能与流程
- #16 Alpha 版本必须改进工作

**标签**：`v1.0.0-beta.1`

---

## v1.0.0-release (正式发布)

**目标**：正式稳定版本

**任务列表**：
- [ ] 所有 RC 验证通过
- [ ] 无 blocker bug
- [ ] 文档齐全
- [ ] 发布公告

**Issue 列表**：
- #20 Phase 4: v1.0.0-release 演示与复盘

---

## v1.1.0-alpha (下一轮)

**目标**：下一版本功能开发

**规划功能**：
- [ ] JOIN 支持
- [ ] 子查询支持
- [ ] 索引优化
- [ ] 查询缓存

---

## 版本推进流程

```
alpha → beta → release → baseline
  ✅       ✅        ⏳        ⏳
```

---

*本文档由 TRAE (GLM-5.0) 创建*
