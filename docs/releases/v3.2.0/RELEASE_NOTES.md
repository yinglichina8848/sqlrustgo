# v3.2.0 Release Notes

> **Version**: 3.2.0
> **Date**: 2026-05-15
> **Status**: In Development (Beta Phase)
> **Target**: GA Release

---

## 概述

**SQLRustGo v3.2.0** 是 **Trusted GMP Data Platform** 的一次重大版本升级。

从 MySQL 替代品 → **GMP Native 可信数据平台**

### 核心定位

v3.2.0 = **Trusted GMP Data Platform**

- **GMP**: Good Manufacturing Practice (良好生产规范)
- **可信**: 数字签名、审计链、不可篡改
- **数据平台**: 不仅是数据库，而是可信数据生命周期管理

---

## 主要新功能

### 1. GMP Framework (P0)

完整实现 GMP 合规所需的核心模块:

| 模块 | 功能 | 状态 |
|------|------|------|
| GMP-1 | 数字签名审计链 | ✅ |
| GMP-2 | 电子签名 (21 CFR Part 11) | ✅ |
| GMP-3 | Immutable Record | ✅ |
| GMP-4 | Correction Chain | ✅ |
| GMP-5 | Provenance Tracking | ✅ |
| GMP-6 | Trusted Timestamp | ✅ |
| GMP-7 | 审计链验证工具 | ✅ |
| GMP-8 | HSM/KMS 集成 | ✅ |
| GMP-9 | Workflow Engine | ✅ |

### 2. Performance Enhancements

| 模块 | 功能 | 状态 |
|------|------|------|
| PERF-3 | 并发 200+ 连接 | ✅ |
| PERF-5 | 内存优化 | ✅ |
| PERF-1 | MySQL flush 优化 | 🔄 |
| PERF-2 | TPC-H SF=10 | 🔄 |

### 3. SQL 增强

| 模块 | 功能 | 状态 |
|------|------|------|
| Multi-Table DML | UPDATE/MERGE 多表操作 | ✅ |
| SQL-1 | RECURSIVE CTE | 🔄 |
| SQL-2 | Performance Schema | 🔄 |

---

## 版本对比

| 功能 | v3.1.0 | v3.2.0 |
|------|---------|--------|
| GMP Framework | 基础 | 完整 P0 |
| 电子签名 | ❌ | ✅ |
| Immutable Record | ❌ | ✅ |
| Correction Chain | ❌ | ✅ |
| Provenance Tracking | ❌ | ✅ |
| Trusted Timestamp | ❌ | ✅ |
| HSM/KMS | ❌ | ✅ |
| Workflow Engine | ❌ | ✅ |
| 并发连接 | 100 | 200+ |
| Multi-Table DML | 部分 | ✅ |
| RECURSIVE CTE | ❌ | 🔄 |

---

## 门禁状态

| Gate | 状态 | 日期 |
|------|------|------|
| Alpha Gate | 🟡 条件性通过 | 2026-05-15 |
| Beta Gate | ⏸️ 进行中 | - |
| RC Gate | ⏸️ 等待 | - |
| GA Gate | ⏸️ 等待 | - |

详细状态见 [ALPHA_GATE_REPORT.md](./ALPHA_GATE_REPORT.md)

---

## 已知问题

| Issue | 描述 | 严重度 | 状态 |
|-------|------|--------|------|
| Coverage | 覆盖率 46.63% < 75% | 中 | 进行中 |
| PERF-1 | MySQL flush 未完成 | 高 | 🔄 |
| PERF-2 | TPC-H SF=10 未完成 | 高 | 🔄 |
| SQL-1 | RECURSIVE CTE | 中 | 🔄 |

---

## 升级指南

### 从 v3.1.0 升级

v3.2.0 兼容 v3.1.0 的 SQL 语法和协议。

新增配置项:
- `max_connections` — 最大并发连接数
- `enable_gmp` — 启用 GMP 框架
- `hsm_provider` — HSM 提供商选择

---

## 文档

| 文档 | 说明 |
|------|------|
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 |
| [ALPHA_GATE_REPORT.md](./ALPHA_GATE_REPORT.md) | Alpha 门禁报告 |
| [BETA_GATE_REPORT.md](./BETA_GATE_REPORT.md) | Beta 门禁报告 |
| [CHANGELOG.md](./CHANGELOG.md) | 变更日志 |

---

## 路线图

```
v3.2.0 ────── Alpha ✅ ────── Beta 🔄 ────── RC 🔄 ────── GA
              │              │              │
           M1-M4 ✅        M5-M6 🔄       M7-M8 🔄
                          PERF 🔄
```

详细路线图见 [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)

---

## 反馈

如有问题，请提交 Issue:
- https://github.com/openclaw/sqlrustgo/issues
- 内部: http://192.168.0.252:3000/openclaw/sqlrustgo/issues

---

**维护人**: hermes-z6g4
**生成日期**: 2026-05-15
