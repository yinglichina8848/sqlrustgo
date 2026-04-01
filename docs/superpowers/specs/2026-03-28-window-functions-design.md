# 窗口函数实现设计规格

> **版本**: v2.0.0
> **日期**: 2026-03-28
> **Issue**: #955
> **状态**: 设计完成，待实现

---

## 1. 概述

实现完整的窗口函数支持，包括序号函数、聚合窗口函数和导航函数，支持 PARTITION BY、ORDER BY 和完整的窗口帧定义。

### 1.1 支持的窗口函数

| 类别 | 函数 | 说明 |
|------|------|------|
| 序号函数 | ROW_NUMBER, RANK, DENSE_RANK | 生成行号/排名 |
| 聚合窗口函数 | SUM, AVG, COUNT, MIN, MAX | 聚合类窗口计算 |
| 导航函数 | LEAD, LAG, FIRST_VALUE, LAST_VALUE, NTH_VALUE | 访问相邻行或首尾值 |

### 1.2 支持的语法

```sql
SELECT
  name,
  department,
  salary,
  ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) as rank,
  SUM(salary) OVER (PARTITION BY department ORDER BY hire_date
    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_total,
  LAG(salary, 1) OVER (ORDER BY hire_date) as prev_salary,
  FIRST_VALUE(salary) OVER (ORDER BY salary DESC) as highest_salary
FROM employees;
```

---

## 2. 数据结构扩展

### 2.1 新增类型

```rust
// crates/planner/src/lib.rs

/// Window frame type
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFrame {
    Rows { start: FrameBound, end: FrameBound, exclude: ExcludeMode },
    Range { start: FrameBound, end: FrameBound, exclude: ExcludeMode },
    Groups { start: FrameBound, end: FrameBound, exclude: ExcludeMode },
}

/// Frame bound for window frame start/end
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    UnboundedPreceding,
    Preceding(i64),
    CurrentRow,
    Following(i64),
    UnboundedFollowing,
}

/// EXCLUDE mode for window frame
#[derive(Debug, Clone, PartialEq)]
pub enum ExcludeMode {
    None,
    CurrentRow,
    Group,
    Ties,
    NoOthers,
}
```

### 2.2 扩展 Expr::WindowFunction

```rust
// 修改现有 Expr::WindowFunction
WindowFunction {
    func: WindowFunction,
    args: Vec<Expr>,
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
    frame: Option<WindowFrame>,  // 新增字段
}
```

---

## 3. 组件实现

### 3.1 Lexer 词法分析

**文件**: `crates/parser/src/lexer.rs`

新增关键字:
- `ROW_NUMBER`, `RANK`, `DENSE_RANK`
- `LEAD`, `LAG`, `FIRST_VALUE`, `LAST_VALUE`, `NTH_VALUE`
- `OVER`, `PARTITION`, `WITHIN`
- `ROWS`, `RANGE`, `GROUPS`
- `UNBOUNDED`, `PRECEDING`, `FOLLOWING`
- `EXCLUDE`, `CURRENT`, `GROUP`, `TIES`, `NO`, `OTHERS`

### 3.2 Parser 语法解析

**文件**: `crates/parser/src/parser.rs`

解析规则:
1. 解析窗口函数调用 (如 `ROW_NUMBER()`)
2. 解析 OVER 子句
3. 解析 PARTITION BY 子句 (可选)
4. 解析 ORDER BY 子句 (可选)
5. 解析 ROWS/RANGE/GROUPS 帧定义 (可选)

### 3.3 Planner 逻辑计划

**文件**: `crates/planner/src/logical_plan.rs`

更新 LogicalPlan::Window:
```rust
Window {
    input: Box<LogicalPlan>,
    window_expr: Vec<Expr>,        // 包含 WindowFunction 的表达式
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
    schema: Schema,
}
```

### 3.4 物理计划生成

**文件**: `crates/planner/src/planner.rs`

将 LogicalPlan::Window 转换为 PhysicalPlan::WindowExec

### 3.5 Executor 执行器

**文件**: `crates/executor/src/window_executor.rs` (新建)

实现 WindowVolcanoExecutor:

```rust
pub struct WindowVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    window_exprs: Vec<WindowExpr>,
    schema: Schema,
    input_schema: Schema,
    // 分区状态
    partition_cache: HashMap<Vec<Value>, PartitionState>,
    current_partition: Option<PartitionState>,
    current_row: usize,
}

struct PartitionState {
    rows: Vec<Vec<Value>>,
    sorted_indices: Vec<usize>,
    aggregations: HashMap<String, WindowAggState>,
}
```

---

## 4. 窗口帧实现

### 4.1 帧计算逻辑

```rust
impl PartitionState {
    /// Calculate window frame rows for a given row index
    fn get_frame_rows(&self, row_idx: usize, frame: &WindowFrame) -> FrameResult {
        match frame {
            WindowFrame::Rows { start, end, exclude } => {
                let start_idx = match start {
                    FrameBound::UnboundedPreceding => 0,
                    FrameBound::Preceding(n) => row_idx.saturating_sub(*n as usize),
                    FrameBound::CurrentRow => row_idx,
                    _ => unreachable!(),
                };
                let end_idx = match end {
                    FrameBound::UnboundedFollowing => self.rows.len(),
                    FrameBound::Following(n) => (row_idx + *n as usize + 1).min(self.rows.len()),
                    FrameBound::CurrentRow => row_idx + 1,
                    _ => unreachable!(),
                };
                FrameResult { start_idx, end_idx, exclude: exclude.clone() }
            }
            // Range and Groups mode similar...
        }
    }
}
```

### 4.2 序号函数实现

```rust
fn compute_row_number(&self, partition: &PartitionState, row_idx: usize) -> Value {
    Value::Integer((row_idx + 1) as i64)
}

fn compute_rank(&self, partition: &PartitionState, row_idx: usize) -> Value {
    // Rank skips gaps
    let sort_key = self.get_sort_key(partition, row_idx);
    let mut rank = 1;
    for i in 0..row_idx {
        if self.get_sort_key(partition, i) < sort_key {
            rank += 1;
        }
    }
    Value::Integer(rank as i64)
}

fn compute_dense_rank(&self, partition: &PartitionState, row_idx: usize) -> Value {
    // Dense rank no gaps
    let sort_key = self.get_sort_key(partition, row_idx);
    let mut rank = 1;
    for i in 0..row_idx {
        if self.get_sort_key(partition, i) < sort_key {
            rank += 1;
        }
    }
    Value::Integer(rank as i64)
}
```

### 4.3 聚合窗口函数

利用现有的 AggregateVolcanoExecutor 模式，但针对窗口帧计算:

```rust
fn compute_window_agg(&self, func: &AggregateFunction, args: &[Expr],
                      partition: &PartitionState, frame: &FrameResult) -> Value {
    let values: Vec<Value> = frame.rows.iter()
        .map(|row| args[0].evaluate(row, &self.input_schema))
        .collect();

    match func {
        AggregateFunction::Sum => compute_sum(&values),
        AggregateFunction::Avg => compute_avg(&values),
        AggregateFunction::Count => Value::Integer(values.len() as i64),
        AggregateFunction::Min => compute_min(&values),
        AggregateFunction::Max => compute_max(&values),
    }
}
```

### 4.4 导航函数

```rust
fn compute_lead(&self, args: &[Expr], partition: &PartitionState,
                row_idx: usize, offset: i64) -> Value {
    let target_idx = (row_idx + offset as usize + 1).min(partition.rows.len() - 1);
    args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
}

fn compute_lag(&self, args: &[Expr], partition: &PartitionState,
               row_idx: usize, offset: i64) -> Value {
    let target_idx = row_idx.saturating_sub(offset as usize);
    args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
}
```

---

## 5. 测试计划

### 5.1 单元测试

| 测试 | 说明 |
|------|------|
| `test_row_number` | ROW_NUMBER() 基本功能 |
| `test_rank_no_gaps` | RANK() 无间隙排名 |
| `test_dense_rank` | DENSE_RANK() 密集排名 |
| `test_sum_over` | SUM() OVER 窗口聚合 |
| `test_avg_over` | AVG() OVER 窗口聚合 |
| `test_count_over` | COUNT() OVER 窗口聚合 |
| `test_lead_lag` | LEAD/LAG 导航函数 |
| `test_first_last_value` | FIRST_VALUE/LAST_VALUE |
| `test_partition_by` | PARTITION BY 多分区 |
| `test_order_by_nulls` | ORDER BY NULLS FIRST/LAST |
| `test_rows_frame` | ROWS BETWEEN ... |
| `test_range_frame` | RANGE BETWEEN ... |
| `test_groups_frame` | GROUPS BETWEEN ... |
| `test_exclude_modes` | EXCLUDE CURRENT ROW/GROUP/TIES |

### 5.2 集成测试

```sql
-- 教学场景测试
SELECT ROW_NUMBER() OVER (ORDER BY id) as row_num FROM t1;
SELECT RANK() OVER (PARTITION BY dept ORDER BY salary DESC) as rank FROM t1;
SELECT SUM(salary) OVER (ORDER BY hire_date ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_sum FROM t1;
SELECT LEAD(salary, 1) OVER (ORDER BY hire_date) as next_salary FROM t1;
```

---

## 6. 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/planner/src/lib.rs` | 修改 | 添加 WindowFrame, FrameBound, ExcludeMode 类型 |
| `crates/parser/src/lexer.rs` | 修改 | 添加窗口函数关键字 |
| `crates/parser/src/parser.rs` | 修改 | 解析窗口函数语法 |
| `crates/planner/src/logical_plan.rs` | 修改 | 更新 LogicalPlan::Window |
| `crates/planner/src/planner.rs` | 修改 | Window 算子物理计划生成 |
| `crates/executor/src/window_executor.rs` | 新建 | WindowVolcanoExecutor 实现 |
| `crates/executor/src/executor.rs` | 修改 | 注册 WindowExecutor |
| `crates/executor/src/lib.rs` | 修改 | 导出 window_executor |
| `tests/window_function_test.rs` | 新建 | 窗口函数测试 |

---

## 7. 风险与限制

1. **性能**: 窗口函数需要排序，当前实现使用内存排序，大数据集可能有性能问题
2. **NULL 处理**: NULL 值的排序顺序需要严格按 SQL 标准
3. **类型系统**: LEAD/LAG 需要支持任意类型参数

---

## 8. 验收标准

- [ ] Lexer 支持所有窗口函数关键字
- [ ] Parser 能解析完整的窗口函数语法
- [ ] ROW_NUMBER, RANK, DENSE_RANK 正确计算
- [ ] SUM, AVG, COUNT, MIN, MAX OVER 正确计算
- [ ] LEAD, LAG, FIRST_VALUE, LAST_VALUE 正确计算
- [ ] PARTITION BY 支持多列
- [ ] ORDER BY 支持 NULLS FIRST/LAST
- [ ] ROWS/RANGE/GROUPS 帧定义正确
- [ ] EXCLUDE 模式正确处理
- [ ] 所有单元测试通过
- [ ] 集成测试通过

---

*设计完成，等待用户批准后进入实现阶段*
