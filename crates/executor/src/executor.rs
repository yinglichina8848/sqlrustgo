//! Volcano Executor trait - 统一的查询执行接口

use crate::filter::FilterVolcanoExecutor;
use sqlrustgo_planner::{PhysicalPlan, Schema};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ExecutorResult {
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: usize,
}

impl ExecutorResult {
    pub fn new(rows: Vec<Vec<Value>>, affected_rows: usize) -> Self {
        Self {
            rows,
            affected_rows,
        }
    }
    pub fn empty() -> Self {
        Self {
            rows: vec![],
            affected_rows: 0,
        }
    }
}

pub trait VolcanoExecutor: Send + Sync {
    fn init(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn close(&mut self) -> SqlResult<()>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn is_initialized(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
}

pub type VolIterator = Box<dyn VolcanoExecutor>;

pub fn execute_collect(executor: &mut dyn VolcanoExecutor) -> SqlResult<ExecutorResult> {
    executor.init()?;
    let mut rows = Vec::new();
    while let Some(row) = executor.next()? {
        rows.push(row);
    }
    executor.close()?;
    Ok(ExecutorResult::new(rows, 0))
}

pub trait Executor: Send + Sync {
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult>;
    fn name(&self) -> &str;
    fn is_ready(&self) -> bool;
}

pub struct SeqScanVolcanoExecutor {
    table_name: String,
    schema: Schema,
    storage: std::sync::Arc<dyn Storage>,
    initialized: bool,
    current_idx: usize,
    rows: Vec<Vec<Value>>,
}

impl SeqScanVolcanoExecutor {
    pub fn new(table_name: String, schema: Schema, storage: std::sync::Arc<dyn Storage>) -> Self {
        Self {
            table_name,
            schema,
            storage,
            initialized: false,
            current_idx: 0,
            rows: Vec::new(),
        }
    }
}

impl VolcanoExecutor for SeqScanVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        if self.initialized {
            return Ok(());
        }
        self.rows = self.storage.scan(&self.table_name).unwrap_or_default();
        self.current_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }
        if self.current_idx >= self.rows.len() {
            return Ok(None);
        }
        let row = self.rows[self.current_idx].clone();
        self.current_idx += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.rows.clear();
        self.current_idx = 0;
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "SeqScan"
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ProjectionVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    expr: Vec<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
}

impl ProjectionVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        expr: Vec<sqlrustgo_planner::Expr>,
        schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            child,
            expr,
            schema,
            input_schema,
            initialized: false,
        }
    }
}

impl VolcanoExecutor for ProjectionVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        self.initialized = true;
        Ok(())
    }
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if let Some(row) = self.child.next()? {
            let projected_row: Vec<Value> = self
                .expr
                .iter()
                .map(|expr| {
                    expr.evaluate(&row, &self.input_schema)
                        .unwrap_or(Value::Null)
                })
                .collect();
            Ok(Some(projected_row))
        } else {
            Ok(None)
        }
    }
    fn close(&mut self) -> SqlResult<()> {
        self.child.close()?;
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "Projection"
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct AggregateVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    group_expr: Vec<sqlrustgo_planner::Expr>,
    aggregate_expr: Vec<sqlrustgo_planner::Expr>,
    having_expr: Option<sqlrustgo_planner::Expr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
    groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>>,
    group_keys: Vec<Vec<Value>>,
    current_group_idx: usize,
}

impl AggregateVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        group_expr: Vec<sqlrustgo_planner::Expr>,
        aggregate_expr: Vec<sqlrustgo_planner::Expr>,
        having_expr: Option<sqlrustgo_planner::Expr>,
        schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            child,
            group_expr,
            aggregate_expr,
            having_expr,
            schema,
            input_schema,
            initialized: false,
            groups: std::collections::HashMap::new(),
            group_keys: Vec::new(),
            current_group_idx: 0,
        }
    }

    fn compute_aggregate(&self, group_rows: &[Vec<Value>]) -> Vec<Value> {
        let mut results = Vec::new();
        for agg_expr in &self.aggregate_expr {
            if let sqlrustgo_planner::Expr::AggregateFunction { func, args, .. } = agg_expr {
                let agg_values: Vec<Value> = group_rows
                    .iter()
                    .flat_map(|row| {
                        if args.is_empty() {
                            vec![Value::Integer(group_rows.len() as i64)]
                        } else {
                            args.iter()
                                .map(|arg| {
                                    arg.evaluate(row, &self.input_schema).unwrap_or(Value::Null)
                                })
                                .collect()
                        }
                    })
                    .collect();

                let result = match func {
                    sqlrustgo_planner::AggregateFunction::Count => {
                        Value::Integer(agg_values.len() as i64)
                    }
                    sqlrustgo_planner::AggregateFunction::Sum => {
                        let mut sum: i64 = 0;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                sum += n;
                            }
                        }
                        Value::Integer(sum)
                    }
                    sqlrustgo_planner::AggregateFunction::Avg => {
                        let mut sum: i64 = 0;
                        let mut count = 0;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                sum += n;
                                count += 1;
                            }
                        }
                        if count > 0 {
                            Value::Integer(sum / count as i64)
                        } else {
                            Value::Null
                        }
                    }
                    sqlrustgo_planner::AggregateFunction::Min => {
                        let mut min_val: Option<i64> = None;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                match min_val {
                                    Some(m) if *n < m => min_val = Some(*n),
                                    None => min_val = Some(*n),
                                    _ => {}
                                }
                            }
                        }
                        min_val.map(Value::Integer).unwrap_or(Value::Null)
                    }
                    sqlrustgo_planner::AggregateFunction::Max => {
                        let mut max_val: Option<i64> = None;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                match max_val {
                                    Some(m) if *n > m => max_val = Some(*n),
                                    None => max_val = Some(*n),
                                    _ => {}
                                }
                            }
                        }
                        max_val.map(Value::Integer).unwrap_or(Value::Null)
                    }
                };
                results.push(result);
            }
        }
        results
    }

    /// Evaluate a HAVING expression that may contain aggregate functions.
    /// Returns true if the row passes the HAVING condition, false otherwise.
    fn evaluate_having_expr(
        &self,
        expr: &sqlrustgo_planner::Expr,
        group_rows: &[Vec<Value>],
    ) -> bool {
        match expr {
            sqlrustgo_planner::Expr::AggregateFunction { func, args, .. } => {
                // Compute the aggregate value for this group
                let agg_values: Vec<Value> = group_rows
                    .iter()
                    .flat_map(|row| {
                        if args.is_empty() {
                            vec![Value::Integer(group_rows.len() as i64)]
                        } else {
                            args.iter()
                                .map(|arg| {
                                    arg.evaluate(row, &self.input_schema).unwrap_or(Value::Null)
                                })
                                .collect()
                        }
                    })
                    .collect();

                let result = match func {
                    sqlrustgo_planner::AggregateFunction::Count => {
                        Value::Integer(agg_values.len() as i64)
                    }
                    sqlrustgo_planner::AggregateFunction::Sum => {
                        let mut sum: i64 = 0;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                sum += n;
                            }
                        }
                        Value::Integer(sum)
                    }
                    sqlrustgo_planner::AggregateFunction::Avg => {
                        let mut sum: i64 = 0;
                        let mut count = 0;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                sum += n;
                                count += 1;
                            }
                        }
                        if count > 0 {
                            Value::Integer(sum / count as i64)
                        } else {
                            Value::Null
                        }
                    }
                    sqlrustgo_planner::AggregateFunction::Min => {
                        let mut min_val: Option<i64> = None;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                match min_val {
                                    Some(m) if *n < m => min_val = Some(*n),
                                    None => min_val = Some(*n),
                                    _ => {}
                                }
                            }
                        }
                        min_val.map(Value::Integer).unwrap_or(Value::Null)
                    }
                    sqlrustgo_planner::AggregateFunction::Max => {
                        let mut max_val: Option<i64> = None;
                        for v in &agg_values {
                            if let Value::Integer(n) = v {
                                match max_val {
                                    Some(m) if *n > m => max_val = Some(*n),
                                    None => max_val = Some(*n),
                                    _ => {}
                                }
                            }
                        }
                        max_val.map(Value::Integer).unwrap_or(Value::Null)
                    }
                };

                if let Value::Boolean(b) = result {
                    b
                } else {
                    false
                }
            }
            sqlrustgo_planner::Expr::BinaryExpr { left, op, right } => {
                let left_val = self.evaluate_having_expr(left, group_rows);
                let right_val = self.evaluate_having_expr(right, group_rows);
                // For boolean results from aggregate comparisons like COUNT(*) > 1
                match op {
                    sqlrustgo_planner::Operator::Gt => left_val > right_val,
                    sqlrustgo_planner::Operator::Lt => left_val < right_val,
                    sqlrustgo_planner::Operator::GtEq => left_val >= right_val,
                    sqlrustgo_planner::Operator::LtEq => left_val <= right_val,
                    sqlrustgo_planner::Operator::Eq => left_val == right_val,
                    sqlrustgo_planner::Operator::NotEq => left_val != right_val,
                    sqlrustgo_planner::Operator::And => left_val && right_val,
                    sqlrustgo_planner::Operator::Or => left_val || right_val,
                    _ => false,
                }
            }
            sqlrustgo_planner::Expr::Literal(val) => {
                if let sqlrustgo_types::Value::Boolean(b) = val {
                    *b
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl VolcanoExecutor for AggregateVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        let mut all_rows = Vec::new();
        while let Some(row) = self.child.next()? {
            all_rows.push(row);
        }
        self.child.close()?;
        for row in &all_rows {
            let key: Vec<Value> = self
                .group_expr
                .iter()
                .map(|expr| {
                    expr.evaluate(row, &self.input_schema)
                        .unwrap_or(Value::Null)
                })
                .collect();
            self.groups.entry(key).or_default().push(row.clone());
        }
        self.group_keys = self.groups.keys().cloned().collect();
        self.group_keys.sort_by(|a, b| {
            for (av, bv) in a.iter().zip(b.iter()) {
                match (av, bv) {
                    (Value::Integer(ai), Value::Integer(bi)) => {
                        let cmp = ai.cmp(bi);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    (Value::Text(ai), Value::Text(bi)) => {
                        let cmp = ai.cmp(bi);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    _ => {}
                }
            }
            a.len().cmp(&b.len())
        });

        // Apply HAVING filter if present
        if let Some(ref having_expr) = self.having_expr {
            let mut keys_to_remove = Vec::new();
            for key in &self.group_keys {
                let group_rows = &self.groups[key];
                if !self.evaluate_having_expr(having_expr, group_rows) {
                    keys_to_remove.push(key.clone());
                }
            }
            for key in keys_to_remove {
                self.groups.remove(&key);
            }
            self.group_keys.retain(|k| self.groups.contains_key(k));
        }

        self.current_group_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }
        if self.current_group_idx >= self.group_keys.len() {
            return Ok(None);
        }
        let key = &self.group_keys[self.current_group_idx];
        let group_rows = &self.groups[key];
        let mut result = key.clone();
        result.extend(self.compute_aggregate(group_rows));
        self.current_group_idx += 1;
        Ok(Some(result))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.groups.clear();
        self.group_keys.clear();
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "Aggregate"
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[allow(dead_code)]
pub struct HashJoinVolcanoExecutor {
    left: Box<dyn VolcanoExecutor>,
    right: Box<dyn VolcanoExecutor>,
    join_type: sqlrustgo_planner::JoinType,
    #[allow(dead_code)]
    left_schema: Schema,
    right_schema: Schema,
    schema: Schema,
    initialized: bool,
    right_hash: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>>,
    current_left_rows: Vec<Vec<Value>>,
    left_idx: usize,
    right_idx: usize,
}

impl HashJoinVolcanoExecutor {
    pub fn new(
        left: Box<dyn VolcanoExecutor>,
        right: Box<dyn VolcanoExecutor>,
        join_type: sqlrustgo_planner::JoinType,
        left_schema: Schema,
        right_schema: Schema,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            left_schema,
            right_schema,
            schema,
            initialized: false,
            right_hash: std::collections::HashMap::new(),
            current_left_rows: Vec::new(),
            left_idx: 0,
            right_idx: 0,
        }
    }

    fn next_inner(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.current_left_rows.len() {
            let left_row = &self.current_left_rows[self.left_idx];
            let key = if !left_row.is_empty() {
                vec![left_row[0].clone()]
            } else {
                vec![Value::Null]
            };
            if let Some(right_rows) = self.right_hash.get(&key) {
                if self.right_idx < right_rows.len() {
                    let mut result = left_row.clone();
                    result.extend(right_rows[self.right_idx].clone());
                    self.right_idx += 1;
                    return Ok(Some(result));
                }
            }
            self.left_idx += 1;
            self.right_idx = 0;
        }
        Ok(None)
    }

    fn next_left(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.current_left_rows.len() {
            let left_row = &self.current_left_rows[self.left_idx];
            let key = if !left_row.is_empty() {
                vec![left_row[0].clone()]
            } else {
                vec![Value::Null]
            };
            if let Some(right_rows) = self.right_hash.get(&key) {
                if self.right_idx < right_rows.len() {
                    let mut result = left_row.clone();
                    result.extend(right_rows[self.right_idx].clone());
                    self.right_idx += 1;
                    return Ok(Some(result));
                }
            } else {
                let mut result = left_row.clone();
                for _ in 0..self.right_schema.fields.len() {
                    result.push(Value::Null);
                }
                self.left_idx += 1;
                self.right_idx = 0;
                return Ok(Some(result));
            }
            self.left_idx += 1;
            self.right_idx = 0;
        }
        Ok(None)
    }
}

impl VolcanoExecutor for HashJoinVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.left.init()?;
        self.right.init()?;
        while let Some(row) = self.right.next()? {
            let key = if !row.is_empty() {
                vec![row[0].clone()]
            } else {
                vec![Value::Null]
            };
            self.right_hash.entry(key).or_default().push(row);
        }
        while let Some(row) = self.left.next()? {
            self.current_left_rows.push(row);
        }
        self.left_idx = 0;
        self.right_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }
        match self.join_type {
            sqlrustgo_planner::JoinType::Inner => self.next_inner(),
            sqlrustgo_planner::JoinType::Left => self.next_left(),
            _ => Ok(None),
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.left.close()?;
        self.right.close()?;
        self.right_hash.clear();
        self.current_left_rows.clear();
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "HashJoin"
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// SortMergeJoinVolcanoExecutor - Sort-Merge Join algorithm implementation
/// Alternative to HashJoin, works best when inputs are already sorted by join keys
#[allow(dead_code)]
pub struct SortMergeJoinVolcanoExecutor {
    left: Box<dyn VolcanoExecutor>,
    right: Box<dyn VolcanoExecutor>,
    join_type: sqlrustgo_planner::JoinType,
    left_schema: Schema,
    right_schema: Schema,
    schema: Schema,
    initialized: bool,
    // Sorted data for merge
    left_sorted: Vec<Vec<Value>>,
    right_sorted: Vec<Vec<Value>>,
    // Current positions in merge
    left_idx: usize,
    right_idx: usize,
    // For handling multiple matches
    left_matches: Vec<Vec<Value>>,
    left_match_idx: usize,
}

impl SortMergeJoinVolcanoExecutor {
    pub fn new(
        left: Box<dyn VolcanoExecutor>,
        right: Box<dyn VolcanoExecutor>,
        join_type: sqlrustgo_planner::JoinType,
        left_schema: Schema,
        right_schema: Schema,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            left_schema,
            right_schema,
            schema,
            initialized: false,
            left_sorted: Vec::new(),
            right_sorted: Vec::new(),
            left_idx: 0,
            right_idx: 0,
            left_matches: Vec::new(),
            left_match_idx: 0,
        }
    }
}

/// Compare two rows by their first column (join key)
fn compare_join_keys(left_row: &[Value], right_row: &[Value]) -> std::cmp::Ordering {
    let left_key = left_row.first().unwrap_or(&Value::Null);
    let right_key = right_row.first().unwrap_or(&Value::Null);

    match (left_key, right_key) {
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r),
        (Value::Text(l), Value::Text(r)) => l.cmp(r),
        (Value::Float(l), Value::Float(r)) => {
            if l < r {
                std::cmp::Ordering::Less
            } else if l > r {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
        _ => std::cmp::Ordering::Equal,
    }
}

impl SortMergeJoinVolcanoExecutor {
    fn next_inner(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            // If we have cached matches, return them
            if self.left_match_idx < self.left_matches.len() {
                let mut result = self.left_matches[self.left_match_idx].clone();
                result.extend(self.right_sorted[self.right_idx].clone());
                self.left_match_idx += 1;
                self.right_idx += 1;
                return Ok(Some(result));
            }

            // Clear cached matches after consuming them all
            self.left_matches.clear();
            self.left_match_idx = 0;

            // Check if we've exhausted either input
            if self.left_idx >= self.left_sorted.len() || self.right_idx >= self.right_sorted.len()
            {
                return Ok(None);
            }

            let left_row = &self.left_sorted[self.left_idx];
            let right_row = &self.right_sorted[self.right_idx];

            let cmp = compare_join_keys(left_row, right_row);

            if cmp == std::cmp::Ordering::Equal {
                // Found match - collect all matching right rows for this left row
                // Collect all matching right rows
                while self.right_idx < self.right_sorted.len()
                    && compare_join_keys(left_row, &self.right_sorted[self.right_idx])
                        == std::cmp::Ordering::Equal
                {
                    self.left_matches.push(left_row.clone());
                    self.right_idx += 1;
                }

                // Return first match
                if !self.left_matches.is_empty() {
                    let mut result = self.left_matches[0].clone();
                    // Get the first right row from the match set (we already advanced right_idx)
                    result.extend(self.right_sorted[self.right_idx - 1].clone());
                    self.left_match_idx = 1;
                    return Ok(Some(result));
                }
            } else if cmp == std::cmp::Ordering::Less {
                // Left key is smaller, advance left
                self.left_idx += 1;
            } else {
                // Right key is smaller, advance right
                self.right_idx += 1;
            }
        }
    }

    fn next_left(&mut self) -> SqlResult<Option<Vec<Value>>> {
        // Loop to find next output
        loop {
            // Check if we've exhausted left input
            if self.left_idx >= self.left_sorted.len() {
                return Ok(None);
            }

            // If we have cached matches, return them
            if self.left_match_idx < self.left_matches.len() {
                let mut result = self.left_matches[self.left_match_idx].clone();
                result.extend(self.right_sorted[self.right_idx - 1].clone());
                self.left_match_idx += 1;
                return Ok(Some(result));
            }

            // Clear cached matches and continue searching
            self.left_matches.clear();
            self.left_match_idx = 0;

            let left_row = &self.left_sorted[self.left_idx];

            // If right is exhausted, return left row with nulls
            if self.right_idx >= self.right_sorted.len() {
                let mut result = left_row.clone();
                for _ in 0..self.right_schema.fields.len() {
                    result.push(Value::Null);
                }
                self.left_idx += 1;
                self.right_idx = 0;
                return Ok(Some(result));
            }

            let right_row = &self.right_sorted[self.right_idx];
            let cmp = compare_join_keys(left_row, right_row);

            if cmp == std::cmp::Ordering::Equal {
                // Found match - collect all matching right rows
                while self.right_idx < self.right_sorted.len()
                    && compare_join_keys(left_row, &self.right_sorted[self.right_idx])
                        == std::cmp::Ordering::Equal
                {
                    self.left_matches.push(left_row.clone());
                    self.right_idx += 1;
                }

                if !self.left_matches.is_empty() {
                    let mut result = self.left_matches[0].clone();
                    result.extend(self.right_sorted[self.right_idx - 1].clone());
                    self.left_match_idx = 1;
                    // After exhausting all matches for this left row, advance to next left
                    self.left_idx += 1;
                    self.right_idx = 0;
                    return Ok(Some(result));
                }
            } else if cmp == std::cmp::Ordering::Less {
                // Left key doesn't have a match in right, return with nulls
                let mut result = left_row.clone();
                for _ in 0..self.right_schema.fields.len() {
                    result.push(Value::Null);
                }
                self.left_idx += 1;
                self.right_idx = 0;
                return Ok(Some(result));
            } else {
                // Right key is smaller, advance right
                self.right_idx += 1;
            }
        }
    }
}

impl VolcanoExecutor for SortMergeJoinVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        // Initialize and collect left input
        self.left.init()?;
        while let Some(row) = self.left.next()? {
            self.left_sorted.push(row);
        }
        self.left.close()?;

        // Initialize and collect right input
        self.right.init()?;
        while let Some(row) = self.right.next()? {
            self.right_sorted.push(row);
        }
        self.right.close()?;

        // Sort both inputs by join key
        self.left_sorted.sort_by(|a, b| compare_join_keys(a, b));
        self.right_sorted.sort_by(|a, b| compare_join_keys(a, b));

        // Reset indices
        self.left_idx = 0;
        self.right_idx = 0;
        self.left_matches.clear();
        self.left_match_idx = 0;
        self.initialized = true;

        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }

        match self.join_type {
            sqlrustgo_planner::JoinType::Inner => self.next_inner(),
            sqlrustgo_planner::JoinType::Left => self.next_left(),
            _ => Ok(None),
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.left_sorted.clear();
        self.right_sorted.clear();
        self.left_matches.clear();
        self.left_idx = 0;
        self.right_idx = 0;
        self.left_match_idx = 0;
        self.initialized = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "SortMergeJoin"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct SortVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    sort_expr: Vec<sqlrustgo_planner::SortExpr>,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
    sorted_rows: Vec<Vec<Value>>,
    current_idx: usize,
}

impl SortVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        sort_expr: Vec<sqlrustgo_planner::SortExpr>,
        schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            child,
            sort_expr,
            schema,
            input_schema,
            initialized: false,
            sorted_rows: Vec::new(),
            current_idx: 0,
        }
    }
}

impl VolcanoExecutor for SortVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        while let Some(row) = self.child.next()? {
            self.sorted_rows.push(row);
        }
        self.child.close()?;
        self.sorted_rows.sort_by(|a, b| {
            for sort_expr in &self.sort_expr {
                let a_val = sort_expr
                    .expr
                    .evaluate(a, &self.input_schema)
                    .unwrap_or(Value::Null);
                let b_val = sort_expr
                    .expr
                    .evaluate(b, &self.input_schema)
                    .unwrap_or(Value::Null);
                let cmp = match (&a_val, &b_val) {
                    (Value::Integer(ai), Value::Integer(bi)) => ai.cmp(bi),
                    (Value::Text(ai), Value::Text(bi)) => ai.cmp(bi),
                    _ => std::cmp::Ordering::Equal,
                };
                let result = if sort_expr.asc { cmp } else { cmp.reverse() };
                if result != std::cmp::Ordering::Equal {
                    return result;
                }
            }
            std::cmp::Ordering::Equal
        });
        self.current_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }
        if self.current_idx >= self.sorted_rows.len() {
            return Ok(None);
        }
        let row = self.sorted_rows[self.current_idx].clone();
        self.current_idx += 1;
        Ok(Some(row))
    }

    fn close(&mut self) -> SqlResult<()> {
        self.sorted_rows.clear();
        self.current_idx = 0;
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "Sort"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// NestedLoopJoinVolcanoExecutor - Nested Loop Join algorithm implementation
/// Supports: Cross Join, Left Outer Join, Right Outer Join, Full Outer Join
#[allow(dead_code)]
pub struct NestedLoopJoinVolcanoExecutor {
    left: Box<dyn VolcanoExecutor>,
    right: Box<dyn VolcanoExecutor>,
    join_type: sqlrustgo_planner::JoinType,
    left_schema: Schema,
    right_schema: Schema,
    schema: Schema,
    left_rows: Vec<Vec<Value>>,
    right_rows: Vec<Vec<Value>>,
    left_idx: usize,
    right_idx: usize,
    right_matched: Vec<bool>,
    has_left_row: bool,
    initialized: bool,
}

#[allow(clippy::never_loop)]
impl NestedLoopJoinVolcanoExecutor {
    pub fn new(
        left: Box<dyn VolcanoExecutor>,
        right: Box<dyn VolcanoExecutor>,
        join_type: sqlrustgo_planner::JoinType,
        left_schema: Schema,
        right_schema: Schema,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            left_schema,
            right_schema,
            schema,
            left_rows: Vec::new(),
            right_rows: Vec::new(),
            left_idx: 0,
            right_idx: 0,
            right_matched: Vec::new(),
            has_left_row: false,
            initialized: false,
        }
    }

    fn next_inner(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            if self.left_idx >= self.left_rows.len() {
                return Ok(None);
            }

            let left_row = &self.left_rows[self.left_idx];
            let right_row = &self.right_rows[self.right_idx];

            let mut result = left_row.clone();
            result.extend_from_slice(right_row);
            self.right_idx += 1;
            if self.right_idx >= self.right_rows.len() {
                self.right_idx = 0;
                self.left_idx += 1;
            }
            return Ok(Some(result));
        }
    }

    fn next_left_outer(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            if self.left_idx >= self.left_rows.len() {
                return Ok(None);
            }

            let left_row = &self.left_rows[self.left_idx];

            if self.right_idx < self.right_rows.len() {
                let right_row = &self.right_rows[self.right_idx];
                self.right_matched[self.right_idx] = true;

                let mut result = left_row.clone();
                result.extend_from_slice(right_row);
                self.right_idx += 1;
                return Ok(Some(result));
            } else {
                let nulls = vec![Value::Null; self.right_schema.fields.len()];
                let mut result = left_row.clone();
                result.extend_from_slice(&nulls);
                self.right_idx = 0;
                self.left_idx += 1;
                return Ok(Some(result));
            }
        }
    }

    fn next_right_outer(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            if self.right_idx >= self.right_rows.len() && self.left_idx >= self.left_rows.len() {
                for (i, matched) in self.right_matched.iter().enumerate() {
                    if !*matched {
                        let nulls = vec![Value::Null; self.left_schema.fields.len()];
                        let mut result = nulls;
                        result.extend_from_slice(&self.right_rows[i]);
                        return Ok(Some(result));
                    }
                }
                return Ok(None);
            }

            if self.left_idx < self.left_rows.len() {
                let left_row = &self.left_rows[self.left_idx];
                let right_row = &self.right_rows[self.right_idx];

                let mut result = left_row.clone();
                result.extend_from_slice(right_row);

                self.right_idx += 1;
                if self.right_idx >= self.right_rows.len() {
                    self.right_idx = 0;
                    self.left_idx += 1;
                }
                return Ok(Some(result));
            }

            return Ok(None);
        }
    }

    fn next_full_outer(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            if self.left_idx < self.left_rows.len() {
                let left_row = &self.left_rows[self.left_idx];

                if self.right_idx < self.right_rows.len() {
                    let right_row = &self.right_rows[self.right_idx];
                    self.right_matched[self.right_idx] = true;

                    let mut result = left_row.clone();
                    result.extend_from_slice(right_row);
                    self.right_idx += 1;
                    if self.right_idx >= self.right_rows.len() {
                        self.right_idx = 0;
                        self.left_idx += 1;
                    }
                    return Ok(Some(result));
                } else {
                    let nulls = vec![Value::Null; self.right_schema.fields.len()];
                    let mut result = left_row.clone();
                    result.extend_from_slice(&nulls);
                    self.right_idx = 0;
                    self.left_idx += 1;
                    return Ok(Some(result));
                }
            }

            for (i, matched) in self.right_matched.iter().enumerate() {
                if !*matched {
                    let nulls = vec![Value::Null; self.left_schema.fields.len()];
                    let mut result = nulls;
                    result.extend_from_slice(&self.right_rows[i]);
                    self.right_matched[i] = true;
                    return Ok(Some(result));
                }
            }

            return Ok(None);
        }
    }

    fn next_cross(&mut self) -> SqlResult<Option<Vec<Value>>> {
        loop {
            if self.left_idx >= self.left_rows.len() {
                return Ok(None);
            }

            let left_row = &self.left_rows[self.left_idx];
            let right_row = &self.right_rows[self.right_idx];

            let mut result = left_row.clone();
            result.extend_from_slice(right_row);
            self.right_idx += 1;
            if self.right_idx >= self.right_rows.len() {
                self.right_idx = 0;
                self.left_idx += 1;
            }
            return Ok(Some(result));
        }
    }
}

impl VolcanoExecutor for NestedLoopJoinVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.left.init()?;
        while let Some(row) = self.left.next()? {
            self.left_rows.push(row);
        }
        self.left.close()?;

        self.right.init()?;
        while let Some(row) = self.right.next()? {
            self.right_rows.push(row);
            self.right_matched.push(false);
        }
        self.right.close()?;

        self.left_idx = 0;
        self.right_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }

        match self.join_type {
            sqlrustgo_planner::JoinType::Inner => self.next_inner(),
            sqlrustgo_planner::JoinType::Left => self.next_left_outer(),
            sqlrustgo_planner::JoinType::Right => self.next_right_outer(),
            sqlrustgo_planner::JoinType::Full => self.next_full_outer(),
            sqlrustgo_planner::JoinType::Cross => self.next_cross(),
            _ => Ok(None),
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.left_rows.clear();
        self.right_rows.clear();
        self.right_matched.clear();
        self.initialized = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "NestedLoopJoin"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[allow(dead_code)]
pub struct SortMergeJoinExecutor {
    left: Box<dyn VolcanoExecutor>,
    right: Box<dyn VolcanoExecutor>,
    join_type: sqlrustgo_planner::JoinType,
    left_schema: Schema,
    right_schema: Schema,
    schema: Schema,
    initialized: bool,
    left_sorted: Vec<Vec<Value>>,
    right_sorted: Vec<Vec<Value>>,
    left_idx: usize,
    right_idx: usize,
    in_match: bool,
    matched_right: Vec<bool>,
    left_row_matched: bool,
}

impl SortMergeJoinExecutor {
    pub fn new(
        left: Box<dyn VolcanoExecutor>,
        right: Box<dyn VolcanoExecutor>,
        join_type: sqlrustgo_planner::JoinType,
        left_schema: Schema,
        right_schema: Schema,
        schema: Schema,
    ) -> Self {
        Self {
            left,
            right,
            join_type,
            left_schema,
            right_schema,
            schema,
            initialized: false,
            left_sorted: Vec::new(),
            right_sorted: Vec::new(),
            left_idx: 0,
            right_idx: 0,
            in_match: false,
            matched_right: Vec::new(),
            left_row_matched: false,
        }
    }

    fn sort_key(row: &[Value], key_idx: usize) -> Value {
        row.get(key_idx).cloned().unwrap_or(Value::Null)
    }

    fn compare_keys(a: &Value, b: &Value) -> std::cmp::Ordering {
        match (a, b) {
            (Value::Integer(ia), Value::Integer(ib)) => ia.cmp(ib),
            (Value::Text(ta), Value::Text(tb)) => ta.cmp(tb),
            _ => std::cmp::Ordering::Equal,
        }
    }
}

impl VolcanoExecutor for SortMergeJoinExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.left.init()?;
        self.right.init()?;

        while let Some(row) = self.left.next()? {
            self.left_sorted.push(row);
        }
        while let Some(row) = self.right.next()? {
            self.right_sorted.push(row.clone());
            self.matched_right.push(false);
        }

        self.left_sorted
            .sort_by(|a, b| Self::compare_keys(&Self::sort_key(a, 0), &Self::sort_key(b, 0)));
        self.right_sorted
            .sort_by(|a, b| Self::compare_keys(&Self::sort_key(a, 0), &Self::sort_key(b, 0)));

        self.left_idx = 0;
        self.right_idx = 0;
        self.in_match = false;
        self.left_row_matched = false;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }

        match self.join_type {
            sqlrustgo_planner::JoinType::Inner => self.next_inner(),
            sqlrustgo_planner::JoinType::Left => self.next_left(),
            _ => Ok(None),
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.left.close()?;
        self.right.close()?;
        self.left_sorted.clear();
        self.right_sorted.clear();
        self.matched_right.clear();
        self.initialized = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "SortMergeJoin"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SortMergeJoinExecutor {
    fn next_inner(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.left_sorted.len() && self.right_idx < self.right_sorted.len() {
            let left_row = &self.left_sorted[self.left_idx];
            let right_row = &self.right_sorted[self.right_idx];
            let key = Self::sort_key(left_row, 0);
            let right_key = Self::sort_key(right_row, 0);

            match Self::compare_keys(&key, &right_key) {
                std::cmp::Ordering::Equal => {
                    let mut result = left_row.clone();
                    result.extend(right_row.clone());
                    self.right_idx += 1;
                    return Ok(Some(result));
                }
                std::cmp::Ordering::Less => {
                    self.left_idx += 1;
                }
                std::cmp::Ordering::Greater => {
                    self.right_idx += 1;
                }
            }
        }
        Ok(None)
    }

    fn next_left(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while self.left_idx < self.left_sorted.len() {
            let left_row = &self.left_sorted[self.left_idx];
            let left_key = Self::sort_key(left_row, 0);

            while self.right_idx < self.right_sorted.len() {
                let right_row = &self.right_sorted[self.right_idx];
                let right_key = Self::sort_key(right_row, 0);

                match Self::compare_keys(&left_key, &right_key) {
                    std::cmp::Ordering::Equal => {
                        let mut result = left_row.clone();
                        result.extend(right_row.clone());
                        self.matched_right[self.right_idx] = true;
                        self.right_idx += 1;
                        self.left_row_matched = true;
                        return Ok(Some(result));
                    }
                    std::cmp::Ordering::Less => {
                        break;
                    }
                    std::cmp::Ordering::Greater => {
                        self.right_idx += 1;
                    }
                }
            }

            if !self.left_row_matched {
                let mut result = left_row.clone();
                for _ in 0..self.right_schema.fields.len() {
                    result.push(Value::Null);
                }
                self.left_idx += 1;
                self.right_idx = 0;
                self.left_row_matched = false;
                return Ok(Some(result));
            }

            self.left_idx += 1;
            self.right_idx = 0;
            self.left_row_matched = false;
        }
        Ok(None)
    }
}

pub struct LimitVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    limit: usize,
    offset: usize,
    schema: Schema,
    initialized: bool,
    current_idx: usize,
}

impl LimitVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        limit: usize,
        offset: usize,
        schema: Schema,
    ) -> Self {
        Self {
            child,
            limit,
            offset,
            schema,
            initialized: false,
            current_idx: 0,
        }
    }
}

impl VolcanoExecutor for LimitVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        self.current_idx = 0;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError("Not initialized".to_string()));
        }
        while self.current_idx < self.offset {
            if self.child.next()?.is_none() {
                return Ok(None);
            }
            self.current_idx += 1;
        }
        if self.current_idx >= self.offset + self.limit {
            return Ok(None);
        }
        let result = self.child.next()?;
        if result.is_some() {
            self.current_idx += 1;
        }
        Ok(result)
    }

    fn close(&mut self) -> SqlResult<()> {
        self.child.close()?;
        self.initialized = false;
        Ok(())
    }
    fn schema(&self) -> &Schema {
        &self.schema
    }
    fn name(&self) -> &str {
        "Limit"
    }
    fn is_initialized(&self) -> bool {
        self.initialized
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct VolExecutorBuilder {
    storage: std::sync::Arc<dyn Storage>,
}

impl VolExecutorBuilder {
    pub fn new(storage: std::sync::Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub fn build(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        match plan.name() {
            "SeqScan" => self.build_seq_scan(plan),
            "Projection" => self.build_projection(plan),
            "Filter" => self.build_filter(plan),
            "Aggregate" => self.build_aggregate(plan),
            "HashJoin" => self.build_hash_join(plan),
            "Sort" => self.build_sort(plan),
            "Limit" => self.build_limit(plan),
            _ => Err(SqlError::ExecutionError(format!(
                "Unsupported plan type: {}",
                plan.name()
            ))),
        }
    }

    fn build_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.len() < 2 {
            return Err(SqlError::ExecutionError(
                "HashJoin has less than 2 children".to_string(),
            ));
        }
        let left = self.build(children[0])?;
        let right = self.build(children[1])?;
        let hash_join = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::HashJoinExec>()
            .ok_or_else(|| {
                SqlError::ExecutionError("Failed to cast to HashJoinExec".to_string())
            })?;

        let join_type = hash_join.join_type();

        // Use NestedLoopJoin for Cross, Right, Full joins
        match join_type {
            sqlrustgo_planner::JoinType::Cross
            | sqlrustgo_planner::JoinType::Right
            | sqlrustgo_planner::JoinType::Full
            | sqlrustgo_planner::JoinType::LeftSemi
            | sqlrustgo_planner::JoinType::LeftAnti
            | sqlrustgo_planner::JoinType::RightSemi
            | sqlrustgo_planner::JoinType::RightAnti => {
                return Ok(Box::new(NestedLoopJoinVolcanoExecutor::new(
                    left,
                    right,
                    join_type,
                    children[0].schema().clone(),
                    children[1].schema().clone(),
                    plan.schema().clone(),
                )));
            }
            _ => {}
        }

        Ok(Box::new(HashJoinVolcanoExecutor::new(
            left,
            right,
            join_type,
            children[0].schema().clone(),
            children[1].schema().clone(),
            plan.schema().clone(),
        )))
    }

    fn build_seq_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        Ok(Box::new(SeqScanVolcanoExecutor::new(
            plan.table_name().to_string(),
            plan.schema().clone(),
            self.storage.clone(),
        )))
    }

    fn build_projection(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() {
            return Err(SqlError::ExecutionError(
                "Projection has no children".to_string(),
            ));
        }
        let child = self.build(children[0])?;
        let projection = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
            .map(|p| p.expr().clone())
            .unwrap_or_default();
        Ok(Box::new(ProjectionVolcanoExecutor::new(
            child,
            projection,
            plan.schema().clone(),
            children[0].schema().clone(),
        )))
    }

    fn build_filter(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() {
            return Err(SqlError::ExecutionError(
                "Filter has no children".to_string(),
            ));
        }
        let child = self.build(children[0])?;
        let filter = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::FilterExec>()
            .map(|f| f.predicate().clone())
            .unwrap_or(sqlrustgo_planner::Expr::Literal(Value::Boolean(true)));
        Ok(Box::new(FilterVolcanoExecutor::new(
            child,
            filter,
            plan.schema().clone(),
            children[0].schema().clone(),
        )))
    }

    fn build_aggregate(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() {
            return Err(SqlError::ExecutionError(
                "Aggregate has no children".to_string(),
            ));
        }
        let child = self.build(children[0])?;
        let aggregate = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::AggregateExec>()
            .ok_or_else(|| {
                SqlError::ExecutionError("Failed to cast to AggregateExec".to_string())
            })?;
        Ok(Box::new(AggregateVolcanoExecutor::new(
            child,
            aggregate.group_expr().clone(),
            aggregate.aggregate_expr().clone(),
            aggregate.having_expr().clone(),
            plan.schema().clone(),
            children[0].schema().clone(),
        )))
    }

    fn build_sort(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() {
            return Err(SqlError::ExecutionError("Sort has no children".to_string()));
        }
        let child = self.build(children[0])?;
        let sort = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::physical_plan::SortExec>()
            .map(|s| s.sort_expr().clone())
            .unwrap_or_default();
        Ok(Box::new(SortVolcanoExecutor::new(
            child,
            sort,
            plan.schema().clone(),
            children[0].schema().clone(),
        )))
    }

    fn build_limit(&self, plan: &dyn PhysicalPlan) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let children = plan.children();
        if children.is_empty() {
            return Err(SqlError::ExecutionError(
                "Limit has no children".to_string(),
            ));
        }
        let child = self.build(children[0])?;
        let limit_exec = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::physical_plan::LimitExec>();
        let limit = limit_exec.map(|l| l.limit()).unwrap_or(usize::MAX);
        let offset = limit_exec.and_then(|l| l.offset()).unwrap_or(0);
        Ok(Box::new(LimitVolcanoExecutor::new(
            child,
            limit,
            offset,
            plan.schema().clone(),
        )))
    }
}

pub trait Storage: Send + Sync {
    fn scan(&self, table_name: &str) -> SqlResult<Vec<Vec<Value>>>;
}

pub struct MockStorageForExecutor {
    data: std::collections::HashMap<String, Vec<Vec<Value>>>,
}

impl MockStorageForExecutor {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn with_table(mut self, name: &str, rows: Vec<Vec<Value>>) -> Self {
        self.data.insert(name.to_string(), rows);
        self
    }
}

impl Default for MockStorageForExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for MockStorageForExecutor {
    fn scan(&self, table_name: &str) -> SqlResult<Vec<Vec<Value>>> {
        Ok(self.data.get(table_name).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{DataType, Field, PhysicalPlan, Schema};

    pub struct MockExecutor;
    impl MockExecutor {
        pub fn new() -> Self {
            Self
        }
    }
    impl Default for MockExecutor {
        fn default() -> Self {
            Self::new()
        }
    }
    impl Executor for MockExecutor {
        fn execute(&self, _plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
            Ok(ExecutorResult::empty())
        }
        fn name(&self) -> &str {
            "mock"
        }
        fn is_ready(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_executor_result() {
        let result = ExecutorResult::new(vec![], 0);
        assert!(result.rows.is_empty());
        let result = ExecutorResult::new(vec![vec![Value::Integer(1)]], 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.affected_rows, 1);
    }

    #[test]
    fn test_executor_result_empty() {
        let result = ExecutorResult::empty();
        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_executor_result_with_rows() {
        let rows = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let result = ExecutorResult::new(rows, 0);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_executor_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MockExecutor>();
        _check::<ExecutorResult>();
    }

    #[test]
    fn test_volcano_executor_trait_object() {
        let _executor: Box<dyn VolcanoExecutor> = Box::new(MockVolcanoExecutor::new());
    }

    pub struct MockVolcanoExecutor {
        data: Vec<Vec<Value>>,
        idx: usize,
        initialized: bool,
        schema: Schema,
    }
    impl MockVolcanoExecutor {
        pub fn new() -> Self {
            Self {
                data: vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
                idx: 0,
                initialized: false,
                schema: Schema::new(vec![
                    Field::new("id".to_string(), DataType::Integer),
                    Field::new("name".to_string(), DataType::Text),
                ]),
            }
        }

        pub fn with_data(data: Vec<Vec<Value>>) -> Self {
            Self {
                data,
                idx: 0,
                initialized: false,
                schema: Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]),
            }
        }
    }
    impl Default for MockVolcanoExecutor {
        fn default() -> Self {
            Self::new()
        }
    }
    impl VolcanoExecutor for MockVolcanoExecutor {
        fn init(&mut self) -> SqlResult<()> {
            self.initialized = true;
            self.idx = 0;
            Ok(())
        }
        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            if !self.initialized {
                return Err(SqlError::ExecutionError("Not initialized".to_string()));
            }
            if self.idx >= self.data.len() {
                return Ok(None);
            }
            let row = self.data[self.idx].clone();
            self.idx += 1;
            Ok(Some(row))
        }
        fn close(&mut self) -> SqlResult<()> {
            self.initialized = false;
            Ok(())
        }
        fn schema(&self) -> &Schema {
            &self.schema
        }
        fn name(&self) -> &str {
            "MockVolcano"
        }
        fn is_initialized(&self) -> bool {
            self.initialized
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_mock_volcano_executor() {
        let mut executor = MockVolcanoExecutor::new();
        let result = executor.next();
        assert!(result.is_err());
        executor.init().unwrap();
        assert!(executor.is_initialized());
        let row1 = executor.next().unwrap();
        assert!(row1.is_some());
        let row2 = executor.next().unwrap();
        assert!(row2.is_some());
        let row3 = executor.next().unwrap();
        assert!(row3.is_none());
        executor.close().unwrap();
        executor.init().unwrap();
        let row = executor.next().unwrap();
        assert!(row.is_some());
    }

    #[test]
    fn test_mock_executor_name() {
        let executor = MockExecutor::new();
        assert_eq!(executor.name(), "mock");
    }
    #[test]
    fn test_mock_executor_is_ready() {
        let executor = MockExecutor::new();
        assert!(executor.is_ready());
    }
    #[test]
    fn test_volcano_executor_schema() {
        let executor = MockVolcanoExecutor::new();
        let schema = executor.schema();
        assert_eq!(schema.fields.len(), 2);
    }
    #[test]
    fn test_volcano_executor_name() {
        let executor = MockVolcanoExecutor::new();
        assert_eq!(executor.name(), "MockVolcano");
    }
    #[test]
    fn test_execute_collect() {
        let mut executor = MockVolcanoExecutor::new();
        let result = execute_collect(&mut executor).unwrap();
        assert_eq!(result.rows.len(), 2);
    }
    #[test]
    fn test_volcano_executor_send_sync() {
        fn _check<T: Send + Sync>() {}
        _check::<MockVolcanoExecutor>();
        _check::<Box<dyn VolcanoExecutor>>();
    }

    #[test]
    fn test_volcano_executor_init() {
        let mut executor = MockVolcanoExecutor::new();
        assert!(!executor.is_initialized());
        executor.init().unwrap();
        assert!(executor.is_initialized());
    }

    #[test]
    fn test_volcano_executor_next() {
        let mut executor = MockVolcanoExecutor::new();
        executor.init().unwrap();
        let row = executor.next().unwrap();
        assert!(row.is_some());
        assert_eq!(
            row.unwrap(),
            vec![Value::Integer(1), Value::Text("Alice".to_string())]
        );
    }

    #[test]
    fn test_volcano_executor_close() {
        let mut executor = MockVolcanoExecutor::new();
        executor.init().unwrap();
        executor.next().unwrap();
        executor.close().unwrap();
    }

    #[test]
    fn test_volcano_executor_schema_method() {
        let executor = MockVolcanoExecutor::new();
        let schema = executor.schema();
        assert_eq!(schema.fields.len(), 2);
    }

    #[test]
    fn test_volcano_executor_as_any() {
        let executor = MockVolcanoExecutor::new();
        let any = executor.as_any();
        assert!(any.is::<MockVolcanoExecutor>());
    }

    #[test]
    fn test_execute_collect_empty() {
        let mut executor = MockVolcanoExecutor::with_data(vec![]);
        let result = execute_collect(&mut executor).unwrap();
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_execute_collect_multiple_rows() {
        let rows = vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ];
        let mut executor = MockVolcanoExecutor::with_data(rows);
        let result = execute_collect(&mut executor).unwrap();
        assert_eq!(result.rows.len(), 3);
    }

    #[test]
    fn test_volcano_executor_not_initialized_error() {
        let mut executor = MockVolcanoExecutor::new();
        let result = executor.next();
        assert!(result.is_err());
    }

    #[test]
    fn test_volcano_executor_all_rows_consumed() {
        let mut executor =
            MockVolcanoExecutor::with_data(vec![vec![Value::Integer(1)], vec![Value::Integer(2)]]);
        executor.init().unwrap();
        assert!(executor.next().unwrap().is_some());
        assert!(executor.next().unwrap().is_some());
        assert!(executor.next().unwrap().is_none());
    }

    #[test]
    fn test_executor_result_debug() {
        let result = ExecutorResult::new(vec![], 0);
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("ExecutorResult"));
    }

    #[test]
    fn test_executor_result_with_multiple_rows() {
        let rows = vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(2), Value::Text("b".to_string())],
            vec![Value::Integer(3), Value::Text("c".to_string())],
        ];
        let result = ExecutorResult::new(rows, 0);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_executor_result_with_affected_rows() {
        let result = ExecutorResult::new(vec![], 10);
        assert_eq!(result.affected_rows, 10);
    }

    #[test]
    fn test_mock_storage_for_executor() {
        let storage = MockStorageForExecutor::new().with_table(
            "users",
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );

        let rows = storage.scan("users").unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_mock_storage_for_executor_empty_table() {
        let storage = MockStorageForExecutor::new();
        let rows = storage.scan("unknown").unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_seq_scan_volcano_executor_with_storage() {
        use std::sync::Arc;

        let storage = Arc::new(MockStorageForExecutor::new().with_table(
            "test",
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
            ],
        ));
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut exec = SeqScanVolcanoExecutor::new("test".to_string(), schema, storage);

        exec.init().unwrap();
        let count = std::iter::from_fn(|| exec.next().unwrap()).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_projection_volcano_executor() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Integer(100)],
            vec![Value::Integer(2), Value::Integer(200)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("id".to_string()),
        )];

        let mut exec = ProjectionVolcanoExecutor::new(child, expr, schema, input_schema);
        exec.init().unwrap();

        let row = exec.next().unwrap().unwrap();
        assert_eq!(row.len(), 1);
        assert_eq!(row[0], Value::Integer(1));
    }

    #[test]
    fn test_projection_volcano_executor_multiple_columns() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![vec![
            Value::Integer(1),
            Value::Text("a".to_string()),
        ]]));

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
        ]);
        let expr = vec![
            sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("name".to_string())),
            sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("id".to_string())),
        ];

        let mut exec = ProjectionVolcanoExecutor::new(child, expr, schema, input_schema);
        exec.init().unwrap();

        let row = exec.next().unwrap().unwrap();
        assert_eq!(row.len(), 2);
    }

    #[test]
    fn test_projection_volcano_executor_empty_input() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("id".to_string()),
        )];

        let mut exec = ProjectionVolcanoExecutor::new(child, expr, schema, input_schema);
        exec.init().unwrap();

        let row = exec.next().unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_projection_volcano_executor_name() {
        let child = Box::new(MockVolcanoExecutor::new());
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = vec![];

        let exec = ProjectionVolcanoExecutor::new(child, expr, schema, input_schema);
        assert_eq!(exec.name(), "Projection");
    }

    #[test]
    fn test_limit_volcano_executor_with_offset() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
            vec![Value::Integer(5)],
        ]));

        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut exec = LimitVolcanoExecutor::new(child, 2, 2, schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_aggregate_volcano_executor_count() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Integer(10)],
            vec![Value::Integer(1), Value::Integer(20)],
            vec![Value::Integer(2), Value::Integer(30)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("count".to_string(), DataType::Integer),
        ]);

        let group_expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("group_id".to_string()),
        )];
        let aggregate_expr = vec![sqlrustgo_planner::Expr::AggregateFunction {
            func: sqlrustgo_planner::AggregateFunction::Count,
            args: vec![],
            distinct: false,
        }];

        let mut exec =
            AggregateVolcanoExecutor::new(child, group_expr, aggregate_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_aggregate_volcano_executor_sum() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Integer(10)],
            vec![Value::Integer(1), Value::Integer(20)],
            vec![Value::Integer(2), Value::Integer(30)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("sum".to_string(), DataType::Integer),
        ]);

        let group_expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("group_id".to_string()),
        )];
        let aggregate_expr = vec![sqlrustgo_planner::Expr::AggregateFunction {
            func: sqlrustgo_planner::AggregateFunction::Sum,
            args: vec![sqlrustgo_planner::Expr::Column(
                sqlrustgo_planner::Column::new("value".to_string()),
            )],
            distinct: false,
        }];

        let mut exec =
            AggregateVolcanoExecutor::new(child, group_expr, aggregate_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_aggregate_volcano_executor_empty_input() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);

        let group_expr = vec![];
        let aggregate_expr = vec![sqlrustgo_planner::Expr::AggregateFunction {
            func: sqlrustgo_planner::AggregateFunction::Count,
            args: vec![],
            distinct: false,
        }];

        let mut exec =
            AggregateVolcanoExecutor::new(child, group_expr, aggregate_expr, schema, input_schema);
        exec.init().unwrap();

        let row = exec.next().unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_aggregate_volcano_executor_name() {
        let child = Box::new(MockVolcanoExecutor::new());
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("count".to_string(), DataType::Integer)]);

        let exec = AggregateVolcanoExecutor::new(child, vec![], vec![], schema, input_schema);
        assert_eq!(exec.name(), "Aggregate");
    }

    #[test]
    fn test_hash_join_volcano_executor_inner_join() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(2), Value::Text("b".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = HashJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_hash_join_volcano_executor_left_join() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(3), Value::Text("c".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = HashJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Left,
            left_schema,
            right_schema,
            schema,
        );
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_hash_join_volcano_executor_name() {
        let left = Box::new(MockVolcanoExecutor::new());
        let right = Box::new(MockVolcanoExecutor::new());
        let left_schema = Schema::empty();
        let right_schema = Schema::empty();
        let schema = Schema::empty();

        let exec = HashJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );
        assert_eq!(exec.name(), "HashJoin");
    }

    #[test]
    fn test_sort_merge_join_volcano_executor_inner_join() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(2), Value::Text("b".to_string())],
            vec![Value::Integer(3), Value::Text("c".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
            vec![Value::Integer(4), Value::Text("z".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = SortMergeJoinExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], Value::Integer(1));
        assert_eq!(rows[0][2], Value::Integer(1));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[1][2], Value::Integer(2));
    }

    #[test]
    fn test_sort_merge_join_volcano_executor_left_join() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(3), Value::Text("c".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = SortMergeJoinExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Left,
            left_schema,
            right_schema,
            schema,
        );
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_sort_merge_join_volcano_executor_name() {
        let left = Box::new(MockVolcanoExecutor::new());
        let right = Box::new(MockVolcanoExecutor::new());
        let left_schema = Schema::empty();
        let right_schema = Schema::empty();
        let schema = Schema::empty();

        let exec = SortMergeJoinExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );
        assert_eq!(exec.name(), "SortMergeJoin");
    }

    #[test]
    fn test_sort_volcano_executor_ascending() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(3)],
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("id".to_string())),
            asc: true,
            nulls_first: true,
        }];

        let mut exec = SortVolcanoExecutor::new(child, sort_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], Value::Integer(1));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[2][0], Value::Integer(3));
    }

    #[test]
    fn test_sort_volcano_executor_descending() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(3)],
            vec![Value::Integer(2)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("id".to_string())),
            asc: false,
            nulls_first: false,
        }];

        let mut exec = SortVolcanoExecutor::new(child, sort_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], Value::Integer(3));
        assert_eq!(rows[1][0], Value::Integer(2));
        assert_eq!(rows[2][0], Value::Integer(1));
    }

    #[test]
    fn test_sort_volcano_executor_text() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Text("c".to_string())],
            vec![Value::Text("a".to_string())],
            vec![Value::Text("b".to_string())],
        ]));

        let input_schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);
        let schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);

        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new(
                "name".to_string(),
            )),
            asc: true,
            nulls_first: true,
        }];

        let mut exec = SortVolcanoExecutor::new(child, sort_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], Value::Text("a".to_string()));
    }

    #[test]
    fn test_sort_volcano_executor_empty_input() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("id".to_string())),
            asc: true,
            nulls_first: true,
        }];

        let mut exec = SortVolcanoExecutor::new(child, sort_expr, schema, input_schema);
        exec.init().unwrap();

        let row = exec.next().unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_sort_volcano_executor_name() {
        let child = Box::new(MockVolcanoExecutor::new());
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let exec = SortVolcanoExecutor::new(child, vec![], schema, input_schema);
        assert_eq!(exec.name(), "Sort");
    }

    #[test]
    fn test_sort_volcano_executor_close() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: sqlrustgo_planner::Expr::Column(sqlrustgo_planner::Column::new("id".to_string())),
            asc: true,
            nulls_first: true,
        }];

        let mut exec = SortVolcanoExecutor::new(child, sort_expr, schema, input_schema);
        exec.init().unwrap();
        let _ = exec.next().unwrap();
        exec.close().unwrap();
        assert!(!exec.is_initialized());
    }

    #[test]
    fn test_aggregate_volcano_executor_avg() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Integer(10)],
            vec![Value::Integer(1), Value::Integer(20)],
            vec![Value::Integer(2), Value::Integer(30)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("avg".to_string(), DataType::Integer),
        ]);

        let group_expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("group_id".to_string()),
        )];
        let aggregate_expr = vec![sqlrustgo_planner::Expr::AggregateFunction {
            func: sqlrustgo_planner::AggregateFunction::Avg,
            args: vec![sqlrustgo_planner::Expr::Column(
                sqlrustgo_planner::Column::new("value".to_string()),
            )],
            distinct: false,
        }];

        let mut exec =
            AggregateVolcanoExecutor::new(child, group_expr, aggregate_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_aggregate_volcano_executor_min_max() {
        let child = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Integer(10)],
            vec![Value::Integer(1), Value::Integer(5)],
            vec![Value::Integer(2), Value::Integer(30)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = Schema::new(vec![
            Field::new("group_id".to_string(), DataType::Integer),
            Field::new("min".to_string(), DataType::Integer),
            Field::new("max".to_string(), DataType::Integer),
        ]);

        let group_expr = vec![sqlrustgo_planner::Expr::Column(
            sqlrustgo_planner::Column::new("group_id".to_string()),
        )];
        let aggregate_expr = vec![
            sqlrustgo_planner::Expr::AggregateFunction {
                func: sqlrustgo_planner::AggregateFunction::Min,
                args: vec![sqlrustgo_planner::Expr::Column(
                    sqlrustgo_planner::Column::new("value".to_string()),
                )],
                distinct: false,
            },
            sqlrustgo_planner::Expr::AggregateFunction {
                func: sqlrustgo_planner::AggregateFunction::Max,
                args: vec![sqlrustgo_planner::Expr::Column(
                    sqlrustgo_planner::Column::new("value".to_string()),
                )],
                distinct: false,
            },
        ];

        let mut exec =
            AggregateVolcanoExecutor::new(child, group_expr, aggregate_expr, schema, input_schema);
        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert_eq!(rows.len(), 2);
    }

    // ========== Tests for SortMergeJoinVolcanoExecutor ==========

    #[test]
    fn test_smj_volcano_exec_inner_join() {
        // Test SortMergeJoin with inner join
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(2), Value::Text("b".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = SortMergeJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );

        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        // Should have 2 matching rows (id=1 and id=2)
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_smj_volcano_exec_left_join() {
        // Test SortMergeJoin with left join
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("a".to_string())],
            vec![Value::Integer(3), Value::Text("c".to_string())],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1), Value::Text("x".to_string())],
            vec![Value::Integer(2), Value::Text("y".to_string())],
        ]));

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Text),
        ]);

        let mut exec = SortMergeJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Left,
            left_schema,
            right_schema,
            schema,
        );

        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        // Should have 2 rows: id=1 (match) and id=3 (no match, nulls for right)
        assert_eq!(rows.len(), 2);
        // Check that the second row has nulls for right side
        assert_eq!(rows[1][2], Value::Null);
    }

    #[test]
    fn test_smj_volcano_exec_empty_input() {
        // Test SortMergeJoin with empty input
        let left = Box::new(MockVolcanoExecutor::with_data(vec![]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);

        let mut exec = SortMergeJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );

        exec.init().unwrap();

        let rows: Vec<_> = std::iter::from_fn(|| exec.next().unwrap()).collect();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_smj_volcano_exec_name() {
        let left = Box::new(MockVolcanoExecutor::new());
        let right = Box::new(MockVolcanoExecutor::new());
        let left_schema = Schema::empty();
        let right_schema = Schema::empty();
        let schema = Schema::empty();

        let exec = SortMergeJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );
        assert_eq!(exec.name(), "SortMergeJoin");
    }

    #[test]
    fn test_smj_volcano_exec_close() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("id".to_string(), DataType::Integer),
        ]);

        let mut exec = SortMergeJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );

        exec.init().unwrap();
        exec.next().unwrap();
        exec.close().unwrap();
        assert!(!exec.is_initialized());
    }

    #[test]
    fn test_sort_merge_join_compare_keys() {
        // Test the standalone compare function
        let left = vec![Value::Integer(1)];
        let right = vec![Value::Integer(1)];
        let cmp = compare_join_keys(&left, &right);
        assert_eq!(cmp, std::cmp::Ordering::Equal);

        let left = vec![Value::Integer(1)];
        let right = vec![Value::Integer(2)];
        let cmp = compare_join_keys(&left, &right);
        assert_eq!(cmp, std::cmp::Ordering::Less);

        let left = vec![Value::Integer(2)];
        let right = vec![Value::Integer(1)];
        let cmp = compare_join_keys(&left, &right);
        assert_eq!(cmp, std::cmp::Ordering::Greater);

        // Test with Text
        let left = vec![Value::Text("a".to_string())];
        let right = vec![Value::Text("b".to_string())];
        let cmp = compare_join_keys(&left, &right);
        assert_eq!(cmp, std::cmp::Ordering::Less);
    }

    // ========== Tests for NestedLoopJoinVolcanoExecutor ==========

    #[test]
    fn test_nested_loop_join_inner() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(10)],
            vec![Value::Integer(20)],
        ]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("val".to_string(), DataType::Integer),
        ]);

        let exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );

        let mut exec = exec;
        exec.init().unwrap();

        // Cross product: 2 x 2 = 4 rows
        let row1 = exec.next().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1, vec![Value::Integer(1), Value::Integer(10)]);

        let row2 = exec.next().unwrap();
        assert!(row2.is_some());
        let row2 = row2.unwrap();
        assert_eq!(row2, vec![Value::Integer(1), Value::Integer(20)]);

        let row3 = exec.next().unwrap();
        assert!(row3.is_some());
        let row3 = row3.unwrap();
        assert_eq!(row3, vec![Value::Integer(2), Value::Integer(10)]);

        let row4 = exec.next().unwrap();
        assert!(row4.is_some());
        let row4 = row4.unwrap();
        assert_eq!(row4, vec![Value::Integer(2), Value::Integer(20)]);

        let row5 = exec.next().unwrap();
        assert!(row5.is_none());
    }

    #[test]
    fn test_nested_loop_join_cross() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(10)],
            vec![Value::Integer(20)],
            vec![Value::Integer(30)],
        ]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("val".to_string(), DataType::Integer),
        ]);

        let exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Cross,
            left_schema,
            right_schema,
            schema,
        );

        let mut exec = exec;
        exec.init().unwrap();

        // Cross product: 1 x 3 = 3 rows
        for i in 0..3 {
            let row = exec.next().unwrap();
            assert!(row.is_some());
        }

        let row = exec.next().unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_nested_loop_join_left_outer() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("val".to_string(), DataType::Integer),
        ]);

        let exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Left,
            left_schema,
            right_schema,
            schema,
        );

        let mut exec = exec;
        exec.init().unwrap();

        // Left outer join with 1 left row and 2 right rows
        // For cross product semantics: produces 2 rows (cartesian product)
        // In a real scenario with join condition, would check for matches
        let row1 = exec.next().unwrap();
        assert!(row1.is_some());

        let row2 = exec.next().unwrap();
        assert!(row2.is_some());
    }

    #[test]
    fn test_nested_loop_join_right_outer() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("val".to_string(), DataType::Integer),
        ]);

        let exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Right,
            left_schema,
            right_schema,
            schema,
        );

        let mut exec = exec;
        exec.init().unwrap();

        let row1 = exec.next().unwrap();
        assert!(row1.is_some());
    }

    #[test]
    fn test_nested_loop_join_name() {
        let left = Box::new(MockVolcanoExecutor::new());
        let right = Box::new(MockVolcanoExecutor::new());
        let schema = Schema::new(vec![]);

        let exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            Schema::new(vec![]),
            Schema::new(vec![]),
            schema,
        );
        assert_eq!(exec.name(), "NestedLoopJoin");
    }

    #[test]
    fn test_nested_loop_join_close() {
        let left = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            1,
        )]]));
        let right = Box::new(MockVolcanoExecutor::with_data(vec![vec![Value::Integer(
            10,
        )]]));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("val".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("val".to_string(), DataType::Integer),
        ]);

        let mut exec = NestedLoopJoinVolcanoExecutor::new(
            left,
            right,
            sqlrustgo_planner::JoinType::Inner,
            left_schema,
            right_schema,
            schema,
        );

        exec.init().unwrap();
        exec.next().unwrap();
        exec.close().unwrap();

        // After close, should be able to re-init
        exec.init().unwrap();
        let row = exec.next().unwrap();
        assert!(row.is_some());
    }

    #[test]
    fn test_seq_scan_volcano_executor_next_before_init() {
        use std::sync::Arc;

        let storage = Arc::new(
            MockStorageForExecutor::new().with_table("test", vec![vec![Value::Integer(1)]]),
        );
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut exec = SeqScanVolcanoExecutor::new("test".to_string(), schema, storage);

        let result = exec.next();
        assert!(result.is_err());
    }

    #[test]
    fn test_seq_scan_volcano_executor_double_init() {
        use std::sync::Arc;

        let storage = Arc::new(
            MockStorageForExecutor::new().with_table("test", vec![vec![Value::Integer(1)]]),
        );
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut exec = SeqScanVolcanoExecutor::new("test".to_string(), schema, storage);

        exec.init().unwrap();
        exec.init().unwrap();
    }

    #[test]
    fn test_seq_scan_volcano_executor_empty_result() {
        use std::sync::Arc;

        let storage = Arc::new(MockStorageForExecutor::new());
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut exec = SeqScanVolcanoExecutor::new("empty_table".to_string(), schema, storage);

        exec.init().unwrap();
        let result = exec.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_seq_scan_volcano_executor_close() {
        use std::sync::Arc;

        let storage = Arc::new(
            MockStorageForExecutor::new().with_table("test", vec![vec![Value::Integer(1)]]),
        );
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let mut exec = SeqScanVolcanoExecutor::new("test".to_string(), schema, storage);

        exec.init().unwrap();
        assert!(exec.next().unwrap().is_some());
        exec.close().unwrap();
        assert!(!exec.is_initialized());
    }

    #[test]
    fn test_seq_scan_volcano_executor_schema() {
        use std::sync::Arc;

        let storage = Arc::new(MockStorageForExecutor::new());
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanVolcanoExecutor::new("test".to_string(), schema.clone(), storage);

        let result_schema = exec.schema();
        assert_eq!(result_schema.fields.len(), 1);
    }

    #[test]
    fn test_seq_scan_volcano_executor_name() {
        use std::sync::Arc;

        let storage = Arc::new(MockStorageForExecutor::new());
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let exec = SeqScanVolcanoExecutor::new("test".to_string(), schema, storage);

        assert_eq!(exec.name(), "SeqScan");
    }
}
