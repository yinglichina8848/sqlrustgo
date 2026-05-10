# JOIN 算法详解

> Hash Join, Nested Loop Join, Merge Join 算法实现与分析

## 1. JOIN 算法概述

```
┌─────────────────────────────────────────────────────────────┐
│                    JOIN 算法分类                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   Hash Join     │  │ Nested Loop     │  │ Merge Join  │ │
│  │                 │  │                 │  │             │ │
│  │ O(n + m)       │  │ O(n * m)       │  │ O(n log n)  │ │
│  │ 等值连接首选    │  │ 小表+索引      │  │ 已排序输入  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.1 SQLRustGo 支持的 JOIN 类型

| JOIN 类型 | 关键字 | 算法 | 状态 |
|----------|--------|------|------|
| INNER JOIN | JOIN, CROSS JOIN | Hash Join | ✅ |
| LEFT JOIN | LEFT OUTER JOIN | Hash Join | ✅ |
| RIGHT JOIN | RIGHT OUTER JOIN | Hash Join | ✅ |
| FULL JOIN | FULL OUTER JOIN | Hash Join | ✅ |
| NATURAL JOIN | NATURAL JOIN | Hash Join | ✅ |
| USING clause | USING (col) | Hash Join | ✅ |

## 2. Hash Join 算法

### 2.1 Hash Join 原理

```
┌─────────────────────────────────────────────────────────────┐
│                    Hash Join 执行原理                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  表 A (Build)              表 B (Probe)                     │
│  ┌─────────┐              ┌─────────┐                      │
│  │ id: 1  │              │ id: 1   │                      │
│  │ id: 2  │  ────►      │ id: 3   │                      │
│  │ id: 3  │              │ id: 5   │                      │
│  └─────────┘              └─────────┘                      │
│       │                         │                           │
│       ▼                         │                           │
│  ┌─────────────────┐             │                           │
│  │   Hash Table    │             │                           │
│  │  1 -> [row_a1] │             │                           │
│  │  2 -> [row_a2] │             │                           │
│  │  3 -> [row_a3] │             │                           │
│  └─────────────────┘             │                           │
│                                  ▼                           │
│                           ┌─────────────────┐                │
│                           │   Probe Phase   │                │
│                           │  1 -> match     │                │
│                           │  3 -> match     │                │
│                           └─────────────────┘                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Hash Join 时序图

```
SELECT * FROM orders o JOIN customers c ON o.customer_id = c.id
    │
    ▼
┌─────────────────────────────────────────────┐
│              Join Planner                    │
│  JoinPlan {                                  │
│    base_table: "orders",                    │
│    joins: [JoinStep {                       │
│      right_table: "customers",              │
│      on: JoinPredicate {                   │
│        left_col: "customer_id",            │
│        right_col: "id"                     │
│      }                                     │
│    }]                                      │
│  }                                         │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Physical Plan                   │
│  HashJoinExec {                              │
│    join_type: INNER,                        │
│    condition: orders.customer_id =          │
│              customers.id,                  │
│    left: SeqScan(orders),                  │
│    right: SeqScan(customers)               │
│  }                                         │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Build Phase                     │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 遍历左表 (orders)               │   │
│  │ 2. 对每行计算 join key 的 hash      │   │
│  │ 3. 插入 hash table                  │   │
│  │ 4. 处理 hash 冲突 (链地址法)        │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Probe Phase                     │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 遍历右表 (customers)            │   │
│  │ 2. 对每行计算 join key 的 hash      │   │
│  │ 3. 在 hash table 中查找匹配        │   │
│  │ 4. 输出匹配行                       │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 2.3 Hash Join 状态机

```
                  ┌──────────────────┐
                  │    INITIAL       │
                  └────────┬─────────┘
                           │ start hash join
                           ▼
                  ┌──────────────────┐
                  │   BUILD_START    │
                  └────────┬─────────┘
                           │ building hash table
                           ▼
                  ┌──────────────────┐
                  │   PROBE_START   │
                  └────────┬─────────┘
                           │ probing
                           ▼
                  ┌──────────────────┐
                  │  MATCH_FOUND     │
                  └────────┬─────────┘
                           │ emit row
                           ▼
                  ┌──────────────────┐
                  │   NO_MATCH      │
                  └────────┬─────────┘
                           │ continue probing
                           ▼
                  ┌──────────────────┐
                  │     DONE        │
                  └──────────────────┘
```

### 2.4 Hash Join 实现

```rust
// crates/executor/src/local_executor.rs
async fn try_build_hash_join(
    left: RecordBatch,
    right: RecordBatch,
    condition: &Expr,
    left_schema: &Schema,
    right_schema: &Schema,
) -> SqlResult<RecordBatch> {
    // 1. 创建 hash table
    let mut hash_table: HashMap<Value, Vec<Row>> = HashMap::new();

    // 2. Build phase: 遍历左表
    for left_row in left.rows() {
        let key = evaluate_join_key(left_row, &condition.left(), left_schema)?;
        let hash = compute_hash(&key);
        hash_table.entry(hash).or_insert_with(Vec::new).push(left_row.clone());
    }

    // 3. Probe phase: 遍历右表
    let mut results = Vec::new();
    for right_row in right.rows() {
        let key = evaluate_join_key(right_row, &condition.right(), right_schema)?;
        let hash = compute_hash(&key);

        if let Some(matches) = hash_table.get(&hash) {
            for left_row in matches {
                if compare_keys(left_row, &key, right_row, &condition)? {
                    results.push(concatenate_rows(left_row, right_row));
                }
            }
        }
    }

    Ok(RecordBatch::from_rows(results))
}
```

### 2.5 外连接处理

```rust
match join_type {
    JoinType::Inner => {
        // 只输出匹配行
        output_matches_only();
    }
    JoinType::Left => {
        // 左表所有行都输出，未匹配的右表列填 NULL
        output_with_null_fill();
    }
    JoinType::Right => {
        // 右表所有行都输出，未匹配的左表列填 NULL
        output_with_null_fill();
    }
    JoinType::Full => {
        // 左右表所有行都输出
        output_with_null_fill();
    }
    JoinType::Cross => {
        // 笛卡尔积
        output_cartesian_product();
    }
}
```

## 3. Nested Loop Join 算法

### 3.1 Nested Loop Join 原理

```
┌─────────────────────────────────────────────────────────────┐
│                Nested Loop Join 执行原理                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  for each row in outer_table:        // 外层循环              │
│      for each row in inner_table:   // 内层循环              │
│          if condition(row_outer, row_inner):               │
│              output row_outer + row_inner                   │
│                                                              │
│  ┌─────────┐      ┌─────────┐                              │
│  │ 表 A    │      │ 表 B    │                              │
│  │ n=1000  │  ×   │ m=1000  │  =  1,000,000 次比较       │
│  └─────────┘      └─────────┘                              │
│                                                              │
│  优化: 如果内表有索引 → Index Nested Loop Join              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Nested Loop Join 时序图

```
SELECT * FROM large_table l JOIN small_table s ON l.id = s.id
    │
    ▼
┌─────────────────────────────────────────────┐
│              Optimizer                      │
│  决策: 使用 Index Nested Loop Join          │
│  原因: small_table 有主键索引               │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Execution                       │
│  ┌─────────────────────────────────────┐   │
│  │  for row in large_table:           │   │
│  │      key = row.id                   │   │
│  │      probe_index(s.primary_key, key)│   │
│  │      if found:                       │   │
│  │          emit combined row          │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 3.3 Nested Loop vs Hash Join 选择

| 条件 | 选择 Nested Loop | 选择 Hash Join |
|------|-----------------|---------------|
| 表大小 | 小表 (< 1000 行) | 大表 |
| 连接条件 | 非等值 (>, <, BETWEEN) | 等值 (=) |
| 索引 | 内表有连接列索引 | 无合适索引 |
| 内存 | 有限 | 足够构建 hash table |
| 数据分布 | 有序 | 无序 |

## 4. Merge Join 算法

### 4.1 Merge Join 原理

```
┌─────────────────────────────────────────────────────────────┐
│                    Merge Join 执行原理                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  表 A (已排序)         表 B (已排序)                        │
│  ┌─────────┐           ┌─────────┐                         │
│  │ 1       │           │ 2       │                         │
│  │ 3       │           │ 3       │                         │
│  │ 5       │           │ 5       │                         │
│  │ 7       │           │ 7       │                         │
│  └─────────┘           └─────────┘                         │
│       │                      │                             │
│       ▼                      ▼                             │
│  ┌─────────────────────────────────────┐                   │
│  │        双指针扫描合并                 │                   │
│  │                                     │                   │
│  │  pointer_a = 0, pointer_b = 0       │                   │
│  │  while not end:                     │                   │
│  │    if A[pa] < B[pb]: pa++          │                   │
│  │    else if A[pa] > B[pb]: pb++     │                   │
│  │    else: // 相等                    │                   │
│  │        输出所有 A[pa] == B[pb]     │                   │
│  │        pa++, pb++                  │                   │
│  └─────────────────────────────────────┘                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Merge Join 时序图

```
SELECT * FROM A JOIN B ON A.id = B.a_id ORDER BY id
    │
    ▼
┌─────────────────────────────────────────────┐
│              Sort Phase                      │
│  ┌─────────────────────────────────────┐   │
│  │ 1. 确保 A 按 id 排序                │   │
│  │ 2. 确保 B 按 a_id 排序              │   │
│  │ 3. 如果未排序，添加 Sort 算子      │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Merge Phase                    │
│  ┌─────────────────────────────────────┐   │
│  │ a_ptr = A.begin(), b_ptr = B.begin()│   │
│  │ while a_ptr && b_ptr:              │   │
│  │   if a_ptr.key < b_ptr.key:        │   │
│  │       a_ptr++                       │   │
│  │   elif a_ptr.key > b_ptr.key:      │   │
│  │       b_ptr++                       │   │
│  │   else:                             │   │
│  │       output all matches            │   │
│  │       a_ptr++, b_ptr++             │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## 5. Join 排序优化

### 5.1 Join Order 问题

```
-- 4 表连接的排列数
A, B, C, D 的排列: 4! = 24 种

-- 最优顺序 vs 最差顺序
最优: A(100行) ⋈ B(1000行) ⋈ C(10000行) ⋈ D(100000行)
     = 100 + 1000 + 10000 + 100000 = 111100 行扫描

最差: D(100000行) ⋈ C(10000行) ⋈ B(1000行) ⋈ A(100行)
     = 100000 × 10000 × 1000 × 100 = 10^17 行扫描!
```

### 5.2 Greedy Join Ordering 算法

```rust
impl JoinPlan {
    /// 使用贪心算法构建连接顺序
    pub fn greedy_ordering(tables: Vec<Table>, predicates: Vec<JoinPredicate>) -> Self {
        let mut remaining: HashSet<_> = tables.iter().collect();
        let mut joins = Vec::new();

        // 1. 选择最小的表作为起始表
        let first = remaining.iter().min_by_key(|t| t.row_count()).unwrap();
        let mut base = first.clone();
        remaining.remove(&base);

        // 2. 贪心选择下一个连接的表
        while !remaining.is_empty() {
            let (best_table, best_predicate) = find_best_next_join(
                &base, &remaining, &predicates
            );

            joins.push(JoinStep {
                right_table: best_table.name().to_string(),
                on: best_predicate,
            });

            remaining.remove(&best_table);
            base = Table::joined(base, best_table);
        }

        JoinPlan { base_table: first.name(), joins, filters: vec![] }
    }
}
```

## 6. 并行 Hash Join

### 6.1 并行执行架构

```
┌─────────────────────────────────────────────────────────────┐
│                 Parallel Hash Join 架构                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                    ┌─────────────────┐                       │
│                    │  Coordinator   │                       │
│                    └────────┬────────┘                       │
│                             │                                │
│          ┌──────────────────┼──────────────────┐            │
│          │                  │                  │            │
│          ▼                  ▼                  ▼            │
│    ┌───────────┐      ┌───────────┐      ┌───────────┐      │
│    │  Worker 1 │      │  Worker 2 │      │  Worker N │      │
│    │           │      │           │      │           │      │
│    │ build hash│      │build hash │      │build hash │      │
│    │ partition │      │ partition │      │ partition │      │
│    └───────────┘      └───────────┘      └───────────┘      │
│          │                  │                  │            │
│          └──────────────────┼──────────────────┘            │
│                             │                                │
│                             ▼                                │
│                    ┌─────────────────┐                       │
│                    │   Shuffled     │                       │
│                    │   Partitions  │                       │
│                    └────────┬────────┘                       │
│                             │                                │
│          ┌──────────────────┼──────────────────┐            │
│          ▼                  ▼                  ▼            │
│    ┌───────────┐      ┌───────────┐      ┌───────────┐      │
│    │ Partition │      │ Partition │      │ Partition │      │
│    │     0     │      │     1     │      │    N-1   │      │
│    └───────────┘      └───────────┘      └───────────┘      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 分区 Hash Join 算法

```rust
fn partition_hash_join(
    left: RecordBatch,
    right: RecordBatch,
    condition: &Expr,
    num_partitions: usize,
) -> SqlResult<Vec<(RecordBatch, RecordBatch)>> {
    // 1. 为每行计算 partition key (hash of join key)
    let left_partition_keys: Vec<u64> = left.rows()
        .map(|row| compute_partition_hash(row, &condition.left()))
        .collect();

    let right_partition_keys: Vec<u64> = right.rows()
        .map(|row| compute_partition_hash(row, &condition.right()))
        .collect();

    // 2. 根据 partition key 重新分区
    let mut partitions: Vec<(Vec<Row>, Vec<Row>)> = vec![
        (Vec::new(), Vec::new()); num_partitions
    ];

    for (row, pkey) in left.rows().zip(left_partition_keys) {
        let pid = (pkey % num_partitions as u64) as usize;
        partitions[pid].0.push(row.clone());
    }

    for (row, pkey) in right.rows().zip(right_partition_keys) {
        let pid = (pkey % num_partitions as u64) as usize;
        partitions[pid].1.push(row.clone());
    }

    Ok(partitions)
}
```

## 7. Join 性能分析

### 7.1 复杂度对比

| 算法 | 时间复杂度 | 空间复杂度 | 适用场景 |
|------|----------|-----------|----------|
| Hash Join | O(n + m) | O(n) | 等值连接，大表 |
| Nested Loop | O(n × m) | O(1) | 小表，有索引 |
| Merge Join | O(n log n + m log m) | O(n + m) | 已排序输入 |
| Block NL | O(n × b + m × b) | O(b) | 减少 IO |

### 7.2 Hash Join 内存估算

```rust
// 内存需求估算
fn estimate_hash_join_memory(
    left_rows: usize,
    row_size: usize,
    num_partitions: usize,
) -> usize {
    // Hash table 大小 = 行数 × 行大小 × 负载因子
    let load_factor = 0.75;
    let hash_table_size = left_rows * row_size / load_factor;

    // 分区大小 = Hash table / 分区数
    let partition_size = hash_table_size / num_partitions;

    // 总内存 = Hash table + 分区缓冲
    hash_table_size + partition_size * 2
}
```

## 8. 测试计划

### 8.1 功能测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| JOIN-T01 | INNER JOIN 两表 | 返回匹配行 |
| JOIN-T02 | LEFT JOIN 右表为空 | 返回左表所有行 |
| JOIN-T03 | RIGHT JOIN 左表为空 | 返回右表所有行 |
| JOIN-T04 | FULL OUTER JOIN 两表都部分匹配 | 返回所有行 |
| JOIN-T05 | CROSS JOIN | 返回笛卡尔积 |
| JOIN-T06 | 多表 JOIN (4 表) | 正确合并结果 |
| JOIN-T07 | 自连接 | 正确处理自身引用 |
| JOIN-T08 | JOIN 包含 NULL 值 | 正确处理 NULL |
| JOIN-T09 | USING clause | 简化列名 |
| JOIN-T10 | NATURAL JOIN | 自动按同名列连接 |

### 8.2 性能测试

| 测试编号 | 测试内容 | 目标 |
|----------|----------|------|
| JOIN-P01 | 100K × 100K Hash Join | < 1s |
| JOIN-P02 | 1M × 1M Hash Join (分区) | < 5s |
| JOIN-P03 | 1000 × 1000 Nested Loop | < 100ms (有索引) |
| JOIN-P04 | 并行 Hash Join 8 线程 | 线性加速比 > 0.7 |
| JOIN-P05 | 内存受限下的 Graceful degradation | 正确分区处理 |

## 9. 覆盖率差距分析

### 9.1 当前覆盖率

| 组件 | 行覆盖率 | 差距 |
|------|----------|------|
| join_planner.rs | ~75% | Join ordering 算法 |
| local_executor.rs (hash join) | ~70% | 复杂 join 类型 |
| parallel_executor.rs | ~60% | 并行 join 边界情况 |

### 9.2 差距原因

1. **Merge Join 未实现**: 只有 Hash Join 和 Nested Loop
2. **Join 排序算法简单**: 贪心算法，未考虑统计信息
3. **Hash Join 优化不足**: 没有 Bloom filter、graceful degradation
4. **并行 Join 不完整**: 分区算法有边界问题

### 9.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 实现 Merge Join | 75% |
| v3.1.0 | 优化 Join ordering (CBO) | 80% |
| v3.2.0 | Bloom filter 优化 | 85% |
| v3.2.0 | 完善并行 Join | 90% |

## 10. 核心文件索引

| 文件 | 说明 |
|------|------|
| `crates/executor/src/join_planner.rs` | Join 逻辑计划构建 |
| `crates/executor/src/local_executor.rs` | Hash Join 执行 (~1040-1100 行) |
| `crates/executor/src/parallel_executor.rs` | 并行 Hash Join (~200-300 行) |
| `crates/planner/src/physical_plan.rs` | HashJoinExec 定义 |

## 11. 相关文档

| 文档 | 说明 |
|------|------|
| [CBO_JOIN_ORDERING.md](../cbo/CBO_JOIN_ORDERING.md) | CBO Join 排序 |
| [CBO_COST_MODEL.md](../cbo/CBO_COST_MODEL.md) | 代价模型 |
| [EXECUTION_PIPELINE.md](../execution/EXECUTION_PIPELINE.md) | 执行流水线 |
