---
entity_type: index
confidence: 100
domains: [sqlrustgo, database]
last_updated: 2026-04-17
---

# Concept Index - SQLRustGo

## 模块 (Modules)

| 概念 | 类型 | 路径 | 状态 |
|------|------|------|------|
| Parser | tool | `crates/parser` | 完整 |
| Planner | tool | `crates/planner` | 部分 |
| Optimizer | tool | `crates/optimizer` | 部分 |
| Executor | tool | `crates/executor` | 部分 |
| Storage Engine | tool | `crates/storage` | 部分 |
| Catalog | tool | `crates/catalog` | 完整 |
| Transaction Manager | tool | `crates/transaction` | 完整 |
| Distributed | tool | `crates/distributed` | 框架 |

## 架构模式 (Patterns)

| 模式 | 说明 |
|------|------|
| MVCC | 多版本并发控制 - 快照隔离 |
| 2PC | 两阶段提交 - 分布式事务 |
| WAL | Write-Ahead Log - 崩溃恢复 |
| B+Tree | 索引结构 |
| CBO | Cost-Based Optimizer - 成本优化 |

## SQL 特性 (Features)

| 特性 | 状态 | 说明 |
|------|------|------|
| SELECT | ✅ | 完整支持 |
| INSERT/UPDATE/DELETE | ✅ | 完整支持 |
| JOIN | ⚠️ | LEFT/RIGHT/SEMI/ANTI 完整，FULL OUTER 部分 |
| 子查询 | ⚠️ | EXISTS/IN/ALL/ANY 支持 |
| CTE | ✅ | WITH 子句 |
| Prepared Statement | ✅ | 参数化查询 |
| FOREIGN KEY | ⚠️ | Parser 完整，Executor 未验证 |
| 触发器 | ⚠️ | Parser/Executor 框架存在，未集成 |
| 存储过程 | ⚠️ | Parser/Executor 框架存在，未集成 |

## 存储层 (Storage)

| 类型 | 说明 |
|------|------|
| MemoryStorage | 内存存储 |
| FileStorage | 文件存储 + B+Tree |
| ColumnarStorage | 列式存储 |
| VectorStorage | 向量存储 |
| GraphStore | 图存储 |

## 测试 (Testing)

| 类型 | 数量 |
|------|------|
| 单元测试 | 2000+ |
| 集成测试 | 100+ |
| 压力测试 | 20+ |
| TPC-H | SF=1, SF=10 |

## 人员 (People)

| 角色 | 说明 |
|------|------|
| Core Team | 主要开发者 |
| Contributors | 贡献者 |

---

*由 AI Agent 维护*
