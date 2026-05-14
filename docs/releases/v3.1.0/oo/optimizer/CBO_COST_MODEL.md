# CBO 代价模型详解 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> Cost-Based Optimizer: 代价计算、统计信息、Join 排序

## 1. CBO 架构

### 1.1 核心数据结构

```rust
pub struct Cost {
    pub io_cost: f64,
    pub cpu_cost: f64,
    pub memory_cost: f64,
    pub network_cost: f64,
}

pub struct CostConstants {
    pub cpu_cost_per_row: f64,         // 1.0
    pub cpu_cost_index_search: f64,    // 0.1
    pub seq_io_latency_ms: f64,        // 0.1
    pub random_io_latency_ms: f64,     // 1.0
    pub memory_latency_ns: f64,        // 100.0
    pub page_size_bytes: u64,          // 16384 (16KB)
    pub sort_buffer_pages: u64,        // 1024
    pub hash_probe_cost_factor: f64,   // 1.2
    pub network_latency_ms: f64,       // 0.5
}

pub struct CboOptimizer {
    cost_model: SimpleCostModel,
    default_row_count: u64,   // 1000
    default_page_count: u64,  // 10
}
```

### 1.2 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [cost.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/cost.rs) | 1001 | CBO 优化器 + 代价模型 |
| [stats.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/stats.rs) | - | 统计信息定义 |
| [stats_collector.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/stats_collector.rs) | - | 统计信息收集 |
| [stats_provider.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/stats_provider.rs) | 323 | 存储后端统计提供 |
| [rules.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/rules.rs) | - | RBO 规则 + IndexSelect |
| [query_planner.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/optimizer/src/query_planner.rs) | 829 | 查询规划器 |

## 2. 代价公式

### 2.1 算子代价计算

| 算子 | 公式 | 说明 |
|------|------|------|
| **SeqScan** | `pages * seq_io + rows * cpu_per_row` | 全表扫描 |
| **IndexScan** | `height * index_search_cpu + index_pages * random_io + data_pages * seq_io + rows * cpu` | 索引扫描 |
| **Index Point** | `1 * random_io + data_pages * seq_io + height * index_search_cpu` | 索引点查 |
| **Hash Join** | `build_rows * cpu + probe_rows * cpu * hash_probe_factor + hash_table_pages * mem_latency` | 哈希连接 |
| **NLJ** | `outer * inner * cpu_per_row` | 嵌套循环 |
| **Sort-Merge Join** | `left_sort + right_sort + (left+right) * cpu` | 排序归并 |
| **Hash Agg** | `rows * cpu + groups * log2(groups) * cpu + hash_table_pages * mem_latency` | 哈希聚合 |
| **Sort (内存)** | `rows * log2(rows) * cpu` | 内存排序 |
| **Sort (外部)** | `pages * seq_io * 2 + rows * log2(rows) * cpu` | 外部排序 |
| **Filter** | `input_cost * 1.1` | ⚠️ 固定乘数 |
| **Limit** | `input_cost.min(limit * 0.5)` | ⚠️ 粗糙估算 |

### 2.2 CBO 决策流程活动图

```
    ┌──────────────────────────────────┐
    │     LogicalPlan 输入              │
    └──────────────┬───────────────────┘
                   │
                   ▼
    ┌──────────────────────────────────┐
    │  RBO 规则优化                     │
    │  ├── PredicatePushdown           │
    │  ├── ProjectionPruning           │
    │  └── ConstantFolding             │
    └──────────────┬───────────────────┘
                   │
                   ▼
    ┌──────────────────────────────────┐
    │  CBO 代价优化                     │
    │  ├── 获取统计信息                 │
    │  │   ├── 行数/页面数             │
    │  │   ├── NDV (唯一值数)          │
    │  │   └── NULL 比例               │
    │  ├── 估算各方案代价               │
    │  │   ├── SeqScan vs IndexScan    │
    │  │   ├── HashJoin vs NLJ vs SMJ  │
    │  │   └── Join 顺序选择           │
    │  └── 选择最低代价方案             │
    └──────────────┬───────────────────┘
                   │
                   ▼
    ┌──────────────────────────────────┐
    │     PhysicalPlan 输出             │
    └──────────────────────────────────┘
```

## 3. Join 排序算法

### 3.1 贪心算法

```
JoinPlanner.build_join_plan(tables, join_preds, filters)
    │
    ▼
┌──────────────────────────────────────────────┐
│ 1. 分离谓词                                   │
│    ├── 跨表等值 → JoinPredicate               │
│    └── 同表/非等值 → FilterPredicate          │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 2. 贪心构建 JoinPlan                          │
│    ├── base_table = tables[0]                 │
│    ├── while 有未连接的表:                     │
│    │   ├── 找到与已连接表有 join_pred 的表     │
│    │   └── 添加 JoinStep                      │
│    └── 输出: JoinPlan { base, joins, filters }│
└──────────────────────────────────────────────┘
```

### 3.2 Join 排序时序图

```
SELECT * FROM a JOIN b ON a.id = b.aid
                JOIN c ON b.id = c.bid
                JOIN d ON c.id = d.cid

    │
    ▼
Step 1: base_table = "a"
    │
    ▼
Step 2: 找到 a.id = b.aid → JoinStep(b, on: a.id=b.aid)
    │
    ▼
Step 3: 找到 b.id = c.bid → JoinStep(c, on: b.id=c.bid)
    │
    ▼
Step 4: 找到 c.id = d.cid → JoinStep(d, on: c.id=d.cid)
    │
    ▼
JoinPlan:
  base: a
  joins: [b, c, d] (左深树)
```

## 4. 算法复杂度与性能分析

### 4.1 操作复杂度

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| 代价估算 | O(P) | P=计划节点数 |
| Join 排序 (贪心) | O(t² * p) | t=表数, p=谓词数 |
| Join 排序 (DP) | O(2^t) | 指数级，t≤10 可用 |
| 统计信息收集 | O(N) | N=表行数 |
| 规则优化 | O(P) | 线性遍历 |

### 4.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **无统计信息集成** | 🔴 严重 | 使用硬编码默认值 | 接入真实表统计 |
| **Filter 代价乘数固定** | 🟡 中等 | 不考虑选择性 | 基于直方图估算 |
| **Limit 代价估算粗糙** | 🟡 中等 | 假设每行 0.5 | 考虑子计划代价 |
| **Join 方法选择简单** | 🟡 中等 | 仅按 join_type | 考虑数据量选择 |
| **贪心算法不考虑表大小** | 🟡 中等 | 可能选错顺序 | 基于行数贪心 |

### 4.3 代价模型准确性评估

```
当前状态:
  SeqScan 代价: ✅ 较准确 (基于页面数和行数)
  IndexScan 代价: ✅ 较准确 (考虑 B+Tree 高度)
  Hash Join 代价: ⚠️ 中等 (未考虑数据倾斜)
  NLJ 代价: ✅ 准确 (O(L*R))
  Filter 代价: ❌ 不准确 (固定 1.1 乘数)
  Limit 代价: ❌ 不准确 (limit * 0.5)

改进路线:
  v3.1.1: 接入统计信息 (ANALYZE TABLE)
  v3.2.0: 直方图选择性估算
  v3.3.0: DP Join 排序 + 数据倾斜感知
```

## 5. 与其他模块的依赖

```
CboOptimizer
  ├── 依赖: optimizer::unified_plan (计划表示)
  ├── 依赖: optimizer::rules (JoinType)
  ├── 被依赖: QueryPlanner (规划入口)
  ├── 被依赖: ExecutionEngine (执行入口)
  └── 被依赖: EXPLAIN (代价输出)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充代价公式、Join 排序算法、准确性评估 |
