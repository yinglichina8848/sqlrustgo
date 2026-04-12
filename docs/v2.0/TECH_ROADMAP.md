# SQLRustGo 技术路线图 (1.x → 3.0)

> **版本**: v1.0
> **状态**: 战略规划
> **目标**: 建立完整数据库内核发展路径
> **更新日期**: 2026-03-11

---

## 一、SQLRustGo 总体演进路线

### 1.1 整体路线

```
1.x   单机数据库内核
  ↓
2.x   向量化数据库
  ↓
3.x   分布式数据库
```

### 1.2 完整结构

```
                SQLRustGo
                    │
        ┌───────────┼───────────┐
        │           │           │
      1.x         2.x         3.x
  Core Engine   Vector DB   Distributed DB
```

---

## 二、SQLRustGo 1.x（数据库核心阶段）

### 2.1 目标

建立完整数据库内核

### 2.2 能力范围

| 模块 | 内容 |
|------|------|
| Parser | SQL 解析 |
| Planner | LogicalPlan |
| Optimizer | Rule-based |
| Executor | Volcano Model |
| Storage | Row Store |
| Transaction | MVCC 基础 |
| Catalog | 元数据 |

### 2.3 版本规划

#### v1.2 - 数据库原型

| 项目 | 内容 |
|------|------|
| 目标 | 数据库原型 |
| 能力 | SELECT, INSERT, Basic Scan, Basic Filter |
| 性能 | 30k rows/s |

#### v1.3 - 稳定执行引擎（关键版本）

| 项目 | 内容 |
|------|------|
| 目标 | 稳定执行引擎 |
| 新增 Executor | Volcano 稳定化 |
| 新增 Operators | Projection, HashJoin |
| 新增 Optimizer | Predicate Pushdown |
| 性能目标 | 50k rows/s |

#### v1.5 - 完整 SQL Engine

| 项目 | 内容 |
|------|------|
| 目标 | 完整 SQL Engine |
| 新增 GroupBy | Aggregation |
| 新增 Sort | ORDER BY |
| 新增 Index | BTree |
| 新增 Transaction | MVCC |
| 性能 | 100k rows/s |

### 2.4 1.x 架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SQLRustGo 1.x 架构                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         SQL Layer                                    │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│   │  │   Parser    │  │  Optimizer  │  │  Executor   │                  │   │
│   │  │   (SQL)     │  │   (Rules)   │  │  (Volcano)  │                  │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                       Storage Layer                                  │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│   │  │   B+Tree    │  │ BufferPool  │  │    WAL      │                  │   │
│   │  │   (Index)   │  │   (Cache)   │  │   (Log)     │                  │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 三、SQLRustGo 2.0（向量化数据库）

### 3.1 核心变化

```
Volcano → Vectorized Execution
```

类似系统: DuckDB, ClickHouse

### 3.2 核心模块

| 模块 | 内容 |
|------|------|
| Vector Engine | 批量执行 |
| Pipeline Engine | 流水线 |
| Cascades Optimizer | CBO |
| Statistics | 统计信息 |
| Column Store | 列存 |

### 3.3 执行架构

```
                SQL
                 │
              Parser
                 │
            LogicalPlan
                 │
         Cascades Optimizer
                 │
            PhysicalPlan
                 │
         Pipeline Builder
                 │
       Vectorized Execution
                 │
            Storage Engine
```

### 3.4 新能力

#### Vector Execution

- 从 tuple 转为 vector
- 每次处理 1024 rows
- 性能提升: 5x – 20x

#### Pipeline Execution

```
Scan → Filter → Join → Aggregate
```

每次处理: 1024 rows

#### Cascades Optimizer

| 优化 | 作用 |
|------|------|
| Join Order | 最优连接顺序 |
| Predicate Pushdown | 减少扫描 |
| Projection Pushdown | 减少 IO |

### 3.5 性能目标

| 操作 | 目标 |
|------|------|
| Insert | 500k rows/s |
| Scan | 1M rows/s |
| Join | 200k rows/s |
| Aggregate | 1M rows/s |

### 3.6 2.x 架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SQLRustGo 2.x 架构                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Query Processing Layer                          │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│   │  │   Parser    │  │  Cascades   │  │    Cost     │                  │   │
│   │  │             │  │  Optimizer  │  │   Model     │                  │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    Execution Layer                                   │   │
│   │  ┌─────────────────────────────────────────────────────────────┐    │   │
│   │  │                 Vectorized Pipeline                         │    │   │
│   │  │   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │    │   │
│   │  │   │  Scan   │ │ Filter  │ │  Join   │ │ Aggregate│           │    │   │
│   │  │   │ (Vector)│ │(Vector) │ │(Vector) │ │ (Vector) │           │    │   │
│   │  │   └─────────┘ └─────────┘ └─────────┘ └─────────┘           │    │   │
│   │  └─────────────────────────────────────────────────────────────┘    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                         │
│                                    ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                      Storage Layer                                  │   │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│   │  │Column Store │  │ BufferPool  │  │    WAL      │                  │   │
│   │  │  (Vector)   │  │   (Cache)   │  │   (Log)     │                  │   │
│   │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 四、SQLRustGo 3.0（分布式数据库）

### 4.1 定位

进入 **Distributed SQL** 领域

类似系统: CockroachDB, TiDB

### 4.2 核心架构

```
              Distributed SQL
                     │
           ┌─────────┴─────────┐
           │                   │
        Query Layer        Storage Layer
           │                   │
     Distributed Planner     Raft
           │                   │
      Distributed Exec     KV Store
```

### 4.3 新模块

| 模块 | 内容 |
|------|------|
| Distributed Planner | 分布式计划 |
| Scheduler | 任务调度 |
| Shuffle | 数据交换 |
| Consensus | Raft |

### 4.4 查询执行示例

```sql
SELECT *
FROM orders
JOIN customers
```

执行流程:

```
Node1: Scan orders
Node2: Scan customers
       ↓
     Shuffle
       ↓
     Join
```

### 4.5 3.x 架构

```
                SQLRustGo 3.0
                     │
               SQL Gateway
                     │
             Distributed Planner
                     │
         ┌───────────┼───────────┐
         │           │           │
      Node1       Node2       Node3
         │           │           │
     Vector Exec  Vector Exec  Vector Exec
         │           │           │
        Storage     Storage     Storage
```

---

## 五、SQLRustGo 最终架构（完整版）

```
                     SQLRustGo
                         │
        ┌────────────────┼────────────────┐
        │                │                │
     Parser          Optimizer         Executor
        │                │                │
   SQL Parser      Cascades CBO     Vector Engine
        │                │                │
     AST            Memo Engine      Pipeline Engine
        │                │                │
     LogicalPlan     Cost Model        Operators
                         │
                    PhysicalPlan
                         │
                     Storage
                         │
                   Transaction
                         │
                       Disk
```

---

## 六、完整版本规划

| 版本 | 阶段 | 特征 |
|------|------|------|
| 1.0 | Prototype | SQL parser |
| 1.2 | Engine | 基础执行 |
| 1.3 | Stable | Executor 稳定化 |
| 1.5 | SQL | 完整 SQL |
| 2.0 | Vector | 向量化执行 |
| 2.5 | Parallel | 并行执行 |
| 3.0 | Distributed | 分布式 |

---

## 七、技术复杂度对比

| 系统 | 难度 |
|------|------|
| SQL Parser | ★ |
| Executor | ★★ |
| Optimizer | ★★★ |
| Vector Engine | ★★★ |
| Distributed DB | ★★★★★ |

**数据库最难部分**: Optimizer + Distributed Execution

---

## 八、SQLRustGo 战略定位

### 8.1 潜在定位

**Rust 原生数据库内核**

类似: DuckDB + Velox 思路

### 8.2 对标系统

| 系统 | 类型 |
|------|------|
| DuckDB | 单机分析数据库 |
| ClickHouse | 分布式分析 |
| CockroachDB | 分布式事务 |

---

## 九、关键成功因素

### 9.1 核心原则

> **不是功能多，而是架构稳定**

### 9.2 正确路线

```
1.x 先稳定
2.0 再革命
3.0 才分布式
```

### 9.3 阶段重点

| 版本 | 重点 |
|------|------|
| 1.x | 稳定核心内核 |
| 2.0 | 性能突破 |
| 3.0 | 扩展性 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-11 | 初始版本 |