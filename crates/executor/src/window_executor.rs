// SQLRustGo window function executor module

use crate::executor::VolcanoExecutor;
use sqlrustgo_planner::{Expr as PlannerExpr, FrameBound, Schema, SortExpr as PlannerSortExpr, WindowFrame, WindowFunction};
use sqlrustgo_types::{SqlResult, Value};
use std::collections::HashMap;

pub struct WindowVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    window_exprs: Vec<PlannerExpr>,
    input_schema: Schema,
    partition_by: Vec<PlannerExpr>,
    order_by: Vec<PlannerSortExpr>,
    partition_cache: HashMap<Vec<Value>, PartitionState>,
    current_rows: Vec<Vec<Value>>,
    current_position: usize,
    initialized: bool,
}

struct PartitionState {
    rows: Vec<Vec<Value>>,
    indices: Vec<usize>,
}

impl WindowVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        window_exprs: Vec<PlannerExpr>,
        input_schema: Schema,
        partition_by: Vec<PlannerExpr>,
        order_by: Vec<PlannerSortExpr>,
    ) -> Self {
        Self {
            child,
            window_exprs,
            input_schema,
            partition_by,
            order_by,
            partition_cache: HashMap::new(),
            current_rows: Vec::new(),
            current_position: 0,
            initialized: false,
        }
    }

    fn compute_partitions(&mut self, rows: &[Vec<Value>]) -> SqlResult<()> {
        self.partition_cache.clear();

        for (idx, row) in rows.iter().enumerate() {
            let partition_key: Vec<Value> = if self.partition_by.is_empty() {
                Vec::new()
            } else {
                self.partition_by
                    .iter()
                    .filter_map(|expr| self.evaluate_planner_expr(expr, row))
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
                        let val_a = Self::eval_expr_static(&sort_expr.expr, row_a, &input_schema);
                        let val_b = Self::eval_expr_static(&sort_expr.expr, row_b, &input_schema);

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

    fn evaluate_planner_expr(&self, expr: &PlannerExpr, row: &[Value]) -> Option<Value> {
        Self::eval_expr_static(expr, row, &self.input_schema)
    }

    fn eval_expr_static(expr: &PlannerExpr, row: &[Value], schema: &Schema) -> Option<Value> {
        match expr {
            PlannerExpr::Column(col) => {
                if let Some(idx) = schema.field_index(&col.name) {
                    row.get(idx).cloned()
                } else {
                    None
                }
            }
            PlannerExpr::Literal(v) => Some(v.clone()),
            PlannerExpr::BinaryExpr { left, op, right } => {
                let l = Self::eval_expr_static(left, row, schema)?;
                let r = Self::eval_expr_static(right, row, schema)?;
                Some(eval_binary_op(&l, op, &r))
            }
            PlannerExpr::UnaryExpr { op, expr } => {
                let v = Self::eval_expr_static(expr, row, schema)?;
                Some(eval_unary_op(op, &v))
            }
            _ => None,
        }
    }

    fn compute_window_expression(
        &self,
        expr: &PlannerExpr,
        partition: &PartitionState,
        local_idx: usize,
    ) -> SqlResult<Value> {
        match expr {
            PlannerExpr::WindowFunction {
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
        args: &[PlannerExpr],
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
                    self.evaluate_planner_expr(&args[1], &partition.rows[partition.indices[local_idx]])
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as usize
                } else {
                    1
                };
                let target_local_idx = local_idx + offset;
                if target_local_idx < partition.indices.len() {
                    let target_idx = partition.indices[target_local_idx];
                    Ok(self.evaluate_planner_expr(&args[0], &partition.rows[target_idx])
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::Lag => {
                let offset = if args.len() > 1 {
                    self.evaluate_planner_expr(&args[1], &partition.rows[partition.indices[local_idx]])
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as usize
                } else {
                    1
                };
                if local_idx >= offset {
                    let target_local_idx = local_idx - offset;
                    let target_idx = partition.indices[target_local_idx];
                    Ok(self.evaluate_planner_expr(&args[0], &partition.rows[target_idx])
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::FirstValue => {
                let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;
                if let Some(&first_idx) = frame_rows.first() {
                    Ok(self.evaluate_planner_expr(&args[0], &partition.rows[first_idx])
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            WindowFunction::LastValue => {
                let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;
                if let Some(&last_idx) = frame_rows.last() {
                    Ok(self.evaluate_planner_expr(&args[0], &partition.rows[last_idx])
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
                        self.evaluate_planner_expr(e, &partition.rows[partition.indices[local_idx]])
                    })
                    .and_then(|v| v.as_integer())
                    .unwrap_or(1) as usize;
                if n > 0 && n <= frame_rows.len() {
                    let target_idx = frame_rows[n - 1];
                    Ok(self.evaluate_planner_expr(&args[0], &partition.rows[target_idx])
                        .unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
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
            WindowFunction::Ntile => {
                let num_buckets = args
                    .first()
                    .and_then(|e| {
                        self.evaluate_planner_expr(e, &partition.rows[partition.indices[local_idx]])
                    })
                    .and_then(|v| v.as_integer())
                    .unwrap_or(1) as usize;
                if num_buckets == 0 {
                    return Ok(Value::Null);
                }
                let total_rows = partition.indices.len();
                if total_rows == 0 {
                    return Ok(Value::Null);
                }
                let row_number = local_idx + 1;
                let bucket = ((row_number - 1) * num_buckets / total_rows) + 1;
                Ok(Value::Integer(bucket as i64))
            }
        }
    }

    fn compute_rank(&self, partition: &PartitionState, local_idx: usize) -> SqlResult<Value> {
        if local_idx == 0 {
            return Ok(Value::Integer(1));
        }
        let current_row_idx = partition.indices[local_idx];
        let current_row = &partition.rows[current_row_idx];

        let mut rank = 1i64;
        for i in 0..local_idx {
            let prev_row_idx = partition.indices[i];
            let prev_row = &partition.rows[prev_row_idx];
            for sort_expr in &self.order_by {
                let val_curr = self.evaluate_planner_expr(&sort_expr.expr, current_row);
                let val_prev = self.evaluate_planner_expr(&sort_expr.expr, prev_row);
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

    fn compute_dense_rank(&self, partition: &PartitionState, local_idx: usize) -> SqlResult<Value> {
        if local_idx == 0 {
            return Ok(Value::Integer(1));
        }
        let current_row_idx = partition.indices[local_idx];
        let current_row = &partition.rows[current_row_idx];

        let mut current_vals: Vec<Option<Value>> = Vec::new();
        for sort_expr in &self.order_by {
            current_vals.push(self.evaluate_planner_expr(&sort_expr.expr, current_row));
        }

        let mut smaller_distinct_values: Vec<Vec<Option<Value>>> = Vec::new();
        for i in 0..local_idx {
            let prev_row_idx = partition.indices[i];
            let prev_row = &partition.rows[prev_row_idx];
            let mut prev_vals: Vec<Option<Value>> = Vec::new();
            for sort_expr in &self.order_by {
                prev_vals.push(self.evaluate_planner_expr(&sort_expr.expr, prev_row));
            }

            if self.values_less_than(&prev_vals, &current_vals) {
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

    fn values_less_than(&self, values1: &[Option<Value>], values2: &[Option<Value>]) -> bool {
        if values1.len() != values2.len() {
            return false;
        }
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

    fn get_frame_rows(
        &self,
        partition: &PartitionState,
        local_idx: usize,
        frame: &Option<WindowFrame>,
    ) -> SqlResult<Vec<usize>> {
        let partition_size = partition.indices.len();

        let (start_bound, end_bound) = match frame {
            Some(f) => (&f.start, &f.end),
            None => {
                return Ok(partition.indices[..=local_idx].to_vec());
            }
        };

        let start_idx = match start_bound {
            FrameBound::UnboundedPreceding => 0,
            FrameBound::Preceding(n) => local_idx.saturating_sub(*n as usize),
            FrameBound::CurrentRow => local_idx,
            FrameBound::Following(n) => (local_idx + *n as usize).min(partition_size),
        };

        let end_idx = match end_bound {
            FrameBound::UnboundedPreceding => 0,
            FrameBound::Preceding(n) => local_idx.saturating_sub(*n as usize),
            FrameBound::CurrentRow => local_idx,
            FrameBound::Following(n) => (local_idx + *n as usize).min(partition_size.saturating_sub(1)),
        };

        if start_idx > end_idx {
            return Ok(Vec::new());
        }

        Ok(partition.indices[start_idx..=end_idx].to_vec())
    }

    fn compute_agg<F>(
        &self,
        args: &[PlannerExpr],
        partition: &PartitionState,
        local_idx: usize,
        frame: &Option<WindowFrame>,
        f: F,
    ) -> SqlResult<Value>
    where
        F: Fn(&[Value]) -> Value,
    {
        let frame_rows = self.get_frame_rows(partition, local_idx, frame)?;

        let values: Vec<Value> = frame_rows
            .iter()
            .map(|&idx| {
                if args.is_empty() {
                    Value::Integer(1)
                } else {
                    self.evaluate_planner_expr(&args[0], &partition.rows[idx])
                        .unwrap_or(Value::Null)
                }
            })
            .collect();

        Ok(f(&values))
    }
}

fn eval_binary_op(l: &Value, op: &sqlrustgo_planner::Operator, r: &Value) -> Value {
    use sqlrustgo_planner::Operator;
    match op {
        Operator::Eq => Value::Boolean(l == r),
        Operator::NotEq => Value::Boolean(l != r),
        Operator::Lt => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a < b)
            } else {
                Value::Null
            }
        }
        Operator::LtEq => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a <= b)
            } else {
                Value::Null
            }
        }
        Operator::Gt => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a > b)
            } else {
                Value::Null
            }
        }
        Operator::GtEq => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a >= b)
            } else {
                Value::Null
            }
        }
        Operator::Plus => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Integer(a + b)
            } else {
                Value::Null
            }
        }
        Operator::Minus => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Integer(a - b)
            } else {
                Value::Null
            }
        }
        Operator::Multiply => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Integer(a * b)
            } else {
                Value::Null
            }
        }
        Operator::Divide => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                if b == 0 {
                    Value::Null
                } else {
                    Value::Integer(a / b)
                }
            } else {
                Value::Null
            }
        }
        Operator::And => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a != 0 && b != 0)
            } else {
                Value::Null
            }
        }
        Operator::Or => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                Value::Boolean(a != 0 || b != 0)
            } else {
                Value::Null
            }
        }
        Operator::Not => Value::Null,
        Operator::Like => Value::Null,
        Operator::Modulo => {
            if let (Some(a), Some(b)) = (l.as_integer(), r.as_integer()) {
                if b == 0 {
                    Value::Null
                } else {
                    Value::Integer(a % b)
                }
            } else {
                Value::Null
            }
        }
    }
}

fn eval_unary_op(op: &sqlrustgo_planner::Operator, v: &Value) -> Value {
    use sqlrustgo_planner::Operator;
    match op {
        Operator::Minus => {
            if let Some(n) = v.as_integer() {
                Value::Integer(-n)
            } else {
                Value::Null
            }
        }
        Operator::Not => {
            if let Some(n) = v.as_integer() {
                Value::Integer(if n == 0 { 1 } else { 0 })
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

impl VolcanoExecutor for WindowVolcanoExecutor {
    fn open(&mut self) -> SqlResult<()> {
        self.current_position = 0;
        self.initialized = false;
        self.current_rows.clear();
        self.child.open()
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            let mut all_rows = Vec::new();
            self.child.open()?;
            while let Some(row) = self.child.next()? {
                all_rows.push(row);
            }
            self.child.close()?;

            if !all_rows.is_empty() {
                self.compute_partitions(&all_rows)?;
                let mut partition_keys: Vec<_> = self.partition_cache.keys().cloned().collect();
                partition_keys.sort();
                for partition_key in partition_keys {
                    let partition_state = self.partition_cache.get(&partition_key).unwrap();
                    for row_idx in &partition_state.indices {
                        let mut output_row = partition_state.rows[*row_idx].clone();
                        for expr in &self.window_exprs {
                            let value = self.compute_window_expression(
                                expr,
                                partition_state,
                                partition_state.indices.iter().position(|&i| i == *row_idx).unwrap(),
                            )?;
                            output_row.push(value);
                        }
                        self.current_rows.push(output_row);
                    }
                }
            }
            self.initialized = true;
            self.current_position = 0;
        }

        if self.current_position >= self.current_rows.len() {
            return Ok(None);
        }
        let row = self.current_rows[self.current_position].clone();
        self.current_position += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.current_position = 0;
        self.initialized = false;
        self.current_rows.clear();
        self.partition_cache.clear();
        self.child.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Column, DataType, Field};

    struct MockExecutor;

    impl MockExecutor {
        fn new() -> Self {
            Self
        }
    }

    impl VolcanoExecutor for MockExecutor {
        fn open(&mut self) -> SqlResult<()> {
            Ok(())
        }

        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            Ok(None)
        }

        fn close(&mut self) -> SqlResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_window_volcano_lifecycle() {
        let child = Box::new(MockExecutor::new());
        let _executor = WindowVolcanoExecutor::new(
            child,
            vec![],
            Schema::empty(),
            vec![],
            vec![],
        );
    }

    #[test]
    fn test_evaluate_column() {
        let child = Box::new(MockExecutor::new());
        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let executor = WindowVolcanoExecutor::new(
            child,
            vec![],
            input_schema,
            vec![],
            vec![],
        );

        let row = vec![Value::Integer(1), Value::Integer(100)];
        let result = executor.evaluate_planner_expr(
            &PlannerExpr::Column(Column::new("value".to_string())),
            &row,
        );
        assert_eq!(result, Some(Value::Integer(100)));
    }

    #[test]
    fn test_evaluate_literal() {
        let child = Box::new(MockExecutor::new());
        let executor = WindowVolcanoExecutor::new(
            child,
            vec![],
            Schema::empty(),
            vec![],
            vec![],
        );

        let row = vec![];
        let result = executor.evaluate_planner_expr(
            &PlannerExpr::Literal(Value::Integer(42)),
            &row,
        );
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_evaluate_binary_expr() {
        let child = Box::new(MockExecutor::new());
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let executor = WindowVolcanoExecutor::new(
            child,
            vec![],
            input_schema,
            vec![],
            vec![],
        );

        let row = vec![Value::Integer(10), Value::Integer(3)];
        let result = executor.evaluate_planner_expr(
            &PlannerExpr::BinaryExpr {
                left: Box::new(PlannerExpr::Column(Column::new("a".to_string()))),
                op: sqlrustgo_planner::Operator::Plus,
                right: Box::new(PlannerExpr::Column(Column::new("b".to_string()))),
            },
            &row,
        );
        assert_eq!(result, Some(Value::Integer(13)));
    }

    #[test]
    fn test_row_number() {
        let child = Box::new(MockExecutor::new());
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let executor = WindowVolcanoExecutor::new(
            child,
            vec![],
            input_schema,
            vec![],
            vec![],
        );

        let partition = PartitionState {
            rows: vec![vec![Value::Integer(1)], vec![Value::Integer(2)], vec![Value::Integer(3)]],
            indices: vec![0, 1, 2],
        };

        for i in 0..3 {
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
}
