# SQLRustGo v2.0 文档入口

> **版本**: v2.0.0 (Vector Engine + Cascades)
> **发布日期**: 2026-03-26
> **状态**: 历史版本

---

## 版本概述

v2.0 是 SQLRustGo 的**向量化引擎版本**：

- **Cascades 优化器**: 基于成本的优化
- **向量化执行**: SIMD 加速
- **Buffer Pool**: LRU/CLOCK 页面管理

> 注意: v2.0 不包含 MVCC 事务、图引擎、向量索引等特性。这些在 v2.5.0 中实现。

---

## 文档索引

### OO 架构文档

| 文档 | 说明 |
|------|------|
| [oo/README.md](./oo/README.md) | OO 文档目录 |
| [oo/architecture/ARCHITECTURE_V2.md](./oo/architecture/ARCHITECTURE_V2.md) | v2.0 架构设计 |

### 发布文档

| 文档 | 说明 |
|------|------|
| [USER_MANUAL.md](./USER_MANUAL.md) | 用户手册与 SQL 参考 |
| [ARCHITECTURE_DECISIONS.md](./ARCHITECTURE_DECISIONS.md) | 架构决策 |
| [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) | 部署指南 |
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |

### v2.0 规划文档

| 文档 | 说明 |
|------|------|
| [do../../v2.0/CASCADES_OPTIMIZER.md](../../v2.0/CASCADES_OPTIMIZER.md) | Cascades 优化器设计 |
| [do../../v2.0/BENCHMARK_FRAMEWORK.md](../../v2.0/BENCHMARK_FRAMEWORK.md) | 基准测试框架 |
| [do../../v2.0/WHITEPAPER.md](../../v2.0/WHITEPAPER.md) | 技术白皮书 |

---

## 功能对比

| 功能 | v1.x | v2.0 | v2.5 |
|------|------|------|------|
| 优化器 | RBO | Cascades | CBO |
| 执行模型 | Volcano | 向量化 | 向量化+并行 |
| 事务 | 无 | 无 | MVCC+WAL |
| 图引擎 | 无 | 无 | ✅ |
| 向量索引 | 无 | IVF | HNSW/IVFPQ |

---

## 相关版本

| 版本 | 日期 | 说明 |
|------|------|------|
| [v2.5.0](../v2.5.0/README.md) | 2026-04-16 | 里程碑版本 |
| v2.0 | 2026-03-26 | 向量化引擎版本 |
| v1.x | 2026-03 | 初始版本 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-03-26*