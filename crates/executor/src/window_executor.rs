// SQLRustGo window function executor module

use crate::executor::{ExecutorResult, VolcanoExecutor};
use sqlrustgo_planner::{Expr, FrameBound, Schema, SortExpr, WindowFrame, WindowFunction};
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
            WindowFunction::Rank => self.compute_rank(partition, local_idx),
            WindowFunction::DenseRank => self.compute_dense_rank(partition, local_idx),
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
                    Ok(args[0]
                        .evaluate(&partition.rows[target_idx], &self.input_schema)
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
                    Ok(args[0]
                        .evaluate(&partition.rows[target_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::FirstValue => {
                let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;
                if let Some(&first_idx) = frame_rows.first() {
                    Ok(args[0]
                        .evaluate(&partition.rows[first_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::LastValue => {
                let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;
                if let Some(&last_idx) = frame_rows.last() {
                    Ok(args[0]
                        .evaluate(&partition.rows[last_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::NthValue => {
                let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;
                let n = args
                    .get(1)
                    .and_then(|e| {
                        let target_idx = partition.indices[local_idx];
                        e.evaluate(&partition.rows[target_idx], &self.input_schema)
                    })
                    .and_then(|v| v.as_integer())
                    .unwrap_or(1) as usize;
                if n > 0 && n <= frame_rows.len() {
                    let target_idx = frame_rows[n - 1];
                    Ok(args[0]
                        .evaluate(&partition.rows[target_idx], &self.input_schema)
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            // Aggregate window functions
            WindowFunction::Sum => self.compute_agg(args, partition, local_idx, frame, |vals| {
                let mut sum = 0i64;
                for v in vals {
                    if let Some(n) = v.as_integer() {
                        sum += n;
                    }
                }
                Value::Integer(sum)
            }),
            WindowFunction::Avg => self.compute_agg(args, partition, local_idx, frame, |vals| {
                let mut sum = 0i64;
                let mut count = 0i64;
                for v in vals {
                    if let Some(n) = v.as_integer() {
                        sum += n;
                        count += 1;
                    }
                }
                if count > 0 {
                    Value::Float(sum as f64 / count as f64)
                } else {
                    Value::Null
                }
            }),
            WindowFunction::Count => self.compute_agg(args, partition, local_idx, frame, |vals| {
                let count = vals.iter().filter(|v| !matches!(v, Value::Null)).count() as i64;
                Value::Integer(count)
            }),
            WindowFunction::Min => self.compute_agg(args, partition, local_idx, frame, |vals| {
                let mut min: Option<i64> = None;
                for v in vals {
                    if let Some(n) = v.as_integer() {
                        min = Some(min.map(|m| n.min(m)).unwrap_or(n));
                    }
                }
                min.map(Value::Integer).unwrap_or(Value::Null)
            }),
            WindowFunction::Max => self.compute_agg(args, partition, local_idx, frame, |vals| {
                let mut max: Option<i64> = None;
                for v in vals {
                    if let Some(n) = v.as_integer() {
                        max = Some(max.map(|m| n.max(m)).unwrap_or(n));
                    }
                }
                max.map(Value::Integer).unwrap_or(Value::Null)
            }),
        }
    }

    /// Compute rank with proper ranking logic (1, 2, 2, 5 for ties)
    fn compute_rank(&self, partition: &PartitionState, local_idx: usize) -> SqlResult<Value> {
        if local_idx == 0 {
            return Ok(Value::Integer(1));
        }
        // Get current row's order_by value
        let current_row_idx = partition.indices[local_idx];
        let current_row = &partition.rows[current_row_idx];

        // Count how many rows have strictly smaller order_by values
        let mut rank = 1i64;
        for i in 0..local_idx {
            let prev_row_idx = partition.indices[i];
            let prev_row = &partition.rows[prev_row_idx];
            // Compare with order_by columns
            for sort_expr in &self.order_by {
                let val_curr = sort_expr.expr.evaluate(current_row, &self.input_schema);
                let val_prev = sort_expr.expr.evaluate(prev_row, &self.input_schema);
                // If prev_row is less than current_row (current row is greater), increment rank
                let is_less = match (val_curr, val_prev) {
                    (Some(vc), Some(vp)) => {
                        let cmp = if sort_expr.asc {
                            vp.cmp(&vc)
                        } else {
                            vp.cmp(&vc).reverse()
                        };
                        cmp == std::cmp::Ordering::Less
                    }
                    (Some(_), None) => false,
                    (None, Some(_)) => true,
                    (None, None) => false,
                };
                if is_less {
                    rank += 1;
                    break;
                }
            }
        }
        Ok(Value::Integer(rank))
    }

    /// Compute dense rank without gaps (1, 2, 2, 3 for ties)
    fn compute_dense_rank(&self, partition: &PartitionState, local_idx: usize) -> SqlResult<Value> {
        if local_idx == 0 {
            return Ok(Value::Integer(1));
        }
        let current_row_idx = partition.indices[local_idx];
        let current_row = &partition.rows[current_row_idx];

        // Get current row's order_by values
        let mut current_vals: Vec<Option<Value>> = Vec::new();
        for sort_expr in &self.order_by {
            current_vals.push(sort_expr.expr.evaluate(current_row, &self.input_schema));
        }

        // Count distinct order_by values strictly smaller than current
        let mut smaller_distinct_values: Vec<Vec<Option<Value>>> = Vec::new();
        for i in 0..local_idx {
            let prev_row_idx = partition.indices[i];
            let prev_row = &partition.rows[prev_row_idx];
            let mut prev_vals: Vec<Option<Value>> = Vec::new();
            for sort_expr in &self.order_by {
                prev_vals.push(sort_expr.expr.evaluate(prev_row, &self.input_schema));
            }

            // Only count values strictly smaller than current
            if self.values_less_than(&prev_vals, &current_vals) {
                // Check if this is a new distinct value
                let mut is_new = true;
                for existing in &smaller_distinct_values {
                    if existing == &prev_vals {
                        is_new = false;
                        break;
                    }
                }
                if is_new {
                    smaller_distinct_values.push(prev_vals);
                }
            }
        }
        Ok(Value::Integer((smaller_distinct_values.len() + 1) as i64))
    }

    /// Compare if values1 < values2 (for all order_by expressions)
    fn values_less_than(&self, values1: &[Option<Value>], values2: &[Option<Value>]) -> bool {
        if values1.len() != values2.len() {
            return false;
        }
        for i in 0..values1.len() {
            let v1 = &values1[i];
            let v2 = &values2[i];
            if let (Some(a), Some(b)) = (v1, v2) {
                if let Some(ord) = a.partial_cmp(b) {
                    if ord == std::cmp::Ordering::Greater {
                        return false;
                    }
                    if ord == std::cmp::Ordering::Less {
                        // Found a smaller value, check if all following are also not greater
                        let mut all_not_greater = true;
                        for j in (i + 1)..values1.len() {
                            let (next_v1, next_v2) = (&values1[j], &values2[j]);
                            if let (Some(na), Some(nb)) = (next_v1, next_v2) {
                                if let Some(next_ord) = na.partial_cmp(nb) {
                                    if next_ord == std::cmp::Ordering::Greater {
                                        all_not_greater = false;
                                    }
                                }
                            }
                        }
                        if all_not_greater {
                            return true;
                        }
                    }
                }
            }
        }
        // Actually simpler: check if ALL values are less than or equal, and at least one is less
        let mut any_less = false;
        for i in 0..values1.len() {
            let v1 = &values1[i];
            let v2 = &values2[i];
            if let (Some(a), Some(b)) = (v1, v2) {
                if let Some(ord) = a.partial_cmp(b) {
                    if ord == std::cmp::Ordering::Greater {
                        return false;
                    }
                    if ord == std::cmp::Ordering::Less {
                        any_less = true;
                    }
                }
            }
        }
        any_less
    }

    /// Get frame rows based on window frame specification
    fn get_frame_rows(
        &self,
        partition: &PartitionState,
        local_idx: usize,
        frame: &Option<WindowFrame>,
    ) -> SqlResult<Vec<usize>> {
        let partition_size = partition.indices.len();

        // Default frame: ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
        let (start_bound, end_bound) = match frame {
            Some(WindowFrame::Rows { start, end, .. }) => (start, end),
            Some(WindowFrame::Range { start, end, .. }) => (start, end),
            Some(WindowFrame::Groups { start, end, .. }) => (start, end),
            None => {
                // Default frame: UNBOUNDED PRECEDING to CURRENT ROW
                return Ok(partition.indices[..=local_idx].to_vec());
            }
        };

        let start_idx = match start_bound {
            FrameBound::UnboundedPreceding => 0,
            FrameBound::Preceding(n) => {
                let offset = *n as usize;
                local_idx.saturating_sub(offset)
            }
            FrameBound::CurrentRow => local_idx,
            FrameBound::Following(n) => {
                let offset = *n as usize;
                (local_idx + offset).min(partition_size)
            }
            FrameBound::UnboundedFollowing => 0,
        };

        let end_idx = match end_bound {
            FrameBound::UnboundedPreceding => 0,
            FrameBound::Preceding(n) => {
                let offset = *n as usize;
                local_idx.saturating_sub(offset)
            }
            FrameBound::CurrentRow => local_idx,
            FrameBound::Following(n) => {
                let offset = *n as usize;
                (local_idx + offset).min(partition_size - 1)
            }
            FrameBound::UnboundedFollowing => partition_size - 1,
        };

        if start_idx > end_idx {
            return Ok(Vec::new());
        }

        Ok(partition.indices[start_idx..=end_idx].to_vec())
    }

    fn compute_agg<F>(
        &self,
        args: &[Expr],
        partition: &PartitionState,
        local_idx: usize,
        frame: &Option<WindowFrame>,
        f: F,
    ) -> SqlResult<Value>
    where
        F: Fn(&[Value]) -> Value,
    {
        // Get frame rows based on window frame specification
        let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;

        // Collect values for aggregation
        let values: Vec<Value> = frame_rows
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Expr, SortExpr};

    fn create_test_partition() -> PartitionState {
        // Create test rows: (id, value)
        let rows = vec![
            vec![Value::Integer(1), Value::Integer(100)],
            vec![Value::Integer(2), Value::Integer(200)],
            vec![Value::Integer(3), Value::Integer(100)],
            vec![Value::Integer(4), Value::Integer(300)],
            vec![Value::Integer(5), Value::Integer(100)],
        ];
        let indices = vec![0, 1, 2, 3, 4];
        PartitionState { rows, indices }
    }

    #[test]
    fn test_row_number() {
        let partition = create_test_partition();
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            Schema::empty(),
            vec![],
            vec![],
        );

        for i in 0..5 {
            let result = executor.compute_window_function(
                &WindowFunction::RowNumber,
                &[],
                &partition,
                i,
                &None,
            );
            assert_eq!(result.unwrap(), Value::Integer((i + 1) as i64));
        }
    }

    #[test]
    fn test_rank_with_ties() {
        // Create partition with ties in order_by values
        let rows = vec![
            vec![Value::Integer(1), Value::Integer(100)],
            vec![Value::Integer(2), Value::Integer(200)],
            vec![Value::Integer(3), Value::Integer(200)],
            vec![Value::Integer(4), Value::Integer(300)],
            vec![Value::Integer(5), Value::Integer(400)],
        ];
        let indices = vec![0, 1, 2, 3, 4];
        let partition = PartitionState { rows, indices };

        // Use index-based evaluation (column 1)
        let order_by = vec![SortExpr {
            expr: Expr::Column(Column {
                relation: None,
                name: "value".to_string(),
            }),
            asc: true,
            nulls_first: false,
        }];
        // Provide schema with field to enable evaluation
        let input_schema = Schema::new(vec![
            sqlrustgo_planner::Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            sqlrustgo_planner::Field::new(
                "value".to_string(),
                sqlrustgo_planner::DataType::Integer,
            ),
        ]);
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            input_schema,
            vec![],
            order_by,
        );

        // Row 1 (value=100): rank 1
        let result = executor.compute_rank(&partition, 0).unwrap();
        assert_eq!(result, Value::Integer(1));

        // Row 2 (value=200): rank 2 (skips 1)
        let result = executor.compute_rank(&partition, 1).unwrap();
        assert_eq!(result, Value::Integer(2));

        // Row 3 (value=200): rank 2 (same as row 2)
        let result = executor.compute_rank(&partition, 2).unwrap();
        assert_eq!(result, Value::Integer(2));

        // Row 4 (value=300): rank 4 (skips 1, 2, 3)
        let result = executor.compute_rank(&partition, 3).unwrap();
        assert_eq!(result, Value::Integer(4));

        // Row 5 (value=400): rank 5
        let result = executor.compute_rank(&partition, 4).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_dense_rank_with_ties() {
        let rows = vec![
            vec![Value::Integer(1), Value::Integer(100)],
            vec![Value::Integer(2), Value::Integer(200)],
            vec![Value::Integer(3), Value::Integer(200)],
            vec![Value::Integer(4), Value::Integer(300)],
            vec![Value::Integer(5), Value::Integer(400)],
        ];
        let indices = vec![0, 1, 2, 3, 4];
        let partition = PartitionState { rows, indices };

        let order_by = vec![SortExpr {
            expr: Expr::Column(Column {
                relation: None,
                name: "value".to_string(),
            }),
            asc: true,
            nulls_first: false,
        }];
        let input_schema = Schema::new(vec![
            sqlrustgo_planner::Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            sqlrustgo_planner::Field::new(
                "value".to_string(),
                sqlrustgo_planner::DataType::Integer,
            ),
        ]);
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            input_schema,
            vec![],
            order_by,
        );

        // Dense rank: 1, 2, 2, 3, 4 (no gaps)
        let results: Vec<Value> = (0..5)
            .map(|i| executor.compute_dense_rank(&partition, i).unwrap())
            .collect();

        assert_eq!(results[0], Value::Integer(1));
        assert_eq!(results[1], Value::Integer(2));
        assert_eq!(results[2], Value::Integer(2));
        assert_eq!(results[3], Value::Integer(3));
        assert_eq!(results[4], Value::Integer(4));
    }

    #[test]
    fn test_get_frame_rows_default() {
        let partition = create_test_partition();
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            Schema::empty(),
            vec![],
            vec![],
        );

        // Default frame: UNBOUNDED PRECEDING to CURRENT ROW
        let frame_rows = executor.get_frame_rows(&partition, 2, &None).unwrap();
        // Should include indices 0, 1, 2 (local_idx=2)
        assert_eq!(frame_rows.len(), 3);
    }

    #[test]
    fn test_get_frame_rows_with_offset() {
        let partition = create_test_partition();
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            Schema::empty(),
            vec![],
            vec![],
        );

        // Frame: ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING
        let frame = WindowFrame::Rows {
            start: FrameBound::Preceding(1),
            end: FrameBound::Following(1),
            exclude: ExcludeMode::None,
        };

        let frame_rows = executor
            .get_frame_rows(&partition, 2, &Some(frame))
            .unwrap();
        // Should include indices 1, 2, 3 (local_idx=2, 1 before, 1 after)
        assert_eq!(frame_rows.len(), 3);
    }

    #[test]
    fn test_aggregate_window_sum() {
        let partition = create_test_partition();
        let input_schema = Schema::new(vec![
            sqlrustgo_planner::Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            sqlrustgo_planner::Field::new(
                "value".to_string(),
                sqlrustgo_planner::DataType::Integer,
            ),
        ]);
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            input_schema,
            vec![],
            vec![],
        );

        let sum = executor.compute_window_function(
            &WindowFunction::Sum,
            &[Expr::Column(Column {
                relation: None,
                name: "value".to_string(),
            })],
            &partition,
            2,
            &None,
        );
        // Default frame includes rows 0,1,2 with values 100,200,100 = 400
        assert_eq!(sum.unwrap(), Value::Integer(400));
    }

    #[test]
    fn test_aggregate_window_count() {
        let partition = create_test_partition();
        let input_schema = Schema::new(vec![
            sqlrustgo_planner::Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            sqlrustgo_planner::Field::new(
                "value".to_string(),
                sqlrustgo_planner::DataType::Integer,
            ),
        ]);
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            input_schema,
            vec![],
            vec![],
        );

        let count = executor.compute_window_function(
            &WindowFunction::Count,
            &[Expr::Column(Column {
                relation: None,
                name: "value".to_string(),
            })],
            &partition,
            2,
            &None,
        );
        // Default frame includes rows 0,1,2 = 3 rows
        assert_eq!(count.unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_aggregate_window_avg() {
        let partition = create_test_partition();
        let input_schema = Schema::new(vec![
            sqlrustgo_planner::Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            sqlrustgo_planner::Field::new(
                "value".to_string(),
                sqlrustgo_planner::DataType::Integer,
            ),
        ]);
        let executor = WindowVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            vec![],
            Schema::empty(),
            input_schema,
            vec![],
            vec![],
        );

        let avg = executor.compute_window_function(
            &WindowFunction::Avg,
            &[Expr::Column(Column {
                relation: None,
                name: "value".to_string(),
            })],
            &partition,
            2,
            &None,
        );
        // Default frame includes rows 0,1,2 with values 100,200,100, avg = 400/3
        let expected = Value::Float(400.0 / 3.0);
        assert!((avg.unwrap().as_float().unwrap() - expected.as_float().unwrap()).abs() < 0.001);
    }

    // Mock executor for tests
    struct MockExecutor {
        schema: Schema,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                schema: Schema::empty(),
            }
        }
    }

    impl VolcanoExecutor for MockExecutor {
        fn init(&mut self) -> SqlResult<()> {
            Ok(())
        }

        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            Ok(None)
        }

        fn close(&mut self) -> SqlResult<()> {
            Ok(())
        }

        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn name(&self) -> &str {
            "MockExecutor"
        }

        fn is_initialized(&self) -> bool {
            true
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}
