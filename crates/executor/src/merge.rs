//! MERGE statement executor
//!
//! Implements the SQL MERGE statement which combines INSERT, UPDATE, and DELETE
//! operations in a single statement based on a condition.

use sqlrustgo_parser::{Expression, MergeStatement};
use sqlrustgo_storage::{StorageEngine, TableInfo};
use sqlrustgo_types::{SqlResult, Value};
use std::sync::{Arc, RwLock};

use crate::executor::ExecutorResult;

/// MERGE executor that handles SQL MERGE statements
pub struct MergeExecutor<S: StorageEngine> {
    storage: Arc<RwLock<S>>,
}

impl<S: StorageEngine> MergeExecutor<S> {
    /// Create a new MergeExecutor
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
        Self { storage }
    }

    /// Execute a MERGE statement
    pub fn execute_merge(&self, merge: &MergeStatement) -> SqlResult<ExecutorResult> {
        let target_table = &merge.target_table;
        let source_table = &merge.source_table;

        let source_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(source_table)?
        };

        let target_rows = {
            let storage = self.storage.read().unwrap();
            storage.scan(target_table)?
        };

        let target_table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(target_table)?.clone()
        };

        let source_table_info = {
            let storage = self.storage.read().unwrap();
            storage.get_table_info(source_table)?.clone()
        };

        let target_pk_idx = target_table_info.columns.iter().position(|c| c.primary_key);

        let mut matched_count: usize = 0;
        let mut inserted_count: usize = 0;

        for source_row in source_rows.iter() {
            let matching_target_idx = target_rows.iter().position(|target_row| {
                self.eval_merge_condition(
                    &merge.on_condition,
                    source_row,
                    target_row,
                    &source_table_info,
                    &target_table_info,
                )
            });

            if let Some(idx) = matching_target_idx {
                if let Some(ref clause) = merge.matched_clause {
                    let target_row = &target_rows[idx];
                    let updates: Vec<(usize, Value)> = clause
                        .update_columns
                        .iter()
                        .zip(clause.update_values.iter())
                        .filter_map(|(col, val)| {
                            find_column_index(col, &target_table_info).map(|col_idx| {
                                let evaluated = self.eval_merge_value(
                                    val,
                                    source_row,
                                    target_row,
                                    &source_table_info,
                                    &target_table_info,
                                );
                                (col_idx, evaluated)
                            })
                        })
                        .collect();

                    let filter = target_pk_idx.and_then(|pk_idx| target_row.get(pk_idx).cloned());
                    let mut storage = self.storage.write().unwrap();
                    let _ = storage.update(target_table, filter.as_slice(), &updates);
                    matched_count += 1;
                }
            } else if let Some(ref clause) = merge.not_matched_clause {
                let values: Vec<Value> = clause
                    .insert_values
                    .iter()
                    .map(|val| {
                        self.eval_merge_value(
                            val,
                            source_row,
                            &[],
                            &source_table_info,
                            &target_table_info,
                        )
                    })
                    .collect();

                let mut storage = self.storage.write().unwrap();
                let _ = storage.insert(target_table, vec![values]);
                inserted_count += 1;
            }
        }

        Ok(ExecutorResult::new(vec![], matched_count + inserted_count))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn eval_merge_condition(
        &self,
        condition: &Expression,
        source_row: &[Value],
        target_row: &[Value],
        source_table_info: &TableInfo,
        target_table_info: &TableInfo,
    ) -> bool {
        match condition {
            Expression::BinaryOp(left, op, right) if op.to_uppercase() == "AND" => {
                self.eval_merge_condition(
                    left,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                ) && self.eval_merge_condition(
                    right,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                )
            }
            Expression::BinaryOp(left, op, right) if op.to_uppercase() == "OR" => {
                self.eval_merge_condition(
                    left,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                ) || self.eval_merge_condition(
                    right,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                )
            }
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.eval_merge_expr(
                    left,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                );
                let right_val = self.eval_merge_expr(
                    right,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                );
                sql_compare(op, &left_val, &right_val)
            }
            _ => false,
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn eval_merge_expr(
        &self,
        expr: &Expression,
        source_row: &[Value],
        target_row: &[Value],
        source_table_info: &TableInfo,
        target_table_info: &TableInfo,
    ) -> Value {
        match expr {
            Expression::Literal(_) => expression_to_value(expr),
            Expression::Identifier(name) => {
                if let Some((qualifier, col)) = name.split_once('.') {
                    let qualifier_lower = qualifier.to_lowercase();
                    if qualifier_lower == source_table_info.name.to_lowercase()
                        || qualifier_lower == "source"
                    {
                        if let Some(idx) = source_table_info
                            .columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col))
                        {
                            return source_row.get(idx).cloned().unwrap_or(Value::Null);
                        }
                    }
                    if qualifier_lower == target_table_info.name.to_lowercase()
                        || qualifier_lower == "target"
                    {
                        if let Some(idx) = target_table_info
                            .columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col))
                        {
                            return target_row.get(idx).cloned().unwrap_or(Value::Null);
                        }
                    }
                    Value::Null
                } else if let Some(idx) = target_table_info
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case(name))
                {
                    target_row.get(idx).cloned().unwrap_or(Value::Null)
                } else if let Some(idx) = source_table_info
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case(name))
                {
                    source_row.get(idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expression::BinaryOp(left, op, right) => {
                let l = self.eval_merge_expr(
                    left,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                );
                let r = self.eval_merge_expr(
                    right,
                    source_row,
                    target_row,
                    source_table_info,
                    target_table_info,
                );
                evaluate_binary_op(&l, &r, op)
            }
            _ => Value::Null,
        }
    }

    fn eval_merge_value(
        &self,
        expr: &Expression,
        source_row: &[Value],
        target_row: &[Value],
        source_table_info: &TableInfo,
        target_table_info: &TableInfo,
    ) -> Value {
        self.eval_merge_expr(
            expr,
            source_row,
            target_row,
            source_table_info,
            target_table_info,
        )
    }
}

/// Compare two values for a binary operation
fn evaluate_binary_op(left: &Value, right: &Value, op: &str) -> Value {
    match op.to_uppercase().as_str() {
        "=" | "==" | "IS" => Value::Boolean(left == right),
        "!=" | "<>" => Value::Boolean(left != right),
        ">" => Value::Boolean(compare_values(left, right) > 0),
        ">=" => Value::Boolean(compare_values(left, right) >= 0),
        "<" => Value::Boolean(compare_values(left, right) < 0),
        "<=" => Value::Boolean(compare_values(left, right) <= 0),
        "AND" | "&&" => {
            if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                Value::Boolean(*l && *r)
            } else {
                Value::Boolean(false)
            }
        }
        "OR" | "||" => {
            if let (Value::Boolean(l), Value::Boolean(r)) = (left, right) {
                Value::Boolean(*l || *r)
            } else {
                Value::Boolean(false)
            }
        }
        _ => Value::Null,
    }
}

/// Compare two values and return -1, 0, or 1
fn compare_values(left: &Value, right: &Value) -> i32 {
    match (left, right) {
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r) as i32,
        (Value::Float(l), Value::Float(r)) => {
            if l < r {
                -1
            } else if l > r {
                1
            } else {
                0
            }
        }
        (Value::Text(l), Value::Text(r)) => l.cmp(r) as i32,
        (Value::Null, Value::Null) => 0,
        (Value::Null, _) => -1,
        (_, Value::Null) => 1,
        _ => 0,
    }
}

/// SQL comparison operator
fn sql_compare(op: &str, left: &Value, right: &Value) -> bool {
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        return false;
    }

    match op.to_uppercase().as_str() {
        "=" | "==" => left == right,
        "!=" | "<>" => left != right,
        ">" => compare_values(left, right) > 0,
        ">=" => compare_values(left, right) >= 0,
        "<" => compare_values(left, right) < 0,
        "<=" => compare_values(left, right) <= 0,
        _ => false,
    }
}

/// Find column index in table info
fn find_column_index(col_name: &str, table_info: &TableInfo) -> Option<usize> {
    if let Some((_qualifier, col)) = col_name.split_once('.') {
        table_info
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(col))
    } else {
        table_info
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(col_name))
    }
}

/// Convert an expression to a Value
fn expression_to_value(expr: &Expression) -> Value {
    match expr {
        Expression::Literal(s) => {
            let s = s.trim();
            if s.eq_ignore_ascii_case("NULL") {
                Value::Null
            } else if s.eq_ignore_ascii_case("TRUE") {
                Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("FALSE") {
                Value::Boolean(false)
            } else if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Float(f)
            } else if s.starts_with('\'') && s.ends_with('\'') {
                Value::Text(s[1..s.len() - 1].to_string())
            } else {
                Value::Text(s.to_string())
            }
        }
        Expression::Identifier(name) => Value::Text(name.clone()),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_values_integer() {
        assert_eq!(compare_values(&Value::Integer(1), &Value::Integer(1)), 0);
        assert_eq!(compare_values(&Value::Integer(1), &Value::Integer(2)), -1);
        assert_eq!(compare_values(&Value::Integer(2), &Value::Integer(1)), 1);
    }

    #[test]
    fn test_compare_values_text() {
        assert_eq!(
            compare_values(&Value::Text("a".to_string()), &Value::Text("a".to_string())),
            0
        );
        assert_eq!(
            compare_values(&Value::Text("a".to_string()), &Value::Text("b".to_string())),
            -1
        );
    }

    #[test]
    fn test_compare_values_null() {
        assert_eq!(compare_values(&Value::Null, &Value::Null), 0);
        assert_eq!(compare_values(&Value::Null, &Value::Integer(1)), -1);
        assert_eq!(compare_values(&Value::Integer(1), &Value::Null), 1);
    }

    #[test]
    fn test_evaluate_binary_op() {
        assert_eq!(
            evaluate_binary_op(&Value::Integer(1), &Value::Integer(1), "="),
            Value::Boolean(true)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Integer(1), &Value::Integer(2), "<"),
            Value::Boolean(true)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Boolean(true), &Value::Boolean(true), "AND"),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_expression_to_value() {
        assert_eq!(
            expression_to_value(&Expression::Literal("42".to_string())),
            Value::Integer(42)
        );
        assert_eq!(
            expression_to_value(&Expression::Literal("'hello'".to_string())),
            Value::Text("hello".to_string())
        );
        assert_eq!(
            expression_to_value(&Expression::Literal("NULL".to_string())),
            Value::Null
        );
    }
}
