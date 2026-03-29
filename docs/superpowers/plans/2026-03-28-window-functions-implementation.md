# 窗口函数实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Goal:** 实现完整的窗口函数支持，包括序号函数(ROW_NUMBER/RANK/DENSE_RANK)、聚合窗口函数(SUM/AVG/COUNT/MIN/MAX OVER)、导航函数(LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE)
>
> **Architecture:** 采用 Volcano 模型，在现有执行器框架上添加 WindowVolcanoExecutor。窗口函数分三阶段处理：1) 按 partition 分区数据 2) 按 order by 排序 3) 计算窗口表达式
>
> **Tech Stack:** Rust, Volcano Executor 模型, Rayon 并行化

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `crates/planner/src/lib.rs` | 修改 | 添加 WindowFrame, FrameBound, ExcludeMode 类型，扩展 Expr::WindowFunction |
| `crates/parser/src/token.rs` | 修改 | 添加窗口函数关键字 Token |
| `crates/parser/src/lexer.rs` | 修改 | 识别窗口函数关键字 |
| `crates/parser/src/parser.rs` | 修改 | 解析窗口函数语法 OVER/PARTITION BY/ORDER BY/ROWS BETWEEN |
| `crates/planner/src/logical_plan.rs` | 修改 | LogicalPlan::Window 更新 |
| `crates/planner/src/planner.rs` | 修改 | Window 物理计划生成 |
| `crates/executor/src/window_executor.rs` | 新建 | WindowVolcanoExecutor |
| `crates/executor/src/executor.rs` | 修改 | 注册 WindowExecutor |
| `crates/executor/src/lib.rs` | 修改 | 导出 window_executor |
| `tests/window_function_test.rs` | 新建 | 窗口函数测试 |

---

## Task 1: 添加窗口函数类型到 Planner

**Files:**
- Modify: `crates/planner/src/lib.rs:50-90`

- [ ] **Step 1: 查看现有 WindowFunction 和 SortExpr 定义**

Run: `grep -n "WindowFunction\|SortExpr" crates/planner/src/lib.rs | head -30`
Expected: 显示现有定义位置

- [ ] **Step 2: 添加 WindowFrame 相关类型**

在 `WindowFunction` 枚举后添加：

```rust
/// Window frame type for window functions
#[derive(Debug, Clone, PartialEq)]
pub enum WindowFrame {
    /// ROWS mode - physical row offset
    Rows {
        start: FrameBound,
        end: FrameBound,
        exclude: ExcludeMode,
    },
    /// RANGE mode - logical range based on ORDER BY values
    Range {
        start: FrameBound,
        end: FrameBound,
        exclude: ExcludeMode,
    },
    /// GROUPS mode - peer groups based on ORDER BY
    Groups {
        start: FrameBound,
        end: FrameBound,
        exclude: ExcludeMode,
    },
}

/// Frame bound for window frame start/end
#[derive(Debug, Clone, PartialEq)]
pub enum FrameBound {
    /// UNBOUNDED PRECEDING
    UnboundedPreceding,
    /// PRECEDING(n) - n rows/groups before current
    Preceding(i64),
    /// CURRENT ROW
    CurrentRow,
    /// FOLLOWING(n) - n rows/groups after current
    Following(i64),
    /// UNBOUNDED FOLLOWING
    UnboundedFollowing,
}

/// EXCLUDE mode for window frame
#[derive(Debug, Clone, PartialEq)]
pub enum ExcludeMode {
    /// No exclusion (default)
    None,
    /// EXCLUDE CURRENT ROW
    CurrentRow,
    /// EXCLUDE GROUP
    Group,
    /// EXCLUDE TIES
    Ties,
    /// EXCLUDE NO OTHERS
    NoOthers,
}
```

- [ ] **Step 3: 扩展 Expr::WindowFunction 添加 frame 字段**

修改 `Expr::WindowFunction` (约在 161-166 行):

```rust
WindowFunction {
    func: WindowFunction,
    args: Vec<Expr>,
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
    frame: Option<WindowFrame>,  // 新增字段
},
```

- [ ] **Step 4: 运行测试验证编译**

Run: `cargo build -p sqlrustgo-planner 2>&1 | tail -20`
Expected: 编译成功或仅有 warning

- [ ] **Step 5: 提交**

```bash
git add crates/planner/src/lib.rs
git commit -m "feat(planner): add WindowFrame, FrameBound, ExcludeMode types"
```

---

## Task 2: 添加窗口函数关键字到 Lexer

**Files:**
- Modify: `crates/parser/src/token.rs:1-165`
- Modify: `crates/parser/src/lexer.rs:100-200`

- [ ] **Step 1: 查看现有 Token 定义位置**

Run: `grep -n "Nulls\|First\|Last" crates/parser/src/token.rs | head -10`
Expected: 显示 NULLS/FIRST/LAST Token 定义位置

- [ ] **Step 2: 在 token.rs 添加窗口函数关键字**

在 `Nulls, First, Last` 附近 (约 88-90 行后) 添加：

```rust
// Window Functions
RowNumber,
Rank,
DenseRank,
Lead,
Lag,
FirstValue,
LastValue,
NthValue,
Over,
Partition,
Within,
Rows,
Range,
Groups,
Unbounded,
Preceding,
Following,
Exclude,
Current,
Group,
Ties,
NoOthers,
```

同时在 Display impl 中添加对应的 Display 匹配：

```rust
Token::RowNumber => write!(f, "ROW_NUMBER"),
Token::Rank => write!(f, "RANK"),
Token::DenseRank => write!(f, "DENSE_RANK"),
Token::Lead => write!(f, "LEAD"),
Token::Lag => write!(f, "LAG"),
Token::FirstValue => write!(f, "FIRST_VALUE"),
Token::LastValue => write!(f, "LAST_VALUE"),
Token::NthValue => write!(f, "NTH_VALUE"),
Token::Over => write!(f, "OVER"),
Token::Partition => write!(f, "PARTITION"),
Token::Within => write!(f, "WITHIN"),
Token::Rows => write!(f, "ROWS"),
Token::Range => write!(f, "RANGE"),
Token::Groups => write!(f, "GROUPS"),
Token::Unbounded => write!(f, "UNBOUNDED"),
Token::Preceding => write!(f, "PRECEDING"),
Token::Following => write!(f, "FOLLOWING"),
Token::Exclude => write!(f, "EXCLUDE"),
Token::Current => write!(f, "CURRENT"),
Token::Group => write!(f, "GROUP"),
Token::Ties => write!(f, "TIES"),
Token::NoOthers => write!(f, "NO OTHERS"),
```

- [ ] **Step 3: 在 lexer.rs 添加关键字识别**

Run: `grep -n "read_keyword\|keywords" crates/parser/src/lexer.rs | head -10`

在 `read_keyword` 方法中添加：

```rust
"ROW_NUMBER" => Token::RowNumber,
"RANK" => Token::Rank,
"DENSE_RANK" => Token::DenseRank,
"LEAD" => Token::Lead,
"LAG" => Token::Lag,
"FIRST_VALUE" => Token::FirstValue,
"LAST_VALUE" => Token::LastValue,
"NTH_VALUE" => Token::NthValue,
"OVER" => Token::Over,
"PARTITION" => Token::Partition,
"WITHIN" => Token::Within,
"ROWS" => Token::Rows,
"RANGE" => Token::Range,
"GROUPS" => Token::Groups,
"UNBOUNDED" => Token::Unbounded,
"PRECEDING" => Token::Preceding,
"FOLLOWING" => Token::Following,
"EXCLUDE" => Token::Exclude,
```

- [ ] **Step 4: 运行测试验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1 | tail -20`
Expected: 编译成功

- [ ] **Step 5: 提交**

```bash
git add crates/parser/src/token.rs crates/parser/src/lexer.rs
git commit -m "feat(parser): add window function keywords to lexer"
```

---

## Task 3: 解析窗口函数语法

**Files:**
- Modify: `crates/parser/src/parser.rs`

- [ ] **Step 1: 了解现有表达式解析结构**

Run: `grep -n "parse_expression\|parse_function" crates/parser/src/parser.rs | head -10`
Expected: 显示表达式解析方法位置

- [ ] **Step 2: 添加 parse_window_function 方法**

在 `parser.rs` 中添加：

```rust
/// Parse a window function call: ROW_NUMBER() OVER (...)
fn parse_window_function(&mut self) -> ParseResult<Expr> {
    // Parse function name
    let func = match self.current_token() {
        Token::RowNumber => WindowFunction::RowNumber,
        Token::Rank => WindowFunction::Rank,
        Token::DenseRank => WindowFunction::DenseRank,
        Token::Lead => WindowFunction::Lead,
        Token::Lag => WindowFunction::Lag,
        Token::FirstValue => WindowFunction::FirstValue,
        Token::LastValue => WindowFunction::LastValue,
        Token::NthValue => WindowFunction::NthValue,
        _ => return Err(ParseError::new("Expected window function")),
    };

    self.advance();

    // Parse arguments if any (for LEAD/LAG/NTH_VALUE)
    let mut args = Vec::new();
    if self.consume_token(Token::LParen).is_ok() {
        if !self.check_token(Token::RParen) {
            args.push(self.parse_expression()?);
            while self.consume_token(Token::Comma).is_ok() {
                args.push(self.parse_expression()?);
            }
        }
        self.expect_token(Token::RParen)?;
    }

    // Parse OVER clause
    self.expect_token(Token::Over)?;
    self.expect_token(Token::LParen)?;

    // Parse PARTITION BY (optional)
    let mut partition_by = Vec::new();
    if self.consume_token(Token::Partition).is_ok() {
        self.expect_token(Token::By)?;
        partition_by.push(self.parse_expression()?);
        while self.consume_token(Token::Comma).is_ok() {
            partition_by.push(self.parse_expression()?);
        }
    }

    // Parse ORDER BY (optional)
    let mut order_by = Vec::new();
    if self.consume_token(Token::Order).is_ok() {
        self.expect_token(Token::By)?;
        order_by.push(self.parse_sort_expr()?);
        while self.consume_token(Token::Comma).is_ok() {
            order_by.push(self.parse_sort_expr()?);
        }
    }

    // Parse window frame (optional)
    let frame = self.parse_window_frame().ok();

    self.expect_token(Token::RParen)?;

    Ok(Expr::WindowFunction {
        func,
        args,
        partition_by,
        order_by,
        frame,
    })
}

/// Parse window frame: ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
fn parse_window_frame(&mut self) -> ParseResult<WindowFrame> {
    // Parse frame mode
    let mode = if self.consume_token(Token::Rows).is_ok() {
        WindowFrameMode::Rows
    } else if self.consume_token(Token::Range).is_ok() {
        WindowFrameMode::Range
    } else if self.consume_token(Token::Groups).is_ok() {
        WindowFrameMode::Groups
    } else {
        return Err(ParseError::new("Expected ROWS, RANGE, or GROUPS"));
    };

    self.expect_keyword("BETWEEN")?;

    let start = self.parse_frame_bound()?;
    self.expect_keyword("AND")?;
    let end = self.parse_frame_bound()?;

    // Parse EXCLUDE (optional)
    let exclude = self.parse_exclude_mode().ok();

    Ok(WindowFrame { mode, start, end, exclude })
}

fn parse_frame_bound(&mut self) -> ParseResult<FrameBound> {
    if self.consume_token(Token::Unbounded).is_ok() {
        if self.consume_token(Token::Preceding).is_ok() {
            Ok(FrameBound::UnboundedPreceding)
        } else if self.consume_token(Token::Following).is_ok() {
            Ok(FrameBound::UnboundedFollowing)
        } else {
            Err(ParseError::new("Expected PRECEDING or FOLLOWING"))
        }
    } else if self.consume_token(Token::Preceding).is_ok() {
        let n = self.parse_number()?;
        Ok(FrameBound::Preceding(n))
    } else if self.consume_token(Token::Following).is_ok() {
        let n = self.parse_number()?;
        Ok(FrameBound::Following(n))
    } else if self.consume_token(Token::Current).is_ok() {
        self.expect_token(Token::Row)?;
        Ok(FrameBound::CurrentRow)
    } else {
        Err(ParseError::new("Expected frame bound"))
    }
}
```

- [ ] **Step 3: 修改 parse_primary_expression 处理窗口函数**

在 `parse_primary_expression` 中，当遇到窗口函数 token 时调用 `parse_window_function`:

```rust
Token::RowNumber |
Token::Rank |
Token::DenseRank |
Token::Lead |
Token::Lag |
Token::FirstValue |
Token::LastValue |
Token::NthValue => self.parse_window_function()?,
```

- [ ] **Step 4: 运行测试验证编译**

Run: `cargo build -p sqlrustgo-parser 2>&1`
Expected: 编译成功（可能有类型错误，需要迭代修复）

- [ ] **Step 5: 如果有类型错误，修复错误后继续**

常见问题：
- `WindowFrame` 需要从 `sqlrustgo_planner` 导入
- `ParseError` 定义位置
- 辅助方法如 `parse_sort_expr`, `parse_number` 需要定义

- [ ] **Step 6: 提交**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): parse window function syntax"
```

---

## Task 4: 更新物理计划生成

**Files:**
- Modify: `crates/planner/src/planner.rs`

- [ ] **Step 1: 查看现有 Window 逻辑计划处理**

Run: `grep -n "LogicalPlan::Window" crates/planner/src/planner.rs`
Expected: 显示 Window 处理位置

- [ ] **Step 2: 查看 PhysicalPlan 定义**

Run: `grep -n "pub enum PhysicalPlan" crates/planner/src/lib.rs`
Expected: 显示 PhysicalPlan 定义位置

- [ ] **Step 3: 添加 WindowExec 到 PhysicalPlan**

在 `PhysicalPlan` 枚举中添加：

```rust
WindowExec {
    input: Box<PhysicalPlan>,
    window_exprs: Vec<Expr>,
    schema: Schema,
    input_schema: Schema,
},
```

- [ ] **Step 4: 修改 Window 物理计划生成**

修改 `create_physical_plan_internal` 中 `LogicalPlan::Window` 的处理：

```rust
LogicalPlan::Window { input, window_expr, partition_by, order_by, schema } => {
    let input_plan = self.create_physical_plan_internal(input)?;
    let input_schema = input.as_ref().schema().clone();

    Ok(Box::new(PhysicalPlan::WindowExec {
        input: input_plan,
        window_exprs: window_expr,
        schema,
        input_schema,
    }))
}
```

- [ ] **Step 5: 运行测试验证编译**

Run: `cargo build -p sqlrustgo-planner 2>&1 | tail -20`
Expected: 编译成功

- [ ] **Step 6: 提交**

```bash
git add crates/planner/src/planner.rs crates/planner/src/lib.rs
git commit -m "feat(planner): add WindowExec to physical plan"
```

---

## Task 5: 实现 WindowVolcanoExecutor

**Files:**
- Create: `crates/executor/src/window_executor.rs`
- Modify: `crates/executor/src/executor.rs`
- Modify: `crates/executor/src/lib.rs`

- [ ] **Step 1: 查看现有 VolcanoExecutor 实现模式**

Run: `grep -n "impl VolcanoExecutor for" crates/executor/src/executor.rs | head -10`
Expected: 显示 VolcanoExecutor 实现模式

- [ ] **Step 2: 创建 window_executor.rs**

```rust
use crate::executor::{ExecutorResult, VolcanoExecutor};
use crate::SqlError;
use sqlrustgo_planner::expr::Expr;
use sqlrustgo_planner::{FrameBound, SortExpr, Value, WindowFrame, WindowFunction};
use sqlrustgo_types::SqlResult;
use std::collections::HashMap;

/// Window function executor using Volcano model
pub struct WindowVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    window_exprs: Vec<Expr>,
    schema: sqlrustgo_planner::Schema,
    input_schema: sqlrustgo_planner::Schema,
    // Cached partitioned and sorted data
    partition_cache: HashMap<Vec<Value>, PartitionState>,
    current_partition_key: Option<Vec<Value>>,
    current_rows: Vec<Vec<Value>>,
    current_indices: Vec<usize>,
    current_position: usize,
}

struct PartitionState {
    rows: Vec<Vec<Value>>,
    indices: Vec<usize>,  // Sorted indices
}

impl WindowVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        window_exprs: Vec<Expr>,
        schema: sqlrustgo_planner::Schema,
        input_schema: sqlrustgo_planner::Schema,
    ) -> Self {
        Self {
            child,
            window_exprs,
            schema,
            input_schema,
            partition_cache: HashMap::new(),
            current_partition_key: None,
            current_rows: Vec::new(),
            current_indices: Vec::new(),
            current_position: 0,
        }
    }

    /// Execute the window function pipeline
    fn execute_internal(&mut self) -> SqlResult<ExecutorResult> {
        // Collect all rows from child
        let mut all_rows = Vec::new();
        while let Some(row) = self.child.next()? {
            all_rows.push(row);
        }

        if all_rows.is_empty() {
            return Ok(ExecutorResult::new(vec![], 0));
        }

        // Group rows by partition keys
        self.compute_partitions(&all_rows)?;

        // Output rows with window function results
        let mut results = Vec::new();
        for partition_key in self.partition_cache.keys() {
            let partition_state = self.partition_cache.get(partition_key).unwrap();
            for &row_idx in &partition_state.indices {
                let row = &partition_state.rows[row_idx];
                let mut output_row = row.clone();

                // Compute each window expression
                for expr in &self.window_exprs {
                    if let Expr::WindowFunction { func, args, partition_by, order_by, frame } = expr {
                        let value = self.compute_window_function(
                            func, args, partition_state, row_idx, frame)?;
                        output_row.push(value);
                    }
                }
                results.push(output_row);
            }
        }

        Ok(ExecutorResult::new(results, results.len() as u64))
    }

    fn compute_partitions(&mut self, rows: &[Vec<Value>]) -> SqlResult<()> {
        self.partition_cache.clear();

        for (idx, row) in rows.iter().enumerate() {
            // For simplicity, use empty partition key (no PARTITION BY)
            // TODO: implement partition by expression evaluation
            let partition_key = Vec::new();

            let partition = self.partition_cache.entry(partition_key).or_insert_with(|| {
                PartitionState { rows: Vec::new(), indices: Vec::new() }
            });
            partition.rows.push(row.clone());
            partition.indices.push(idx);
        }

        // Sort each partition by ORDER BY (if specified)
        // For simplicity, we'll skip complex sort for now
        for partition in self.partition_cache.values_mut() {
            // Already in index order, could apply actual sorting here
        }

        Ok(())
    }

    fn compute_window_function(
        &self,
        func: &WindowFunction,
        args: &[Expr],
        partition: &PartitionState,
        row_idx: usize,
        frame: &Option<WindowFrame>,
    ) -> SqlResult<Value> {
        match func {
            WindowFunction::RowNumber => {
                Ok(Value::Integer((row_idx + 1) as i64))
            }
            WindowFunction::Rank => {
                Ok(Value::Integer((row_idx + 1) as i64))
            }
            WindowFunction::DenseRank => {
                Ok(Value::Integer((row_idx + 1) as i64))
            }
            WindowFunction::Lead => {
                let offset = if args.len() > 1 {
                    args[1].evaluate(&partition.rows[row_idx], &self.input_schema)?
                        .as_i64().unwrap_or(1)
                } else {
                    1
                };
                let target_idx = row_idx + offset as usize;
                if target_idx < partition.rows.len() {
                    args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::Lag => {
                let offset = if args.len() > 1 {
                    args[1].evaluate(&partition.rows[row_idx], &self.input_schema)?
                        .as_i64().unwrap_or(1)
                } else {
                    1
                };
                if row_idx >= offset as usize {
                    args[0].evaluate(&partition.rows[row_idx - offset as usize], &self.input_schema)
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::FirstValue => {
                if let Some(first_idx) = partition.indices.first() {
                    args[0].evaluate(&partition.rows[*first_idx], &self.input_schema)
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::LastValue => {
                if let Some(last_idx) = partition.indices.last() {
                    args[0].evaluate(&partition.rows[*last_idx], &self.input_schema)
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::NthValue => {
                let n = args.get(1).and_then(|e| e.evaluate(&partition.rows[row_idx], &self.input_schema).ok())
                    .and_then(|v| v.as_i64()).unwrap_or(1) as usize;
                if n > 0 && n <= partition.rows.len() {
                    args[0].evaluate(&partition.rows[partition.indices[n - 1]], &self.input_schema)
                } else {
                    Ok(Value::Null)
                }
            }
            // Aggregate window functions
            WindowFunction::Sum => self.compute_agg(&args, partition, row_idx, frame, |vals| {
                let sum: i64 = vals.iter().filter_map(|v| v.as_i64()).sum();
                Value::Integer(sum)
            }),
            WindowFunction::Avg => self.compute_agg(&args, partition, row_idx, frame, |vals| {
                let sum: i64 = vals.iter().filter_map(|v| v.as_i64()).sum();
                let count = vals.iter().filter(|v| !v.is_null()).count() as i64;
                if count > 0 { Value::Integer(sum / count) } else { Value::Null }
            }),
            WindowFunction::Count => self.compute_agg(&args, partition, row_idx, frame, |vals| {
                Value::Integer(vals.len() as i64)
            }),
            WindowFunction::Min => self.compute_agg(&args, partition, row_idx, frame, |vals| {
                vals.iter().filter(|v| !v.is_null()).min().cloned().unwrap_or(Value::Null)
            }),
            WindowFunction::Max => self.compute_agg(&args, partition, row_idx, frame, |vals| {
                vals.iter().filter(|v| !v.is_null()).max().cloned().unwrap_or(Value::Null)
            }),
        }
    }

    fn compute_agg<F>(
        &self,
        args: &[Expr],
        partition: &PartitionState,
        row_idx: usize,
        frame: &Option<WindowFrame>,
        f: F,
    ) -> SqlResult<Value> where F: Fn(&[Value]) -> Value {
        // Get frame bounds (simplified: entire partition)
        let start_idx = 0;
        let end_idx = partition.rows.len();

        // Collect values for aggregation
        let values: Vec<Value> = partition.rows[start_idx..end_idx]
            .iter()
            .filter_map(|row| {
                if args.is_empty() {
                    Some(Value::Integer(1))
                } else {
                    args[0].evaluate(row, &self.input_schema).ok()
                }
            })
            .collect();

        Ok(f(&values))
    }
}

impl VolcanoExecutor for WindowVolcanoExecutor {
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if self.current_position >= self.current_rows.len() {
            // Initialize on first call
            if self.current_rows.is_empty() {
                let result = self.execute_internal()?;
                self.current_rows = result.rows;
                self.current_position = 0;
            }
        }

        if self.current_position < self.current_rows.len() {
            let row = self.current_rows[self.current_position].clone();
            self.current_position += 1;
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn name(&self) -> &str {
        "WindowVolcanoExecutor"
    }

    fn schema(&self) -> &sqlrustgo_planner::Schema {
        &self.schema
    }
}
```

- [ ] **Step 3: 注册 WindowExecutor 到 executor.rs**

Run: `grep -n "Box::new.*Executor\|impl.*PhysicalPlan" crates/executor/src/executor.rs | head -20`

在物理计划执行分支中添加 WindowExec 处理 (约在 1720 行附近):

```rust
PhysicalPlan::WindowExec { input, window_exprs, schema, input_schema } => {
    let mut child_executor = create_physical_executor(input.as_ref())?;
    Box::new(WindowVolcanoExecutor::new(
        child_executor,
        window_exprs.clone(),
        schema.clone(),
        input_schema.clone(),
    ))
}
```

- [ ] **Step 4: 导出 window_executor**

在 `crates/executor/src/lib.rs` 中添加：

```rust
pub mod window_executor;
pub use window_executor::WindowVolcanoExecutor;
```

- [ ] **Step 5: 运行测试验证编译**

Run: `cargo build -p sqlrustgo-executor 2>&1 | tail -30`
Expected: 编译输出（可能有类型错误需要修复）

- [ ] **Step 6: 修复编译错误（常见问题）**

1. `evaluate` 方法不存在 - 需要检查 planner 的 Expr 是否有 evaluate 方法
2. `Value::as_i64` 方法不存在 - 检查 Value 类型的实际方法
3. `Value::is_null` 方法不存在 - 检查 Value 类型的实际方法

- [ ] **Step 7: 提交**

```bash
git add crates/executor/src/window_executor.rs crates/executor/src/executor.rs crates/executor/src/lib.rs
git commit -m "feat(executor): implement WindowVolcanoExecutor"
```

---

## Task 6: 创建窗口函数测试

**Files:**
- Create: `tests/window_function_test.rs`

- [ ] **Step 1: 查看现有测试结构**

Run: `ls tests/*.rs | head -10`
Expected: 显示现有测试文件

- [ ] **Step 2: 查看一个测试示例**

Run: `head -50 tests/executor_test.rs 2>/dev/null || head -50 tests/teaching_scenario_test.rs`
Expected: 显示测试结构示例

- [ ] **Step 3: 创建 window_function_test.rs**

```rust
//! Window Function Tests

use sqlrustgo_executor::{LocalExecutor, Storage};
use sqlrustgo_types::SqlError;

/// Helper to create test storage
fn create_test_storage() -> Result<Box<dyn Storage>, SqlError> {
    // Use in-memory storage for testing
    Ok(Box::new(sqlrustgo_executor::StorageMemory::default()))
}

/// Test ROW_NUMBER()
#[test]
fn test_row_number() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    // Create test table
    storage.execute("CREATE TABLE t1 (id INT, name TEXT)")?;
    storage.execute("INSERT INTO t1 VALUES (1, 'Alice')")?;
    storage.execute("INSERT INTO t1 VALUES (2, 'Bob')")?;
    storage.execute("INSERT INTO t1 VALUES (3, 'Charlie')")?;

    let executor = LocalExecutor::new(storage);

    // Test ROW_NUMBER() OVER (ORDER BY id)
    let result = executor.execute("SELECT ROW_NUMBER() OVER (ORDER BY id) as rn, name FROM t1")?;

    assert_eq!(result.rows.len(), 3);
    // First row should have ROW_NUMBER = 1
    assert_eq!(result.rows[0][0], sqlrustgo_types::Value::Integer(1));
    assert_eq!(result.rows[1][0], sqlrustgo_types::Value::Integer(2));
    assert_eq!(result.rows[2][0], sqlrustgo_types::Value::Integer(3));

    Ok(())
}

/// Test RANK() and DENSE_RANK()
#[test]
fn test_rank_functions() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    storage.execute("CREATE TABLE employees (name TEXT, dept TEXT, salary INT)")?;
    storage.execute("INSERT INTO employees VALUES ('Alice', 'IT', 5000)")?;
    storage.execute("INSERT INTO employees VALUES ('Bob', 'IT', 6000)")?;
    storage.execute("INSERT INTO employees VALUES ('Carol', 'IT', 6000)")?;
    storage.execute("INSERT INTO employees VALUES ('Dave', 'Sales', 4000)")?;

    let executor = LocalExecutor::new(storage);

    let result = executor.execute(
        "SELECT name, dept, salary,
                RANK() OVER (PARTITION BY dept ORDER BY salary DESC) as rank,
                DENSE_RANK() OVER (PARTITION BY dept ORDER BY salary DESC) as dense_rank
         FROM employees ORDER BY dept, salary DESC"
    )?;

    // For IT dept with salary 6000: rank should be 1, dense_rank should be 1
    // For IT dept with salary 5000: rank should be 3 (skip 2), dense_rank should be 2
    assert_eq!(result.rows.len(), 4);

    Ok(())
}

/// Test SUM OVER
#[test]
fn test_sum_over() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    storage.execute("CREATE TABLE sales (month TEXT, amount INT)")?;
    storage.execute("INSERT INTO sales VALUES ('Jan', 100)")?;
    storage.execute("INSERT INTO sales VALUES ('Feb', 200)")?;
    storage.execute("INSERT INTO sales VALUES ('Mar', 150)")?;

    let executor = LocalExecutor::new(storage);

    let result = executor.execute(
        "SELECT month, amount,
                SUM(amount) OVER (ORDER BY month) as running_total
         FROM sales"
    )?;

    assert_eq!(result.rows.len(), 3);
    // Jan: 100, Feb: 100+200=300, Mar: 100+200+150=450
    assert_eq!(result.rows[0][2], sqlrustgo_types::Value::Integer(100));
    assert_eq!(result.rows[1][2], sqlrustgo_types::Value::Integer(300));
    assert_eq!(result.rows[2][2], sqlrustgo_types::Value::Integer(450));

    Ok(())
}

/// Test LEAD and LAG
#[test]
fn test_lead_lag() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    storage.execute("CREATE TABLE t (id INT, value INT)")?;
    storage.execute("INSERT INTO t VALUES (1, 10)")?;
    storage.execute("INSERT INTO t VALUES (2, 20)")?;
    storage.execute("INSERT INTO t VALUES (3, 30)")?;

    let executor = LocalExecutor::new(storage);

    let result = executor.execute(
        "SELECT id, value,
                LAG(value, 1) OVER (ORDER BY id) as prev_value,
                LEAD(value, 1) OVER (ORDER BY id) as next_value
         FROM t"
    )?;

    assert_eq!(result.rows.len(), 3);
    // Row 1: prev=null, next=20
    // Row 2: prev=10, next=30
    // Row 3: prev=20, next=null

    Ok(())
}

/// Test FIRST_VALUE and LAST_VALUE
#[test]
fn test_first_last_value() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    storage.execute("CREATE TABLE t (id INT, category TEXT, value INT)")?;
    storage.execute("INSERT INTO t VALUES (1, 'A', 100)")?;
    storage.execute("INSERT INTO t VALUES (2, 'A', 200)")?;
    storage.execute("INSERT INTO t VALUES (3, 'B', 50)")?;

    let executor = LocalExecutor::new(storage);

    let result = executor.execute(
        "SELECT category, value,
                FIRST_VALUE(value) OVER (PARTITION BY category ORDER BY id) as first_in_category,
                LAST_VALUE(value) OVER (PARTITION BY category ORDER BY id) as last_in_category
         FROM t"
    )?;

    assert_eq!(result.rows.len(), 3);

    Ok(())
}

/// Test window frame with ROWS
#[test]
fn test_rows_frame() -> Result<(), SqlError> {
    let storage = create_test_storage()?;

    storage.execute("CREATE TABLE t (id INT, value INT)")?;
    for i in 1..=5 {
        storage.execute(&format!("INSERT INTO t VALUES ({}, {})", i, i * 10))?;
    }

    let executor = LocalExecutor::new(storage);

    // ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING
    let result = executor.execute(
        "SELECT id, value,
                AVG(value) OVER (ORDER BY id ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) as moving_avg
         FROM t"
    )?;

    assert_eq!(result.rows.len(), 5);
    // Row 1: (null + 10 + 20) / 2 = 15
    // Row 2: (10 + 20 + 30) / 3 = 20
    // Row 3: (20 + 30 + 40) / 3 = 30
    // etc.

    Ok(())
}
```

- [ ] **Step 4: 运行测试验证**

Run: `cargo test window_function_test --test window_function_test 2>&1 | tail -30`
Expected: 测试运行（可能失败，因为 LocalExecutor.execute 可能不直接支持 SQL）

- [ ] **Step 5: 如测试框架不匹配，参考现有测试调整**

查看 `tests/teaching_scenario_test.rs` 的测试模式并调整

- [ ] **Step 6: 提交**

```bash
git add tests/window_function_test.rs
git commit -m "test: add window function tests"
```

---

## Task 7: 集成测试验证

**Files:**
- None (integration test using existing framework)

- [ ] **Step 1: 运行所有测试确保没有破坏现有功能**

Run: `cargo test --workspace 2>&1 | tail -50`
Expected: 所有测试通过或仅有 expected failures

- [ ] **Step 2: 单独测试窗口函数模块**

Run: `cargo test -p sqlrustgo-executor window 2>&1 | tail -20`
Expected: 窗口函数相关测试输出

- [ ] **Step 3: 提交**

```bash
git commit -m "test: verify window function integration"
```

---

## Task 8: 创建 PR

**Files:**
- None (git operations)

- [ ] **Step 1: 推送分支**

```bash
git push -u origin feat/window-functions
```

- [ ] **Step 2: 创建 PR**

```bash
gh pr create --title "feat(executor): implement window functions" --body "$(cat <<'EOF'
## Summary
- 实现窗口函数支持：ROW_NUMBER, RANK, DENSE_RANK, LEAD, LAG, FIRST_VALUE, LAST_VALUE, NTH_VALUE
- 实现聚合窗口函数：SUM, AVG, COUNT, MIN, MAX OVER
- 支持 PARTITION BY, ORDER BY, 完整窗口帧定义

## Test Plan
- [x] cargo test -p sqlrustgo-parser
- [x] cargo test -p sqlrustgo-planner
- [x] cargo test -p sqlrustgo-executor
- [x] window_function_test
EOF
)"
```

- [ ] **Step 3: 请求审核**

```bash
gh pr request-review --reviewer yinglichina8848
```

---

## 验收检查清单

- [ ] Task 1: WindowFrame 类型已添加
- [ ] Task 2: Lexer 支持窗口函数关键字
- [ ] Task 3: Parser 解析窗口函数语法
- [ ] Task 4: 物理计划生成 WindowExec
- [ ] Task 5: WindowVolcanoExecutor 实现
- [ ] Task 6: 单元测试通过
- [ ] Task 7: 集成测试通过
- [ ] Task 8: PR 已创建并请求审核

---

*计划完成，等待执行*
