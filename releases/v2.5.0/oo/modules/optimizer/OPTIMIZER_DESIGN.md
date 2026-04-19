# Optimizer 模块设计

**版本**: v2.5.0
**模块**: Optimizer (基于成本的优化器)

---

## 一、What (是什么)

Optimizer 是 SQLRustGo 的查询优化器，负责将逻辑执行计划转换为最优的物理执行计划，基于成本模型选择最优执行策略。

## 二、Why (为什么)

- **查询优化**: 选择最优执行计划
- **成本评估**: 基于统计信息估算成本
- **规则优化**: 应用优化规则提升性能
- **连接重排**: 找到最优连接顺序

## 三、How (如何实现)

### 3.1 优化器架构

```
┌─────────────────────────────────────────┐
│           Optimizer                       │
├─────────────────────────────────────────┤
│  - Rule-based Optimization (RBO)       │
│  - Cost-based Optimization (CBO)       │
│  - Statistics Management                 │
└─────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│         LogicalPlan                       │
├─────────────────────────────────────────┤
│  - Predicate Pushdown                   │
│  - Projection Pushdown                   │
│  - Join Reordering                      │
└─────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│         PhysicalPlan                      │
├─────────────────────────────────────────┤
│  - Algorithm Selection                   │
│  - Access Path Selection                 │
│  - Parallel Execution Plan              │
└─────────────────────────────────────────┘
```

### 3.2 成本模型

```rust
pub struct CostModel {
    // 统计信息
    stats: Statistics,
    // 配置参数
    config: CostConfig,
}

pub struct CostConfig {
    cpu_cost_per_row: f64,
    io_cost_per_page: f64,
    memory_cost_per_hash: f64,
    network_cost_per_tuple: f64,
}

impl CostModel {
    // 估算扫描成本
    pub fn estimate_scan_cost(&self, table: &Table) -> Cost {
        let num_pages = table.stats().num_pages();
        let cpu_cost = table.stats().num_rows() as f64 * self.config.cpu_cost_per_row;
        let io_cost = num_pages as f64 * self.config.io_cost_per_page;
        Cost { cpu_cost, io_cost, total: cpu_cost + io_cost }
    }

    // 估算连接成本
    pub fn estimate_join_cost(&self, left: Plan, right: Plan, join_type: JoinType) -> Cost {
        let left_cost = self.estimate_cost(&left);
        let right_cost = self.estimate_cost(&right);
        let build_cost = self.estimate_hash_build_cost(&right);
        let probe_cost = left.stats().num_rows() as f64 * self.config.memory_cost_per_hash;
        Cost {
            cpu_cost: left_cost.cpu_cost + right_cost.cpu_cost + probe_cost,
            io_cost: left_cost.io_cost + right_cost.io_cost,
            total: left_cost.total + right_cost.total + probe_cost,
        }
    }
}
```

### 3.3 统计信息

```rust
pub struct Statistics {
    table_stats: HashMap<TableId, TableStatistics>,
    column_stats: HashMap<(TableId, ColumnId), ColumnStatistics>,
}

pub struct TableStatistics {
    num_rows: u64,
    num_pages: u64,
    data_size: u64,
    key_min: Value,
    key_max: Value,
}

pub struct ColumnStatistics {
    null_count: u64,
    distinct_count: u64,
    min_value: Value,
    max_value: Value,
    histogram: Histogram,
}

pub enum Histogram {
    EqualWidth { buckets: Vec<Bucket> },
    EqualHeight { buckets: Vec<Bucket> },
}
```

### 3.4 优化规则

```rust
pub struct OptimizerRules {
    rules: Vec<Box<dyn OptimizationRule>>,
}

impl OptimizerRules {
    pub fn default_rules() -> Self {
        Self {
            rules: vec![
                Box::new(PredicatePushdownRule),
                Box::new(ProjectionPushdownRule),
                Box::new(ColumnPruningRule),
                Box::new(FilterMergeRule),
                Box::new(JoinCommutativityRule),
                Box::new(JoinAssociativityRule),
                Box::new(IndexSelectionRule),
            ],
        }
    }
}
```

### 3.5 连接重排

```rust
pub struct JoinReorder {
    config: JoinOrderConfig,
}

impl JoinReorder {
    // 基于贪心的连接重排
    pub fn reorder(&self, tables: Vec<LogicalPlan>) -> Vec<LogicalPlan> {
        // IKKB (Iterative Kruskal's Based)
        let mut result = Vec::new();
        let mut used_tables = HashSet::new();

        // 选择最小基数的表作为起点
        let start = tables.iter().min_by_key(|t| t.stats().num_rows()).unwrap().clone();
        result.push(start.clone());
        used_tables.insert(start.id());

        while result.len() < tables.len() {
            // 找最小代价的下一个连接
            let next = self.find_best_next_join(&result, &tables, &used_tables);
            result.push(next);
            used_tables.insert(next.id());
        }

        result
    }
}
```

## 四、接口设计

### 4.1 公开 API

```rust
pub trait Optimizer: Send + Sync {
    // 优化逻辑计划
    fn optimize_logical(&self, plan: LogicalPlan) -> Result<LogicalPlan>;

    // 生成物理计划
    fn optimize_physical(&self, plan: LogicalPlan) -> Result<PhysicalPlan>;

    // 统计信息管理
    fn update_stats(&self, table_id: TableId, stats: TableStatistics) -> Result<()>;
}
```

### 4.2 计划类型

```rust
// 逻辑计划
pub enum LogicalPlan {
    TableScan { table: TableRef },
    Selection { input: Box<LogicalPlan>, predicate: Expr },
    Projection { input: Box<LogicalPlan>, exprs: Vec<Expr> },
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, condition: Expr },
    Aggregate { input: Box<LogicalPlan>, group_by: Vec<Expr>, aggrs: Vec<Expr> },
}

// 物理计划
pub enum PhysicalPlan {
    SeqScan { table: TableRef },
    IndexScan { table: TableRef, index: IndexRef, range: Range },
    HashJoin { left: Box<PhysicalPlan>, right: Box<PhysicalPlan>, condition: Expr },
    MergeJoin { left: Box<PhysicalPlan>, right: Box<PhysicalPlan>, condition: Expr },
    HashAgg { input: Box<PhysicalPlan>, group_by: Vec<Expr>, aggrs: Vec<Expr> },
}
```

## 五、优化策略

| 优化类型 | 规则 | 效果 |
|----------|------|------|
|谓词下推 | Filter 下推到 Scan | 减少读取数据量 |
| 投影下推 | Project 下推到 Scan | 减少列传输 |
| 连接重排 | CBO 选择最优顺序 | 减少中间结果 |
| 索引选择 | 选择最优索引 | 减少 IO |

## 六、相关文档

- [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md) - 整体架构
- [PLANNER_DESIGN.md](./PLANNER_DESIGN.md) - 查询规划

---

*Optimizer 模块设计 v2.5.0*
