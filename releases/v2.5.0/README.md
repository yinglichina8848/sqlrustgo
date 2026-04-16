# SQLRustGo v2.5.0 文档入口

> **版本**: v2.5.0 GA (Full Integration + GMP)
> **发布日期**: 2026-04-16

---

## 版本概述

v2.5.0 是 SQLRustGo 的**里程碑版本**，实现全面的企业级数据库功能：

- **MVCC 事务**: 快照隔离，WAL 崩溃恢复
- **向量化执行**: SIMD 加速，并行查询
- **图引擎**: Cypher 查询，BFS/DFS 遍历
- **向量索引**: HNSW/IVF/IVFPQ + SIMD
- **统一查询**: SQL + 向量 + 图 融合
- **OpenClaw 接口**: Agent 框架

---

## 文档索引

### OO 架构文档

| 文档 | 说明 |
|------|------|
| [oo/README.md](./oo/README.md) | OO 文档目录 |
| [oo/architecture/ARCHITECTURE_V2.5.md](./oo/architecture/ARCHITECTURE_V2.5.md) | v2.5 架构设计 |

### 模块设计

| 模块 | 文档 |
|------|------|
| MVCC | [oo/modules/mvcc/MVCC_DESIGN.md](../v2.5.0/MVCC_DESIGN.md) |
| WAL | [oo/modules/wal/WAL_DESIGN.md](../v2.5.0/MVCC_DESIGN.md) |
| 图引擎 | [oo/modules/graph/GRAPH_ENGINE_DESIGN.md](../v2.5.0/GRAPH_ENGINE_DESIGN.md) |
| 向量索引 | [oo/modules/vector/VECTOR_INDEX_DESIGN.md](../v2.5.0/VECTOR_INDEX_DESIGN.md) |
| 统一查询 | [oo/modules/unified-query/UNIFIED_API.md](./oo/modules/unified-query/UNIFIED_API.md) |
| OpenClaw | [oo/modules/openclaw/AGENT_GATEWAY.md](./oo/modules/openclaw/AGENT_GATEWAY.md) |

### 用户指南

| 文档 | 说明 |
|------|------|
| [oo/user-guide/USER_MANUAL.md](./oo/user-guide/USER_MANUAL.md) | 用户手册 |

### 报告

| 文档 | 说明 |
|------|------|
| [oo/reports/SECURITY_ANALYSIS.md](./oo/reports/SECURITY_ANALYSIS.md) | 安全分析 |
| [oo/reports/PERFORMANCE_REPORT.md](./oo/reports/PERFORMANCE_REPORT.md) | 性能测试报告 |

### 发布文档

| 文档 | 说明 |
|------|------|
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | 发布说明 |
| [FEATURE_MATRIX.md](./FEATURE_MATRIX.md) | 功能矩阵 |
| [CHANGELOG.md](./CHANGELOG.md) | 变更日志 |
| [GATE_CHECKLIST.md](./GATE_CHECKLIST.md) | 门禁清单 |

---

## 快速开始

### 构建

```bash
cargo build --release
```

### 测试

```bash
cargo test --workspace
```

### 运行 REPL

```bash
cargo run --release
```

---

## 功能一览

### SQL 功能

| 功能 | 状态 |
|------|------|
| SELECT/INSERT/UPDATE/DELETE | ✅ |
| 外键约束 | ✅ |
| 预处理语句 | ✅ |
| 子查询 (EXISTS/ANY/ALL/IN) | ✅ |
| JOIN (INNER/LEFT/RIGHT/SEMI/ANTI) | ✅ |
| 窗口函数 | ✅ |

### 存储

| 功能 | 状态 |
|------|------|
| MVCC 快照隔离 | ✅ |
| WAL 崩溃恢复 | ✅ |
| PITR 时间点恢复 | ✅ |
| B+Tree 索引 | ✅ |
| 向量索引 | ✅ |
| 列式存储 | ✅ |

### 引擎

| 功能 | 状态 |
|------|------|
| SQL 查询 | ✅ |
| Cypher 图查询 | ✅ |
| 向量搜索 | ✅ |
| 统一查询 | ✅ |
| CBO 优化器 | ✅ |

---

## 性能基准

| 场景 | 性能 |
|------|------|
| 点查 (32并发) | > 50,000 TPS |
| TPC-H Q1 (SF=1) | < 500ms |
| 向量搜索 (1M) | < 5ms |

---

## 里程碑

| 日期 | 里程碑 |
|------|---------|
| 2026-04-01 | MVCC + WAL 集成 |
| 2026-04-06 | Cypher Phase-1 |
| 2026-04-10 | 预处理语句 + 子查询 |
| 2026-04-14 | TPC-H SF=10 |
| 2026-04-15 | 统一查询 API |
| 2026-04-16 | **v2.5.0 GA** |

---

## 相关链接

- [GitHub Releases](https://github.com/minzuuniversity/sqlrustgo/releases)
- [ROADMAP.md](../../ROADMAP.md) - 路线图

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-16*