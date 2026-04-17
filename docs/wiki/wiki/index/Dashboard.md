---
entity_type: overview
confidence: 100
domains: [sqlrustgo, database, distributed-systems]
last_updated: 2026-04-17
---

# SQLRustGo Wiki Dashboard

> 持续更新的 SQLRustGo 知识库 - AI 驱动的项目知识管理

## 项目概览

| 维度 | 值 |
|------|-----|
| **当前版本** | v2.6.0 (开发中) |
| **架构** | Rust Kernel + Go API Layer |
| **类型** | 分布式 RDBMS (MySQL 兼容) |
| **许可证** | GPL-3.0 |
| **分支策略** | main (锁定), release/* (冻结), develop/* (开发) |

## 核心模块

| 模块 | 路径 | 状态 | 说明 |
|------|------|------|------|
| **Parser** | `crates/parser` | ✅ 完整 | SQL 解析、语法树生成 |
| **Planner** | `crates/planner` | ⚠️ 部分 | 逻辑计划、Prepared Statement |
| **Optimizer** | `crates/optimizer` | ⚠️ 部分 | CBO、成本模型 (未集成) |
| **Executor** | `crates/executor` | ⚠️ 部分 | 执行引擎 (外键/触发器未集成) |
| **Storage** | `crates/storage` | ⚠️ 部分 | B+Tree, WAL (WAL 未默认启用) |
| **Catalog** | `crates/catalog` | ✅ 完整 | 元数据管理 |
| **Transaction** | `crates/transaction` | ✅ 完整 | MVCC、锁管理 |
| **Distributed** | `crates/distributed` | ⚠️ 框架 | 2PC/Raft (仅框架) |

## 已知未集成功能 (Issue #1497)

| 功能 | 实现 | 集成状态 |
|------|------|----------|
| 索引扫描 | ✅ | ❌ `should_use_index() = false` |
| CBO 优化 | ✅ | ❌ 未调用 `optimizer.optimize()` |
| 存储过程 | ✅ | ❌ 未连接调用链 |
| 触发器 | ✅ | ❌ 未集成到 DML |
| 外键约束 | ✅ | ❌ `table_foreign_keys: None` |
| WAL | ✅ | ❌ 默认未启用 |
| 并行执行 | ✅ | ❌ 未启用 |
| SIMD | ✅ | ❌ 未启用 |

## 开发流程

```
develop/v2.x.0 → PR → develop/v2.x+1.0 → PR → release/v2.x.0 → PR → main
```

- **main**: 只读锁定，需 PR 审核合并
- **release/v2.x.0**: 冻结状态，仅修复 bug
- **develop/v2.x.0**: 开发分支，所有功能在此开发

## 常用命令

```bash
# 测试
cargo test

# 构建
cargo build --release

# 运行
cargo run --bin sqlrustgo-server

# 覆盖率
cargo tarpaulin
```

## 最近更新

- 2026-04-17: v2.6.0 模块测试覆盖率提升
- 2026-04-17: 触发器类型 API 基础框架
- 2026-04-17: Catalog 类型定义

## 待完成任务

- [ ] v2.6.0: MVCC SSI 实现
- [ ] v2.6.0: FULL OUTER JOIN 完整支持
- [ ] v2.6.0: 并发压力测试框架
- [ ] v2.6.0: SQL Regression Corpus 5000+ 用例

## 相关链接

- [GitHub Repository](https://github.com/minzuuniversity/sqlrustgo)
- [Version Roadmap](../../releases/VERSION_ROADMAP.md)
- [Architecture](../../architecture/ARCHITECTURE.md)

---

*由 AI Agent 维护，持续更新*
