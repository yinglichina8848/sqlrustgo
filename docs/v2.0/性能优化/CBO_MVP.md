# CBO 最小可运行实现

> Cost-Based Optimizer MVP

---

## 架构层级

```
SQL
  ↓
Parser
  ↓
LogicalPlan
  ↓
Rule Rewriter
  ↓
Cost Estimator
  ↓
PhysicalPlan
  ↓
Executor
```

---

## 核心数据结构

### Logical Plan

```rust
pub enum LogicalPlan {
    Scan { table: String },
    Filter { predicate: Expr, input: Box<LogicalPlan> },
    Projection { columns: Vec<String>, input: Box<LogicalPlan> },
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        on: Expr,
    },
}
```

### Physical Plan

```rust
pub enum PhysicalPlan {
    SeqScan { table: String },
    FilterExec { predicate: Expr, input: Box<PhysicalPlan> },
    ProjectionExec { columns: Vec<String>, input: Box<PhysicalPlan> },
    HashJoinExec {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        on: Expr,
    },
}
```

---

## 统计信息

```rust
use std::collections::HashMap;

#[derive(Clone)]
struct Stats {
    rows: f64,
    pages: f64,
    distinct: HashMap<String, f64>,
}
```

---

## 成本模型

```rust
struct CostModel {
    seq_page_cost: f64,
    cpu_tuple_cost: f64,
}

impl CostModel {
    fn scan_cost(&self, stats: &Stats) -> f64 {
        stats.pages * self.seq_page_cost
            + stats.rows * self.cpu_tuple_cost
    }

    fn hash_join_cost(&self, left: &Stats, right: &Stats) -> f64 {
        self.scan_cost(left)
            + self.scan_cost(right)
            + right.rows * self.cpu_tuple_cost
    }
}
```

---

## Join 选择器

```rust
fn choose_join(
    model: &CostModel,
    a: &Stats,
    b: &Stats,
) {
    let cost_ab = model.hash_join_cost(a, b);
    let cost_ba = model.hash_join_cost(b, a);

    if cost_ab < cost_ba {
        println!("Choose A as build side");
    } else {
        println!("Choose B as build side");
    }
}
```

---

## 优化器流程

```rust
pub fn optimize(plan: LogicalPlan) -> PhysicalPlan {
    let candidates = rewrite(plan);
    choose_lowest_cost(candidates)
}
```

---

## 成本公式

### 单表扫描

```
Cost_scan = B × C_seq + N × C_cpu
```

- B = 表页数
- N = 表行数
- C_seq = 顺序 I/O 成本
- C_cpu = 单行 CPU 成本

### Hash Join

```
Cost_hash = B_build + B_probe + N_probe × C_cpu
```

### Nested Loop Join

```
Cost_NLJ = Cost_outer + N_outer × Cost_inner
```

---

## Selectivity 估计

### 等值条件

```
s = 1 / distinct
```

### 范围条件

```
s = (high - low) / (max - min)
```

---

## MVP 范围

第一版只做：

1. Join 顺序比较
2. Scan 方式选择
3. 简单成本估计

---

## 扩展方向

- 统计信息收集
- 直方图
- 多列统计
- 自适应成本模型
