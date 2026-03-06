# SQLRustGo Architecture

> **版本**: 1.0
> **更新日期**: 2026-03-07
> **维护人**: yinglichina8848

---

## 核心目标

- 高性能 SQL 执行
- Cascades Query Optimizer
- 向量化执行引擎
- 分布式扩展能力
- AI Native Software Engineering

---

# 1. 系统总体架构

```mermaid
graph TB

Client[SQL Client]

Parser[SQL Parser]

Binder[Binder]

LogicalPlan[Logical Plan]

Optimizer[Cascades Optimizer]

PhysicalPlan[Physical Plan]

Executor[Execution Engine]

Storage[Storage Engine]

Client --> Parser
Parser --> Binder
Binder --> LogicalPlan
LogicalPlan --> Optimizer
Optimizer --> PhysicalPlan
PhysicalPlan --> Executor
Executor --> Storage
```

---

# 2. 分层架构

SQLRustGo 分为五个核心层：

| 层级 | 描述 |
|------|------|
| **SQL Layer** | SQL 解析与绑定 |
| **Query Optimization Layer** | Cascades 优化器 |
| **Execution Layer** | 向量化执行引擎 |
| **Storage Layer** | 存储引擎 |
| **Distributed Layer** | 分布式执行 (2.0) |

---

# 3. 详细模块架构

```mermaid
graph TB

subgraph SQL Layer
    Client[SQL Client]
    Parser[Parser]
    Binder[Binder]
    LogicalPlan[Logical Plan]
end

subgraph Query Optimization Layer
    Optimizer[Cascades Optimizer]
    Memo[Memo Structure]
    Rules[Optimization Rules]
    CostModel[Cost Model]
end

subgraph Execution Layer
    Executor[Execution Engine]
    VectorEngine[Vector Engine]
    Pipeline[Pipeline Execution]
end

subgraph Storage Layer
    Storage[Storage Engine]
    BPlusTree[B+ Tree]
    BufferPool[Buffer Pool]
    WAL[Write-Ahead Log]
end

Client --> Parser
Parser --> Binder
Binder --> LogicalPlan
LogicalPlan --> Optimizer
Optimizer --> Memo
Memo --> Rules
Rules --> CostModel
CostModel --> Executor
Executor --> VectorEngine
VectorEngine --> Pipeline
Pipeline --> Storage
Storage --> BPlusTree
Storage --> BufferPool
Storage --> WAL
```

---

# 4. 查询执行流程

```mermaid
flowchart LR

SQL[SQL Query]
    --> Parser
    --> Binder
    --> LogicalPlan
    --> Optimizer
    --> PhysicalPlan
    --> Executor
    --> Storage
    --> Result[Result Set]

subgraph Optimization
    LogicalPlan --> Memo[Memo Construction]
    Memo --> Explore[Rule Exploration]
    Explore --> Physical[Physical Plan]
    Physical --> Cost[Cost Evaluation]
end
```

---

# 5. 核心设计原则

## 5.1 模块化

- 每个组件独立 crate
- 清晰的依赖关系
- 最小化公开 API

## 5.2 可扩展

- 优化器规则可插件化
- 执行算子可扩展
- 存储引擎可替换

## 5.3 高性能

- 向量化执行 (Vectorized Execution)
- Pipeline 流水线执行
- 无锁数据结构

## 5.4 分布式 (2.0)

- MPP 架构
- 分布式查询规划
- Shuffle 机制
- 故障容错

---

# 6. 核心模块

## 6.1 Parser (SQL 解析器)

```
┌─────────────┐
│    SQL      │
└──────┬──────┘
       │
       ▼
┌─────────────┐    ┌─────────────┐
│   Lexer     │───►│    AST      │
│  (词法)     │    │  (语法树)   │
└─────────────┘    └─────────────┘
```

## 6.2 Optimizer (Cascades 优化器)

```mermaid
graph TD

LogicalPlan[Logical Plan] --> Memo[Memo]
Memo --> Group[Group]
Group --> Expr[Expr]
RuleEngine[Rule Engine] --> Memo
CostModel[Cost Model] --> Memo
Memo --> BestPlan[Best Plan]
```

## 6.3 Executor (执行引擎)

```mermaid
graph LR

PhysicalPlan[Physical Plan] --> Operators[Operators]
Operators --> Vector[Vectorized Execution]
Vector --> Pipeline[Pipeline]
Pipeline --> Result[Record Batch]
```

## 6.4 Storage (存储引擎)

```mermaid
graph TB

Storage[Storage Engine] --> Table[Table Manager]
Storage --> Index[Index Manager]
Storage --> Buffer[Buffer Pool]
Table --> BPlusTree[B+ Tree]
Index --> BPlusTree
Buffer --> Cache[Page Cache]
```

---

# 7. 版本演进

```mermaid
graph LR

v1x[1.x<br/>单机内核] --> v2x[2.0<br/>分布式]
v2x --> v3x[3.0<br/>云原生]

subgraph v1x
    Parser1[Parser]
    Executor1[Executor]
    Storage1[Storage]
end

subgraph v2x
    Coordinator[Coordinator]
    Worker1[Worker 1]
    Worker2[Worker 2]
    Worker3[Worker 3]
end

subgraph v3x
    Cloud[Cloud Native]
    K8s[Kubernetes]
end
```

| 版本 | 特性 |
|------|------|
| **1.x** | SQL 执行原型、Cascades 优化器、向量化执行 |
| **2.0** | 分布式 MPP、Shuffle Exchange、故障容错 |
| **3.0** | 云原生、Kubernetes 集成、弹性伸缩 |

---

# 8. 技术栈

| 组件 | 技术 |
|------|------|
| **语言** | Rust |
| **解析** | nom |
| **优化器** | Cascades Framework |
| **执行** | Vectorized / Pipeline |
| **存储** | B+ Tree / Buffer Pool |
| **网络** | Tokio / Tower |
| **测试** | Cargo Test / Criterion |

---

# 9. 相关文档

| 文档 | 说明 |
|------|------|
| [CASCADES_OPTIMIZER.md](./CASCADES_OPTIMIZER.md) | Cascades 优化器设计 |
| [DISTRIBUTED_EXECUTION.md](./DISTRIBUTED_EXECUTION.md) | 分布式执行架构 |
| [DIRECTORY_STRUCTURE.md](./DIRECTORY_STRUCTURE.md) | 目录结构规范 |
| [BRANCH_GOVERNANCE.md](../governance/BRANCH_GOVERNANCE.md) | 分支治理规范 |

---

# 10. 变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 yinglichina8848 维护*
