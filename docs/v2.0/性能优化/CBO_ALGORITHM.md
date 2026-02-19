# CBO 详细算法（含 Join DP 公式）

> 版本：v1.0
> 日期：2026-02-18
> 类型：工程可执行版本

---

## 一、基础成本公式

### 1.1 符号定义

| 符号 | 含义 |
|:-----|:-----|
| N(T) | 表 T 的行数 |
| V(T, c) | 列 c 的 distinct 值数量 |
| Sel(p) | 谓词 p 的选择率 |
| min(c), max(c) | 列 c 的最小/最大值 |

### 1.2 过滤选择率估算

#### 等值谓词

```
Sel(c = v) = 1 / V(T, c)
```

**过滤后行数**：
```
N' = N(T) × Sel
```

**示例**：
```
表 users: N = 10000
列 city: V = 100

Sel(city = 'Beijing') = 1/100 = 0.01
过滤后行数 = 10000 × 0.01 = 100
```

#### 范围谓词

假设均匀分布：
```
Sel(c > v) = (max(c) - v) / (max(c) - min(c))
Sel(c < v) = (v - min(c)) / (max(c) - min(c))
Sel(c BETWEEN a AND b) = (b - a) / (max(c) - min(c))
```

**示例**：
```
列 age: min=0, max=100
Sel(age > 60) = (100 - 60) / (100 - 0) = 0.4
```

#### AND/OR 组合

```
Sel(p1 AND p2) = Sel(p1) × Sel(p2)
Sel(p1 OR p2) = Sel(p1) + Sel(p2) - Sel(p1) × Sel(p2)
Sel(NOT p) = 1 - Sel(p)
```

---

## 二、Join 基数估算

### 2.1 等值 Join

```
N(A ⋈ B) = N(A) × N(B) / max(V(A,a), V(B,b))
```

**示例**：
```
表 orders: N = 10000, V(user_id) = 1000
表 users: N = 1000, V(id) = 1000

N(orders ⋈ users) = 10000 × 1000 / max(1000, 1000) = 10000
```

### 2.2 外连接

```
N(A LEFT JOIN B) = N(A)  // 左表行数不变
N(A RIGHT JOIN B) = N(B) // 右表行数不变
N(A FULL JOIN B) = N(A) + N(B) - N(A ⋈ B)
```

### 2.3 交叉连接

```
N(A × B) = N(A) × N(B)
```

---

## 三、成本模型

### 3.1 简化 CPU 成本

```
Cost(HashJoin) = N(A) + N(B) + build_cost
Cost(NestedLoop) = N(A) × N(B)
Cost(SortMergeJoin) = N(A) × log(N(A)) + N(B) × log(N(B))
```

### 3.2 I/O 成本

```
Cost(Scan) = N(T) / page_size
Cost(IndexScan) = log(index_pages) + matching_pages
```

### 3.3 总代价

```
TotalCost = α × CPU + β × IO + γ × Memory
```

**推荐权重**：
```
α = 1.0   // CPU 权重
β = 10.0  // IO 权重（IO 更昂贵）
γ = 0.5   // Memory 权重
```

---

## 四、Join Reorder — DP 算法

### 4.1 Selinger 风格 DP

设表集合 S：
```
dp[S] = 最优子计划
```

### 4.2 算法实现

```rust
pub struct JoinReorderOptimizer {
    stats: HashMap<String, TableStats>,
    cost_model: CostModel,
}

impl JoinReorderOptimizer {
    pub fn optimize(&self, tables: Vec<String>, join_conditions: Vec<JoinCondition>) -> Result<JoinTree> {
        let n = tables.len();
        
        if n > 10 {
            return Err(Error::TooManyTables(n));
        }
        
        let mut dp: HashMap<HashSet<String>, JoinPlan> = HashMap::new();
        
        for table in &tables {
            let mut set = HashSet::new();
            set.insert(table.clone());
            dp.insert(set, JoinPlan::Scan(table.clone()));
        }
        
        for size in 2..=n {
            for subset in self.subsets_of_size(&tables, size) {
                let mut best_plan = None;
                let mut best_cost = f64::MAX;
                
                for partition in self.partitions(&subset) {
                    if let (Some(left), Some(right)) = (dp.get(&partition.left), dp.get(&partition.right)) {
                        if self.is_connected(&partition.left, &partition.right, &join_conditions) {
                            let join_plan = JoinPlan::Join {
                                left: Box::new(left.clone()),
                                right: Box::new(right.clone()),
                            };
                            let cost = self.estimate_cost(&join_plan);
                            
                            if cost < best_cost {
                                best_cost = cost;
                                best_plan = Some(join_plan);
                            }
                        }
                    }
                }
                
                if let Some(plan) = best_plan {
                    dp.insert(subset, plan);
                }
            }
        }
        
        let all_tables: HashSet<String> = tables.into_iter().collect();
        dp.get(&all_tables).cloned().ok_or(Error::NoValidPlan)
    }
    
    fn estimate_cost(&self, plan: &JoinPlan) -> f64 {
        match plan {
            JoinPlan::Scan(table) => {
                let stats = self.stats.get(table).unwrap();
                stats.row_count as f64
            }
            JoinPlan::Join { left, right } => {
                let left_cost = self.estimate_cost(left);
                let right_cost = self.estimate_cost(right);
                let left_rows = self.estimate_rows(left);
                let right_rows = self.estimate_rows(right);
                
                left_cost + right_cost + left_rows + right_rows
            }
        }
    }
    
    fn estimate_rows(&self, plan: &JoinPlan) -> f64 {
        match plan {
            JoinPlan::Scan(table) => {
                self.stats.get(table).unwrap().row_count as f64
            }
            JoinPlan::Join { left, right } => {
                let left_rows = self.estimate_rows(left);
                let right_rows = self.estimate_rows(right);
                left_rows * right_rows / 1000.0
            }
        }
    }
}
```

### 4.3 时间复杂度

```
O(n² × 2ⁿ)
```

**适合**：≤ 10 张表

| 表数 | 子问题数 | 计算量 |
|:-----|:----------|:-------|
| 5 | 32 | ~800 |
| 10 | 1024 | ~100K |
| 15 | 32768 | ~7M |
| 20 | 1M | ~400M |

---

## 五、完整 CBO 实现

### 5.1 统计信息收集

```rust
pub struct StatisticsCollector {
    storage: Arc<dyn StorageEngine>,
}

impl StatisticsCollector {
    pub fn collect(&self, table: &str) -> Result<TableStatistics> {
        let rows = self.storage.scan(table)?;
        
        let row_count = rows.len();
        let mut column_stats = HashMap::new();
        
        for column in self.storage.schema(table)?.columns() {
            let mut values = HashSet::new();
            let mut null_count = 0;
            let mut min = None;
            let mut max = None;
            
            for row in &rows {
                if let Some(value) = row.get(&column.name) {
                    values.insert(value.clone());
                    
                    match &min {
                        None => min = Some(value.clone()),
                        Some(m) if value < m => min = Some(value.clone()),
                        _ => {}
                    }
                    
                    match &max {
                        None => max = Some(value.clone()),
                        Some(m) if value > m => max = Some(value.clone()),
                        _ => {}
                    }
                } else {
                    null_count += 1;
                }
            }
            
            column_stats.insert(column.name.clone(), ColumnStatistics {
                distinct_count: values.len(),
                null_count,
                min_value: min,
                max_value: max,
            });
        }
        
        Ok(TableStatistics {
            row_count,
            column_stats,
        })
    }
}
```

### 5.2 成本估算器

```rust
pub struct CostEstimator {
    weights: CostWeights,
}

impl CostEstimator {
    pub fn estimate_scan(&self, stats: &TableStatistics) -> Cost {
        Cost {
            cpu: stats.row_count as f64 * 0.001,
            io: stats.row_count as f64 / 100.0,
            memory: 0.0,
        }
    }
    
    pub fn estimate_filter(&self, input_cost: &Cost, selectivity: f64) -> Cost {
        Cost {
            cpu: input_cost.cpu * selectivity,
            io: 0.0,
            memory: 0.0,
        }
    }
    
    pub fn estimate_hash_join(
        &self,
        left_stats: &TableStatistics,
        right_stats: &TableStatistics,
    ) -> Cost {
        Cost {
            cpu: (left_stats.row_count + right_stats.row_count) as f64 * 0.01,
            io: 0.0,
            memory: left_stats.row_count.min(right_stats.row_count) as f64 * 0.001,
        }
    }
    
    pub fn estimate_nested_loop(
        &self,
        left_stats: &TableStatistics,
        right_stats: &TableStatistics,
    ) -> Cost {
        Cost {
            cpu: (left_stats.row_count * right_stats.row_count) as f64 * 0.001,
            io: 0.0,
            memory: 0.0,
        }
    }
}
```

---

## 六、使用示例

```rust
fn main() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    let collector = StatisticsCollector::new(storage.clone());
    let cost_estimator = CostEstimator::default();
    let join_reorder = JoinReorderOptimizer::new(cost_estimator);
    
    let stats = collector.collect("users")?;
    
    let tables = vec!["users".to_string(), "orders".to_string(), "products".to_string()];
    let conditions = vec![
        JoinCondition::new("users.id", "orders.user_id"),
        JoinCondition::new("orders.product_id", "products.id"),
    ];
    
    let optimal_plan = join_reorder.optimize(tables, conditions)?;
    
    println!("Optimal join order: {:?}", optimal_plan);
    
    Ok(())
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
