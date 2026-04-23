# v2.8.0 Release Notes

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **发布日期**: (待定 - Alpha 阶段)
> **状态**: 🔄 开发中

---

## 版本概述

v2.8.0 是 SQLRustGo 的**生产化+分布式+安全**版本，目标是：

1. **MySQL 5.7 功能覆盖率**: 83% → 92%
2. **初步分布式能力**: 分区表、主从复制、故障转移、负载均衡
3. **安全性评分**: 85% → 92%

### 目标

| 目标 | 说明 |
|------|------|
| 兼容性增强 | FULL OUTER JOIN、TRUNCATE/REPLACE、窗口函数完善 |
| 分布式基础 | 分区表、主从复制、故障转移、负载均衡 |
| 性能优化 | SIMD 向量化、Hash Join 并行化、查询计划器优化 |
| 安全加固 | 列级权限、审计告警、数据加密基础 |

---

## 新增功能

### Phase A: 兼容性增强 (T-11, T-12, T-13)

| 功能 | PR | 说明 |
|------|-----|------|
| FULL OUTER JOIN (T-11) | #1754 | Hash-based matching algorithm, 3/3 tests passing |
| TRUNCATE/REPLACE (T-12) | #1754 | TRUNCATE TABLE + REPLACE INTO syntax |
| 窗口函数完善 (T-13) | #1754 | ROW_NUMBER, RANK, DENSE_RANK |

### Phase B: 初步分布式能力 (T-23~T-27) - 规划中

| 功能 | 状态 | 说明 |
|------|------|------|
| 分区表 (T-23) | ⏳ 未开始 | Range/List/Hash/Key 分区 + 裁剪优化 |
| 主从复制 (T-24) | ⏳ 未开始 | GTID 复制协议、半同步复制 |
| 故障转移 (T-25) | ⏳ 未开始 | 自动切换 < 30s |
| 负载均衡 (T-26) | ⏳ 未开始 | 轮询/最少连接策略 |
| 读写分离 (T-27) | ⏳ 未开始 | SELECT 路由到从节点 |

### Phase C: 性能优化 (T-14, T-15, T-16)

| 功能 | 状态 | 说明 |
|------|------|------|
| SIMD 向量化加速 (T-14) | ✅ 完成 | crates/vector/src/simd_explicit.rs, 5 tests passing |
| Hash Join 并行化 (T-15) | ⚠️ 未集成 | parallel_executor.rs 存在但未集成 |
| 查询计划器优化 (T-16) | ✅ 完成 | 81 planner tests passing |

### Phase D: 安全加固 (T-17, T-18, T-19)

| 功能 | 状态 | 说明 |
|------|------|------|
| 列级权限控制 (T-17) | ⚠️ 部分实现 | ColumnMasker 存在，缺少 GRANT/REVOKE 解析器 |
| 审计告警系统 (T-18) | ✅ 完成 | security/src/audit.rs, 78 tests passing |
| 数据加密基础 (T-19) | ⏳ 未开始 | AES-256 加密 |

### Phase E: 文档与多语言 (T-20, T-21, T-22)

| 功能 | 状态 | 说明 |
|------|------|------|
| 英文错误消息 (T-20) | ✅ 完成 | ERROR_MESSAGES.md |
| 英文 API 文档 (T-21) | ✅ 完成 | API_REFERENCE.md |
| 安全加固指南 (T-22) | ✅ 完成 | SECURITY_HARDENING.md |

---

## 破坏性变更

本版本包含以下破坏性变更 (如有)，请在升级前仔细阅读：

1. **分区表语法**: 需要使用新的 PARTITION BY 语法
2. **主从复制协议**: 需配置 GTID

详见 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) (待创建)

---

## Bug 修复

### 已修复问题

| Issue | 描述 | 修复时间 |
|-------|------|----------|
| #1733 | FULL OUTER JOIN 执行器修复 | 2026-04-23 |
| #1734 | TRUNCATE TABLE 支持 | 2026-04-23 |
| #1735 | REPLACE INTO 支持 | 2026-04-23 |

### 已知问题 (待 GA)

1. **覆盖率**: executor 覆盖率目标 60%+
2. **分布式事务**: 跨节点事务支持 (规划中 v2.9.0)
3. **SSI 隔离级别**: 可串行化快照隔离 (规划中)

---

## 性能改进 (规划中)

### 基准测试目标

| 场景 | v2.7.0 | v2.8.0 目标 |
|------|--------|--------------|
| SIMD 加速比 | 1x | ≥ 2x |
| Hash Join 并行化 | 单线程 | ≥ 1.5x |
| 查询计划器优化 | 基础 | CBO 命中率 ≥ 80% |

---

## 发布里程碑

| 阶段 | 计划日期 | 状态 |
|------|----------|------|
| Alpha | 2026-05-20 | 🔄 开发中 |
| Beta | 2026-06-10 | ⏳ 未开始 |
| RC | 2026-06-25 | ⏳ 未开始 |
| GA | 2026-07-05 | ⏳ 未开始 |

---

## 升级指南

### 从 v2.7.0 升级

详见 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) (待创建)

---

## 贡献者

感谢所有贡献者的支持!

---

## 相关链接

- [版本计划](./VERSION_PLAN.md)
- [门禁检查清单](./RELEASE_GATE_CHECKLIST.md)
- [测试计划](./TEST_PLAN.md)
- [用户指南](./user-guide/README.md)
- [GitHub Issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- [GitHub Releases](https://github.com/minzuuniversity/sqlrustgo/releases)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-23*
*开发版本: commit 13d2645d (develop/v2.8.0)*