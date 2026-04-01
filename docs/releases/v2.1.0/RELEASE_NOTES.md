# v2.1.0 Release Notes

## 概述
v2.1.0 是 SQLRustGo 的企业级增强版本，专注于可观测性、安全增强、工具链完善和 SQL 功能扩展。

## 发布日期
2026-04-02

## 代号
**Enterprise Observability** - 可观测性 + 安全增强版

---

## 主要功能

### 可观测性增强

| 功能 | Issue | 状态 |
|------|-------|------|
| Prometheus 指标端点 | M-004 | ✅ |
| Grafana Dashboard | M-005 | ✅ |
| 慢查询日志 | M-006 | ✅ |
| Executor Metrics | M-003 | ✅ |
| Buffer Pool Metrics | M-002 | ✅ |

### SQL Firewall 安全增强

| 功能 | Issue | 状态 |
|------|-------|------|
| SQL 防火墙核心 | #1134 | ✅ |
| 告警系统 | #1134 | ✅ |
| KILL 语句支持 | #1135 | ✅ |
| PROCESSLIST | #1135 | ✅ |

### 工具链完善

| 功能 | Issue | 状态 |
|------|-------|------|
| Physical Backup CLI | #1018 | ✅ |
| 备份保留策略 | #1198 | ✅ |
| mysqldump 导入导出 | #1022 | ✅ |
| 日志轮转 | #1022 | ✅ |

### SQL 功能扩展

| 功能 | Issue | 状态 |
|------|-------|------|
| TPC-H Phase 1 (BETWEEN/DATE/IN) | #1210 | ✅ |
| UUID 类型 | #1128 | ✅ |
| ARRAY 类型 | #1128 | ✅ |
| ENUM 类型 | #1128 | ✅ |
| AgentSQL Extension | #1128 | 开发中 |

### 存储过程

| 功能 | Issue | 状态 |
|------|-------|------|
| 控制流语句 | #1164 | ✅ |
| SQL 集成 | #1164 | ✅ |

---

## 核心技术特性

### 1. 可观测性
- `/metrics` - Prometheus 格式指标端点
- `/health` - 健康检查端点
- `/ready` - 就绪检查端点
- 慢查询日志记录
- Executor 执行器指标

### 2. SQL 防火墙
- SQL 注入检测
- 异常查询告警
- KILL 超时连接
- 进程列表查看

### 3. 备份工具
- Physical Backup (全量/增量)
- WAL 归档打包
- GZIP 压缩
- SHA256 完整性验证
- 保留策略 (按数量/天数)

### 4. 新数据类型
- UUID - 128位唯一标识符
- ARRAY - 可变长度数组
- ENUM - 枚举类型

---

## 升级说明

从 v2.0.0 升级无需特殊迁移。

---

## 测试统计

| 测试类别 | 数量 | 状态 |
|----------|------|------|
| 单元测试 | 1900+ | ✅ 通过 |
| 集成测试 | 70+ | ✅ 通过 |
| TPC-H 测试 | 57 | ✅ 通过 |
| 覆盖率 | 80%+ | ✅ |

---

## Issue 完成统计

| 模块 | Issue | 状态 |
|------|-------|------|
| 可观测性 | M-001 ~ M-006 | ✅ |
| SQL Firewall | #1134 | ✅ |
| KILL/PROCESSLIST | #1135 | ✅ |
| Physical Backup | #1018 | ✅ |
| Retention Policy | #1198 | ✅ |
| mysqldump | #1022 | ✅ |
| TPC-H Phase 1 | #1210 | ✅ |
| UUID/ARRAY/ENUM | #1128 | ✅ |
| Stored Procedure | #1164 | ✅ |

---

## 门禁状态

| 检查项 | 状态 |
|--------|------|
| 编译检查 | ✅ 通过 |
| 测试检查 | ✅ 通过 |
| Clippy | ✅ 通过 (warnings only) |
| 格式化 | ✅ 通过 |
| 覆盖率 | ✅ 80%+ |
| MockStorage 移除 | ✅ 完成 |
| MemoryStorage 测试 | ✅ 完成 |

---

## 重要 PR 列表

| PR | 描述 | 日期 |
|----|------|------|
| #1217 | feat(types): add UUID, ARRAY, and ENUM types | 2026-04-02 |
| #1216 | feat: deprecate MockStorage | 2026-04-02 |
| #1211 | feat(tpch): TPC-H Phase 1 - BETWEEN, DATE, IN | 2026-04-02 |
| #1208 | feat(agentsql): Phase 3 - Security, Explain, Optimizer | 2026-04-01 |
| #1191 | feat(executor): Stored Procedure Executor SQL Integration | 2026-03-31 |
| #1190 | fix(executor): optimize LRU cache O(n) to O(1) | 2026-03-31 |
| #1181 | feat(tools): add mysqldump import tool | 2026-03-31 |
| #1183 | feat(common): implement log rotation | 2026-03-31 |

---

## 已知问题

| Issue | 描述 | 状态 |
|-------|------|------|
| #1207 | MockStorage stub 修复 | 进行中 |
| #1214 | TPC-H Phase 2+3 | 待处理 |

---

## 贡献者

感谢所有参与 v2.1.0 开发的团队成员及 AI 助手 (OpenCode A/B, Claude A/B)。

---

*发布版本: v2.1.0*
*生成日期: 2026-04-02*
