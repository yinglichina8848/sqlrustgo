//! Physical Plan Module
//!
//! Defines the physical execution representation of query plans.

#![allow(dead_code)]

use crate::AggregateFunction;
use crate::Expr;
use crate::Operator;
use crate::Schema;
use sqlrustgo_types::Value;
use std::any::Any;
use std::collections::HashMap;

/// Physical plan trait - common interface for all physical operators
pub trait PhysicalPlan: Send + Sync {
    /// Get the schema of this physical plan
    fn schema(&self) -> &Schema;

    /// Get children of this plan
    fn children(&self) -> Vec<&dyn PhysicalPlan>;

    /// Get the name of this plan node
    fn name(&self) -> &str;

    /// Execute this physical plan and return results
    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
    }

    /// Get table name for scan operators
    fn table_name(&self) -> &str {
        ""
    }

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;

    /// Estimated cost for this plan node (startup_cost, total_cost, rows, width)
    /// Default implementation returns estimated values based on schema
    fn estimated_cost(&self) -> (f64, f64, u64, u32) {
        let row_width = self
            .schema()
            .fields
            .iter()
            .map(|f| f.data_type.estimate_size())
            .sum::<usize>() as u32;
        let children = self.children();
        if children.is_empty() {
            (0.0, 100.0, 1000, row_width)
        } else {
            let child_costs: Vec<_> = children.iter().map(|c| c.estimated_cost()).collect();
            let total_cost = child_costs.iter().map(|(_, c, _, _)| c).sum::<f64>() + 50.0;
            let total_rows = child_costs
                .iter()
                .map(|(_, _, r, _)| *r)
                .max()
                .unwrap_or(1000);
            (0.0, total_cost, total_rows, row_width)
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SeqScanExec {
    table_name: String,
    schema: Schema,
    projection: Option<Vec<usize>>,
}

impl SeqScanExec {
    pub fn new(table_name: String, schema: Schema) -> Self {
        Self {
            table_name,
            schema: schema.clone(),
            projection: None,
        }
    }

    pub fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn projection(&self) -> Option<&Vec<usize>> {
        self.projection.as_ref()
    }
}

impl PhysicalPlan for SeqScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "SeqScan"
    }

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn estimated_cost(&self) -> (f64, f64, u64, u32) {
        let row_width = self
            .schema
            .fields
            .iter()
            .map(|f| f.data_type.estimate_size())
            .sum::<usize>() as u32;
        (0.0, 100.0, 1000, row_width)
    }
}

/// ColumnarScan execution operator - optimized scan for columnar storage
///
/// This operator leverages column-oriented storage to read only the required
/// columns (projection pushdown), significantly reducing I/O for analytical queries.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ColumnarScanExec {
    table_name: String,
    schema: Schema,
    /// Column indices to scan (projection pushdown)
    projection: Vec<usize>,
}

impl ColumnarScanExec {
    pub fn new(table_name: String, schema: Schema, projection: Vec<usize>) -> Self {
        Self {
            table_name,
            schema,
            projection,
        }
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn projection(&self) -> &Vec<usize> {
        &self.projection
    }
}

impl PhysicalPlan for ColumnarScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "ColumnarScan"
    }

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn estimated_cost(&self) -> (f64, f64, u64, u32) {
        let row_width = self
            .schema
            .fields
            .iter()
            .map(|f| f.data_type.estimate_size())
            .sum::<usize>() as u32;
        // Columnar scan is cheaper due to projection pushdown
        let savings = 1.0 - (self.projection.len() as f64 / self.schema.fields.len().max(1) as f64);
        let cost = 100.0 * (1.0 - savings * 0.5);
        (0.0, cost, 1000, row_width)
    }
}

/// Index scan execution operator - uses index instead of full table scan
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexScanExec {
    table_name: String,
    index_name: String,
    key_expr: Expr,
    schema: Schema,
    key_range_min: Option<i64>,
    key_range_max: Option<i64>,
}

impl IndexScanExec {
    pub fn new(table_name: String, index_name: String, key_expr: Expr, schema: Schema) -> Self {
        Self {
            table_name,
            index_name,
            key_expr,
            schema,
            key_range_min: None,
            key_range_max: None,
        }
    }

    pub fn with_key_range(mut self, min: i64, max: i64) -> Self {
        self.key_range_min = Some(min);
        self.key_range_max = Some(max);
        self
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    pub fn key_expr(&self) -> &Expr {
        &self.key_expr
    }

    pub fn key_range(&self) -> (Option<i64>, Option<i64>) {
        (self.key_range_min, self.key_range_max)
    }
}

impl PhysicalPlan for IndexScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![]
    }

    fn name(&self) -> &str {
        "IndexScan"
    }

    fn table_name(&self) -> &str {
        &self.table_name
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        Ok(vec![])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Projection execution operator
#[allow(dead_code)]
pub struct ProjectionExec {
    input: Box<dyn PhysicalPlan>,
    expr: Vec<Expr>,
    schema: Schema,
}

impl ProjectionExec {
    pub fn new(input: Box<dyn PhysicalPlan>, expr: Vec<Expr>, schema: Schema) -> Self {
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

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Projection"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;

        if self.expr.is_empty() {
            return Ok(input_rows);
        }

        let input_schema = self.input.schema();
        let mut results = vec![];

        for row in input_rows {
            let mut projected_row = vec![];
            for expr in &self.expr {
                let value = self.evaluate_expr(expr, &row, input_schema);
                projected_row.push(value);
            }
            results.push(projected_row);
        }

        Ok(results)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ProjectionExec {
    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn expr(&self) -> &Vec<Expr> {
        &self.expr
    }

    fn evaluate_expr(&self, expr: &Expr, row: &[Value], schema: &Schema) -> Value {
        match expr {
            Expr::Column(col) => {
                if let Some(idx) = schema.field_index(&col.name) {
                    row.get(idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expr::Literal(val) => val.clone(),
            Expr::Wildcard => Value::Text(
                row.iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            Expr::Alias { expr, .. } => self.evaluate_expr(expr, row, schema),
            Expr::BinaryExpr { left, op, right } => {
                let lval = self.evaluate_expr(left, row, schema);
                let rval = self.evaluate_expr(right, row, schema);
                self.evaluate_arithmetic(op, &lval, &rval)
            }
            _ => Value::Null,
        }
    }

    fn evaluate_arithmetic(&self, op: &Operator, left: &Value, right: &Value) -> Value {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match op {
                Operator::Plus => Value::Integer(l + r),
                Operator::Minus => Value::Integer(l - r),
                Operator::Multiply => Value::Integer(l * r),
                Operator::Divide => {
                    if *r != 0 {
                        Value::Integer(l / r)
                    } else {
                        Value::Null
                    }
                }
                Operator::Modulo => {
                    if *r != 0 {
                        Value::Integer(l % r)
                    } else {
                        Value::Null
                    }
                }
                _ => Value::Null,
            },
            (Value::Float(l), Value::Float(r)) => match op {
                Operator::Plus => Value::Float(l + r),
                Operator::Minus => Value::Float(l - r),
                Operator::Multiply => Value::Float(l * r),
                Operator::Divide => {
                    if *r != 0.0 {
                        Value::Float(l / r)
                    } else {
                        Value::Null
                    }
                }
                _ => Value::Null,
            },
            _ => Value::Null,
        }
    }
}

/// Filter execution operator
#[allow(dead_code)]
pub struct FilterExec {
    input: Box<dyn PhysicalPlan>,
    predicate: Expr,
}

impl FilterExec {
    pub fn new(input: Box<dyn PhysicalPlan>, predicate: Expr) -> Self {
        Self { input, predicate }
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn predicate(&self) -> &Expr {
        &self.predicate
    }
}

impl PhysicalPlan for FilterExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Filter"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;
        let input_schema = self.input.schema();

        let filtered: Vec<Vec<Value>> = input_rows
            .into_iter()
            .filter(|row| self.evaluate_predicate(&self.predicate, row, input_schema))
            .collect();

        Ok(filtered)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FilterExec {
    fn evaluate_predicate(&self, expr: &Expr, row: &[Value], schema: &Schema) -> bool {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let lval = self.evaluate_expr(left, row, schema);
                let rval = self.evaluate_expr(right, row, schema);
                self.compare_values(&lval, op, &rval)
            }
            Expr::Literal(Value::Integer(n)) => *n != 0,
            _ => true,
        }
    }

    fn evaluate_expr(&self, expr: &Expr, row: &[Value], schema: &Schema) -> Value {
        match expr {
            Expr::Column(col) => {
                if let Some(idx) = schema.field_index(&col.name) {
                    row.get(idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expr::Literal(val) => val.clone(),
            _ => Value::Null,
        }
    }

    fn compare_values(&self, left: &Value, op: &Operator, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match op {
                Operator::Eq => l == r,
                Operator::NotEq => l != r,
                Operator::Gt => l > r,
                Operator::Lt => l < r,
                Operator::GtEq => l >= r,
                Operator::LtEq => l <= r,
                _ => false,
            },
            _ => false,
        }
    }
}

/// Scalar subquery execution operator
/// Executes a subquery that returns a single value
#[allow(dead_code)]
pub struct ScalarSubqueryExec {
    subquery: Box<dyn PhysicalPlan>,
    schema: Schema,
}

impl ScalarSubqueryExec {
    pub fn new(subquery: Box<dyn PhysicalPlan>, schema: Schema) -> Self {
        Self { subquery, schema }
    }

    pub fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        self.subquery.execute()
    }
}

impl PhysicalPlan for ScalarSubqueryExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.subquery.as_ref()]
    }

    fn name(&self) -> &str {
        "ScalarSubquery"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        self.subquery.execute()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// IN subquery execution operator
/// Checks if a value exists in the subquery results
#[allow(dead_code)]
pub struct InSubqueryExec {
    expr: Box<Expr>,
    subquery: Box<dyn PhysicalPlan>,
    schema: Schema,
}

impl InSubqueryExec {
    pub fn new(expr: Box<Expr>, subquery: Box<dyn PhysicalPlan>, schema: Schema) -> Self {
        Self {
            expr,
            subquery,
            schema,
        }
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn subquery(&self) -> &dyn PhysicalPlan {
        self.subquery.as_ref()
    }
}

impl PhysicalPlan for InSubqueryExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.subquery.as_ref()]
    }

    fn name(&self) -> &str {
        "InSubquery"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        self.subquery.execute()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// EXISTS subquery execution operator
/// Checks if the subquery returns any rows
#[allow(dead_code)]
pub struct ExistsExec {
    subquery: Box<dyn PhysicalPlan>,
    schema: Schema,
}

impl ExistsExec {
    pub fn new(subquery: Box<dyn PhysicalPlan>, schema: Schema) -> Self {
        Self { subquery, schema }
    }

    pub fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let results = self.subquery.execute()?;
        // EXISTS returns true if subquery returns at least one row
        Ok(vec![vec![Value::Boolean(!results.is_empty())]])
    }
}

impl PhysicalPlan for ExistsExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.subquery.as_ref()]
    }

    fn name(&self) -> &str {
        "Exists"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let results = self.subquery.execute()?;
        Ok(vec![vec![Value::Boolean(!results.is_empty())]])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// ANY/ALL subquery execution operator
/// Compares a value with ALL or ANY results from the subquery
#[allow(dead_code)]
pub struct AnyAllSubqueryExec {
    expr: Box<Expr>,
    op: Operator,
    subquery: Box<dyn PhysicalPlan>,
    any_all: crate::SubqueryType,
    schema: Schema,
}

impl AnyAllSubqueryExec {
    pub fn new(
        expr: Box<Expr>,
        op: Operator,
        subquery: Box<dyn PhysicalPlan>,
        any_all: crate::SubqueryType,
        schema: Schema,
    ) -> Self {
        Self {
            expr,
            op,
            subquery,
            any_all,
            schema,
        }
    }

    pub fn any_all(&self) -> crate::SubqueryType {
        self.any_all.clone()
    }
}

impl PhysicalPlan for AnyAllSubqueryExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.subquery.as_ref()]
    }

    fn name(&self) -> &str {
        match self.any_all {
            crate::SubqueryType::Any => "Any",
            crate::SubqueryType::All => "All",
            _ => "AnyAll",
        }
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        self.subquery.execute()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Aggregate execution operator
#[allow(dead_code)]
pub struct AggregateExec {
    input: Box<dyn PhysicalPlan>,
    group_expr: Vec<Expr>,
    aggregate_expr: Vec<Expr>,
    having_expr: Option<Expr>,
    schema: Schema,
}

impl AggregateExec {
    pub fn new(
        input: Box<dyn PhysicalPlan>,
        group_expr: Vec<Expr>,
        aggregate_expr: Vec<Expr>,
        having_expr: Option<Expr>,
        schema: Schema,
    ) -> Self {
        Self {
            input,
            group_expr,
            aggregate_expr,
            having_expr,
            schema,
        }
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn group_expr(&self) -> &Vec<Expr> {
        &self.group_expr
    }

    pub fn aggregate_expr(&self) -> &Vec<Expr> {
        &self.aggregate_expr
    }

    pub fn having_expr(&self) -> &Option<Expr> {
        &self.having_expr
    }

    fn evaluate_expr(&self, expr: &Expr, row: &[Value], schema: &Schema) -> Value {
        match expr {
            Expr::Column(col) => {
                if let Some(idx) = schema.field_index(&col.name) {
                    row.get(idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expr::Literal(val) => val.clone(),
            Expr::Wildcard => Value::Integer(row.len() as i64),
            Expr::Alias { expr, .. } => self.evaluate_expr(expr, row, schema),
            _ => Value::Null,
        }
    }

    fn compute_aggregate(
        &self,
        func: &AggregateFunction,
        args: &[Expr],
        values: &[Value],
    ) -> Value {
        match func {
            AggregateFunction::Count => {
                if args.is_empty() {
                    Value::Integer(values.len() as i64)
                } else {
                    let non_null_count =
                        values.iter().filter(|v| !matches!(v, Value::Null)).count();
                    Value::Integer(non_null_count as i64)
                }
            }
            AggregateFunction::Sum => {
                let mut sum: i64 = 0;
                let mut sum_float: f64 = 0.0;
                let mut has_float = false;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        sum += n;
                    } else if let Value::Float(n) = v {
                        has_float = true;
                        sum_float += n;
                    }
                }
                if has_float {
                    Value::Float(sum_float + sum as f64)
                } else {
                    Value::Integer(sum)
                }
            }
            AggregateFunction::Avg => {
                let mut sum: i64 = 0;
                let mut sum_float: f64 = 0.0;
                let mut count = 0;
                let mut has_float = false;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        sum += n;
                        count += 1;
                    } else if let Value::Float(n) = v {
                        has_float = true;
                        sum_float += n;
                        count += 1;
                    }
                }
                if count > 0 {
                    if has_float {
                        Value::Float((sum_float + sum as f64) / count as f64)
                    } else {
                        Value::Integer(sum / count as i64)
                    }
                } else {
                    Value::Null
                }
            }
            AggregateFunction::Min => {
                let mut min_val: Option<(bool, i64, f64)> = None;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        let n = *n;
                        match min_val {
                            Some((false, m, _)) if n < m => min_val = Some((false, n, 0.0)),
                            None => min_val = Some((false, n, 0.0)),
                            _ => {}
                        }
                    } else if let Value::Float(n) = v {
                        let n = *n;
                        match min_val {
                            Some((true, _, m)) if n < m => min_val = Some((true, 0, n)),
                            None => min_val = Some((true, 0, n)),
                            Some((false, _, _)) => min_val = Some((true, 0, n)),
                            _ => {}
                        }
                    }
                }
                match min_val {
                    Some((true, _, n)) => Value::Float(n),
                    Some((false, n, _)) => Value::Integer(n),
                    None => Value::Null,
                }
            }
            AggregateFunction::Max => {
                let mut max_val: Option<(bool, i64, f64)> = None;
                for v in values {
                    if let Value::Null = v {
                        continue;
                    }
                    if let Value::Integer(n) = v {
                        let n = *n;
                        match max_val {
                            Some((false, m, _)) if n > m => max_val = Some((false, n, 0.0)),
                            None => max_val = Some((false, n, 0.0)),
                            _ => {}
                        }
                    } else if let Value::Float(n) = v {
                        let n = *n;
                        match max_val {
                            Some((true, _, m)) if n > m => max_val = Some((true, 0, n)),
                            None => max_val = Some((true, 0, n)),
                            Some((false, _, _)) => max_val = Some((true, 0, n)),
                            _ => {}
                        }
                    }
                }
                match max_val {
                    Some((true, _, n)) => Value::Float(n),
                    Some((false, n, _)) => Value::Integer(n),
                    None => Value::Null,
                }
            }
        }
    }
}

impl PhysicalPlan for AggregateExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Aggregate"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let input_rows = self.input.execute()?;

        if self.group_expr.is_empty() {
            let mut results = vec![];
            let mut agg_results = vec![];

            for agg_expr in &self.aggregate_expr {
                if let Expr::AggregateFunction {
                    func,
                    args,
                    distinct,
                } = agg_expr
                {
                    let values: Vec<Value> = if args.is_empty() {
                        input_rows.iter().map(|_| Value::Null).collect()
                    } else {
                        input_rows
                            .iter()
                            .map(|row| {
                                self.evaluate_expr(
                                    args.first().unwrap_or(&Expr::Wildcard),
                                    row,
                                    self.input.schema(),
                                )
                            })
                            .collect()
                    };
                    let values = if *distinct {
                        let mut unique_values = values.clone();
                        unique_values.sort();
                        unique_values.dedup();
                        unique_values
                    } else {
                        values
                    };
                    let result = self.compute_aggregate(func, args, &values);
                    agg_results.push(result);
                }
            }

            if !agg_results.is_empty() {
                results.push(agg_results);
            }

            Ok(results)
        } else {
            let mut groups: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();

            for row in &input_rows {
                let key: Vec<Value> = self
                    .group_expr
                    .iter()
                    .map(|expr| self.evaluate_expr(expr, row, self.input.schema()))
                    .collect();
                groups.entry(key).or_default().push(row.clone());
            }

            let mut results = vec![];
            for (key, group_rows) in groups {
                let mut row = key;
                for agg_expr in &self.aggregate_expr {
                    if let Expr::AggregateFunction {
                        func,
                        args,
                        distinct,
                    } = agg_expr
                    {
                        let values: Vec<Value> = if args.is_empty() {
                            group_rows.iter().map(|_| Value::Null).collect()
                        } else {
                            group_rows
                                .iter()
                                .map(|r| {
                                    self.evaluate_expr(
                                        args.first().unwrap_or(&Expr::Wildcard),
                                        r,
                                        self.input.schema(),
                                    )
                                })
                                .collect()
                        };
                        let values = if *distinct {
                            let mut unique_values = values.clone();
                            unique_values.sort();
                            unique_values.dedup();
                            unique_values
                        } else {
                            values
                        };
                        let result = self.compute_aggregate(func, args, &values);
                        row.push(result);
                    }
                }
                results.push(row);
            }

            Ok(results)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Hash join execution operator
#[allow(dead_code)]
pub struct HashJoinExec {
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    join_type: crate::JoinType,
    condition: Option<Expr>,
    schema: Schema,
}

impl HashJoinExec {
    pub fn new(
        left: Box<dyn PhysicalPlan>,
        right: Box<dyn PhysicalPlan>,
        join_type: crate::JoinType,
        condition: Option<Expr>,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
            schema,
        }
    }

    pub fn left(&self) -> &dyn PhysicalPlan {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn PhysicalPlan {
        self.right.as_ref()
    }

    pub fn join_type(&self) -> crate::JoinType {
        self.join_type.clone()
    }

    pub fn condition(&self) -> Option<&Expr> {
        self.condition.as_ref()
    }
}

impl PhysicalPlan for HashJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.left.as_ref(), self.right.as_ref()]
    }

    fn name(&self) -> &str {
        "HashJoin"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Sort-Merge join execution operator
#[allow(dead_code)]
pub struct SortMergeJoinExec {
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    join_type: crate::JoinType,
    condition: Option<Expr>,
    schema: Schema,
    left_keys: Vec<Expr>,
    right_keys: Vec<Expr>,
}

impl SortMergeJoinExec {
    pub fn new(
        left: Box<dyn PhysicalPlan>,
        right: Box<dyn PhysicalPlan>,
        join_type: crate::JoinType,
        condition: Option<Expr>,
        schema: Schema,
        left_keys: Vec<Expr>,
        right_keys: Vec<Expr>,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            condition,
            schema,
            left_keys,
            right_keys,
        }
    }

    pub fn left(&self) -> &dyn PhysicalPlan {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn PhysicalPlan {
        self.right.as_ref()
    }

    pub fn join_type(&self) -> crate::JoinType {
        self.join_type.clone()
    }

    pub fn condition(&self) -> Option<&Expr> {
        self.condition.as_ref()
    }

    pub fn left_keys(&self) -> &Vec<Expr> {
        &self.left_keys
    }

    pub fn right_keys(&self) -> &Vec<Expr> {
        &self.right_keys
    }
}

impl PhysicalPlan for SortMergeJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.left.as_ref(), self.right.as_ref()]
    }

    fn name(&self) -> &str {
        "SortMergeJoin"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Sort execution operator
#[allow(dead_code)]
pub struct SortExec {
    input: Box<dyn PhysicalPlan>,
    sort_expr: Vec<crate::SortExpr>,
}

impl SortExec {
    pub fn new(input: Box<dyn PhysicalPlan>, sort_expr: Vec<crate::SortExpr>) -> Self {
        Self { input, sort_expr }
    }

    pub fn sort_expr(&self) -> &Vec<crate::SortExpr> {
        &self.sort_expr
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }
}

impl PhysicalPlan for SortExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Sort"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Limit execution operator
#[allow(dead_code)]
pub struct LimitExec {
    input: Box<dyn PhysicalPlan>,
    limit: usize,
    offset: Option<usize>,
}

impl LimitExec {
    pub fn new(input: Box<dyn PhysicalPlan>, limit: usize, offset: Option<usize>) -> Self {
        Self {
            input,
            limit,
            offset,
        }
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }
}

impl PhysicalPlan for LimitExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Limit"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Set operation execution operator (UNION, INTERSECT, EXCEPT)
#[allow(dead_code)]
pub struct SetOperationExec {
    op_type: crate::SetOperationType,
    left: Box<dyn PhysicalPlan>,
    right: Box<dyn PhysicalPlan>,
    schema: Schema,
}

impl SetOperationExec {
    pub fn new(
        op_type: crate::SetOperationType,
        left: Box<dyn PhysicalPlan>,
        right: Box<dyn PhysicalPlan>,
        schema: Schema,
    ) -> Self {
        Self {
            op_type,
            left,
            right,
            schema,
        }
    }

    pub fn op_type(&self) -> crate::SetOperationType {
        self.op_type
    }

    pub fn left(&self) -> &dyn PhysicalPlan {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn PhysicalPlan {
        self.right.as_ref()
    }
}

impl PhysicalPlan for SetOperationExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.left.as_ref(), self.right.as_ref()]
    }

    fn name(&self) -> &str {
        "SetOperation"
    }

    fn execute(&self) -> Result<Vec<Vec<Value>>, String> {
        let left_results = self.left.execute()?;
        let right_results = self.right.execute()?;
        match self.op_type {
            crate::SetOperationType::Union | crate::SetOperationType::UnionAll => {
                let mut results = left_results;
                results.extend(right_results);
                Ok(results)
            }
            _ => Ok(vec![]),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Window execution operator
#[allow(dead_code)]
pub struct WindowExec {
    input: Box<dyn PhysicalPlan>,
    window_exprs: Vec<crate::Expr>,
    partition_by: Vec<crate::Expr>,
    order_by: Vec<crate::SortExpr>,
    schema: Schema,
    input_schema: Schema,
}

impl WindowExec {
    pub fn new(
        input: Box<dyn PhysicalPlan>,
        window_exprs: Vec<crate::Expr>,
        partition_by: Vec<crate::Expr>,
        order_by: Vec<crate::SortExpr>,
        schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            input,
            window_exprs,
            partition_by,
            order_by,
            schema,
            input_schema,
        }
    }

    pub fn input(&self) -> &dyn PhysicalPlan {
        self.input.as_ref()
    }

    pub fn window_exprs(&self) -> &Vec<crate::Expr> {
        &self.window_exprs
    }

    pub fn partition_by(&self) -> &Vec<crate::Expr> {
        &self.partition_by
    }

    pub fn order_by(&self) -> &Vec<crate::SortExpr> {
        &self.order_by
    }
}

impl PhysicalPlan for WindowExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> Vec<&dyn PhysicalPlan> {
        vec![self.input.as_ref()]
    }

    fn name(&self) -> &str {
        "Window"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Expr, Field, Schema, SortExpr};
    use sqlrustgo_types::Value;

    #[test]
    fn test_seq_scan_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("test_table".to_string(), schema.clone());

        assert_eq!(scan.name(), "SeqScan");
        assert_eq!(scan.schema().fields.len(), 1);
        assert!(scan.children().is_empty());
    }

    #[test]
    fn test_seq_scan_exec_with_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("name".to_string(), crate::DataType::Text),
        ]);
        let scan = SeqScanExec::new("test_table".to_string(), schema).with_projection(vec![0]);

        assert!(scan.schema().fields.len() >= 1);
    }

    #[test]
    fn test_projection_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], schema);

        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.schema().fields.len(), 1);
        assert!(!proj.children().is_empty());
    }

    #[test]
    fn test_filter_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Gt,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        assert_eq!(filter.name(), "Filter");
        assert!(!filter.children().is_empty());
    }

    #[test]
    fn test_aggregate_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let agg = AggregateExec::new(
            input,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
            None,
            schema,
        );

        assert_eq!(agg.name(), "Aggregate");
        assert_eq!(agg.schema().fields.len(), 1);
        assert!(!agg.children().is_empty());
    }

    #[test]
    fn test_hash_join_exec() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let right_schema =
            Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let left = Box::new(SeqScanExec::new(
            "left_table".to_string(),
            left_schema.clone(),
        ));
        let right = Box::new(SeqScanExec::new(
            "right_table".to_string(),
            right_schema.clone(),
        ));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), crate::DataType::Integer),
            Field::new("id".to_string(), crate::DataType::Integer),
        ]);
        let join = HashJoinExec::new(left, right, crate::JoinType::Inner, None, join_schema);

        assert_eq!(join.name(), "HashJoin");
        assert!(!join.children().is_empty());
    }

    #[test]
    fn test_sort_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr);

        assert_eq!(sort.name(), "Sort");
        assert_eq!(sort.schema().fields.len(), 1);
        assert!(!sort.children().is_empty());
    }

    #[test]
    fn test_limit_exec() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let limit = LimitExec::new(input, 10, None);

        assert_eq!(limit.name(), "Limit");
        assert_eq!(limit.schema().fields.len(), 1);
        assert!(!limit.children().is_empty());
    }

    #[test]
    fn test_limit_exec_with_offset() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, Some(5));

        assert_eq!(limit.name(), "Limit");
        assert!(!limit.children().is_empty());
    }

    #[test]
    fn test_projection_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema.clone());
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = ProjectionExec::new(Box::new(child), vec![], schema);
        assert_eq!(exec.children().len(), 1);
    }

    #[test]
    fn test_filter_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate);
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_filter_exec_children() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::column("id");
        let exec = FilterExec::new(Box::new(child), predicate);
        assert_eq!(exec.children().len(), 1);
    }

    #[test]
    fn test_aggregate_exec_new() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], None, schema);
        assert_eq!(exec.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_schema() {
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = AggregateExec::new(Box::new(child), vec![], vec![], None, schema);
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_hash_join_exec_children() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Inner,
            None,
            schema,
        );
        assert_eq!(exec.children().len(), 2);
    }

    #[test]
    fn test_sort_exec_new() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = SortExec::new(Box::new(child), vec![]);
        assert_eq!(exec.name(), "Sort");
    }

    #[test]
    fn test_sort_exec_schema() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("users".to_string(), schema.clone());
        let exec = SortExec::new(Box::new(child), vec![]);
        assert_eq!(exec.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_exec_column() {
        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], output_schema);

        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.schema().fields.len(), 1);
    }

    #[test]
    fn test_projection_exec_alias() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let aliased_expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "my_id".to_string(),
        };
        let proj = ProjectionExec::new(input, vec![aliased_expr], schema);

        assert_eq!(proj.name(), "Projection");
    }

    #[test]
    fn test_aggregate_exec_count_star() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_count_column() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![Expr::column("id")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_sum() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("sum".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_group_by() {
        let schema = Schema::new(vec![
            Field::new("category".to_string(), DataType::Text),
            Field::new("amount".to_string(), DataType::Integer),
        ]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![Expr::column("category")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![
                Field::new("category".to_string(), DataType::Text),
                Field::new("sum".to_string(), DataType::Integer),
            ]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_physical_plan_trait_default_methods() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let scan = SeqScanExec::new("test".to_string(), schema);

        // Test default execute method
        let result = scan.execute();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        // Test default table_name
        assert_eq!(scan.table_name(), "test");
    }

    #[test]
    fn test_seq_scan_exec_projection() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let scan = SeqScanExec::new("test_table".to_string(), schema.clone());
        let proj = scan.projection();
        assert!(proj.is_none());

        let scan_with_proj =
            SeqScanExec::new("test_table".to_string(), schema).with_projection(vec![0]);
        assert!(scan_with_proj.projection().is_some());
    }

    #[test]
    fn test_projection_exec_execute() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_with_exprs() {
        use crate::Operator;
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![Field::new("sum".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Plus,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Gt,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_execute_with_and() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::column("id")),
                op: crate::Operator::Gt,
                right: Box::new(Expr::Literal(Value::Integer(10))),
            }),
            op: crate::Operator::And,
            right: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::column("id")),
                op: crate::Operator::Lt,
                right: Box::new(Expr::Literal(Value::Integer(100))),
            }),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
            None,
            Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_join_exec_execute() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let join = HashJoinExec::new(left, right, crate::JoinType::Inner, None, join_schema);

        let result = join.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_join_exec_execute_with_condition() {
        use crate::Operator;
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let condition = Expr::BinaryExpr {
            left: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
            op: Operator::Eq,
            right: Box::new(Expr::Column(crate::Column::new("id".to_string()))),
        };
        let join = HashJoinExec::new(
            left,
            right,
            crate::JoinType::Inner,
            Some(condition),
            join_schema,
        );

        let result = join.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr);

        let result = sort.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_limit_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, None);

        let result = limit.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_limit_exec_execute_with_offset() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, Some(5));

        let result = limit.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_empty_exprs() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(input, vec![], schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_with_multiple_group_by() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
            Field::new("c".to_string(), DataType::Integer),
        ]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![Expr::column("a"), Expr::column("b")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("c")],
                distinct: false,
            }],
            None,
            Schema::new(vec![
                Field::new("a".to_string(), DataType::Integer),
                Field::new("b".to_string(), DataType::Integer),
                Field::new("sum".to_string(), DataType::Integer),
            ]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_min() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("min".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_max() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("max".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_aggregate_exec_avg() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("avg".to_string(), DataType::Integer)]),
        );

        assert_eq!(agg.name(), "Aggregate");
    }

    #[test]
    fn test_hash_join_exec_left_join() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Left,
            None,
            schema,
        );
        assert_eq!(exec.children().len(), 2);
    }

    #[test]
    fn test_hash_join_exec_right_join() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Right,
            None,
            schema,
        );
        assert_eq!(exec.children().len(), 2);
    }

    #[test]
    fn test_hash_join_exec_cross_join() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let exec = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Cross,
            None,
            schema,
        );
        assert_eq!(exec.children().len(), 2);
    }

    #[test]
    fn test_projection_exec_evaluate_literal() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(input, vec![Expr::Literal(Value::Integer(42))], schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_unary() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(
            input,
            vec![Expr::UnaryExpr {
                op: crate::Operator::Minus,
                expr: Box::new(Expr::Literal(Value::Integer(5))),
            }],
            schema,
        );

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_evaluate_gte() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::GtEq,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_evaluate_lt() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Lt,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_evaluate_neq() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::NotEq,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    // HashJoinExec tests - different join types
    #[test]
    fn test_hash_join_exec_left_outer() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let join = HashJoinExec::new(left, right, crate::JoinType::Left, None, schema);

        assert_eq!(join.name(), "HashJoin");
    }

    // SortExec tests
    #[test]
    fn test_sort_exec_with_sort_expr() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let sort_expr = vec![crate::SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: true,
        }];
        let sort = SortExec::new(input, sort_expr);

        assert_eq!(sort.name(), "Sort");
    }

    #[test]
    fn test_sort_exec_children() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let sort_expr = vec![crate::SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: true,
        }];
        let sort = SortExec::new(input, sort_expr);

        assert_eq!(sort.children().len(), 1);
    }

    #[test]
    fn test_sort_exec_execute_2() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let sort_expr = vec![crate::SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: true,
        }];
        let sort = SortExec::new(input, sort_expr);

        let result = sort.execute();
        assert!(result.is_ok());
    }

    // LimitExec tests
    #[test]
    fn test_limit_exec_new() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let limit = LimitExec::new(input, 10, Some(5));

        assert_eq!(limit.name(), "Limit");
        assert_eq!(limit.limit(), 10);
        assert_eq!(limit.offset(), Some(5));
    }

    #[test]
    fn test_limit_exec_no_offset() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let limit = LimitExec::new(input, 10, None);

        assert_eq!(limit.offset(), None);
    }

    #[test]
    fn test_limit_exec_children() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let limit = LimitExec::new(input, 10, None);

        assert_eq!(limit.children().len(), 1);
    }

    #[test]
    fn test_limit_exec_execute_2() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let limit = LimitExec::new(input, 10, None);

        let result = limit.execute();
        assert!(result.is_ok());
    }

    // Additional tests to increase coverage

    #[test]
    fn test_projection_exec_getters() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let proj = ProjectionExec::new(input, vec![Expr::column("id")], schema.clone());

        // Test getters
        assert_eq!(proj.name(), "Projection");
        assert_eq!(proj.expr().len(), 1);
        // Test that input returns a valid PhysicalPlan
        let _ = proj.input();
    }

    #[test]
    fn test_filter_exec_getters() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let predicate = Expr::column("id");
        let filter = FilterExec::new(input, predicate);

        // Test getters
        assert_eq!(filter.name(), "Filter");
        // predicate() returns &Expr directly
        let _ = filter.predicate();
        // input() returns &dyn PhysicalPlan
        let _ = filter.input();
    }

    #[test]
    fn test_aggregate_exec_getters() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let agg = AggregateExec::new(
            input,
            vec![Expr::column("id")],
            vec![Expr::column("id")],
            None,
            schema,
        );

        // Test getters
        assert_eq!(agg.name(), "Aggregate");
        assert!(agg.input().schema().fields.len() >= 1);
        assert_eq!(agg.group_expr().len(), 1);
        assert_eq!(agg.aggregate_expr().len(), 1);
    }

    #[test]
    fn test_hash_join_exec_getters() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let join = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Inner,
            None,
            schema,
        );

        // Test getters
        assert_eq!(join.name(), "HashJoin");
        assert!(join.left().schema().fields.is_empty());
        assert!(join.right().schema().fields.is_empty());
        assert_eq!(join.join_type(), crate::JoinType::Inner);
        assert_eq!(join.condition(), None);
    }

    #[test]
    fn test_hash_join_exec_getters_with_condition() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let condition = Expr::column("id");
        let join = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Inner,
            Some(condition),
            schema,
        );

        assert!(join.condition().is_some());
    }

    #[test]
    fn test_hash_join_exec_schema() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let left = Box::new(SeqScanExec::new("left_table".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("right_table".to_string(), right_schema));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let join = HashJoinExec::new(left, right, crate::JoinType::Inner, None, join_schema);

        assert_eq!(join.schema().fields.len(), 2);
    }

    #[test]
    fn test_sort_exec_getters() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema.clone()));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr);

        // Test getters
        assert_eq!(sort.name(), "Sort");
        assert_eq!(sort.sort_expr().len(), 1);
        assert!(sort.input().schema().fields.len() >= 1);
    }

    #[test]
    fn test_sort_exec_as_any() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr);

        let _ = sort.as_any().downcast_ref::<SortExec>();
    }

    #[test]
    fn test_limit_exec_getters() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, Some(5));

        // Test getters
        assert_eq!(limit.name(), "Limit");
        assert_eq!(limit.limit(), 10);
        assert_eq!(limit.offset(), Some(5));
        assert!(limit.input().schema().fields.len() >= 1);
    }

    #[test]
    fn test_limit_exec_as_any() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let limit = LimitExec::new(input, 10, None);

        let _ = limit.as_any().downcast_ref::<LimitExec>();
    }

    #[test]
    fn test_aggregate_exec_as_any() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        let _ = agg.as_any().downcast_ref::<AggregateExec>();
    }

    #[test]
    fn test_aggregate_exec_execute_with_count_star() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_avg() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("avg".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_min() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("min".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_max() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("max".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_sum_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("sum".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_avg_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("avg".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_min_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("min".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_max_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("max".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_literal_predicate() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        // Use literal predicate (non-zero integer evaluates to true)
        let predicate = Expr::Literal(Value::Integer(1));
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_complex_binary_expr() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        // Test with Eq operator
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Eq,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_lte_operator() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::LtEq,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_with_alias() {
        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let aliased_expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "my_id".to_string(),
        };
        let proj = ProjectionExec::new(input, vec![aliased_expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_join_exec_as_any() {
        let schema = Schema::new(vec![]);
        let left = SeqScanExec::new("users".to_string(), schema.clone());
        let right = SeqScanExec::new("orders".to_string(), schema.clone());
        let join = HashJoinExec::new(
            Box::new(left),
            Box::new(right),
            crate::JoinType::Inner,
            None,
            schema,
        );

        let _ = join.as_any().downcast_ref::<HashJoinExec>();
    }

    // Tests for evaluate_expr with different expression types

    #[test]
    fn test_projection_exec_evaluate_wildcard() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new("*".to_string(), DataType::Text)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let proj = ProjectionExec::new(input, vec![Expr::Wildcard], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_binary_plus() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![Field::new("sum".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Plus,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_binary_minus() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![Field::new("diff".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Minus,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_binary_multiply() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![Field::new("product".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Multiply,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_binary_divide() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema =
            Schema::new(vec![Field::new("quotient".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Divide,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_binary_modulo() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![Field::new("mod".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Modulo,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_float_operations() {
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let output_schema = Schema::new(vec![Field::new("sum".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Plus,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_unary_minus() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("neg_id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        let expr = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::column("id")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_evaluate_column_not_found() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let output_schema =
            Schema::new(vec![Field::new("not_found".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));

        // Column "not_found" doesn't exist in the input schema
        let expr = Expr::column("not_found");
        let proj = ProjectionExec::new(input, vec![expr], output_schema);

        let result = proj.execute();
        assert!(result.is_ok());
    }

    // Tests for aggregate with different scenarios

    #[test]
    fn test_aggregate_exec_execute_multiple_aggregates() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![
                Expr::AggregateFunction {
                    func: AggregateFunction::Count,
                    args: vec![],
                    distinct: false,
                },
                Expr::AggregateFunction {
                    func: AggregateFunction::Sum,
                    args: vec![Expr::column("amount")],
                    distinct: false,
                },
            ],
            None,
            Schema::new(vec![
                Field::new("count".to_string(), DataType::Integer),
                Field::new("sum".to_string(), DataType::Integer),
            ]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_execute_with_distinct() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));
        let agg = AggregateExec::new(
            input,
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![Expr::column("amount")],
                distinct: true,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_group_by_multiple_columns() {
        let schema = Schema::new(vec![
            Field::new("dept".to_string(), DataType::Text),
            Field::new("category".to_string(), DataType::Text),
            Field::new("amount".to_string(), DataType::Integer),
        ]);
        let child = SeqScanExec::new("test_table".to_string(), schema.clone());
        let agg = AggregateExec::new(
            Box::new(child),
            vec![Expr::column("dept"), Expr::column("category")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![
                Field::new("dept".to_string(), DataType::Text),
                Field::new("category".to_string(), DataType::Text),
                Field::new("sum".to_string(), DataType::Integer),
            ]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_evaluate_expr_wildcard() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![Expr::Wildcard],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    // Additional filter tests for different predicates

    #[test]
    fn test_filter_exec_with_or_predicate() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::Or,
            right: Box::new(Expr::Literal(Value::Integer(0))),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_wildcard() {
        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let proj = ProjectionExec::new(input, vec![Expr::Wildcard], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_alias() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("my_id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "my_id".to_string(),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_minus() {
        use crate::Operator;
        let input_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Minus,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_multiply() {
        use crate::Operator;
        let input_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Multiply,
            right: Box::new(Expr::Literal(Value::Integer(2))),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_divide() {
        use crate::Operator;
        let input_schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let output_schema = Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Divide,
            right: Box::new(Expr::Literal(Value::Integer(2))),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_projection_exec_execute_float() {
        use crate::Operator;
        let input_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let output_schema = Schema::new(vec![Field::new("result".to_string(), DataType::Float)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), input_schema));
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::column("a")),
            op: Operator::Plus,
            right: Box::new(Expr::column("b")),
        };
        let proj = ProjectionExec::new(input, vec![expr], output_schema);
        let result = proj.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_not_equal() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::NotEq,
            right: Box::new(Expr::Literal(Value::Integer(10))),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_less() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::Lt,
            right: Box::new(Expr::Literal(Value::Integer(10))),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_less_equal() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::LtEq,
            right: Box::new(Expr::Literal(Value::Integer(10))),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter_exec_with_greater_equal() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test_table".to_string(), schema));

        let predicate = Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: crate::Operator::GtEq,
            right: Box::new(Expr::Literal(Value::Integer(10))),
        };
        let filter = FilterExec::new(input, predicate);

        let result = filter.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_count_distinct() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![Expr::column("id")],
                distinct: true,
            }],
            None,
            Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_sum_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("sum".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_avg_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("avg".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_min_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("min".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_exec_max_float() {
        let schema = Schema::new(vec![Field::new("amount".to_string(), DataType::Float)]);
        let child = SeqScanExec::new("test_table".to_string(), schema);
        let agg = AggregateExec::new(
            Box::new(child),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            None,
            Schema::new(vec![Field::new("max".to_string(), DataType::Float)]),
        );

        let result = agg.execute();
        assert!(result.is_ok());
    }

    // === Tests for uncovered code paths ===

    #[test]
    fn test_filter_compare_values_non_comparison_operators() {
        // Test non-comparison operators (Plus, Minus, etc.) on integers
        // These should return false per line 318
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let filter = FilterExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Plus, // Non-comparison operator
                right: Box::new(Expr::column("b")),
            },
        );

        // Evaluate predicate - should return false for non-comparison ops on integers
        let row = vec![Value::Integer(5), Value::Integer(3)];
        let result = filter.evaluate_predicate(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Plus,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert!(!result); // Should be false per line 318
    }

    #[test]
    fn test_filter_compare_values_float_comparison() {
        // Test Float comparisons - should return false per line 320
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let filter = FilterExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Gt,
                right: Box::new(Expr::column("b")),
            },
        );

        // Evaluate predicate with Float values - should return false per line 320
        let row = vec![Value::Float(5.0), Value::Float(3.0)];
        let result = filter.evaluate_predicate(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Gt,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert!(!result); // Float comparisons return false
    }

    #[test]
    fn test_projection_arithmetic_integer_division_by_zero() {
        // Test integer division by zero - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Divide,
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]),
        );

        // Evaluate expression with divisor = 0
        let row = vec![Value::Integer(10), Value::Integer(0)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Divide,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Division by zero returns Null
    }

    #[test]
    fn test_projection_arithmetic_integer_modulo_by_zero() {
        // Test integer modulo by zero - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Modulo,
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]),
        );

        // Evaluate expression with divisor = 0
        let row = vec![Value::Integer(10), Value::Integer(0)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Modulo,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Modulo by zero returns Null
    }

    #[test]
    fn test_projection_arithmetic_float_division_by_zero() {
        // Test float division by zero - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Divide,
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Float)]),
        );

        // Evaluate expression with divisor = 0.0
        let row = vec![Value::Float(10.0), Value::Float(0.0)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Divide,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Float division by zero returns Null
    }

    #[test]
    fn test_projection_arithmetic_non_arithmetic_op_on_integer() {
        // Test non-arithmetic operator on integers - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Eq, // Comparison operator in arithmetic context
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Integer)]),
        );

        let row = vec![Value::Integer(10), Value::Integer(5)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Eq,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Non-arithmetic operators return Null
    }

    #[test]
    fn test_projection_arithmetic_float_non_division_ops() {
        // Test non-division arithmetic operators on floats - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Modulo, // Not valid for floats
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Float)]),
        );

        let row = vec![Value::Float(10.0), Value::Float(3.0)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Modulo,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Non-division float ops return Null
    }

    #[test]
    fn test_projection_arithmetic_mixed_types() {
        // Test mixed types (Integer + Float) - should return Value::Null
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let proj = ProjectionExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Plus,
                right: Box::new(Expr::column("b")),
            }],
            Schema::new(vec![Field::new("result".to_string(), DataType::Float)]),
        );

        let row = vec![Value::Integer(10), Value::Float(3.0)];
        let result = proj.evaluate_expr(
            &Expr::BinaryExpr {
                left: Box::new(Expr::column("a")),
                op: Operator::Plus,
                right: Box::new(Expr::column("b")),
            },
            &row,
            &schema,
        );
        assert_eq!(result, Value::Null); // Mixed types return Null
    }

    // ========== Tests for AggregateExec coverage ==========

    #[test]
    fn test_aggregate_sum_with_float_values() {
        // Test Sum aggregate with Float values - covers lines 398-409
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Sum,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Float(1.5), Value::Float(2.5), Value::Float(3.0)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Sum,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(7.0));
    }

    #[test]
    fn test_aggregate_sum_with_mixed_int_float() {
        // Test Sum aggregate with mixed Integer and Float - covers lines 401-409
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Sum,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Integer(5), Value::Float(2.5), Value::Integer(3)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Sum,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(10.5));
    }

    #[test]
    fn test_aggregate_avg_with_float_values() {
        // Test Avg aggregate with Float values - covers lines 420-436
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Avg,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Float(10.0), Value::Float(20.0), Value::Float(30.0)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Avg,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(20.0));
    }

    #[test]
    fn test_aggregate_avg_with_mixed_types() {
        // Test Avg aggregate with mixed Integer and Float - covers lines 423-434
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Avg,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Integer(10), Value::Float(20.0)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Avg,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(15.0));
    }

    #[test]
    fn test_aggregate_min_with_float_values() {
        // Test Min aggregate with Float values - covers lines 445-467
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Min,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Float(5.5), Value::Float(2.2), Value::Float(8.8)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Min,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(2.2));
    }

    #[test]
    fn test_aggregate_min_with_mixed_types() {
        // Test Min aggregate with mixed Integer and Float - covers lines 448-461
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Min,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Integer(10), Value::Float(5.5), Value::Integer(20)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Min,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(5.5));
    }

    #[test]
    fn test_aggregate_max_with_float_values() {
        // Test Max aggregate with Float values - covers lines 474-496
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Max,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Float(5.5), Value::Float(22.2), Value::Float(8.8)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Max,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(22.2));
    }

    #[test]
    fn test_aggregate_max_with_mixed_types() {
        // Test Max aggregate with mixed Integer and Float - covers lines 477-490
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Float)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Max,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Integer(10), Value::Float(55.5), Value::Integer(20)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Max,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Float(55.5));
    }

    #[test]
    fn test_aggregate_count_with_args() {
        // Test Count aggregate with args (non-null count) - covers lines 385-390
        let schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let agg = AggregateExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            vec![],
            vec![Expr::AggregateFunction {
                func: crate::AggregateFunction::Count,
                args: vec![Expr::column("val")],
                distinct: false,
            }],
            None,
            schema.clone(),
        );

        let values = vec![Value::Integer(1), Value::Null, Value::Integer(3)];
        let result = agg.compute_aggregate(
            &crate::AggregateFunction::Count,
            &[Expr::column("val")],
            &values,
        );
        assert_eq!(result, Value::Integer(2)); // Non-null count
    }

    // ========== Tests for FilterExec coverage ==========

    #[test]
    fn test_filter_evaluate_predicate_integer_literal() {
        // Test Filter evaluate_predicate with Integer literal - covers line 290
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let filter = FilterExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            Expr::Literal(Value::Integer(1)),
        );

        // Test with non-zero integer - should be true
        let row = vec![Value::Integer(5)];
        let result = filter.evaluate_predicate(&Expr::Literal(Value::Integer(1)), &row, &schema);
        assert!(result);

        // Test with zero integer - should be false
        let result = filter.evaluate_predicate(&Expr::Literal(Value::Integer(0)), &row, &schema);
        assert!(!result);
    }

    #[test]
    fn test_filter_compare_values_non_integer() {
        // Test Filter compare_values with non-Integer values - covers lines 320-321
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let filter = FilterExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            Expr::literal(Value::Integer(1)),
        );

        // Test with Float values - should return false (non-matching types)
        let left = Value::Float(5.0);
        let right = Value::Float(3.0);
        let result = filter.compare_values(&left, &Operator::Eq, &right);
        assert!(!result);

        // Test with Text values - should return false
        let left = Value::Text("hello".to_string());
        let right = Value::Text("world".to_string());
        let result = filter.compare_values(&left, &Operator::Eq, &right);
        assert!(!result);
    }

    #[test]
    fn test_filter_evaluate_expr_literal() {
        // Test Filter evaluate_expr with Literal - covers lines 304-305
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let filter = FilterExec::new(
            Box::new(SeqScanExec::new("test".to_string(), schema.clone())),
            Expr::literal(Value::Integer(1)),
        );

        let row = vec![Value::Integer(5)];
        let result = filter.evaluate_expr(&Expr::Literal(Value::Integer(100)), &row, &schema);
        assert_eq!(result, Value::Integer(100));
    }

    // ========== Tests for HashJoinExec coverage ==========

    #[test]
    fn test_hash_join_execute_inner_join() {
        // Test HashJoinExec execute with inner join - covers lines 532-535
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(SeqScanExec::new("left".to_string(), left_schema.clone()));
        let right = Box::new(SeqScanExec::new("right".to_string(), right_schema.clone()));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let join = HashJoinExec::new(left, right, crate::JoinType::Inner, None, join_schema);

        let result = join.execute();
        // Empty input produces empty result
        assert!(result.is_ok());
        let rows = result.unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_seq_scan_exec_table_name() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let scan = SeqScanExec::new("my_table".to_string(), schema);

        assert_eq!(scan.table_name(), "my_table");
    }

    #[test]
    fn test_projection_exec_expr() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test".to_string(), schema.clone()));
        let expr = vec![Expr::column("id")];
        let proj = ProjectionExec::new(input, expr.clone(), schema);

        assert_eq!(proj.expr(), &expr);
    }

    #[test]
    fn test_filter_exec_predicate() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test".to_string(), schema.clone()));
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            crate::Operator::Gt,
            Expr::literal(Value::Integer(10)),
        );
        let filter = FilterExec::new(input, predicate.clone());

        assert_eq!(filter.predicate(), &predicate);
    }

    #[test]
    fn test_aggregate_exec_methods() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test".to_string(), schema.clone()));
        let group_expr = vec![Expr::column("id")];
        let agg_expr = vec![];
        let agg = AggregateExec::new(input, group_expr.clone(), agg_expr, None, schema.clone());

        assert_eq!(agg.group_expr(), &group_expr);
    }

    #[test]
    fn test_hash_join_exec_methods() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(SeqScanExec::new("left".to_string(), left_schema.clone()));
        let right = Box::new(SeqScanExec::new("right".to_string(), right_schema.clone()));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let join = HashJoinExec::new(
            left,
            right,
            crate::JoinType::Inner,
            None,
            join_schema.clone(),
        );

        assert_eq!(join.join_type(), crate::JoinType::Inner);
    }

    #[test]
    fn test_sort_exec_methods() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test".to_string(), schema.clone()));
        let sort_expr = vec![SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        }];
        let sort = SortExec::new(input, sort_expr.clone());

        assert_eq!(sort.sort_expr(), &sort_expr);
    }

    #[test]
    fn test_limit_exec_methods() {
        let schema = Schema::new(vec![Field::new("id".to_string(), crate::DataType::Integer)]);
        let input = Box::new(SeqScanExec::new("test".to_string(), schema.clone()));
        let limit = LimitExec::new(input, 100, Some(10));

        assert_eq!(limit.limit(), 100);
        assert_eq!(limit.offset(), Some(10));
    }

    #[test]
    fn test_sort_merge_join_exec_methods() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(SeqScanExec::new("left".to_string(), left_schema.clone()));
        let right = Box::new(SeqScanExec::new("right".to_string(), right_schema.clone()));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let left_keys = vec![Expr::column("id")];
        let right_keys = vec![Expr::column("id")];
        let smj = SortMergeJoinExec::new(
            left,
            right,
            crate::JoinType::Left,
            None,
            join_schema,
            left_keys.clone(),
            right_keys.clone(),
        );

        assert_eq!(smj.join_type(), crate::JoinType::Left);
        assert_eq!(smj.left_keys(), &left_keys);
        assert_eq!(smj.right_keys(), &right_keys);
    }

    #[test]
    fn test_index_scan_exec_new() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let key_expr = Expr::column("id");
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "idx_id".to_string(),
            key_expr,
            schema.clone(),
        );

        assert_eq!(scan.name(), "IndexScan");
        assert_eq!(scan.table_name(), "test_table");
        assert_eq!(scan.index_name(), "idx_id");
        assert_eq!(scan.schema().fields.len(), 1);
        assert!(scan.children().is_empty());
    }

    #[test]
    fn test_index_scan_exec_with_key_range() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let key_expr = Expr::column("id");
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "idx_id".to_string(),
            key_expr,
            schema,
        )
        .with_key_range(10, 100);

        assert_eq!(scan.key_expr().to_string(), "id");
        assert_eq!(scan.key_range(), (Some(10), Some(100)));
    }

    #[test]
    fn test_index_scan_exec_execute() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let key_expr = Expr::column("id");
        let scan = IndexScanExec::new(
            "test_table".to_string(),
            "idx_id".to_string(),
            key_expr,
            schema,
        );

        let result = scan.execute();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scalar_subquery_exec() {
        let subquery_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let subquery = Box::new(SeqScanExec::new("inner".to_string(), subquery_schema));
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let scalar = ScalarSubqueryExec::new(subquery, schema.clone());

        assert_eq!(scalar.name(), "ScalarSubquery");
        assert_eq!(scalar.schema().fields.len(), 1);
    }

    #[test]
    fn test_in_subquery_exec() {
        let subquery_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let subquery = Box::new(SeqScanExec::new("inner".to_string(), subquery_schema));
        let expr = Box::new(Expr::column("id"));
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let in_subquery = InSubqueryExec::new(expr, subquery, schema.clone());

        assert_eq!(in_subquery.name(), "InSubquery");
        assert_eq!(in_subquery.schema().fields.len(), 1);
    }

    #[test]
    fn test_exists_exec() {
        let subquery_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let subquery = Box::new(SeqScanExec::new("inner".to_string(), subquery_schema));
        let schema = Schema::new(vec![Field::new("exists".to_string(), DataType::Boolean)]);

        let exists = ExistsExec::new(subquery, schema.clone());

        assert_eq!(exists.name(), "Exists");
        assert_eq!(exists.schema().fields.len(), 1);
    }

    #[test]
    fn test_any_all_subquery_exec() {
        let subquery_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let subquery = Box::new(SeqScanExec::new("inner".to_string(), subquery_schema));
        let expr = Box::new(Expr::column("id"));
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let any_all = AnyAllSubqueryExec::new(
            expr,
            crate::Operator::Eq,
            subquery,
            crate::SubqueryType::Any,
            schema.clone(),
        );

        assert_eq!(any_all.name(), "Any");
        assert_eq!(any_all.any_all(), crate::SubqueryType::Any);
    }

    #[test]
    fn test_set_operation_exec() {
        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let left = Box::new(SeqScanExec::new("left".to_string(), left_schema));
        let right = Box::new(SeqScanExec::new("right".to_string(), right_schema));

        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let set_op =
            SetOperationExec::new(crate::SetOperationType::Union, left, right, schema.clone());

        assert_eq!(set_op.name(), "SetOperation");
        assert_eq!(set_op.op_type(), crate::SetOperationType::Union);
    }
}
