# CBO Join Ordering 算法

> Join Order 优化算法 - 动态规划、贪心算法、遗传算法

## 1. Join Ordering 问题定义

```
┌─────────────────────────────────────────────────────────────┐
│                   Join Order 问题                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  给定 N 个表的连接查询:                                     │
│                                                              │
│    SELECT * FROM t1, t2, t3, ..., tN                      │
│    WHERE t1.x = t2.x AND t2.y = t3.y AND ...               │
│                                                              │
│  问题: 如何选择最优的连接顺序使得总代价最小?                  │
│                                                              │
│  搜索空间: (2N)! / (N+1)!  (Catalan number)              │
│  ─────────────────────────────────                         │
│  N=3: 3 种连接顺序                                        │
│  N=5: 42 种连接顺序                                       │
│  N=7: 792 种连接顺序                                       │
│  N=10: 176,226 种连接顺序                                  │
│                                                              │
│  这是 NP-hard 问题，需要启发式算法                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 2. 动态规划算法 (DP)

### 2.1 Selinger 算法 (最优解)

```
┌─────────────────────────────────────────────────────────────┐
│              Selinger Style Join Ordering (DP)               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  核心思想:                                                  │
│  ─────────                                                  │
│  对于每一种表的组合，找最优的连接计划                        │
│  利用最优子结构，避免重复计算                               │
│                                                              │
│  状态定义:                                                  │
│  ─────────                                                  │
│  DP[S] = 在表集合 S 上的最优连接计划                        │
│                                                              │
│  递推公式:                                                  │
│  ─────────                                                  │
│  DP[S] = min_{A ⊂ S, B = S\A} (                           │
│            cost(join(Dp[A], DP[B]))                        │
│          )                                                  │
│                                                              │
│  时间复杂度: O(3^N)  // 每种分割尝试                         │
│  空间复杂度: O(2^N)  // 存储每个子集的最优计划               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 DP 算法流程

```
Query: t1 ⋈ t2 ⋈ t3 ⋈ t4

初始化: DP[{t1}], DP[{t2}], DP[{t3}], DP[{t4}]
        (单表的最优访问路径: 全表扫描 或 索引扫描)

Step 1: 计算大小为 2 的集合
───────────────────────────────────────────────────────────────
DP[{t1,t2}] = min(
    join(DP[{t1}], DP[{t2}]),   // t1 ⋈ t2
    join(DP[{t2}], DP[{t1}])    // t2 ⋈ t1 (如果 join 不是对称的)
)
DP[{t1,t3}] = min(...)
DP[{t1,t4}] = min(...)
DP[{t2,t3}] = min(...)
DP[{t2,t4}] = min(...)
DP[{t3,t4}] = min(...)

Step 2: 计算大小为 3 的集合
───────────────────────────────────────────────────────────────
DP[{t1,t2,t3}] = min(
    join(DP[{t1}], DP[{t2,t3}]),   // (t1 ⋈ (t2 ⋈ t3))
    join(DP[{t2}], DP[{t1,t3}]),   // (t2 ⋈ (t1 ⋈ t3))
    join(DP[{t3}], DP[{t1,t2}])    // (t3 ⋈ (t1 ⋈ t2))
)

Step 3: 计算大小为 4 的集合
───────────────────────────────────────────────────────────────
DP[{t1,t2,t3,t4}] = min(
    join(DP[{t1}], DP[{t2,t3,t4}]),
    join(DP[{t2}], DP[{t1,t3,t4}]),
    join(DP[{t3}], DP[{t1,t2,t4}]),
    join(DP[{t4}], DP[{t1,t2,t3}]),
    join(DP[{t1,t2}], DP[{t3,t4}]),  // 左深树
    join(DP[{t1,t3}], DP[{t2,t4}]),  // bushy tree
    join(DP[{t1,t4}], DP[{t2,t3}])
)

结果: DP[{t1,t2,t3,t4}] 就是最优 join order
```

### 2.3 DP 实现代码

```rust
pub fn compute_optimal_join_order(
    tables: &[TableRef],
    join_conditions: &[(TableRef, TableRef, Expr)],
    stats: &StatsCache,
    cost_config: &CostConfig,
) -> JoinPlan {
    let n = tables.len();
    let mut dp: HashMap<BitSet, JoinPlan> = HashMap::new();

    // 初始化: 单表的最优访问路径
    for (i, table) in tables.iter().enumerate() {
        let access_path = find_best_access_path(table, stats, cost_config);
        let mut plan = JoinPlan::new(table.clone());
        plan.set_access_path(access_path);
        dp.insert(bitmask(i), plan);
    }

    // 逐步增加连接表数量
    for size in 2..=n {
        for subset in subsets_of_size(size, n) {
            let complement = complement(&subset, n);

            // 尝试所有分割方式
            let mut best_cost = f64::MAX;
            let mut best_plan = None;

            // 分割成 A 和 B
            for split in all_splits(&subset) {
                let a = &dp[&split];
                let b = &dp[&complement_without(&subset, &split)];

                // 尝试 A ⋈ B 和 B ⋈ A
                for (left, right) in [(a.clone(), b.clone()), (b.clone(), a.clone())] {
                    let join_predicates = find_join_predicates(&left, &right, join_conditions);

                    if join_predicates.is_empty() {
                        continue;  // 这两个子集不能直接连接
                    }

                    let cost = compute_join_cost(&left, &right, &join_predicates, cost_config);

                    if cost < best_cost {
                        best_cost = cost;
                        best_plan = Some(JoinPlan::binary(left, right, join_predicates, cost));
                    }
                }
            }

            if let Some(plan) = best_plan {
                dp.insert(subset, plan);
            }
        }
    }

    dp[&bitmask_all(n)].clone()
}
```

## 3. 贪心算法 (Greedy)

### 3.1 贪心 Join Ordering

```
┌─────────────────────────────────────────────────────────────┐
│                  贪心 Join Ordering                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  核心思想:                                                   │
│  ─────────                                                   │
│  每一步选择代价最小的两个子集进行连接                         │
│  快速得到"还算不错"的解，但可能不是最优                     │
│                                                              │
│  算法步骤:                                                   │
│  ─────────                                                   │
│  1. 初始化: 每个表作为一个集合                              │
│  2. 重复直到只剩一个集合:                                   │
│     a. 计算每对集合的连接代价                               │
│     b. 选择代价最小的 pair 进行连接                         │
│     c. 合并为新集合                                        │
│                                                              │
│  时间复杂度: O(N^3)  // 每次 O(N^2) 搜索，共 O(N) 步       │
│  空间复杂度: O(N)                                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 贪心算法流程图

```
初始状态: [{t1}, {t2}, {t3}, {t4}]

Step 1: 计算所有 pair 的连接代价
───────────────────────────────────────────────────────────────
  cost(t1⋈t2) = 100   ← 最小!
  cost(t1⋈t3) = 200
  cost(t1⋈t4) = 300
  cost(t2⋈t3) = 150
  cost(t2⋈t4) = 250
  cost(t3⋈t4) = 400

  选择: t1 ⋈ t2
  合并: [{t1,t2}, {t3}, {t4}]

Step 2: 计算所有 pair 的连接代价
───────────────────────────────────────────────────────────────
  cost((t1,t2)⋈t3) = 180
  cost((t1,t2)⋈t4) = 250   ← 最小!
  cost(t3⋈t4) = 400

  选择: (t1,t2) ⋈ t4
  合并: [{t1,t2,t4}, {t3}]

Step 3: 最后一次连接
───────────────────────────────────────────────────────────────
  cost((t1,t2,t4)⋈t3) = 300

  最终结果: ((t1 ⋈ t2) ⋈ t4) ⋈ t3
```

### 3.3 贪心算法实现

```rust
pub fn greedy_join_ordering(
    tables: &[TableRef],
    join_conditions: &[(TableRef, TableRef, Expr)],
    stats: &StatsCache,
    cost_config: &CostConfig,
) -> JoinPlan {
    let mut sets: Vec<JoinSet> = tables
        .iter()
        .enumerate()
        .map(|(i, t)| JoinSet {
            tables: bitmask(1 << i),
            plan: build_single_table_plan(t, stats, cost_config),
        })
        .collect();

    while sets.len() > 1 {
        let mut best_cost = f64::MAX;
        let mut best_i = 0;
        let mut best_j = 0;

        // 找代价最小的 pair
        for i in 0..sets.len() {
            for j in (i + 1)..sets.len() {
                let cost = compute_pair_join_cost(&sets[i], &sets[j], join_conditions, cost_config);
                if cost < best_cost {
                    best_cost = cost;
                    best_i = i;
                    best_j = j;
                }
            }
        }

        // 合并 pair
        let left = sets.remove(best_j);
        let right = sets.remove(best_i);
        let join_predicates = find_join_predicates(&left, &right, join_conditions);
        let new_plan = JoinPlan::binary(left.plan, right.plan, join_predicates, best_cost);
        let new_set = JoinSet {
            tables: left.tables | right.tables,
            plan: new_plan,
        };
        sets.push(new_set);
    }

    sets[0].plan.clone()
}
```

## 4. 左深树 vs 浓密树

### 4.1 树形状对比

```
┌─────────────────────────────────────────────────────────────┐
│                 Join Tree 形状                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  左深树 (Left-Deep Tree)           浓密树 (Bushy Tree)      │
│  ──────────────────────────────   ──────────────────────── │
│                                                              │
│           ⋈                              ⋈                   │
│          /                              / \                  │
│         ⋈                            ⋈   ⋈                 │
│        / \                          / \ / \                 │
│       ⋈   ⋈                        t1 t2 t3 t4             │
│      / \                                                       │
│    t1   t2                                                        │
│                                                              │
│  深度: N                        深度: log2(N)                 │
│  最大并发: 1                  最大并发: N/2                    │
│                                                              │
│  优点: 易于流水线执行              优点: 更好的并行度         │
│  缺点: 并行度低                   缺点: 搜索空间大            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 树形状选择策略

```rust
pub enum TreeShape {
    LeftDeep,
    Bushy,
    Both,
}

pub fn should_use_bushy_tree(
    num_tables: usize,
    available_memory: u64,
    parallelism: usize,
) -> bool {
    // 浓密树的条件:
    // 1. 表数量较多 (> 6)
    // 2. 有足够内存支持并行
    // 3. 系统支持并行执行
    num_tables > 6 && available_memory > 1_GB && parallelism > 2
}
```

## 5. Bushy DP 算法

### 5.1 Bushy DP 状态机

```
┌─────────────────────────────────────────────────────────────┐
│                   Bushy DP 算法                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  与左深 DP 的区别:                                          │
│  - 左深 DP: 只考虑分割成 (单表, 多表)                       │
│  - Bushy DP: 分割成 (多表, 多表)                           │
│                                                              │
│  搜索空间: 2^N × 2^N = 4^N (更大)                         │
│  但可以通过剪枝优化                                          │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  DP[S] = min_{A ⊂ S, B = S\A} (                   │   │
│  │            cost(join(DP[A], DP[B]))                 │   │
│  │          )                                         │   │
│  │                                                     │   │
│  │  其中 A 和 B 都可以是任意大小的子集                   │   │
│  │  而不仅仅是 (1, N-1)                              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 Bushy DP 优化: 分区剪枝

```rust
pub fn bushy_dp_with_partitioning(
    tables: &[TableRef],
    join_graph: &JoinGraph,
    stats: &StatsCache,
) -> JoinPlan {
    // 1. 根据连接谓词构建连接图
    let connected_components = find_connected_components(tables, join_graph);

    // 2. 分别优化每个连通分量
    let mut partial_plans = Vec::new();
    for component in connected_components {
        if component.len() == 1 {
            partial_plans.push(build_single_table_plan(&component[0]));
        } else {
            // 只在这个连通分量内做 DP
            partial_plans.push(bushy_dp(&component, stats));
        }
    }

    // 3. 交叉连接不同连通分量的结果 (如果有多个)
    if partial_plans.len() > 1 {
        cross_join_all(partial_plans)
    } else {
        partial_plans[0].clone()
    }
}
```

## 6. 遗传算法 (Genetic Algorithm)

### 6.1 GA Join Ordering

```
┌─────────────────────────────────────────────────────────────┐
│                  遗传算法 Join Ordering                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  编码方式: 表的排列顺序                                      │
│  例如: [t3, t1, t4, t2] → 连接顺序为 t3 ⋈ t1 ⋈ t4 ⋈ t2  │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  个体编码: permutation of [0, 1, 2, ..., N-1]       │   │
│  │  适应度函数: fitness = 1 / join_cost               │   │
│  │  选择算子: tournament selection                      │   │
│  │  交叉算子: PMX, OX                                  │   │
│  │  变异算子: swap mutation                           │   │
│  │  终止条件: 固定代数 / 收敛                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                              │
│  优点: 可以处理大规模表 (> 15)                              │
│  缺点: 不保证最优解                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 GA 算法流程

```
初始化种群 (100 个个体)
    ↓
┌─────────────────────────────────────────────────────────────┐
│                    迭代进化                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                                                     │   │
│  │  for generation in 1..MAX_GENERATIONS:            │   │
│  │                                                     │   │
│  │    1. 评估: 计算每个个体的代价                      │   │
│  │                                                     │   │
│  │    2. 选择: tournament selection (k=3)             │   │
│  │                                                     │   │
│  │    3. 交叉: PMX 产生两个后代                       │   │
│  │                                                     │   │
│  │    4. 变异: swap mutation (5% 概率)               │   │
│  │                                                     │   │
│  │    5. 替换: elitism 保留最优个体                   │   │
│  │                                                     │   │
│  │    6. 收敛检测: 最优解连续 N 代无变化               │   │
│  │                                                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
    ↓
返回最优个体
```

### 6.3 GA 实现代码

```rust
pub struct GeneticJoinOptimizer {
    population_size: usize,
    generations: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    tournament_size: usize,
}

impl GeneticJoinOptimizer {
    pub fn optimize(
        &self,
        tables: &[TableRef],
        join_conditions: &[(TableRef, TableRef, Expr)],
        stats: &StatsCache,
    ) -> JoinPlan {
        // 初始化种群
        let mut population = self.init_population(tables.len());

        // 评估初始种群
        let mut costs = population
            .iter()
            .map(|p| self.evaluate(p, tables, join_conditions, stats))
            .collect::<Vec<_>>();

        for gen in 0..self.generations {
            let mut new_population = Vec::with_capacity(self.population_size);

            // Elitism: 保留最优个体
            let best_idx = costs.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
            new_population.push(population[best_idx].clone());

            // 生成新个体
            while new_population.len() < self.population_size {
                // Tournament selection
                let p1 = self.tournament_select(&population, &costs);
                let p2 = self.tournament_select(&population, &costs);

                // Crossover
                let (c1, c2) = if rand::random::<f64>() < self.crossover_rate {
                    self.crossover(&p1, &p2)
                } else {
                    (p1.clone(), p2.clone())
                };

                // Mutation
                let c1 = if rand::random::<f64>() < self.mutation_rate {
                    self.mutate(c1)
                } else {
                    c1
                };
                let c2 = if rand::random::<f64>() < self.mutation_rate {
                    self.mutate(c2)
                } else {
                    c2
                };

                new_population.push(c1);
                if new_population.len() < self.population_size {
                    new_population.push(c2);
                }
            }

            population = new_population;
            costs = population
                .iter()
                .map(|p| self.evaluate(p, tables, join_conditions, stats))
                .collect::<Vec<_>>();
        }

        // 返回最优解
        let best_idx = costs.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        self.decode_to_plan(&population[best_idx], tables, join_conditions, stats)
    }

    fn crossover(&self, p1: &[usize], p2: &[usize]) -> (Vec<usize>, Vec<usize>) {
        // PMX (Partially Mapped Crossover)
        let n = p1.len();
        let (i, j) = {
            let mut rng = rand::thread_rng();
            let i = rng.gen_range(0..n);
            let j = rng.gen_range(i..n);
            (i, j)
        };

        let mut c1 = vec![0usize; n];
        let mut c2 = vec![0usize; n];

        // Copy middle segment
        c1[i..=j].copy_from_slice(&p1[i..=j]);
        c2[i..=j].copy_from_slice(&p2[i..=j]);

        // Fill remaining positions
        self.fill_remaining(&mut c1, p2, i, j);
        self.fill_remaining(&mut c2, p1, i, j);

        (c1, c2)
    }
}
```

## 7. 互联外小表优化

### 7.1 互联外小表 (Small Outer Table)

```
┌─────────────────────────────────────────────────────────────┐
│                  互联外小表优化                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  核心思想:                                                  │
│  ─────────                                                  │
│  小表应该作为外表(outer)，大表作为内表(inner)               │
│  这样内表只需要扫描一次                                    │
│                                                              │
│  场景:                                                     │
│  ─────                                                     │
│  SELECT * FROM large_table L                               │
│  JOIN small_table S ON L.id = S.id                         │
│                                                              │
│  正确顺序: L ⋈ S (L 是 inner, S 是 outer)                 │
│  错误顺序: S ⋈ L (S 是 inner, L 是 outer) ← 极慢!        │
│                                                              │
│  原因:                                                     │
│  ─────                                                     │
│  NLJoin: outer_rows × inner_scan_cost                     │
│  如果 outer=大表, inner=小表: 1000000 × 1 = 1000000       │
│  如果 outer=小表, inner=大表: 100 × 10000 = 1000000       │
│  (一样?)                                                   │
│                                                              │
│  但如果考虑索引:                                            │
│  NLJoin with index: outer_rows × index_lookup_cost        │
│  小表做 outer → outer_rows × 小 → 代价小                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 基于代价的 Outer/Inner 选择

```rust
pub fn choose_join_order_with_outer(
    left: &TableRef,
    right: &TableRef,
    join_condition: &Expr,
    stats: &StatsCache,
    cost_config: &CostConfig,
) -> (JoinPlan, JoinSide) {
    let left_rows = stats.row_count(left);
    let right_rows = stats.row_count(right);

    // 方案 1: left ⋈ right
    let cost1 = compute_join_cost(
        left, right, join_condition,
        /* left_is_outer = */ true,
        cost_config
    );

    // 方案 2: right ⋈ left
    let cost2 = compute_join_cost(
        right, left, join_condition,
        /* left_is_outer = */ false,
        cost_config
    );

    if cost1 < cost2 {
        (build_join_plan(left, right, join_condition, JoinSide::Left), JoinSide::Left)
    } else {
        (build_join_plan(right, left, join_condition, JoinSide::Right), JoinSide::Right)
    }
}
```

## 8. Join Reorder 规则

### 8.1 转换规则

```rust
// Join 交换律
(A ⋈ B) = (B ⋈ A)

// Join 结合律
(A ⋈ B) ⋈ C = A ⋈ (B ⋈ C)

// 笛卡尔积 + 条件 → 实际连接
A × B WHERE A.x = B.x = A ⋈ B

// 关联规则重写
(A ⋈ B) WHERE condition = A ⋈ (B WHERE condition)  // 如果 condition 只涉及 B
```

### 8.2 Join 消除

```rust
pub fn eliminate_redundant_join(
    tables: &[TableRef],
    join_conditions: &[(TableRef, TableRef, Expr)],
) -> (Vec<TableRef>, Vec<JoinCondition>) {
    // 构建连接图
    let mut graph = Graph::new();
    for t in tables {
        graph.add_node(t);
    }
    for (t1, t2, cond) in join_conditions {
        graph.add_edge(t1, t2, cond);
    }

    // 找出桥接表 (只用于连接其他两个表，本身条件无独立用途)
    let mut result_tables = Vec::new();
    let mut result_joins = Vec::new();

    for table in tables {
        if !is_bridge_table(table, &graph) {
            result_tables.push(table.clone());
        }
    }

    // 保留非桥接表的 join
    for join in join_conditions {
        if result_tables.contains(&join.0) && result_tables.contains(&join.1) {
            result_joins.push(join.clone());
        }
    }

    (result_tables, result_joins)
}
```

## 9. 关键测试用例

### 9.1 DP 正确性测试

```rust
#[test]
fn test_dp_finds_optimal_for_3_tables() {
    let tables = vec![t1, t2, t3];
    let joins = vec![
        (t1.clone(), t2.clone(), t1.x.eq(t2.x)),
        (t2.clone(), t3.clone(), t2.y.eq(t3.y)),
    ];

    let plan = compute_optimal_join_order(&tables, &joins, &stats, &config);

    // 验证是最优的
    let dp_cost = plan.cost;
    let enumerated_cost = enumerate_all_join_orders(&tables, &joins, &stats)
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    assert!((dp_cost - enumerated_cost).abs() < 0.001);
}
```

### 9.2 小表先连接测试

```rust
#[test]
fn test_small_table_first() {
    let large = Table::new("large", 1_000_000 rows);
    let medium = Table::new("medium", 10_000 rows);
    let small = Table::new("small", 100 rows);

    let plan = compute_optimal_join_order(
        &[large.clone(), medium.clone(), small.clone()],
        &[
            (large.clone(), medium.clone(), ...),
            (medium.clone(), small.clone(), ...),
        ],
        &stats,
        &config,
    );

    // small 应该在前面
    assert!(is_joined_early(&plan, &small));
}
```

### 9.3 贪心 vs DP 对比测试

```rust
#[test]
fn test_greedy_vs_dp() {
    let tables = (0..10).map(|i| Table::new(format!("t{}", i), 1000)).collect::<Vec<_>>();
    let joins = generate_random_joins(&tables, 15);

    let dp_plan = dp_join_ordering(&tables, &joins, &stats, &config);
    let greedy_plan = greedy_join_ordering(&tables, &joins, &stats, &config);

    // DP 应该不差于贪心
    assert!(dp_plan.cost <= greedy_plan.cost * 1.1);  // 允许 10% 误差
}
```

### 9.4 遗传算法收敛测试

```rust
#[test]
fn test_ga_convergence() {
    let tables = (0..15).map(|i| Table::new(format!("t{}", i), rand())).collect::<Vec<_>>();
    let joins = generate_random_joins(&tables, 20);

    let ga_cost = genetic_join_ordering(&tables, &joins, &stats, &config);
    let dp_cost = dp_join_ordering(&tables, &joins, &stats, &config);

    // GA 的解应该在 DP 解的 105% 以内
    assert!(ga_cost <= dp_cost * 1.05);
}
```

## 10. 覆盖率差距

| 场景 | 当前覆盖 | 目标覆盖 | 差距 |
|------|---------|---------|------|
| DP 算法 (Selinger) | 80% | 95% | 15% |
| 贪心算法 | 75% | 90% | 15% |
| Bushy Tree DP | 60% | 90% | 30% |
| 遗传算法 | 50% | 85% | 35% |
| 小表先连接优化 | 85% | 95% | 10% |
| Join 消除 | 70% | 90% | 20% |
| 互联外小表选择 | 75% | 95% | 20% |
| 大规模表 (N>10) | 55% | 85% | 30% |

## 11. 相关文件

| 文件 | 说明 |
|------|------|
| `crates/optimizer/src/join_ordering.rs` | Join ordering 算法实现 |
| `crates/optimizer/src/cost.rs` | 代价计算 |
| `crates/optimizer/src/stats.rs` | 统计信息 |
| `crates/planner/src/optimizer.rs` | 优化器入口 |

---

*文档版本: v3.0.0*
*最后更新: 2026-05-11*