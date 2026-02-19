# 成本优化器（Cost-Based Optimizer）设计

> 版本：v1.0
> 日期：2026-02-18
> 目标：从 L2（规则优化）升级到 L4（成本优化）

---

## 一、CBO 核心组成

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          CBO 架构                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Statistics（统计信息）                                                    │
│       │                                                                      │
│       ▼                                                                      │
│   Cost Model（成本模型）                                                    │
│       │                                                                      │
│       ▼                                                                      │
│   Plan Enumerator（计划枚举器）                                             │
│       │                                                                      │
│       ▼                                                                      │
│   Best Plan（最优计划）                                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、统计信息（Statistics）

**没有统计信息，就没有成本优化**。

### 2.1 表级统计

```rust
#[derive(Clone, Debug)]
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub avg_row_size: f64,
}

impl TableStats {
    pub fn new(row_count: usize, total_bytes: usize) -> Self {
        Self {
            row_count,
            total_bytes,
            avg_row_size: if row_count > 0 {
                total_bytes as f64 / row_count as f64
            } else {
                0.0
            },
        }
    }
}
```

### 2.2 列级统计

```rust
#[derive(Clone, Debug)]
pub struct ColumnStats {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<ScalarValue>,
    pub max_value: Option<ScalarValue>,
    pub histogram: Option<Histogram>,
}

#[derive(Clone, Debug)]
pub struct Histogram {
    pub buckets: Vec<Bucket>,
    pub num_buckets: usize,
}

#[derive(Clone, Debug)]
pub struct Bucket {
    pub lower: ScalarValue,
    pub upper: ScalarValue,
    pub count: usize,
    pub distinct: usize,
}
```

### 2.3 统计信息收集

```rust
pub trait StatisticsCollector: Send + Sync {
    fn collect_table_stats(&self, table: &str) -> Result<TableStats>;
    fn collect_column_stats(&self, table: &str, column: &str) -> Result<ColumnStats>;
    fn analyze_table(&self, table: &str) -> Result<()>;
}

pub struct DefaultStatisticsCollector {
    storage: Arc<dyn StorageEngine>,
    cache: RwLock<HashMap<String, TableStats>>,
}

impl StatisticsCollector for DefaultStatisticsCollector {
    fn collect_table_stats(&self, table: &str) -> Result<TableStats> {
        if let Some(stats) = self.cache.read().unwrap().get(table) {
            return Ok(stats.clone());
        }
        
        let rows = self.storage.scan(table)?;
        let row_count = rows.len();
        let total_bytes = rows.iter().map(|r| r.size()).sum();
        
        let stats = TableStats::new(row_count, total_bytes);
        self.cache.write().unwrap().insert(table.to_string(), stats.clone());
        
        Ok(stats)
    }
}
```

---

## 三、成本模型（Cost Model）

**成本 = I/O + CPU + 内存**

### 3.1 成本结构

```rust
#[derive(Clone, Debug, Default)]
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub memory: f64,
}

impl Cost {
    pub fn total(&self, weights: &CostWeights) -> f64 {
        self.cpu * weights.cpu
            + self.io * weights.io
            + self.memory * weights.memory
    }
    
    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            cpu: self.cpu + other.cpu,
            io: self.io + other.io,
            memory: self.memory + other.memory,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CostWeights {
    pub cpu: f64,
    pub io: f64,
    pub memory: f64,
}

impl Default for CostWeights {
    fn default() -> Self {
        Self {
            cpu: 1.0,
            io: 10.0,
            memory: 0.5,
        }
    }
}
```

### 3.2 操作成本估算

```rust
pub trait CostEstimator: Send + Sync {
    fn estimate_table_scan(&self, stats: &TableStats) -> Cost;
    fn estimate_filter(&self, input_cost: &Cost, selectivity: f64) -> Cost;
    fn estimate_join(&self, left: &Cost, right: &Cost, join_type: JoinType) -> Cost;
    fn estimate_aggregate(&self, input_cost: &Cost, groups: usize) -> Cost;
}

pub struct DefaultCostEstimator {
    weights: CostWeights,
}

impl CostEstimator for DefaultCostEstimator {
    fn estimate_table_scan(&self, stats: &TableStats) -> Cost {
        Cost {
            cpu: stats.row_count as f64 * 0.001,
            io: stats.total_bytes as f64 / 4096.0,
            memory: 0.0,
        }
    }
    
    fn estimate_filter(&self, input_cost: &Cost, selectivity: f64) -> Cost {
        Cost {
            cpu: input_cost.cpu * selectivity,
            io: 0.0,
            memory: 0.0,
        }
    }
    
    fn estimate_join(&self, left: &Cost, right: &Cost, join_type: JoinType) -> Cost {
        match join_type {
            JoinType::NestedLoop => Cost {
                cpu: left.cpu * right.cpu,
                io: left.io + right.io,
                memory: 0.0,
            },
            JoinType::HashJoin => Cost {
                cpu: left.cpu + right.cpu,
                io: left.io + right.io,
                memory: left.memory + right.memory,
            },
            JoinType::SortMerge => Cost {
                cpu: left.cpu * left.cpu.log2() + right.cpu * right.cpu.log2(),
                io: left.io + right.io,
                memory: 0.0,
            },
        }
    }
}
```

---

## 四、Join 选择示例

### 4.1 NestedLoop 成本

```
cost = left_rows * right_rows
```

**示例**：
```
left: 50,000 rows
right: 50,000 rows
cost = 50k × 50k = 2,500,000,000

🚨 不可接受
```

### 4.2 HashJoin 成本

```
cost = left_rows + right_rows
memory = build_side_size
```

**示例**：
```
left: 50,000 rows
right: 50,000 rows
cost = 50k + 50k = 100,000
memory = 5MB

✅ 可接受
```

### 4.3 优化器选择

```rust
pub struct JoinCostModel;

impl JoinCostModel {
    pub fn choose_join_method(
        &self,
        left_stats: &TableStats,
        right_stats: &TableStats,
        memory_limit: usize,
    ) -> JoinMethod {
        let nested_loop_cost = left_stats.row_count * right_stats.row_count;
        let hash_join_cost = left_stats.row_count + right_stats.row_count;
        let hash_memory = left_stats.total_bytes.min(right_stats.total_bytes);
        
        if hash_memory <= memory_limit && hash_join_cost < nested_loop_cost {
            JoinMethod::HashJoin
        } else {
            JoinMethod::NestedLoop
        }
    }
}
```

---

## 五、Plan 枚举器

### 5.1 简单版本

```rust
pub struct PlanEnumerator {
    cost_estimator: Arc<dyn CostEstimator>,
}

impl PlanEnumerator {
    pub fn enumerate_join_orders(&self, tables: Vec<String>) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        self.permute(&tables, &mut vec![], &mut result);
        result
    }
    
    fn permute(
        &self,
        remaining: &[String],
        current: &mut Vec<String>,
        result: &mut Vec<Vec<String>>,
    ) {
        if remaining.is_empty() {
            result.push(current.clone());
            return;
        }
        
        for i in 0..remaining.len() {
            let mut new_remaining = remaining.to_vec();
            let table = new_remaining.remove(i);
            current.push(table);
            self.permute(&new_remaining, current, result);
            current.pop();
        }
    }
    
    pub fn find_best_plan(
        &self,
        plans: Vec<LogicalPlan>,
        stats: &Statistics,
    ) -> Result<LogicalPlan> {
        let mut best_plan = None;
        let mut best_cost = f64::MAX;
        
        for plan in plans {
            let cost = self.estimate_plan_cost(&plan, stats)?;
            if cost < best_cost {
                best_cost = cost;
                best_plan = Some(plan);
            }
        }
        
        best_plan.ok_or(Error::NoValidPlan)
    }
}
```

### 5.2 动态规划版本（DP-based Join Reorder）

```rust
pub struct DPJoinReorder {
    cost_estimator: Arc<dyn CostEstimator>,
}

impl DPJoinReorder {
    pub fn optimize(&self, tables: Vec<TableInfo>) -> Result<JoinTree> {
        let n = tables.len();
        let mut dp = vec![vec![None; n]; n];
        
        for i in 0..n {
            dp[i][i] = Some(JoinTree::Leaf(tables[i].clone()));
        }
        
        for len in 2..=n {
            for i in 0..=(n - len) {
                let j = i + len - 1;
                for k in i..j {
                    if let (Some(left), Some(right)) = (&dp[i][k], &dp[k + 1][j]) {
                        let new_tree = JoinTree::Join {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        };
                        let new_cost = self.cost_estimator.estimate_join_tree(&new_tree);
                        
                        if dp[i][j].is_none() || new_cost < self.cost_estimator.estimate_join_tree(dp[i][j].as_ref().unwrap()) {
                            dp[i][j] = Some(new_tree);
                        }
                    }
                }
            }
        }
        
        dp[0][n - 1].clone().ok_or(Error::NoValidPlan)
    }
}
```

---

## 六、完整 CBO 架构

```rust
pub struct CostBasedOptimizer {
    statistics: Arc<dyn StatisticsCollector>,
    cost_estimator: Arc<dyn CostEstimator>,
    plan_enumerator: Arc<dyn PlanEnumerator>,
    rules: Vec<Arc<dyn OptimizerRule>>,
}

impl CostBasedOptimizer {
    pub fn optimize(&self, plan: &LogicalPlan) -> Result<LogicalPlan> {
        let mut current_plan = plan.clone();
        
        for rule in &self.rules {
            current_plan = rule.optimize(current_plan)?;
        }
        
        let alternative_plans = self.plan_enumerator.enumerate(current_plan)?;
        let best_plan = self.plan_enumerator.find_best_plan(
            alternative_plans,
            &self.statistics,
        )?;
        
        Ok(best_plan)
    }
}
```

---

## 七、与 RBO 对比

| 特性 | RBO（规则优化） | CBO（成本优化） |
|:-----|:----------------|:----------------|
| 依据 | 预定义规则 | 统计信息 + 成本模型 |
| 灵活性 | 固定 | 动态 |
| 适应性 | 差 | 好 |
| 复杂度 | 低 | 高 |
| 成熟度 | L2 | L4 |

---

*本文档由 TRAE (GLM-5.0) 创建*
