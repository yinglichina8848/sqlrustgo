//! Physical Plan Definitions
//!
//! Defines the PhysicalPlan trait and concrete physical operator implementations.

use super::{Column, Expr, JoinType, Schema};
use crate::types::{SqlResult, Value};
use std::fmt;
use std::sync::Arc;

pub trait PhysicalPlan: Send + Sync {
    fn schema(&self) -> &Schema;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
    fn execute(&self) -> SqlResult<Vec<Vec<Value>>>;
}

pub struct SeqScanExec {
    pub table_name: String,
    pub projection: Option<Vec<usize>>,
    pub filters: Vec<Expr>,
    pub limit: Option<usize>,
    pub schema: Schema,
    pub rows: Vec<Vec<Value>>,
}

impl fmt::Debug for SeqScanExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SeqScanExec")
            .field("table_name", &self.table_name)
            .field("schema", &self.schema)
            .finish()
    }
}

impl SeqScanExec {
    pub fn new(
        table_name: String,
        schema: Schema,
        rows: Vec<Vec<Value>>,
        projection: Option<Vec<usize>>,
        filters: Vec<Expr>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            table_name,
            projection,
            filters,
            limit,
            schema,
            rows,
        }
    }
}

impl PhysicalPlan for SeqScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let mut result = self.rows.clone();

        if let Some(ref projection) = self.projection {
            result = result
                .iter()
                .map(|row| {
                    projection
                        .iter()
                        .filter_map(|&idx| row.get(idx).cloned())
                        .collect()
                })
                .collect();
        }

        if !self.filters.is_empty() {
            let column_map: std::collections::HashMap<String, usize> = self
                .schema
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| (f.name.clone(), i))
                .collect();

            result.retain(|row| evaluate_filters(row, &self.filters, &column_map));
        }

        if let Some(limit) = self.limit {
            result.truncate(limit);
        }

        Ok(result)
    }
}

fn evaluate_filters(
    row: &[Value],
    filters: &[Expr],
    column_map: &std::collections::HashMap<String, usize>,
) -> bool {
    filters
        .iter()
        .all(|filter| evaluate_expr(row, filter, column_map))
}

fn evaluate_expr(
    row: &[Value],
    expr: &Expr,
    column_map: &std::collections::HashMap<String, usize>,
) -> bool {
    match expr {
        Expr::BinaryExpr { left, op, right } => {
            let left_val = evaluate_expr(row, left, column_map);
            let right_val = evaluate_expr(row, right, column_map);
            match op {
                super::Operator::And => left_val && right_val,
                super::Operator::Or => left_val || right_val,
                _ => false,
            }
        }
        Expr::Column(col) => {
            if let Some(&idx) = column_map.get(&col.name) {
                if let Some(val) = row.get(idx) {
                    return !matches!(val, Value::Null);
                }
            }
            false
        }
        _ => true,
    }
}

pub struct ProjectionExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub expr: Vec<Expr>,
    pub schema: Schema,
}

impl fmt::Debug for ProjectionExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProjectionExec")
            .field("schema", &self.schema)
            .finish()
    }
}

impl ProjectionExec {
    pub fn new(input: Arc<dyn PhysicalPlan>, expr: Vec<Expr>, schema: Schema) -> Self {
        Self {
            input,
            expr,
            schema,
        }
    }
}

impl PhysicalPlan for ProjectionExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let input_rows = self.input.execute()?;
        let input_schema = self.input.schema();
        let column_map: std::collections::HashMap<String, usize> = input_schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        let result: Vec<Vec<Value>> = input_rows
            .iter()
            .map(|row| {
                self.expr
                    .iter()
                    .map(|expr| evaluate_projection_expr(row, expr, &column_map))
                    .collect()
            })
            .collect();

        Ok(result)
    }
}

fn evaluate_projection_expr(
    row: &[Value],
    expr: &Expr,
    column_map: &std::collections::HashMap<String, usize>,
) -> Value {
    match expr {
        Expr::Column(col) => {
            if let Some(&idx) = column_map.get(&col.name) {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        Expr::Literal(val) => val.clone(),
        Expr::Alias { expr, .. } => evaluate_projection_expr(row, expr, column_map),
        _ => Value::Null,
    }
}

pub struct FilterExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub predicate: Expr,
}

impl fmt::Debug for FilterExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterExec")
            .field("predicate", &self.predicate)
            .finish()
    }
}

impl FilterExec {
    pub fn new(input: Arc<dyn PhysicalPlan>, predicate: Expr) -> Self {
        Self { input, predicate }
    }
}

impl PhysicalPlan for FilterExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let input_rows = self.input.execute()?;
        let input_schema = self.input.schema();
        let column_map: std::collections::HashMap<String, usize> = input_schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        let result: Vec<Vec<Value>> = input_rows
            .into_iter()
            .filter(|row| evaluate_predicate(row, &self.predicate, &column_map))
            .collect();

        Ok(result)
    }
}

fn evaluate_predicate(
    row: &[Value],
    expr: &Expr,
    column_map: &std::collections::HashMap<String, usize>,
) -> bool {
    match expr {
        Expr::BinaryExpr { left, op, right } => {
            let left_val = get_value(row, left, column_map);
            let right_val = get_value(row, right, column_map);
            match op {
                super::Operator::Eq => left_val == right_val,
                super::Operator::NotEq => left_val != right_val,
                super::Operator::Gt => compare_values(&left_val, &right_val) > 0,
                super::Operator::Lt => compare_values(&left_val, &right_val) < 0,
                super::Operator::GtEq => compare_values(&left_val, &right_val) >= 0,
                super::Operator::LtEq => compare_values(&left_val, &right_val) <= 0,
                super::Operator::And => {
                    evaluate_predicate(row, left, column_map)
                        && evaluate_predicate(row, right, column_map)
                }
                super::Operator::Or => {
                    evaluate_predicate(row, left, column_map)
                        || evaluate_predicate(row, right, column_map)
                }
                _ => false,
            }
        }
        Expr::Column(col) => {
            if let Some(&idx) = column_map.get(&col.name) {
                if let Some(val) = row.get(idx) {
                    return !matches!(val, Value::Null);
                }
            }
            false
        }
        _ => true,
    }
}

fn get_value(
    row: &[Value],
    expr: &Expr,
    column_map: &std::collections::HashMap<String, usize>,
) -> Value {
    match expr {
        Expr::Column(col) => {
            if let Some(&idx) = column_map.get(&col.name) {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        Expr::Literal(val) => val.clone(),
        _ => Value::Null,
    }
}

fn compare_values(left: &Value, right: &Value) -> i32 {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r) as i32,
        (Value::Float(l), Value::Float(r)) => {
            l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal) as i32
        }
        (Value::Text(l), Value::Text(r)) => l.cmp(r) as i32,
        _ => 0,
    }
}

#[derive(Debug, Clone)]
pub enum AggregateMode {
    Partial,
    Final,
    FinalPartitioned,
}

pub struct AggregateExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub group_expr: Vec<Expr>,
    pub aggr_expr: Vec<(super::AggregateFunction, Expr)>,
    pub schema: Schema,
    pub mode: AggregateMode,
}

impl fmt::Debug for AggregateExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AggregateExec")
            .field("schema", &self.schema)
            .field("mode", &self.mode)
            .finish()
    }
}

impl AggregateExec {
    pub fn new(
        input: Arc<dyn PhysicalPlan>,
        group_expr: Vec<Expr>,
        aggr_expr: Vec<(super::AggregateFunction, Expr)>,
        schema: Schema,
    ) -> Self {
        Self {
            input,
            group_expr,
            aggr_expr,
            schema,
            mode: AggregateMode::Partial,
        }
    }
}

impl PhysicalPlan for AggregateExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let input_rows = self.input.execute()?;
        let input_schema = self.input.schema();
        let column_map: std::collections::HashMap<String, usize> = input_schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        if self.group_expr.is_empty() {
            let mut result_row = Vec::new();
            for (func, expr) in &self.aggr_expr {
                let val = compute_aggregate(func, expr, &input_rows, &column_map);
                result_row.push(val);
            }
            return Ok(vec![result_row]);
        }

        let mut groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>> =
            std::collections::HashMap::new();
        for row in &input_rows {
            let key: Vec<Value> = self
                .group_expr
                .iter()
                .map(|expr| evaluate_projection_expr(row, expr, &column_map))
                .collect();
            groups.entry(key).or_default().push(row.clone());
        }

        let mut result = Vec::new();
        for (key, group_rows) in groups {
            let mut row = key.clone();
            for (func, expr) in &self.aggr_expr {
                let val = compute_aggregate(func, expr, &group_rows, &column_map);
                row.push(val);
            }
            result.push(row);
        }

        Ok(result)
    }
}

fn compute_aggregate(
    func: &super::AggregateFunction,
    expr: &Expr,
    rows: &[Vec<Value>],
    column_map: &std::collections::HashMap<String, usize>,
) -> Value {
    let values: Vec<Value> = rows
        .iter()
        .map(|row| evaluate_projection_expr(row, expr, column_map))
        .collect();

    match func {
        super::AggregateFunction::Count => Value::Integer(rows.len() as i64),
        super::AggregateFunction::Sum => {
            let sum: i64 = values.iter().filter_map(|v| v.as_integer()).sum();
            Value::Integer(sum)
        }
        super::AggregateFunction::Avg => {
            let sum: f64 = values
                .iter()
                .filter_map(|v| match v {
                    Value::Integer(i) => Some(*i as f64),
                    Value::Float(f) => Some(*f),
                    _ => None,
                })
                .sum();
            let count = values.iter().filter(|v| !matches!(v, Value::Null)).count();
            if count > 0 {
                Value::Float(sum / count as f64)
            } else {
                Value::Null
            }
        }
        super::AggregateFunction::Min => {
            let mut min: Option<Value> = None;
            for v in &values {
                if !matches!(v, Value::Null) {
                    match &min {
                        None => min = Some(v.clone()),
                        Some(m) => {
                            if compare_values(v, m) < 0 {
                                min = Some(v.clone());
                            }
                        }
                    }
                }
            }
            min.unwrap_or(Value::Null)
        }
        super::AggregateFunction::Max => {
            let mut max: Option<Value> = None;
            for v in &values {
                if !matches!(v, Value::Null) {
                    match &max {
                        None => max = Some(v.clone()),
                        Some(m) => {
                            if compare_values(v, m) > 0 {
                                max = Some(v.clone());
                            }
                        }
                    }
                }
            }
            max.unwrap_or(Value::Null)
        }
    }
}

pub struct HashJoinExec {
    pub left: Arc<dyn PhysicalPlan>,
    pub right: Arc<dyn PhysicalPlan>,
    pub on: Vec<(Column, Column)>,
    pub join_type: JoinType,
    pub schema: Schema,
}

impl fmt::Debug for HashJoinExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashJoinExec")
            .field("on", &self.on)
            .field("join_type", &self.join_type)
            .field("schema", &self.schema)
            .finish()
    }
}

impl HashJoinExec {
    pub fn new(
        left: Arc<dyn PhysicalPlan>,
        right: Arc<dyn PhysicalPlan>,
        on: Vec<(Column, Column)>,
        join_type: JoinType,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            on,
            join_type,
            schema,
        }
    }
}

impl PhysicalPlan for HashJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.left.clone(), self.right.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let left_rows = self.left.execute()?;
        let right_rows = self.right.execute()?;

        let left_schema = self.left.schema();
        let right_schema = self.right.schema();

        let left_key_indices: Vec<usize> = self
            .on
            .iter()
            .map(|(col, _)| {
                left_schema.field_index(&col.name).ok_or_else(|| {
                    crate::types::SqlError::ExecutionError(format!(
                        "Join column '{}' not found in left schema",
                        col.name
                    ))
                })
            })
            .collect::<SqlResult<Vec<usize>>>()?;

        let right_key_indices: Vec<usize> = self
            .on
            .iter()
            .map(|(_, col)| {
                right_schema.field_index(&col.name).ok_or_else(|| {
                    crate::types::SqlError::ExecutionError(format!(
                        "Join column '{}' not found in right schema",
                        col.name
                    ))
                })
            })
            .collect::<SqlResult<Vec<usize>>>()?;

        let mut hash_table: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>> =
            std::collections::HashMap::new();
        for row in &right_rows {
            let key: Vec<Value> = right_key_indices
                .iter()
                .map(|&idx| row.get(idx).cloned().unwrap_or(Value::Null))
                .collect();
            hash_table.entry(key).or_default().push(row.clone());
        }

        let mut result = Vec::new();

        match self.join_type {
            JoinType::Inner => {
                for left_row in &left_rows {
                    let key: Vec<Value> = left_key_indices
                        .iter()
                        .map(|&idx| left_row.get(idx).cloned().unwrap_or(Value::Null))
                        .collect();

                    if let Some(matching_rows) = hash_table.get(&key) {
                        for right_row in matching_rows {
                            let mut joined_row = left_row.clone();
                            joined_row.extend(right_row.clone());
                            result.push(joined_row);
                        }
                    }
                }
            }
            JoinType::Left => {
                for left_row in &left_rows {
                    let key: Vec<Value> = left_key_indices
                        .iter()
                        .map(|&idx| left_row.get(idx).cloned().unwrap_or(Value::Null))
                        .collect();

                    if let Some(matching_rows) = hash_table.get(&key) {
                        for right_row in matching_rows {
                            let mut joined_row = left_row.clone();
                            joined_row.extend(right_row.clone());
                            result.push(joined_row);
                        }
                    } else {
                        let mut left_extended = left_row.clone();
                        let right_nulls = vec![Value::Null; right_schema.fields.len()];
                        left_extended.extend(right_nulls);
                        result.push(left_extended);
                    }
                }
            }
            _ => {
                return Err(crate::types::SqlError::ExecutionError(
                    "Unsupported join type".to_string(),
                ));
            }
        }

        Ok(result)
    }
}

pub struct SortExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub expr: Vec<super::SortExpr>,
    pub schema: Schema,
}

impl fmt::Debug for SortExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SortExec")
            .field("schema", &self.schema)
            .finish()
    }
}

impl SortExec {
    pub fn new(input: Arc<dyn PhysicalPlan>, expr: Vec<super::SortExpr>, schema: Schema) -> Self {
        Self {
            input,
            expr,
            schema,
        }
    }
}

impl PhysicalPlan for SortExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let mut rows = self.input.execute()?;
        let input_schema = self.input.schema();
        let column_map: std::collections::HashMap<String, usize> = input_schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.clone(), i))
            .collect();

        let sort_exprs = self.expr.clone();
        rows.sort_by(|a, b| {
            for expr in &sort_exprs {
                let val_a = evaluate_projection_expr(a, &expr.expr, &column_map);
                let val_b = evaluate_projection_expr(b, &expr.expr, &column_map);
                let cmp = compare_values(&val_a, &val_b);
                if cmp != 0 {
                    return if expr.asc {
                        if cmp > 0 {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else if cmp > 0 {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    };
                }
            }
            std::cmp::Ordering::Equal
        });

        Ok(rows)
    }
}

pub struct LimitExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub n: usize,
}

impl fmt::Debug for LimitExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LimitExec").field("n", &self.n).finish()
    }
}

impl LimitExec {
    pub fn new(input: Arc<dyn PhysicalPlan>, n: usize) -> Self {
        Self { input, n }
    }
}

impl PhysicalPlan for LimitExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }

    fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
        let mut rows = self.input.execute()?;
        rows.truncate(self.n);
        Ok(rows)
    }
}
