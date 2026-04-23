# v2.8.0 文档索引

> 版本: `v2.8.0`
> 代号: `Production+Distributed+Secure`
> 当前状态: `Alpha`
> 最后更新: 2026-04-22

---

## 一、版本定位

`v2.8.0` 是 SQLRustGo 的生产化+分布式版本，目标是：

1. 达到 MySQL 5.7 功能覆盖率 92%
2. 提供初步分布式能力 (分区表、主从复制、故障转移、负载均衡)
3. 安全性评分达到 92%

---

## 二、核心目标

### 2.1 兼容性增强

1. FULL OUTER JOIN 修复
2. TRUNCATE/REPLACE INTO 支持
3. 窗口函数完善
4. 分区表完整支持

### 2.2 初步分布式能力

1. 分区表 (Range/List/Hash/Key)
2. 主从复制完善 (GTID/Semi-sync)
3. 基础故障转移 (自动切换 < 30s)
4. 基础负载均衡 (轮询/最少连接/健康检查)
5. 读写分离路由

### 2.3 性能优化

1. SIMD 向量化加速
2. Hash Join 并行化
3. 查询计划器优化

### 2.4 安全加固

1. 列级权限控制
2. 审计告警系统
3. 数据加密基础

---

## 三、文档清单

### 核心文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 里程碑、任务矩阵、交付节奏 | ✅ |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | Alpha/Beta/RC/GA 门禁清单 | ✅ |
| [DISTRIBUTED_TEST_DESIGN.md](./DISTRIBUTED_TEST_DESIGN.md) | 分布式测试设计 | ✅ |
| [TEST_PLAN.md](./TEST_PLAN.md) | 完整测试计划 | ✅ |
| [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md) | 性能目标 | ✅ |
| [API_REFERENCE.md](./API_REFERENCE.md) | REST API 参考 | ✅ |
| [ERROR_MESSAGES.md](./ERROR_MESSAGES.md) | 错误消息参考 | ✅ |
| [SECURITY_HARDENING.md](./SECURITY_HARDENING.md) | 安全加固指南 | ✅ |

### 用户指南

| 文档 | 说明 | 状态 |
|------|------|------|
| [QUICK_START.md](./QUICK_START.md) | 快速开始 | ✅ |
| [CLIENT_CONNECTION.md](./CLIENT_CONNECTION.md) | 客户端连接指南 | ✅ |
| [user-guide/USER_MANUAL.md](./user-guide/USER_MANUAL.md) | 用户手册 | ✅ |
| [user-guide/README.md](./user-guide/README.md) | 用户指南索引 | ✅ |
| [user-guide/GMP_USER_GUIDE.md](./user-guide/GMP_USER_GUIDE.md) | GMP 用户指南 | ✅ |
| [user-guide/GRAPH_SEARCH_USER_GUIDE.md](./user-guide/GRAPH_SEARCH_USER_GUIDE.md) | 图检索用户指南 | ✅ |
| [user-guide/VECTOR_SEARCH_USER_GUIDE.md](./user-guide/VECTOR_SEARCH_USER_GUIDE.md) | 向量检索用户指南 | ✅ |

---

## 四、阶段路线

1. `Phase A`: 兼容性增强 + 分布式基础 (T-11~T-24)
2. `Phase B`: 初步分布式能力 (T-25~T-27, T-13)
3. `Phase C`: 性能优化 (T-14~T-16)
4. `Phase D`: 安全加固 (T-17~T-19)
5. `Phase E`: 文档与多语言 (T-20~T-22)

---

## 五、计划时间线

| 里程碑 | 计划日期 | 目标 |
|--------|----------|------|
| v2.8.0-alpha | 2026-05-20 | 完成 Phase A/B |
| v2.8.0-beta | 2026-06-10 | 完成 Phase C/D |
| v2.8.0-rc1 | 2026-06-25 | RC 候选 |
| v2.8.0-ga | 2026-07-05 | 全部门禁通过并发布 |

---

## 六、相关文档

1. [v2.7.0 文档索引](../v2.7.0/README.md)
2. [长期路线图](../LONG_TERM_ROADMAP.md)
3. [版本演化计划](../VERSION_ROADMAP.md)
4. [v2.8.0 开发 Issue](https://github.com/minzuuniversity/sqlrustgo/issues/1731)
