# 查询执行器设计 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> LocalExecutor + ParallelExecutor: Volcano 模型执行器

## 1. 执行器架构

### 1.1 核心数据结构

```rust
pub struct LocalExecutor<'a> {
    storage: &'a dyn StorageEngine,
    cache: Arc<RwLock<QueryCache>>,
    cache_config: QueryCacheConfig,
    slow_query_log: StdRwLock<Option<SlowQueryLog>>,
    current_sql: StdRwLock<String>,
    prepared_statements: StdRwLock<PreparedStatementManager>,
}

struct GroupAccumulator {
    count: i64,
    sum: i64,
    min_val: Option<i64>,
    max_val: Option<i64>,
    distinct_values: HashSet<String>,
}
```

### 1.2 执行器类型

| 执行器 | 文件 | 职责 |
|--------|------|------|
| LocalExecutor | [local_executor.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/executor/src/local_executor.rs) (2612行) | 单线程执行 |
| ParallelExecutor | [parallel_executor.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/executor/src/parallel_executor.rs) | 并行执行 (rayon) |
| TransactionalExecutor | - | 事务内执行 |
| WalTransactionalExecutor | - | WAL 事务执行 |

### 1.3 支持的算子

| 算子 | 方法 | 状态 | 复杂度 |
|------|------|------|--------|
| SeqScan | execute_seq_scan | ✅ | O(N) |
| IndexScan | execute_index_scan | ✅ | O(log N + K) |
| Filter | execute_filter | ✅ | O(N) |
| Projection | execute_projection | ✅ | O(N) |
| Hash Join | execute_hash_join | ✅ | O(L + R) |
| Aggregate | execute_aggregate | ✅ | O(N * G) |
| Sort | execute_sort | ⚠️ 未实现 | O(N log N) |
| Limit | execute_limit | ⚠️ 未实现 | O(K) |
| Delete | execute_delete | ✅ | O(N) |

## 2. SELECT 执行全链路

### 2.1 执行时序图

```
SELECT o.id, c.name, SUM(o.total)
FROM orders o
JOIN customers c ON o.cid = c.id
WHERE o.status = 'active'
GROUP BY o.id, c.name
ORDER BY SUM(o.total) DESC
LIMIT 10
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│ 1. PARSER: Lexer → Tokens → Parser → Statement::Select  │
│    复杂度: O(n), n=SQL长度                               │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────┐
│ 2. PLANNER: AST → LogicalPlan → PhysicalPlan             │
│    ├── TableScan(orders) → TableScan(customers)          │
│    ├── HashJoin(on: o.cid = c.id)                        │
│    ├── Filter(status = 'active')                         │
│    ├── Aggregate(GROUP BY id, name, SUM(total))          │
│    ├── Sort(SUM(total) DESC)                             │
│    └── Limit(10)                                         │
│    复杂度: O(t² * p), t=表数, p=谓词数                   │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────┐
│ 3. OPTIMIZER: RBO + CBO                                  │
│    ├── PredicatePushdown: Filter 下推到 Scan              │
│    ├── ProjectionPruning: 裁剪未使用列                    │
│    ├── ConstantFolding: 常量折叠                          │
│    └── CBO: 选择 Hash Join (代价最低)                    │
│    复杂度: O(2^t) DP, O(t²) 贪心                        │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────────┐
│ 4. EXECUTOR: LocalExecutor.execute(plan)                  │
│    ├── execute_seq_scan(orders) → Vec<Row>               │
│    ├── execute_seq_scan(customers) → Vec<Row>            │
│    ├── execute_hash_join(left, right, on) → Vec<Row>     │
│    ├── execute_filter(rows, status='active') → Vec<Row>  │
│    ├── execute_aggregate(rows, GROUP BY) → Vec<Row>      │
│    ├── execute_sort(rows, SUM DESC) ⚠️ 未实现             │
│    └── execute_limit(rows, 10) ⚠️ 未实现                  │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
                  ResultSet
```

## 3. Hash Join 执行链路

### 3.1 Hash Join 算法

```rust
fn try_build_hash_join(
    left: &[Record], right: &[Record],
    left_col_idx: usize, right_col_idx: usize,
) -> HashMap<String, Vec<usize>> {
    let (build, probe, build_idx, probe_idx, is_swapped) =
        if left.len() <= right.len() {
            (left, right, left_col_idx, right_col_idx, false)
        } else {
            (right, left, right_col_idx, left_col_idx, true)
        };

    let mut hash_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, row) in build.iter().enumerate() {
        let key = format!("{:?}", row.columns[build_idx]);
        hash_map.entry(key).or_default().push(i);
    }
    hash_map
}
```

### 3.2 Hash Join 时序图

```
HashJoin(left=orders, right=customers, on=o.cid=c.id)
    │
    ▼
┌──────────────────────────────────────────────┐
│ Phase 1: BUILD                               │
│  ├── 选择较小表 (orders: 1000行)             │
│  ├── 遍历 orders                             │
│  │   ├── row[0].cid = 1 → hash["1"]=[0]     │
│  │   ├── row[1].cid = 2 → hash["2"]=[1]     │
│  │   └── ...                                 │
│  └── HashMap 构建: O(L)                      │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ Phase 2: PROBE                               │
│  ├── 遍历 customers (5000行)                 │
│  │   ├── row[0].id = 1 → hash["1"] → 匹配  │
│  │   ├── row[1].id = 3 → hash["3"] → 未匹配│
│  │   └── ...                                 │
│  └── 探测: O(R)                              │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ Phase 3: MERGE                               │
│  ├── Inner: 只返回匹配行                     │
│  ├── Left: 匹配行 + 左未匹配(右填NULL)      │
│  └── Full: 匹配行 + 左未匹配 + 右未匹配     │
└──────────────────────────────────────────────┘
```

### 3.3 Hash Join 状态图

```
            ┌──────────┐
            │  INIT    │
            └────┬─────┘
                 │
                 ▼
        ┌────────────────┐
        │ SELECT BUILD   │
        │ SIDE (smaller) │
        └───────┬────────┘
                │
                ▼
        ┌────────────────┐
        │ BUILD HASH MAP │
        │ O(build_rows)  │
        └───────┬────────┘
                │
                ▼
        ┌────────────────┐
        │ PROBE PHASE    │
        │ O(probe_rows)  │
        └───┬────────┬───┘
        MATCH    NO_MATCH
            │        │
            ▼        ▼
    ┌───────────┐ ┌──────────────┐
    │ CONCAT    │ │ JOIN TYPE    │
    │ ROWS      │ │ DEPENDENT    │
    └─────┬─────┘ │ ACTION       │
          │       └──────┬───────┘
          │              │
          ▼              ▼
    ┌──────────────────────────┐
    │     RESULT COLLECTION    │
    └────────────┬─────────────┘
                 │
                 ▼
            ┌──────────┐
            │  DONE    │
            └──────────┘
```

## 4. 并行执行

### 4.1 Parallel Hash Join

```
ParallelExecutor.execute_parallel_hash_join()
    │
    ▼
┌──────────────────────────────────────────────┐
│ rayon::join(                                 │
│   || execute(left_child),   ← 并行扫描左表  │
│   || execute(right_child),  ← 并行扫描右表  │
│ )                                            │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ partition_hash_join(left, right, on)         │
│  ├── 按 join key 哈希分区                    │
│  ├── 每个分区独立构建 Hash 表                │
│  └── 每个分区独立探测                        │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 合并分区结果                                 │
└──────────────────────────────────────────────┘
```

## 5. 算法复杂度与性能分析

### 5.1 算子复杂度

| 算子 | 复杂度 | 内存 | 说明 |
|------|--------|------|------|
| SeqScan | O(N) | O(N) | 全表扫描 |
| IndexScan | O(log N + K) | O(K) | B+Tree + 结果集 |
| Hash Join (Inner) | O(L + R) | O(min(L,R)) | Build + Probe |
| Hash Join (Left) | O(L + R + M*L) | O(L + R) | ⚠️ M=匹配数 |
| Hash Join (Full) | O(L * R) | O(L + R) | ⚠️ 全行克隆 |
| NLJ | O(L * R) | O(1) | 笛卡尔积 |
| GROUP BY | O(N * G) | O(G) | 流式聚合 |
| DISTINCT | O(N * D) | O(D) | HashSet 去重 |
| Sort | ⚠️ 未实现 | - | 直接返回子节点 |
| Limit | ⚠️ 未实现 | - | 直接返回子节点 |

### 5.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **Sort 未实现** | 🔴 严重 | ORDER BY 不生效 | 实现排序算法 |
| **Limit 未实现** | 🔴 严重 | LIMIT 不生效 | 实现 K-heap 或截断 |
| **Left Join O(M*L)** | 🟡 中等 | 大结果集慢 | 使用标记位替代遍历 |
| **Full Join O(L*R)** | 🟡 中等 | 性能极差 | 使用反连接优化 |
| **DISTINCT 用 format!** | 🟡 中等 | 效率低 | 实现 Value::hash() |
| **重复声明 results** | 🟢 轻微 | 编译警告 | 修改变量名 |

### 5.3 性能优化建议

```
优化1: Sort 算子实现
  当前: execute_sort() 直接返回子节点结果
  建议: 实现外部排序 (in-memory: Vec::sort, external: 归并排序)
  预期: ORDER BY 功能正常

优化2: Limit 算子实现
  当前: execute_limit() 直接返回子节点结果
  建议: 截断结果集到 limit 行
  预期: LIMIT 功能正常

优化3: Left Join 标记位
  当前: 对每个左表行遍历所有匹配结果
  建议: 使用 HashSet<row_idx> 标记已匹配的右表行
  预期: O(L + R) → O(L + R)

优化4: DISTINCT 哈希
  当前: format!("{:?}", v) 字符串去重
  建议: 实现 Hash trait for Value
  预期: 避免字符串分配和碰撞风险
```

## 6. 与其他模块的依赖

```
LocalExecutor
  ├── 依赖: sqlrustgo_planner (物理计划)
  ├── 依赖: sqlrustgo_storage::StorageEngine (存储)
  ├── 依赖: sqlrustgo_types::Value (值类型)
  ├── 依赖: crate::query_cache (查询缓存)
  ├── 依赖: crate::operator_profile (性能剖析)
  ├── 被依赖: ExecutionEngine (执行入口)
  ├── 被依赖: TransactionalExecutor (事务执行)
  └── 被依赖: ParallelExecutor (并行执行)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充 Hash Join 时序图/状态图、Sort/Limit gap |
