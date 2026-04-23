# SQLRustGo v2.8.0 用户指南

> **版本**: v2.8.0
> **代号**: Production+Distributed+Secure
> **更新日期**: 2026-04-23

---

## 概述

本文档包含 SQLRustGo v2.8.0 的用户指南。

---

## 文档索引

### 核心用户指南

| 文档 | 说明 |
|------|------|
| [USER_MANUAL.md](./USER_MANUAL.md) | 主用户手册 - SQL 语法、分区表、主从复制、安全特性 |
| [QUICK_START.md](../QUICK_START.md) | 快速开始 - 安装、连接、基础操作 |
| [CLIENT_CONNECTION.md](../CLIENT_CONNECTION.md) | 客户端连接 - MySQL CLI、ODBC、JDBC、REST API |

### 功能用户指南

| 文档 | 说明 | 适用场景 |
|------|------|----------|
| [GMP_USER_GUIDE.md](./GMP_USER_GUIDE.md) | GMP 用户指南 | 药品生产质量管理、审计合规 |
| [GRAPH_SEARCH_USER_GUIDE.md](./GRAPH_SEARCH_USER_GUIDE.md) | 图检索用户指南 | 社交网络、推荐系统、欺诈检测 |
| [VECTOR_SEARCH_USER_GUIDE.md](./VECTOR_SEARCH_USER_GUIDE.md) | 向量检索用户指南 | 语义搜索、相似度匹配、混合检索 |

---

## v2.8.0 新增功能

### 兼容性增强

| 功能 | 说明 |
|------|------|
| FULL OUTER JOIN | 完整的外部连接支持 |
| TRUNCATE TABLE | 表数据快速清空 |
| REPLACE INTO | 唯一键冲突时替换 |
| 窗口函数 | ROW_NUMBER、RANK、DENSE_RANK |

### 分布式能力

| 功能 | 说明 |
|------|------|
| 分区表 | Range、List、Hash、Key 分区 |
| 主从复制 | GTID 复制、半同步复制 |
| 故障转移 | 自动切换 < 30s |
| 负载均衡 | 轮询、最少连接 |

### 安全加固

| 功能 | 说明 |
|------|------|
| 列级权限 | SELECT/INSERT/UPDATE 列控制 |
| 审计日志 | 操作审计、证据链 |
| 数据加密 | AES-256 加密支持 |

### 性能优化

| 功能 | 说明 |
|------|------|
| SIMD 向量化 | 向量操作加速 ≥ 2x |
| Hash Join 并行化 | 多线程 Hash Join |
| 查询计划器 | CBO 优化 |

---

## 相关文档

- [API 参考](../API_REFERENCE.md) - REST API 端点
- [安全加固](../SECURITY_HARDENING.md) - 安全配置
- [错误消息](../ERROR_MESSAGES.md) - 错误代码
- [版本计划](../VERSION_PLAN.md) - 开发计划

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-23*
