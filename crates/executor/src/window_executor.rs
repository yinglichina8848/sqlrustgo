// SQLRustGo window function executor module

use crate::executor::{ExecutorResult, VolcanoExecutor};
use sqlrustgo_planner::{Expr, Schema, SortExpr, WindowFrame, WindowFunction};
use sqlrustgo_types::{SqlResult, Value};
use std::any::Any;
use std::collections::HashMap;

/// Window function executor using Volcano model
pub struct WindowVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    window_exprs: Vec<Expr>,
    schema: Schema,
    input_schema: Schema,
    partition_by: Vec<Expr>,
    order_by: Vec<SortExpr>,
    // Cached partitioned and sorted data
    partition_cache: HashMap<Vec<Value>, PartitionState>,
    current_rows: Vec<Vec<Value>>,
    current_position: usize,
    initialized: bool,
}

struct PartitionState {
    rows: Vec<Vec<Value>>,
    indices: Vec<usize>, // Sorted indices
}

impl WindowVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        window_exprs: Vec<Expr>,
        schema: Schema,
        input_schema: Schema,
        partition_by: Vec<Expr>,
        order_by: Vec<SortExpr>,
    ) -> Self {
        Self {
            child,
            window_exprs,
            schema,
            input_schema,
            partition_by,
            order_by,
            partition_cache: HashMap::new(),
            current_rows: Vec::new(),
            current_position: 0,
            initialized: false,
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
        // Collect partition keys and sort them
        let mut partition_keys: Vec<_> = self.partition_cache.keys().cloned().collect();
        partition_keys.sort();
        for partition_key in partition_keys {
            let partition_state = self.partition_cache.get(&partition_key).unwrap();
            for (local_idx, &row_idx) in partition_state.indices.iter().enumerate() {
                let row = &partition_state.rows[row_idx];
                let mut output_row = row.clone();

                // Compute each window expression
                for expr in &self.window_exprs {
                    let value = self.compute_window_expression(expr, partition_state, local_idx)?;
                    output_row.push(value);
                }
                results.push(output_row);
            }
        }

        let row_count = results.len();
        Ok(ExecutorResult::new(results, row_count))
    }

    fn compute_partitions(&mut self, rows: &[Vec<Value>]) -> SqlResult<()> {
        self.partition_cache.clear();

        for (idx, row) in rows.iter().enumerate() {
            // Compute partition key
            let partition_key: Vec<Value> = if self.partition_by.is_empty() {
                Vec::new()
            } else {
                self.partition_by
                    .iter()
                    .filter_map(|expr| expr.evaluate(row, &self.input_schema))
                    .collect()
            };

            let partition = self
                .partition_cache
                .entry(partition_key)
                .or_insert_with(|| PartitionState {
                    rows: Vec::new(),
                    indices: Vec::new(),
                });
            partition.rows.push(row.clone());
            partition.indices.push(idx);
        }

        // Sort each partition by ORDER BY (if specified)
        if !self.order_by.is_empty() {
            for partition in self.partition_cache.values_mut() {
                let rows_copy = partition.rows.clone();
                let input_schema = self.input_schema.clone();
                let order_by = self.order_by.clone();

                partition.indices.sort_by(|&a, &b| {
                    let row_a = &rows_copy[a];
                    let row_b = &rows_copy[b];

                    let mut cmp = std::cmp::Ordering::Equal;
                    for sort_expr in &order_by {
                        let val_a = sort_expr.expr.evaluate(row_a, &input_schema);
                        let val_b = sort_expr.expr.evaluate(row_b, &input_schema);

                        let ordering = match (val_a, val_b) {
                            (Some(v_a), Some(v_b)) => v_a.cmp(&v_b),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        };

                        cmp = if sort_expr.asc {
                            ordering
                        } else {
                            ordering.reverse()
                        };

                        if cmp != std::cmp::Ordering::Equal {
                            break;
                        }
                    }
                    cmp
                });
            }
        }

        Ok(())
    }

    fn compute_window_expression(
        &self,
        expr: &Expr,
        partition: &PartitionState,
        local_idx: usize,
    ) -> SqlResult<Value> {
        match expr {
            Expr::WindowFunction {
                func,
                args,
                partition_by: _,
                order_by: _,
                frame,
            } => self.compute_window_function(func, args, partition, local_idx, frame),
            _ => Ok(Value::Null),
        }
    }

    fn compute_window_function(
        &self,
        func: &WindowFunction,
        args: &[Expr],
        partition: &PartitionState,
        local_idx: usize,
        frame: &Option<WindowFrame>,
    ) -> SqlResult<Value> {
        match func {
            WindowFunction::RowNumber => Ok(Value::Integer((local_idx + 1) as i64)),
            WindowFunction::Rank => Ok(Value::Integer((local_idx + 1) as i64)),
            WindowFunction::DenseRank => Ok(Value::Integer((local_idx + 1) as i64)),
            WindowFunction::Lead => {
                let offset = if args.len() > 1 {
                    let target_idx = partition.indices[local_idx];
                    args[1]
                        .evaluate(&partition.rows[target_idx], &self.input_schema)
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as usize
                } else {
                    1
                };
                let target_local_idx = local_idx + offset;
                if target_local_idx < partition.indices.len() {
                    let target_idx = partition.indices[target_local_idx];
                    Ok(args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::Lag => {
                let offset = if args.len() > 1 {
                    let target_idx = partition.indices[local_idx];
                    args[1]
                        .evaluate(&partition.rows[target_idx], &self.input_schema)
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as usize
                } else {
                    1
                };
                if local_idx >= offset {
                    let target_local_idx = local_idx - offset;
                    let target_idx = partition.indices[target_local_idx];
                    Ok(args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::FirstValue => {
                if let Some(&first_idx) = partition.indices.first() {
                    Ok(args[0].evaluate(&partition.rows[first_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::LastValue => {
                if let Some(&last_idx) = partition.indices.last() {
                    Ok(args[0].evaluate(&partition.rows[last_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::NthValue => {
                let n = args
                    .get(1)
                    .and_then(|e| {
                        let target_idx = partition.indices[local_idx];
                        e.evaluate(&partition.rows[target_idx], &self.input_schema)
                    })
                    .and_then(|v| v.as_integer())
                    .unwrap_or(1) as usize;
                if n > 0 && n <= partition.indices.len() {
                    let target_idx = partition.indices[n - 1];
                    Ok(args[0].evaluate(&partition.rows[target_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            // Note: Aggregate window functions (Sum, Avg, Count, Min, Max)
            // are handled through aggregate expression evaluation
            // For now, return NULL for unhandled cases
            _ => Ok(Value::Null),
        }
    }

    fn compute_agg<F>(
        &self,
        args: &[Expr],
        partition: &PartitionState,
        local_idx: usize,
        _frame: &Option<WindowFrame>,
        f: F,
    ) -> SqlResult<Value>
    where
        F: Fn(&[Value]) -> Value,
    {
        // For simplicity, use entire partition as frame
        let start_idx = 0;
        let end_idx = partition.indices.len();

        // Collect values for aggregation
        let values: Vec<Value> = partition.indices[start_idx..end_idx]
            .iter()
            .map(|&idx| {
                if args.is_empty() {
                    Value::Integer(1)
                } else {
                    args[0]
                        .evaluate(&partition.rows[idx], &self.input_schema)
                        .unwrap_or(Value::Null)
                }
            })
            .collect();

        Ok(f(&values))
    }
}

impl VolcanoExecutor for WindowVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        // Initialize child executor
        self.child.init()?;
        // Execute and cache all results
        let result = self.execute_internal()?;
        self.current_rows = result.rows;
        self.current_position = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            self.init()?;
        }

        if self.current_position < self.current_rows.len() {
            let row = self.current_rows[self.current_position].clone();
            self.current_position += 1;
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.child.close()?;
        self.current_rows.clear();
        self.partition_cache.clear();
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "WindowVolcanoExecutor"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}