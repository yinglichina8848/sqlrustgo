# 窗口函数执行链路

> Window Functions: ROW_NUMBER, RANK, LEAD, LAG, FIRST_VALUE, etc.

## 1. 窗口函数概述

### 1.1 支持的窗口函数

```
┌─────────────────────────────────────────────────────────────┐
│                    窗口函数分类                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   排名函数      │  │   导航函数      │  │  聚合函数   │ │
│  │                 │  │                 │  │             │ │
│  │ ROW_NUMBER()    │  │ LEAD(expr, n)  │  │ SUM() OVER │ │
│  │ RANK()          │  │ LAG(expr, n)   │  │ AVG() OVER │ │
│  │ DENSE_RANK()   │  │ FIRST_VALUE()   │  │ COUNT()    │ │
│  │ PERCENT_RANK() │  │ LAST_VALUE()    │  │ MIN/MAX    │ │
│  │ CUME_DIST()    │  │ NTH_VALUE()    │  │             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│                                                              │
│  ┌─────────────────┐                                        │
│  │   分布函数       │                                        │
│  │                 │                                        │
│  │ PERCENT_RANK()  │                                        │
│  │ CUME_DIST()    │                                        │
│  └─────────────────┘                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 窗口函数语法

```sql
SELECT
    name,
    department,
    salary,
    ROW_NUMBER() OVER (
        PARTITION BY department
        ORDER BY salary DESC
    ) as rank_in_dept,
    SUM(salary) OVER (
        PARTITION BY department
    ) as dept_total,
    LEAD(salary, 1) OVER (
        ORDER BY hire_date
    ) as next_salary
FROM employees;
```

## 2. 窗口函数执行架构

### 2.1 执行流程

```
┌─────────────────────────────────────────────────────────────┐
│                 窗口函数执行流程                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  SELECT ... ROW_NUMBER() OVER (PARTITION BY d ORDER BY s)   │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Parser                           │           │
│  │  WindowFunction {                             │           │
│  │    func: ROW_NUMBER,                        │           │
│  │    partition_by: [d],                       │           │
│  │    order_by: [s DESC],                      │           │
│  │    frame: Rows between unbounded preceding... │           │
│  │  }                                          │           │
│  └─────────────────────────────────────────────┘           │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Planner                         │           │
│  │  WindowAggExec {                           │           │
│  │    partition_by: [d],                       │           │
│  │    order_by: [s DESC],                      │           │
│  │    window_funcs: [ROW_NUMBER()],           │           │
│  │    child: SortExec { ... }                  │           │
│  │  }                                          │           │
│  └─────────────────────────────────────────────┘           │
│      │                                                        │
│      ▼                                                        │
│  ┌─────────────────────────────────────────────┐           │
│  │              Executor                        │           │
│  │  WindowVolcanoExecutor {                   │           │
│  │    child.next() → rows                    │           │
│  │    compute_partitions()                    │           │
│  │    for each partition:                     │           │
│  │      sort by order_by                      │           │
│  │      compute window functions              │           │
│  │  }                                         │           │
│  └─────────────────────────────────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 窗口帧 (Window Frame)

```
┌─────────────────────────────────────────────────────────────┐
│                    窗口帧类型                                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ROWS 模式 (物理偏移):                                     │
│  ─────────────────────────────────────────────────────────  │
│  ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW          │
│  ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING                  │
│  ROWS BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING          │
│                                                              │
│  RANGE 模式 (逻辑偏移):                                     │
│  ─────────────────────────────────────────────────────────  │
│  RANGE BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW        │
│  RANGE BETWEEN 100 PRECEDING AND 100 FOLLOWING             │
│                                                              │
│  GROUPS 模式 (peer group):                                 │
│  ─────────────────────────────────────────────────────────  │
│  GROUPS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. 窗口函数实现

### 3.1 窗口函数执行器结构

```rust
// crates/executor/src/window_executor.rs
pub struct WindowVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    window_exprs: Vec<Expr>,
    schema: Schema,
    input_schema: Schema,
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
    // 缓存的分区数据
    partition_cache: HashMap<Vec<Value>, PartitionState>,
    current_rows: Vec<Vec<Value>>,
    current_position: usize,
    initialized: bool,
}

struct PartitionState {
    rows: Vec<Vec<Value>>,
    indices: Vec<usize>, // 排序后的索引
}
```

### 3.2 分区计算

```rust
fn compute_partitions(&mut self, all_rows: &[Vec<Value>]) -> SqlResult<()> {
    // 1. 按分区键分组
    for (idx, row) in all_rows.iter().enumerate() {
        let partition_key = self.extract_partition_keys(row)?;

        self.partition_cache
            .entry(partition_key)
            .or_insert_with(|| PartitionState {
                rows: Vec::new(),
                indices: Vec::new(),
            })
            .rows
            .push(row.clone());
    }

    // 2. 对每个分区内的行按 ORDER BY 排序
    for partition_state in self.partition_cache.values_mut() {
        let indices: Vec<usize> = (0..partition_state.rows.len()).collect();
        partition_state.indices = self.sort_indices(&partition_state.rows, indices);
    }

    Ok(())
}
```

## 4. 窗口函数状态机

```
                  ┌──────────────────┐
                  │    INITIAL       │
                  └────────┬─────────┘
                           │ first next() call
                           ▼
                  ┌──────────────────┐
                  │  COLLECT_ROWS   │
                  └────────┬─────────┘
                           │ collect all child rows
                           ▼
                  ┌──────────────────┐
                  │  COMPUTE_PARTS  │
                  └────────┬─────────┘
                           │ group by partition keys
                           ▼
                  ┌──────────────────┐
                  │    SORT_PART    │
                  └────────┬─────────┘
                           │ sort within partition
                           ▼
                  ┌──────────────────┐
                  │   EMIT_ROWS     │
                  └────────┬─────────┘
                           │ emit row with window values
                           ▼
                  ┌──────────────────┐
                  │   MORE_ROWS     │
                  └────────┬─────────┘
                           │ yes → continue
                           │
                           │ no
                           ▼
                  ┌──────────────────┐
                  │      DONE        │
                  └──────────────────┘
```

## 5. 窗口函数详解

### 5.1 ROW_NUMBER

```rust
// 分配唯一的行号，不重复
fn row_number(partition: &PartitionState, position: usize) -> Value {
    Value::BigInt(position as i64 + 1)
}
```

### 5.2 RANK vs DENSE_RANK

```
分区数据: [100, 100, 90, 80]

ROW_NUMBER():  [1, 2, 3, 4]   // 连续编号
RANK():        [1, 1, 3, 4]   // 跳跃排名 (1, 1, 3, 4)
DENSE_RANK():  [1, 1, 2, 3]  // 紧凑排名 (1, 1, 2, 3)
```

### 5.3 LEAD / LAG

```
数据: [10, 20, 30]

LEAD(x, 1):    [20, 30, NULL]   // 下一行的值
LEAD(x, 2):    [30, NULL, NULL] // 下两行的值
LAG(x, 1):     [NULL, 10, 20]   // 上一行的值
LAG(x, 2):     [NULL, NULL, 10] // 上两行的值
```

### 5.4 FIRST_VALUE / LAST_VALUE

```
分区数据 (按时间排序): [t1:10, t2:20, t3:15]

FIRST_VALUE:  10  // 帧的第一个值
LAST_VALUE:   15  // 帧的最后一个值 (默认 RANGE CURRENT ROW)
```

## 6. 帧计算

### 6.1 帧边界

```rust
enum FrameBound {
    UnboundedPreceding,
    Preceding(u64),
    CurrentRow,
    Following(u64),
    UnboundedFollowing,
}

fn compute_frame_bounds(
    partition: &PartitionState,
    current_idx: usize,
    frame_start: &FrameBound,
    frame_end: &FrameBound,
) -> (usize, usize) {
    let start = match frame_start {
        FrameBound::UnboundedPreceding => 0,
        FrameBound::Preceding(n) => (current_idx as i64 - *n as i64).max(0) as usize,
        FrameBound::CurrentRow => current_idx,
        _ => current_idx, // Following not valid for start
    };

    let end = match frame_end {
        FrameBound::UnboundedFollowing => partition.rows.len() - 1,
        FrameBound::Following(n) => (current_idx + *n as usize).min(partition.rows.len() - 1),
        FrameBound::CurrentRow => current_idx,
        _ => current_idx,
    };

    (start, end)
}
```

### 6.2 帧计算示例

```
SELECT SUM(x) OVER (
    ORDER BY x
    ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING
) FROM table;

x: [1, 2, 3, 4, 5]

帧计算:
- x=1: SUM(1, 2) = 3    (frame: [1 PRECEDING, CURRENT, 1 FOLLOWING])
- x=2: SUM(1, 2, 3) = 6
- x=3: SUM(2, 3, 4) = 9
- x=4: SUM(3, 4, 5) = 12
- x=5: SUM(4, 5) = 9     (no following row)
```

## 7. 测试计划

### 7.1 排名函数测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| WIN-T01 | ROW_NUMBER 全局 | 1, 2, 3, 4 |
| WIN-T02 | ROW_NUMBER 分区 | 每分区重新编号 |
| WIN-T03 | RANK 有重复 | 相同值相同排名 |
| WIN-T04 | DENSE_RANK | 紧凑排名 |
| WIN-T05 | PARTITION BY 多列 | 正确分区 |

### 7.2 导航函数测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| WIN-T10 | LEAD 简单 | 获取下一行 |
| WIN-T11 | LEAD n=2 | 获取下两行 |
| WIN-T12 | LAG 首行 | 返回 NULL |
| WIN-T13 | FIRST_VALUE | 获取帧首值 |
| WIN-T14 | LAST_VALUE | 获取帧尾值 |

### 7.3 聚合窗口测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| WIN-T20 | SUM OVER | 累计求和 |
| WIN-T21 | AVG OVER | 累计平均 |
| WIN-T22 | COUNT OVER | 累计计数 |
| WIN-T23 | MAX OVER | 累计最大值 |

### 7.4 帧测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| WIN-T30 | ROWS UNBOUNDED PRECEDING | 全部分区 |
| WIN-T31 | ROWS n PRECEDING | 滑动窗口 |
| WIN-T32 | RANGE mode | 按值分组 |
| WIN-T33 | GROUPS mode | 按 peer group 分组 |

## 8. 覆盖率差距分析

### 8.1 当前覆盖率

| 组件 | 行覆盖率 | 说明 |
|------|----------|------|
| window_executor.rs | ~70% | ROW_NUMBER/RANK/DENSE_RANK |
| window function parsing | ~65% | PARTITION/ORDER 解析 |
| window function planning | ~60% | 物理计划生成 |

### 8.2 差距原因

1. **NTILE**: 未实现
2. **LEAD/LAG**: 未实现完整（包括 IGNORE NULLS）
3. **FIRST_VALUE/LAST_VALUE**: 帧处理不完整
4. **NTH_VALUE**: 未实现
5. **CUME_DIST/PERCENT_RANK**: 未实现

### 8.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 实现 LEAD/LAG | 80% |
| v3.1.0 | 实现 FIRST_VALUE/LAST_VALUE | 80% |
| v3.2.0 | 实现 NTILE/NTH_VALUE | 75% |
| v3.2.0 | 实现 PERCENT_RANK/CUME_DIST | 70% |

## 9. 核心文件索引

| 文件 | 行数 | 说明 |
|------|------|------|
| `crates/executor/src/window_executor.rs` | ~1030 | 窗口函数执行器 |
| `crates/planner/src/physical_plan.rs` | ~500 | WindowAggExec 定义 |
| `tests/window_function_test.rs` | ~300 | 窗口函数测试 |

## 10. 相关文档

| 文档 | 说明 |
|------|------|
| [DML_EXECUTION.md](./DML_EXECUTION.md) | DML 执行链路 |
| [SUBQUERY_EXECUTION.md](./SUBQUERY_EXECUTION.md) | 子查询执行 |
| [AGGREGATE_EXECUTION.md](./AGGREGATE_EXECUTION.md) | 聚合函数执行 |
