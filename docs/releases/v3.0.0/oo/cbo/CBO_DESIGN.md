# CBO 查询优化器设计

> Cost-Based Optimizer 架构与代价模型

## 1. CBO 架构总览

### 1.1 优化器结构

```
┌─────────────────────────────────────────────────────────────────┐
│                    CBO Optimizer                                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │ Stats        │  │ Cost Model    │  │ Plan         │        │
│  │ Collector    │  │               │  │ Enumerator    │        │
│  │              │  │ CPU Cost      │  │              │        │
│  │ - cardinality│  │ IO Cost       │  │ - join order  │        │
│  │ - ndv       │  │ Memory Cost   │  │ - access path│        │
│  │ - histogram │  │ Network Cost  │  │ - phys op    │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│           ↓               ↓                ↓                     │
│  ┌─────────────────────────────────────────────────────┐      │
│  │              Plan Cache (optional)                   │      │
│  └─────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 关键文件

| 文件 | 作用 |
|------|------|
| `crates/optimizer/src/cost.rs` | 代价模型定义 |
| `crates/optimizer/src/graph_cost.rs` | 图代价计算 |
| `crates/optimizer/src/stats.rs` | 统计信息 |
| `crates/optimizer/src/stats_provider.rs` | 统计提供者 |
| `crates/optimizer/src/rules.rs` | 优化规则 |
| `crates/planner/src/optimizer.rs` | 优化器入口 |

## 2. 代价模型

### 2.1 代价要素

```rust
pub struct Cost {
    pub cpu_cost: f64,      // CPU 处理代价
    pub io_cost: f64,       // 磁盘 IO 代价
    pub memory_cost: f64,   // 内存使用代价
    pub network_cost: f64,   // 网络传输代价 (分布式)
}
```

### 2.2 代价计算公式

```
TotalCost = Σ(cpu_cost_i) + Σ(io_cost_i * page_fault_rate) + memory_cost + network_cost

其中:
- cpu_cost_i = rows_processed * cpu_cost_per_row
- io_cost_i = pages_read * read_latency
```

### 2.3 访问路径代价

| 访问路径 | 代价计算 | 适用场景 |
|----------|----------|----------|
| 全表扫描 | pages * read_latency | 小表，低选择性 |
| 索引唯一查询 | 1 page | 高选择性 |
| 索引范围查询 | index_pages + data_pages * selectivity | 中等选择性 |
| 索引全扫描 | index_pages + all_data_pages | 全索引覆盖 |

## 3. 统计信息

### 3.1 统计信息结构

```rust
pub struct TableStats {
    pub row_count: u64,           // 行数
    pub page_count: u64,          // 页数
    pub data_size: u64,           // 数据大小
    pub hidden_files: Vec<u64>,   // 隐藏文件
}

pub struct ColumnStats {
    pub ndv: u64,                 // 不同值数量
    pub null_count: u64,          // NULL 数量
    pub min_value: Value,          // 最小值
    pub max_value: Value,          // 最大值
    pub histogram: Vec<HistogramBucket>, // 直方图
}
```

### 3.2 选择性计算

```
selectivity = ndv / total_rows  (for equality predicate)
selectivity = (high - low) / (max - min)  (for range predicate)
```

## 4. Join Ordering

### 4.1 Join Order 算法

```rust
// 动态规划算法 - 最优 Join Order
fn compute_join_order(tables: &[Table]) -> JoinPlan {
    let n = tables.len();
    let mut dp = vec![HashMap::new(); n + 1];

    // 单表最优访问路径
    for (i, table) in tables.iter().enumerate() {
        dp[1].insert(bitmask(i), compute_best_access_path(table));
    }

    // 逐步增加连接表数量
    for size in 2..=n {
        for subset in subsets_of_size(size) {
            let (left, right) = split_subset(subset);
            let left_plan = dp[left.len()].get(&left).unwrap();
            let right_plan = dp[right.len()].get(&right).unwrap();
            let join_cost = compute_join_cost(left_plan, right_plan);
            // 选择最小代价的分割方式
        }
    }
}
```

### 4.2 Join 策略选择

| 策略 | 适用场景 | 代价估算 |
|------|----------|----------|
| Nested Loop Join | 小表 join 大表，有索引 | inner_rows * index_lookup_cost |
| Hash Join | 等值 join，无索引，等值大小相近 | build_input + probe_input |
| Sort-Merge Join | 已排序数据，等值 join | sort_cost + merge_cost |

## 5. 物理算子选择

### 5.1 算子选择规则

```
IF (selection_cardinality < threshold) THEN
    use INDEX_SCAN
ELSE IF (requires_aggregation) THEN
    use HASH_AGGREGATE or SORT_AGGREGATE
ELSE IF (requires_order) THEN
    use SORT + LIMIT or INDEX_SCAN
ELSE
    use SEQ_SCAN
```

### 5.2 聚合策略

| 聚合类型 | 策略 | 代价 |
|----------|------|------|
| COUNT, SUM, AVG | Streaming Aggregate | O(n), 低内存 |
| COUNT DISTINCT | Hash Aggregate | O(n), 高内存 |
| ORDER BY | Sort Aggregate | O(n log n) |

## 6. CBO 测试要点

### 6.1 覆盖率目标

- [ ] 代价模型计算正确性
- [ ] Join Order 动态规划正确性
- [ ] 统计信息收集完整性
- [ ] 物理算子选择正确性
- [ ] 计划缓存正确性

### 6.2 关键测试用例

```sql
-- T1: Join Order 测试
SELECT * FROM t1, t2, t3 WHERE t1.id = t2.id AND t2.id = t3.id;
-- 验证: 小表先 join (t1.t2) vs 大表先 join (t2.t3)

-- T2: 索引选择测试
SELECT * FROM t WHERE a > 10 AND a < 100;
-- 验证: 索引 vs 全表扫描的选择

-- T3: 多表 Join Order
SELECT * FROM t1, t2, t3, t4 WHERE t1.id = t2.id AND t2.id = t3.id AND t3.id = t4.id;
-- 验证: 笛卡尔积爆炸的避免
```

## 7. CBO 性能评估

### 7.1 优化时间预算

| 查询复杂度 | 最大优化时间 |
|------------|--------------|
| 1-3 表 | 10ms |
| 4-6 表 | 50ms |
| 7+ 表 | 200ms |

### 7.2 优化效果评估

```
优化效果 = (OriginalCost - OptimizedCost) / OriginalCost * 100%

期望: > 20% 提升视为有效优化
```
