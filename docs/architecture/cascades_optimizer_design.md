# 级联优化器设计

> **版本**: 2.x (规划中)
> **更新日期**: 2026-03-05

---

## 1. 为什么需要 Cascades

SQLRustGo 在 2.x 计划实现 **Cascades Optimizer**。

Cascades 是现代数据库优化器的主流架构，被以下系统采用：

| 数据库 | 架构 |
|--------|------|
|SQL服务器|瀑布|
|绿梅|瀑布|
|蟑螂数据库|瀑布|
|阿帕奇兽人|瀑布|

### 传统优化器 vs Cascades

| 特性 | 传统优化器 |瀑布|
|------|------------|----------|
| 搜索空间 | 有限 | 完整 |
| 规则扩展 | 困难 | 容易 |
| 成本模型 | 简单 | 灵活 |
| 执行计划 | 次优 | 更优 |

---

## 2. Cascades 架构

```mermaid
graph TB
    subgraph "Input"
        SQL["SQL Query"]
        Parser["Parser"]
        LogicalPlan["Logical Plan"]
    end

    subgraph "Cascades Engine"
        Memo["Memo<br/>Intermediate Representation"]
        Group["Group<br/>Equivalence Class"]
        Explorer["Explorer<br/>Search Engine"]
    end

    subgraph "Optimization"
        Rules["Transformation<br/>Rules"]
        CostModel["Cost Model"]
        Memoize["Memoize<br/>Deduplication"]
    end

    subgraph "Output"
        BestPlan["Best Physical Plan"]
    end

    SQL --> Parser
    Parser --> LogicalPlan
    LogicalPlan --> Memo
    Memo --> Explorer
    Explorer --> Rules
    Explorer --> CostModel
    Rules --> Memoize
    CostModel --> Memoize
    Memoize --> Memo
    Memo --> BestPlan
```

---

## 3. Memo 数据结构

Memo 用于保存所有等价表达式，避免重复计算。

```mermaid
classDiagram
    class Memo {
        +groups: Vec~Group~
        +group_id_counter: u32
        +create_group() GroupId
        +insert_expr(expr: Expr) GroupId
        +get_best_cost(group_id: GroupId) Cost
    }

    class Group {
        +group_id: GroupId
        +logical_expressions: Vec~Expr~
        +physical_expressions: Vec~Expr~
        +best_cost: Cost
        +best_plan: Option~PhysicalPlan~
        +explored: bool
    }

    class Expr {
        +expr_type: ExprType
        +children: Vec~Expr~
    }

    class GroupId {
        +id: u32
    }

    Memo "1" --> "*" Group
    Group "1" --> "*" Expr
    GroupId --> Group
```

### Memo 结构示意

```rust
struct Memo {
    groups: Vec<Group>
}

struct Group {
    group_id: GroupId,
    expressions: Vec<Expr>,  // 逻辑表达式
    best_cost: Cost,         // 最低成本
    best_plan: PhysicalPlan, // 最优计划
}
```

---

## 4. 优化流程

```mermaid
flowchart TB
    Start["Input: Logical Plan"] --> CreateMemo["Create Memo from Logical Plan"]
    
    subgraph "Exploration Loop"
        CreateMemo --> CheckExplored{All Groups Explored?}
        CheckExplored -->|No| SelectGroup["Select Unexplored Group"]
        SelectGroup --> ApplyRules["Apply Transformation Rules"]
        ApplyRules --> GenerateExprs["Generate Equivalent Expressions"]
        GenerateExprs --> AddToMemo["Add to Memo Groups"]
        AddToMemo --> Costing["Cost Each Physical Plan"]
        Costing --> UpdateBest["Update Best Plan per Group"]
        UpdateBest --> CheckExplored
    end
    
    CheckExplored -->|Yes| ExtractPlan["Extract Best Plan"]
    ExtractPlan --> Output["Output: Physical Plan"]
```

---

## 5. 转换规则

示例规则：

### 5.1 Join 交换律

```mermaid
graph LR
    A["Join(A, B)"] -->|Commute| B["Join(B, A)"]
```

### 5.2 Filter 合并

```mermaid
graph LR
    A["Filter(Filter(X))"] -->|Collapse| B["Filter(X)"]
```

### 5.3谓词下推

```mermaid
graph LR
    A["Join(Filter(A), B)"] -->|PushDown| B["Filter(Join(A, B))"]
```

### 规则接口

```rust
pub trait TransformRule {
    fn match_expr(&self, expr: &Expr) -> bool;
    fn apply(&self, expr: &Expr) -> Vec<Expr>;
}
```

---

## 6. 成本模型

成本函数：

```mermaid
classDiagram
    class Cost {
        +cpu: f64
        +io: f64
        +memory: f64
        +network: f64
        +add(other: Cost) Cost
        +compare(other: Cost) Ordering
    }

    class CostModel {
        +calculate_scan(rows: usize) Cost
        +calculate_join(left: Cost, right: Cost) Cost
        +calculate_filter(cost: Cost) Cost
    }

    CostModel ..> Cost
```

### 成本计算公式

```
TotalCost = CPU_Cost + I/O_Cost + Memory_Cost + Network_Cost

CPU_Cost = rows * cpu_per_row
I/O_Cost = pages * disk_latency
Memory_Cost = bytes * memory_bandwidth
Network_Cost = transfer_bytes * network_bandwidth
```

---

## 7. 搜索策略

### 7.1 搜索方向

```mermaid
flowchart LR
    subgraph "Top-Down"
        TD["从物理计划向下搜索"]
    end

    subgraph "Bottom-Up"
        BU["从逻辑计划向上搜索"]
    end
```

### 7.2 剪枝策略

```mermaid
flowchart TB
    Start["Start Exploration"] --> CheckBound{"Current Cost > Best Known?"}
    CheckBound -->|Yes| Prune["Prune Branch"]
    CheckBound -->|No| Continue["Continue Exploration"]
    Prune --> Done
    Continue --> CheckBound
    Done["Done"]
```

---

## 8. SQLRustGo Cascades 规划

| 阶段 | 功能 | 目标版本 |
|------|------|----------|
| Phase 1 |备忘录引擎| 2.0 |
| Phase 2 |规则引擎| 2.1 |
| Phase 3 |成本模型| 2.2 |
| Phase 4 |搜索策略| 2.3 |

### 8.1 实现计划

```mermaid
gantt
    title Cascades Optimizer Implementation
    dateFormat  YYYY-MM-DD
    section Phase 1
    Memo Structure       :a1, 2026-04-01, 30d
    Group Management     :a2, after a1, 14d
    section Phase 2
    Rule Interface      :b1, after a2, 30d
    Basic Rules         :b2, after b1, 21d
    section Phase 3
    Cost Model Interface:c1, after b2, 21d
    Physical Costing    :c2, after c1, 14d
    section Phase 4
    Search Engine      :d1, after c2, 30d
    Integration         :d2, after d1, 14d
```

---

## 9. Cascades 优势

| 优势 | 说明 |
|------|------|
| **规则扩展** | 新规则容易加入，只需实现 TransformRule trait |
| **计划搜索** | 更全面的搜索空间 |
| **性能优化** | 更优的执行计划 |
| **成本模型** | 可插拔的成本模型 |
| **复用** | Memo 避免重复计算 |

---

## 10. 与其他系统的对标

| 数据库 | 优化器架构 |SQLRustGo|
|--------|------------|-----------|
|SQL服务器|瀑布| ✅ 目标 |
|绿梅|瀑布| ✅ 目标 |
|蟑螂数据库|瀑布| ✅ 目标 |
|PostgreSQL|启发式| ❌ 已超越 |
| MySQL |启发式| ❌ 已超越 |

---

## 11. 相关文档

- [SQLRustGo Architecture](./sqlrustgo_architecture.md)
- [Distributed Scheduler](./distributed_scheduler_design.md)
- [Whitepaper](../whitepaper/sqlrustgo_1.2_release_whitepaper.md)
