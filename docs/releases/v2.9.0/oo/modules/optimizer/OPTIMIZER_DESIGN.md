# Optimizer Design

This document describes the query optimizer design for SqlRustGo, covering cost-based optimization, rule-based optimization, and plan enumeration.

## Overview

The optimizer transforms a parsed SQL statement (logical plan) into an efficient execution plan (physical plan). It uses a two-phase approach:
1. **Rule-Based Optimization (RBO)**: Apply heuristic transformations
2. **Cost-Based Optimization (CBO)**: Select best plan using cost estimates

## Implementation Status

> ⚠️ **警告**: v2.9.0 版本中，优化器大多数规则为 **TODO stub**，实际查询未经过有意义的优化。当前优化器覆盖率为 0% (WEAKNESS_ANALYSIS.md)。

| 组件 | 状态 | 说明 |
|------|------|------|
| **RBO** | ⚠️ 部分实现 | 基础规则已实现 |
| PredicatePushdown | ❌ TODO | `rules.rs` 中为 stub |
| ProjectionPruning | ❌ TODO | `rules.rs` 中为 stub |
| ConstantFolding | ❌ TODO | `rules.rs` 中为 stub |
| **CBO** | ❌ TODO | 代价模型未实现 |
| Cost Model | ❌ TODO | 无统计信息 |
| Plan Enumeration | ❌ TODO | 无枚举逻辑 |
| **物理计划** | ✅ 已实现 | HashJoin, NestedLoopJoin 等 |

### 架构图 (虚线 = TODO)

```
SQL → Parser → [RBO] → [CBO] → Physical Plan
                   ↓           ↓
              ⚠️ TODO    ⚠️ TODO
```

## Plan Representation

### Logical Plan Nodes

```
LogicalPlan
├── TableScan
├── IndexScan
├── HashJoin
├── NestedLoopJoin
├── HashAgg
├── Sort
├── Limit
└── Projection
```

### Physical Plan Nodes

```
PhysicalPlan
├── SeqScan [table]
├── IndexScan [index] [indexexpr]
├── IndexOnlyScan [index]
├── HashJoin [inner|outer]
├── MergeJoin
├── NestedLoopJoin
├── HashAgg [groupby]
├── Sort [orderby]
├── Limit [count]
├── Projection [exprs]
└── Aggregate [aggs]
```

### Plan Properties

Each plan node has properties:
- **Row Count Estimate**: Estimated output rows
- **Cost**: Estimated CPU and I/O cost
- **Distribution**: How rows are partitioned across nodes
- **Sort Order**: Output ordering guarantee

## Rule-Based Optimization (RBO)

### Transformation Rules

RBO applies logical transformations that always improve or preserve query semantics.

#### Predicate Pushdown

Move filter conditions as close to data sources as possible:

```
Before:              After:
Projection           TableScan [pushed filter]
  |                    |
  Filter [c1 > 10]     |
    |                  Projection
    |                    |
    Join                Join [with residual filter]
      |                  |
      TableScan A      IndexScan A [c1 > 10]
      TableScan B        |
                        TableScan B
```

#### Subquery Flattening

Convert correlated subqueries to joins:

```
Before:                      After:
Filter [x IN (SELECT y)]     HashJoin
  |                          / \
  ...                       x    Subquery
                           /|\
                          ... (unnest)
```

#### Join Reordering

Reorder joins to minimize intermediate result size:

```
Good:                  Bad:
Filter                 Join (large)
  |                      / \
  Join                   |   |
   / \                  |   |
  A   Join             A   B
       / \                |
      B   C              Filter
      |                     |
      C                    ...
```

#### Column Pruning

Remove unused columns from scans:

```
Before:                        After:
TableScan columns=[a,b,c,d]    TableScan columns=[a,c]
  |                                 |
  Projection [a, c]                 Projection [a, c]
```

#### Constant Folding

Evaluate constant expressions at optimization time:

```
Before:                    After:
Filter [1 + 2 = 3]        Filter [true]
  |                          |
  ...                        ...
```

### RBO Rule Categories

| Category | Rules |
|----------|-------|
| `PROJECTION` | Prune columns, simplify projections |
| `FILTER` | Push filter, merge filters, remove tautologies |
| `JOIN` | Reorder, eliminate, convert to cross product |
| `AGGREGATION` | Push groupby, split aggregations |
| `SUBQUERY` | Flatten correlated, convert to semi-join |

## Cost-Based Optimizer (CBO)

### Overview

CBO estimates the cost of alternative plans and selects the minimum cost plan. Cost is measured in estimated I/O and CPU units.

### Cost Model

```
TotalCost = IOCost * PageReads + CPUCost * RowCount

IOCost = SequentialReadCost + RandomReadCost
CPUCost = TupleEvalCost + FunctionEvalCost
```

### Statistics

The optimizer maintains table-level and column-level statistics:

**Table Statistics**:
- `row_count`: Number of rows
- `page_count`: Number of pages
- `tuple_width`: Average tuple size in bytes
- `dead_tuples`: Vacuumable tuples

**Column Statistics**:
- `null_frac`: Fraction of NULL values
- `n_distinct`: Number of distinct values
- `most_common_vals`: MCV list
- `most_common_freqs`: MCV frequencies
- `histogram_bounds`: Equi-depth histogram
- `correlation`: Physical-to-logical correlation

### Selectivity Estimation

Selectivity of predicate `col OP value`:

```
selectivity = 
  | equality (col = v):    1 / n_distinct
  | range (col > v):       (max - v) / (max - min)
  | comparison (col > v):  (max - v) / (max - min)
  | conjunction (a AND b): sel(a) * sel(b)
  | disjunction (a OR b):  sel(a) + sel(b) - sel(a AND b)
  | negation (NOT a):     1 - sel(a)
```

### Cost Estimation

**Sequential Scan**:
```
Cost = page_count * seq_page_cost + row_count * cpu_tuple_cost
```

**Index Scan**:
```
Cost = index_pages * index_page_cost + 
       rows_selected * (index_cpu + cpu_tuple) +
       heap_pages * heap_page_cost
```

**Hash Join**:
```
Cost = outer_cost + inner_build_cost + 
       (outer_rows * inner_rows * hash_join_cost)
```

**Sort**:
```
Cost = n * log(n) * sort_cpu_cost + 
       (if spills) pages * disk_write_read_cost
```

## Plan Enumeration

### Overview

Plan enumeration explores the space of possible execution plans and selects the best one based on cost estimates.

### Enumeration Strategy

SqlRustGo uses a Selinger-style dynamic programming approach:

1. **Single Relation Plans**: Enumerate all access paths for each table
2. **Join Ordering**: Enumerate all join orderings for multi-table queries
3. **Aggregate Planning**: Handle groupby, distinct, having clauses
4. **Final Plan**: Select minimum cost plan from enumerated options

### Access Path Selection

For each base relation, enumerate access methods:

```
TableScan [filter]
IndexScan [index] [index filter]
IndexOnlyScan [index] [covering columns]
```

Select based on:
- Index availability for WHERE clauses
- Index selectivity vs sequential scan
- Covering index for index-only scans

### Join Ordering

**Left-Deep vs Bushy**:

```
Left-Deep:           Bushy:
    Join               Join
   /   \             /    \
  A    Join        Join    Join
       /  \        / \    /  \
       B   C      A   B  C   D
```

Left-deep preferred for:
- Pipeline execution (can start returning rows early)
- Hash join (only one hash table at a time)

Bushy useful for:
- Memory-constrained environments
- Parallel execution

### Enumeration Algorithm

```
function enumerate(query):
    // Single relation plans
    for each base rel:
        plans[rel] = enumerate_access_paths(rel)
    
    // Join ordering
    for each join subset (size 2..n):
        for each way to split subset (A join B):
            left_plans = plans[A]
            right_plans = plans[B]
            for each (lp, rp) combination:
                join_plans = create_join_plans(lp, rp)
                plans[subset] += join_plans
    
    // Select best final plan
    return min_cost(plans[all_rels])
```

### Memo Structure

The Memo structure caches intermediate plans:

```
Memo
├── Group 1: [SeqScan, IndexScan, IndexOnlyScan]
├── Group 2: [HashJoin, NLJoin, MergeJoin]
├── Group 3: [Sort, NoSort]
...
```

Each group contains logically equivalent plans. Optimization explores groups, not individual plans.

### Physical Properties

Plans have required physical properties:

| Property | Values |
|----------|--------|
| Sort Order | Any, Ascending, Descending, Constrained |
| Distribution | Any, HashPartitioned, Broadcast |
| Parallelism | Sequential, Parallel |

### Plan Caching

- Memo entries cached across similar queries
- Parameterized plan caching for prepared statements
- Statistics-based reuse for ad-hoc queries

## Optimization Levels

| Level | Description |
|-------|-------------|
| `O0` | No optimization (parse only) |
| `O1` | Basic RBO only |
| `O2` | RBO + basic CBO |
| `O3` | Full optimization |

Set via `SET optimization_level = 'O3';`

## Hint System

Manual hints override optimizer decisions:

```sql
SELECT /*+ INDEX(t idx_name) */
  * FROM t WHERE col = 1;
```

| Hint | Effect |
|------|--------|
| `INDEX(rel idx)` | Force index usage |
| `FULL(rel)` | Force sequential scan |
| `ORDERED` | Use table order for joins |
| `LEADING(t1 t2)` | Specify join order |
| `NO_CACHE` | Disable caching |
