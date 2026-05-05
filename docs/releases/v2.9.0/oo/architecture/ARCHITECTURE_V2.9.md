# SQLRustGo v2.9.0 架构设计

> **版本**: v2.9.0 (RC)
> **代号**: Formal Verification Excellence
> **更新日期**: 2026-05-05

---

## 1. 概述

v2.9.0 是 SQLRustGo 迈向形式化验证卓越的关键版本。本文档描述整体架构设计和核心模块。

### 1.1 版本定位

| 属性 | 值 |
|------|-----|
| 版本 | v2.9.0 |
| 阶段 | RC (Release Candidate) |
| 代号 | Formal Verification Excellence |
| 覆盖率 | 84.18% |
| SQL Corpus | 92.6% (449/485) |
| 形式化证明 | 18/18 |

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                     SQLRustGo v2.9.0                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Parser    │───▶│   Planner   │───▶│  Executor   │     │
│  │  (SQL-92)   │    │             │    │             │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                                     │              │
│         │              ┌─────────────┐      │              │
│         └─────────────▶│  Optimizer   │◀─────┘              │
│                        │    (CBO)    │                     │
│                        └─────────────┘                     │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Storage   │◀───│   Catalog   │◀───│ Transaction │     │
│  │   Engine    │    │             │    │   Manager   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                                       │              │
│         │         ┌─────────────┐                │              │
│         └────────▶│     WAL     │◀───────────────┘              │
│                   │ (Crash Rec.) │                              │
│                   └─────────────┘                                │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ MySQL Server│    │    REPL     │    │  GMP Audit  │     │
│  │  Protocol   │    │             │    │   Support   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐     │
│  │            Unified Search API (qmd-bridge)           │     │
│  │         lex / vec / graph / hybrid                  │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐     │
│  │              Distributed Orchestration                │     │
│  │         Nomad + Runner + Gitea Actions             │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐     │
│  │              Formal Verification Layer              │     │
│  │         TLA+ / Dafny / Formulog / Proof          │     │
│  └─────────────────────────────────────────────────────┘     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块

| 模块 | 职责 | 关键组件 |
|------|------|----------|
| Parser | SQL 解析 | Tokenizer, AST |
| Planner | 查询规划 | LogicalPlan, PhysicalPlan |
| Optimizer | 优化执行 | CBO, Rules |
| Executor | 执行算子 | Scan, Join, Aggregate |
| Storage | 存储引擎 | BufferPool, BTree, Columnar |
| Transaction | 事务管理 | MVCC, WAL |
| WAL | 崩溃恢复 | Write-Ahead Logging |
| Search | 统一检索 | lex, vec, graph, hybrid |
| Distributed | 分布式编排 | Nomad, Runner |

---

## 3. v2.9.0 核心改进

### 3.1 形式化验证 (R10)

v2.9.0 实现多层次形式化验证：

| 工具 | 验证内容 | 结果 |
|------|----------|------|
| TLA+ | 并发协议、事务语义 | 6 PASS |
| Dafny | 算法正确性 | 1 PASS |
| Formulog | 逻辑推理 | 5 PASS |
| Proof Registry | 证明文件管理 | 18/18 |

### 3.2 Coverage CI/CD

自动化覆盖率集成：

- 每次 PR 自动检测覆盖率变化
- L1/L2/L3 分层覆盖率检查
- 报告自动生成并归档

### 3.3 分布式设计

| 组件 | 说明 |
|------|------|
| Nomad | 任务调度 |
| Runner | Gitea Actions 执行器 |
| Gitea Actions | CI/CD 流水线 |
| Gate Policy | 质量门禁判断 |

### 3.4 SQL 兼容性

| 指标 | 值 |
|------|---|
| SQL Corpus 通过率 | 92.6% (449/485) |
| TPC-H 可运行查询 | 9/22 |
| 目标 | ≥85% |

---

## 4. 模块架构

### 4.1 Parser

```
SQL Input
    │
    ▼
┌─────────┐
│Tokenizer │───▶ Tokens
└─────────┘
    │
    ▼
┌─────────┐
│  Parser  │───▶ AST
└─────────┘
    │
    ▼
┌─────────┐
│ Resolver │───▶ Logical Plan
└─────────┘
```

### 4.2 Executor

```
Logical Plan
    │
    ▼
┌─────────────┐
│  Optimizer  │───▶ Physical Plan
└─────────────┘
    │
    ▼
┌─────────────┐
│  Executor   │
├─────────────┤
│  Scan       │
│  Projection │
│  Filter     │
│  Join      │
│  Aggregate  │
│  Sort      │
└─────────────┘
    │
    ▼
   Result
```

### 4.3 Transaction (MVCC)

```
Transaction T1
    │
    ▼
┌─────────────┐
│ Begin Tx    │
└─────────────┘
    │
    ▼
┌─────────────┐     ┌─────────────┐
│ Read Set    │     │ Version Chain│
│ (Snapshot)  │◀───│  Management   │
└─────────────┘     └─────────────┘
    │
    ▼
┌─────────────┐
│ Validate    │───▶ SSI Validation
└─────────────┘
    │
    ▼
┌─────────────┐
│ Commit/WAL  │
└─────────────┘
```

---

## 5. 数据流

### 5.1 查询执行流程

```
Client (MySQL Protocol)
    │
    ▼
Network Layer (Server)
    │
    ▼
Parser (SQL → AST)
    │
    ▼
Planner (AST → LogicalPlan)
    │
    ▼
Optimizer (LogicalPlan → PhysicalPlan)
    │
    ▼
Executor (PhysicalPlan → Result)
    │
    ▼
Transaction Manager (MVCC + WAL)
    │
    ▼
Storage Engine (BTree/Columnar)
```

### 5.2 写入流程

```
INSERT/UPDATE/DELETE
    │
    ▼
Transaction Manager
    │
    ▼
WAL (Write-Ahead Log)
    │
    ▼
Storage Engine
    │
    ▼
Index Update (BTree/Vector/Graph)
```

---

## 6. 安全性

### 6.1 审计证据链

| 组件 | 说明 |
|------|------|
| GMP Audit | 操作审计 |
| Evidence Chain | 完整性校验 |
| Immutability | 证据不可篡改 |

### 6.2 安全扫描

- Cargo Audit: 依赖漏洞扫描
- 形式化验证: 协议安全性证明
- 攻击面分析: AV1-AV10

---

## 7. 性能目标

### 7.1 性能指标

| 测试 | 目标 | 实际 |
|------|------|------|
| MemoryExecutionEngine INSERT QPS | - | 10,770 |
| MemoryExecutionEngine SELECT QPS | - | ~2,200 |
| sysbench OLTP QPS | ≥10,000 | ~2,000 |
| TPC-H P99 延迟 | <1000ms | <200ms |

### 7.2 覆盖率

| 指标 | 目标 | 实际 |
|------|------|------|
| 总行覆盖率 | ≥75% | 84.18% |
| executor 覆盖率 | ≥60% | 71.08% |

---

## 8. 相关文档

- [../VERSION_PLAN.md](../VERSION_PLAN.md) - 版本计划
- [../RELEASE_NOTES.md](../RELEASE_NOTES.md) - 发布说明
- [../FEATURE_MATRIX.md](../FEATURE_MATRIX.md) - 功能矩阵
- [../RELEASE_GATE_CHECKLIST.md](../RELEASE_GATE_CHECKLIST.md) - 门禁清单

---

## 9. 版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| v2.9.0 | 2026-05-05 | Formal Verification Excellence |
| v2.7.0 | 2026-04-22 | Enterprise Resilience |

---

*架构设计 v2.9.0*
*最后更新: 2026-05-05*
