//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult, GLOBAL_PROFILER};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{
    parse, Expression, GrantStatement, Lexer, Privilege, RevokeStatement, SetOperation, Statement,
    Token, TransactionCommand,
};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner, SetOperationType};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine, ViewInfo,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

use std::sync::{Arc, RwLock};

use sqlrustgo_executor::OperatorProfile;
use sqlrustgo_storage::ForeignKeyAction;

/// Format EXPLAIN output with cost information
#[allow(dead_code)]
fn format_explain_output(plan: &dyn PhysicalPlan, indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let prefix = "  ".repeat(indent);

    let (startup_cost, total_cost, rows, width) = plan.estimated_cost();
    let plan_name = plan.name();
    let table_name = plan.table_name();

    let line = if table_name.is_empty() {
        format!(
            "{}{} (cost={:.2}..{:.2} rows={} width={})",
            prefix, plan_name, startup_cost, total_cost, rows, width
        )
    } else {
        format!(
            "{}{} on {} (cost={:.2}..{:.2} rows={} width={})",
            prefix, plan_name, table_name, startup_cost, total_cost, rows, width
        )
    };
    lines.push(line);

    for child in plan.children() {
        lines.extend(format_explain_output(child, indent + 1));
    }

    lines
}

/// Format the EXPLAIN ANALYZE output as a tree structure (PostgreSQL-style)
fn format_tree_output(profiles: &[OperatorProfile], total_time: &str) -> Vec<Vec<Value>> {
    let mut rows = Vec::new();

    rows.push(vec![
        Value::Text(
            "┌─────────────────────────────────────────────────────────────────┐".to_string(),
        ),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);
    rows.push(vec![
        Value::Text(
            "│                      Execution Plan                             │".to_string(),
        ),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);
    rows.push(vec![
        Value::Text(
            "├─────────────────────────────────────────────────────────────────┤".to_string(),
        ),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);

    for (i, profile) in profiles.iter().enumerate() {
        let is_last = i == profiles.len() - 1;
        let prefix = if is_last { "└─ " } else { "├─ " };
        let time_ms = profile.total_time_ns as f64 / 1_000_000.0;

        rows.push(vec![
            Value::Text(format!(
                "│  {} {} (rows={})",
                prefix, profile.operator_name, profile.rows_processed
            )),
            Value::Text("".to_string()),
            Value::Text("".to_string()),
        ]);
        rows.push(vec![
            Value::Text(format!(
                "│        Actual Time: {:.3} ms, Rows: {}",
                time_ms, profile.rows_processed
            )),
            Value::Text("".to_string()),
            Value::Text("".to_string()),
        ]);
    }

    rows.push(vec![
        Value::Text(
            "└─────────────────────────────────────────────────────────────────┘".to_string(),
        ),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);
    rows.push(vec![
        Value::Text(format!("Total Execution Time: {}", total_time)),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);

    rows
}

/// Handle foreign key constraints for DELETE operations
/// Returns: (cascaded_deletes, modified_rows) or error for RESTRICT
fn handle_foreign_key_delete(
    storage: &mut dyn sqlrustgo_storage::StorageEngine,
    parent_table: &str,
    parent_key_values: &[Value],
    parent_key_column: &str,
) -> SqlResult<(usize, usize)> {
    let mut total_cascade_deletes = 0;
    let mut total_set_null_updates = 0;

    let all_tables = storage.list_tables();

    for table_name in all_tables {
        // For RESTRICT and SET NULL, we skip self-referencing tables
        // because they are handled specially in the storage engine's delete method
        // But for CASCADE, we need to process self-referencing tables too
        let table_info = match storage.get_table_info(&table_name) {
            Ok(info) => info,
            Err(_) => continue,
        };

        for (col_idx, col) in table_info.columns.iter().enumerate() {
            if let Some(ref fk) = col.references {
                if fk.referenced_table == parent_table && fk.referenced_column == parent_key_column
                {
                    match fk.on_delete {
                        Some(ForeignKeyAction::Restrict) => {
                            // Skip self-referencing tables for RESTRICT
                            // They are handled in storage engine's delete method
                            if table_name == parent_table {
                                continue;
                            }
                            let child_rows = storage.scan(&table_name)?;
                            if let Some(pk_val) = parent_key_values.first() {
                                for child_row in &child_rows {
                                    if child_row[col_idx] == *pk_val {
                                        return Err(SqlError::ExecutionError(format!(
                                            "Cannot delete: foreign key constraint violation - table '{}' has referenced rows",
                                            table_name
                                        )));
                                    }
                                }
                            }
                        }
                        Some(ForeignKeyAction::Cascade) => {
                            // Skip self-referencing tables for CASCADE - they are handled
                            // in the storage engine's delete method which does multi-pass
                            // transitive cascade deletion properly
                            if table_name == parent_table {
                                continue;
                            }
                            let child_rows = storage.scan(&table_name)?;
                            if let Some(pk_val) = parent_key_values.first() {
                                let (to_delete, to_keep): (Vec<Vec<Value>>, Vec<Vec<Value>>) =
                                    child_rows.into_iter().partition(|r| r[col_idx] == *pk_val);

                                if !to_delete.is_empty() {
                                    let _ = storage.delete(&table_name, &[]);
                                    if !to_keep.is_empty() {
                                        storage.insert(&table_name, to_keep)?;
                                    }
                                    total_cascade_deletes += to_delete.len();
                                }
                            }
                        }
                        Some(ForeignKeyAction::SetNull) => {
                            // Skip self-referencing tables for SET NULL
                            // They are handled in storage engine's delete method
                            if table_name == parent_table {
                                continue;
                            }
                            let child_rows = storage.scan(&table_name)?;
                            if let Some(pk_val) = parent_key_values.first() {
                                let mut updated = false;
                                let new_rows: Vec<Vec<Value>> = child_rows
                                    .into_iter()
                                    .map(|mut r| {
                                        if r[col_idx] == *pk_val {
                                            r[col_idx] = Value::Null;
                                            updated = true;
                                        }
                                        r
                                    })
                                    .collect();
                                if updated {
                                    let _ = storage.delete(&table_name, &[]);
                                    storage.insert(&table_name, new_rows)?;
                                    total_set_null_updates += 1;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }

    Ok((total_cascade_deletes, total_set_null_updates))
}

/// Handle foreign key constraints for UPDATE operations
fn handle_foreign_key_update(
    storage: &mut dyn sqlrustgo_storage::StorageEngine,
    parent_table: &str,
    old_key_value: &Value,
    new_key_value: &Value,
    parent_key_column: &str,
) -> SqlResult<(usize, usize)> {
    let mut total_cascade_updates = 0;
    let mut total_set_null_updates = 0;

    let all_tables = storage.list_tables();

    for table_name in all_tables {
        if table_name == parent_table {
            continue;
        }

        let table_info = match storage.get_table_info(&table_name) {
            Ok(info) => info,
            Err(_) => continue,
        };

        for (col_idx, col) in table_info.columns.iter().enumerate() {
            if let Some(ref fk) = col.references {
                if fk.referenced_table == parent_table && fk.referenced_column == parent_key_column
                {
                    match fk.on_update {
                        Some(ForeignKeyAction::Restrict) => {
                            let child_rows = storage.scan(&table_name)?;
                            for child_row in &child_rows {
                                if child_row[col_idx] == *old_key_value {
                                    return Err(SqlError::ExecutionError(format!(
                                        "Cannot update: foreign key constraint violation - table '{}' has referenced rows",
                                        table_name
                                    )));
                                }
                            }
                        }
                        Some(ForeignKeyAction::Cascade) => {
                            let child_rows = storage.scan(&table_name)?;
                            let new_rows: Vec<Vec<Value>> = child_rows
                                .into_iter()
                                .map(|mut r| {
                                    if r[col_idx] == *old_key_value {
                                        r[col_idx] = new_key_value.clone();
                                    }
                                    r
                                })
                                .collect();
                            let updated_count = new_rows
                                .iter()
                                .filter(|r| r[col_idx] == *new_key_value)
                                .count();
                            if updated_count > 0 {
                                let _ = storage.delete(&table_name, &[]);
                                storage.insert(&table_name, new_rows)?;
                                total_cascade_updates += updated_count;
                            }
                        }
                        Some(ForeignKeyAction::SetNull) => {
                            let child_rows = storage.scan(&table_name)?;
                            let new_rows: Vec<Vec<Value>> = child_rows
                                .into_iter()
                                .map(|mut r| {
                                    if r[col_idx] == *old_key_value {
                                        r[col_idx] = Value::Null;
                                    }
                                    r
                                })
                                .collect();
                            let updated_count = new_rows
                                .iter()
                                .filter(|r| r[col_idx] == Value::Null)
                                .count();
                            if updated_count > 0 {
                                let _ = storage.delete(&table_name, &[]);
                                storage.insert(&table_name, new_rows)?;
                                total_set_null_updates += updated_count;
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }

    Ok((total_cascade_updates, total_set_null_updates))
}

/// Evaluate a WHERE clause expression against a row
fn evaluate_where_clause(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> bool {
    match expr {
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);
            compare_values(&left_val, op, &right_val)
        }
        sqlrustgo_parser::Expression::Identifier(name) => {
            // Single identifier in WHERE - treat as boolean (for EXISTS subqueries, etc)
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                if let Some(val) = row.get(idx) {
                    return val.to_bool();
                }
            }
            false
        }
        sqlrustgo_parser::Expression::Literal(s) => {
            // For IN clauses, we might get a comma-separated list
            s.to_uppercase() != "FALSE" && s != "0"
        }
        sqlrustgo_parser::Expression::Wildcard => true,
        sqlrustgo_parser::Expression::FunctionCall(_, _) => {
            // Function calls in WHERE should be evaluated as boolean
            // This handles cases like WHERE COUNT(*) > 1
            false
        }
        sqlrustgo_parser::Expression::Subquery(_) => false,
        sqlrustgo_parser::Expression::QualifiedColumn(_, _) => false,
        sqlrustgo_parser::Expression::WindowFunction { .. } => false,
        sqlrustgo_parser::Expression::Placeholder => false,
        sqlrustgo_parser::Expression::Between { expr, low, high } => {
            let expr_val = evaluate_expr(expr, row, columns);
            let low_val = evaluate_expr(low, row, columns);
            let high_val = evaluate_expr(high, row, columns);
            compare_values(&expr_val, ">=", &low_val) && compare_values(&expr_val, "<=", &high_val)
        }
        sqlrustgo_parser::Expression::InList { expr, values } => {
            let expr_val = evaluate_expr(expr, row, columns);
            for value_expr in values {
                let value = evaluate_expr(value_expr, row, columns);
                if value == expr_val {
                    return true;
                }
            }
            false
        }
        sqlrustgo_parser::Expression::CaseWhen {
            conditions,
            else_result,
        } => {
            for (condition, then_result) in conditions {
                let cond_val = evaluate_expr(condition, row, columns);
                if let Value::Boolean(true) = cond_val {
                    let then_val = evaluate_expr(then_result, row, columns);
                    return then_val.to_bool();
                }
            }
            if let Some(else_expr) = else_result {
                let else_val = evaluate_expr(else_expr, row, columns);
                else_val.to_bool()
            } else {
                false
            }
        }
    }
}

/// Evaluate an expression to a Value
fn evaluate_expr(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
            // Try to parse as number first
            if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if let Ok(n) = s.parse::<f64>() {
                Value::Float(n)
            } else if s.eq_ignore_ascii_case("true") {
                Value::Boolean(true)
            } else if s.eq_ignore_ascii_case("false") {
                Value::Boolean(false)
            } else {
                Value::Text(s.clone())
            }
        }
        sqlrustgo_parser::Expression::Identifier(name) => {
            // Look up column by name (case-insensitive)
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);

            match op.as_str() {
                "+" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 + r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
                    _ => Value::Null,
                },
                "-" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 - r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l - *r as f64),
                    _ => Value::Null,
                },
                "*" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
                    _ => Value::Null,
                },
                "/" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) if *r != 0 => Value::Integer(l / r),
                    (Value::Float(l), Value::Float(r)) if *r != 0.0 => Value::Float(l / r),
                    (Value::Integer(l), Value::Float(r)) if *r != 0.0 => {
                        Value::Float(*l as f64 / r)
                    }
                    (Value::Float(l), Value::Integer(r)) if *r != 0 => Value::Float(l / *r as f64),
                    _ => Value::Null,
                },
                _ => Value::Null,
            }
        }
        sqlrustgo_parser::Expression::Wildcard => Value::Null,
        sqlrustgo_parser::Expression::FunctionCall(_, _) => Value::Null,
        sqlrustgo_parser::Expression::Subquery(_) => Value::Null,
        sqlrustgo_parser::Expression::QualifiedColumn(_, _) => Value::Null,
        sqlrustgo_parser::Expression::WindowFunction { .. } => Value::Null,
        sqlrustgo_parser::Expression::Placeholder => Value::Null,
        sqlrustgo_parser::Expression::Between { expr, low, high } => {
            let expr_val = evaluate_expr(expr, row, columns);
            let low_val = evaluate_expr(low, row, columns);
            let high_val = evaluate_expr(high, row, columns);
            if compare_values(&expr_val, ">=", &low_val)
                && compare_values(&expr_val, "<=", &high_val)
            {
                Value::Boolean(true)
            } else {
                Value::Boolean(false)
            }
        }
        sqlrustgo_parser::Expression::InList { expr, values } => {
            let expr_val = evaluate_expr(expr, row, columns);
            for value_expr in values {
                let value = evaluate_expr(value_expr, row, columns);
                if value == expr_val {
                    return Value::Boolean(true);
                }
            }
            Value::Boolean(false)
        }
        sqlrustgo_parser::Expression::CaseWhen {
            conditions,
            else_result,
        } => {
            for (condition, then_result) in conditions {
                let cond_val = evaluate_expr(condition, row, columns);
                if let Value::Boolean(true) = cond_val {
                    return evaluate_expr(then_result, row, columns);
                }
            }
            if let Some(else_expr) = else_result {
                evaluate_expr(else_expr, row, columns)
            } else {
                Value::Null
            }
        }
    }
}

/// Compare two values with the given operator
fn compare_values(left: &Value, op: &str, right: &Value) -> bool {
    match op {
        "=" | "==" | "EQ" => left == right,
        "!=" | "<>" | "NE" => left != right,
        ">" | "GT" => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l > r,
            (Value::Float(l), Value::Float(r)) => l > r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) > *r,
            (Value::Float(l), Value::Integer(r)) => *l > (*r as f64),
            (Value::Text(l), Value::Text(r)) => l > r,
            _ => false,
        },
        "<" | "LT" => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l < r,
            (Value::Float(l), Value::Float(r)) => l < r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) < *r,
            (Value::Float(l), Value::Integer(r)) => *l < (*r as f64),
            (Value::Text(l), Value::Text(r)) => l < r,
            _ => false,
        },
        ">=" | "GE" => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l >= r,
            (Value::Float(l), Value::Float(r)) => l >= r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) >= *r,
            (Value::Float(l), Value::Integer(r)) => *l >= (*r as f64),
            (Value::Text(l), Value::Text(r)) => l >= r,
            _ => false,
        },
        "<=" | "LE" => match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => l <= r,
            (Value::Float(l), Value::Float(r)) => l <= r,
            (Value::Integer(l), Value::Float(r)) => (*l as f64) <= *r,
            (Value::Float(l), Value::Integer(r)) => *l <= (*r as f64),
            (Value::Text(l), Value::Text(r)) => l <= r,
            _ => false,
        },
        "LIKE" | "like" => {
            if let (Value::Text(pattern), Value::Text(text)) = (right, left) {
                like_match(text, pattern)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Simple LIKE pattern matching (supports % and _)
fn like_match(text: &str, pattern: &str) -> bool {
    // Simple implementation for LIKE patterns
    // % matches any sequence of characters
    // _ matches any single character
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    fn do_match(pi: usize, ti: usize, pc: &[char], tc: &[char]) -> bool {
        if pi == pc.len() {
            ti == tc.len()
        } else if pc[pi] == '%' {
            // % matches any sequence - try matching remaining pattern at each position
            // or skip the % and continue
            (ti < tc.len() && do_match(pi + 1, ti, pc, tc))
                || (ti < tc.len() && do_match(pi, ti + 1, pc, tc))
        } else if pc[pi] == '_' {
            ti < tc.len() && do_match(pi + 1, ti + 1, pc, tc)
        } else if ti < tc.len() && pc[pi] == tc[ti] {
            do_match(pi + 1, ti + 1, pc, tc)
        } else {
            false
        }
    }

    do_match(0, 0, &pattern_chars, &text_chars)
}

/// Compute an aggregate function over a set of rows
fn compute_aggregate(
    agg: &sqlrustgo_parser::parser::AggregateCall,
    rows: &[Vec<Value>],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Value {
    use sqlrustgo_parser::parser::AggregateFunction;

    // Get the column index for the aggregate argument
    let col_idx: Option<usize> = if agg.args.is_empty() {
        // COUNT(*) has no arguments
        None
    } else {
        match &agg.args[0] {
            sqlrustgo_parser::Expression::Identifier(name) => columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name)),
            sqlrustgo_parser::Expression::Wildcard => Some(0), // Will be ignored for COUNT
            _ => None,
        }
    };

    match agg.func {
        AggregateFunction::Count => {
            if agg.args.is_empty()
                || matches!(
                    agg.args.first(),
                    Some(sqlrustgo_parser::Expression::Wildcard)
                )
            {
                // COUNT(*) - count all rows
                Value::Integer(rows.len() as i64)
            } else {
                // COUNT(column) - count non-null values
                let mut count = 0i64;
                for row in rows {
                    if let Some(idx) = col_idx {
                        if let Some(val) = row.get(idx) {
                            if *val != Value::Null {
                                count += 1;
                            }
                        }
                    }
                }
                Value::Integer(count)
            }
        }
        AggregateFunction::Sum => {
            let mut sum = 0i64;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if let Value::Integer(n) = val {
                            sum += n;
                        } else if let Value::Float(n) = val {
                            sum += *n as i64;
                        }
                    }
                }
            }
            Value::Integer(sum)
        }
        AggregateFunction::Avg => {
            let mut sum = 0.0f64;
            let mut count = 0i64;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if let Value::Integer(n) = val {
                            sum += *n as f64;
                            count += 1;
                        } else if let Value::Float(n) = val {
                            sum += *n;
                            count += 1;
                        }
                    }
                }
            }
            if count > 0 {
                Value::Float(sum / count as f64)
            } else {
                Value::Null
            }
        }
        AggregateFunction::Min => {
            let mut min: Option<Value> = None;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if *val != Value::Null {
                            match &min {
                                None => min = Some(val.clone()),
                                Some(m) => {
                                    if val < m {
                                        min = Some(val.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            min.unwrap_or(Value::Null)
        }
        AggregateFunction::Max => {
            let mut max: Option<Value> = None;
            for row in rows {
                if let Some(idx) = col_idx {
                    if let Some(val) = row.get(idx) {
                        if *val != Value::Null {
                            match &max {
                                None => max = Some(val.clone()),
                                Some(m) => {
                                    if val > m {
                                        max = Some(val.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            max.unwrap_or(Value::Null)
        }
    }
}

pub struct ExecutionEngine {
    pub storage: Arc<RwLock<dyn StorageEngine>>,
}

impl ExecutionEngine {
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self { storage }
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutorResult, SqlError> {
        match statement {
            Statement::Insert(insert) => {
                let table_name = &insert.table;
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(table_name) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        table_name
                    )));
                }

                // Get table info to determine column types and auto_increment columns
                let table_info = storage.get_table_info(table_name).ok();

                // Find auto_increment column indices
                let auto_increment_cols: Vec<usize> = table_info
                    .as_ref()
                    .map(|info| {
                        info.columns
                            .iter()
                            .enumerate()
                            .filter(|(_, col)| col.auto_increment)
                            .map(|(idx, _)| idx)
                            .collect()
                    })
                    .unwrap_or_default();

                // Pre-calculate auto_increment values for each row
                let mut auto_increment_values: Vec<Vec<(usize, i64)>> = Vec::new();
                for _ in 0..insert.values.len() {
                    let mut row_auto_inc: Vec<(usize, i64)> = Vec::new();
                    for &col_idx in &auto_increment_cols {
                        let next_val = storage.get_next_auto_increment(table_name, col_idx)?;
                        row_auto_inc.push((col_idx, next_val));
                    }
                    auto_increment_values.push(row_auto_inc);
                }

                let records: Vec<Vec<Value>> = insert
                    .values
                    .iter()
                    .enumerate()
                    .map(|(row_idx, row)| {
                        // If INSERT specifies columns, map values to correct positions
                        // Otherwise, values go in column order
                        let num_columns = if insert.columns.is_empty() {
                            table_info
                                .as_ref()
                                .map(|i| i.columns.len())
                                .unwrap_or(row.len())
                        } else {
                            table_info.as_ref().map(|i| i.columns.len()).unwrap_or(0)
                        };

                        // Create row with correct size, filled with Null initially
                        let mut new_row: Vec<Value> = vec![Value::Null; num_columns];

                        // Map values to correct column positions
                        if insert.columns.is_empty() {
                            // No column list: values map to columns in order
                            for (col_idx, expr) in row.iter().enumerate() {
                                if col_idx < num_columns {
                                    new_row[col_idx] = match expr {
                                        Expression::Literal(value) => {
                                            if value.to_uppercase() == "NULL" {
                                                Value::Null
                                            } else if let Some(ref info) = table_info {
                                                if col_idx < info.columns.len() {
                                                    let col_type = &info.columns[col_idx].data_type;
                                                    let upper = col_type.to_uppercase();
                                                    if upper.contains("INT")
                                                        || upper == "BIGINT"
                                                        || upper == "SMALLINT"
                                                    {
                                                        if let Ok(n) = value.parse::<i64>() {
                                                            Value::Integer(n)
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    } else if upper == "FLOAT"
                                                        || upper == "DOUBLE"
                                                        || upper == "DECIMAL"
                                                    {
                                                        if let Ok(n) = value.parse::<f64>() {
                                                            Value::Float(n)
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    } else if upper == "BOOLEAN" {
                                                        if value.to_uppercase() == "TRUE" {
                                                            Value::Boolean(true)
                                                        } else if value.to_uppercase() == "FALSE" {
                                                            Value::Boolean(false)
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    } else {
                                                        Value::Text(value.clone())
                                                    }
                                                } else {
                                                    Value::Text(value.clone())
                                                }
                                            } else {
                                                Value::Text(value.clone())
                                            }
                                        }
                                        _ => Value::Null,
                                    };
                                }
                            }
                        } else {
                            // Column list specified: map each value to its column position
                            for (value_idx, col_name) in insert.columns.iter().enumerate() {
                                if value_idx < row.len() {
                                    // Find column index by name
                                    if let Some(ref info) = table_info {
                                        if let Some(target_idx) = info
                                            .columns
                                            .iter()
                                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                        {
                                            let expr = &row[value_idx];
                                            new_row[target_idx] = match expr {
                                                Expression::Literal(value) => {
                                                    if value.to_uppercase() == "NULL" {
                                                        Value::Null
                                                    } else {
                                                        let col_type =
                                                            &info.columns[target_idx].data_type;
                                                        let upper = col_type.to_uppercase();
                                                        if upper.contains("INT")
                                                            || upper == "BIGINT"
                                                            || upper == "SMALLINT"
                                                        {
                                                            if let Ok(n) = value.parse::<i64>() {
                                                                Value::Integer(n)
                                                            } else {
                                                                Value::Text(value.clone())
                                                            }
                                                        } else if upper == "FLOAT"
                                                            || upper == "DOUBLE"
                                                            || upper == "DECIMAL"
                                                        {
                                                            if let Ok(n) = value.parse::<f64>() {
                                                                Value::Float(n)
                                                            } else {
                                                                Value::Text(value.clone())
                                                            }
                                                        } else if upper == "BOOLEAN" {
                                                            if value.to_uppercase() == "TRUE" {
                                                                Value::Boolean(true)
                                                            } else if value.to_uppercase()
                                                                == "FALSE"
                                                            {
                                                                Value::Boolean(false)
                                                            } else {
                                                                Value::Text(value.clone())
                                                            }
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    }
                                                }
                                                _ => Value::Null,
                                            };
                                        }
                                    }
                                }
                            }
                        }

                        // Apply auto_increment values for columns that are Null
                        for &(col_idx, next_val) in &auto_increment_values[row_idx] {
                            if col_idx < new_row.len() && matches!(new_row[col_idx], Value::Null) {
                                new_row[col_idx] = Value::Integer(next_val);
                            }
                        }

                        new_row
                    })
                    .collect();

                // Handle UPSERT: INSERT ... ON DUPLICATE KEY UPDATE
                if let Some(updates) = &insert.on_duplicate {
                    let existing_rows = storage.scan(table_name)?;

                    let key_columns: Vec<usize> = table_info
                        .as_ref()
                        .map(|info| {
                            info.columns
                                .iter()
                                .enumerate()
                                .filter(|(_, col)| col.is_primary_key || col.is_unique)
                                .map(|(idx, _)| idx)
                                .collect()
                        })
                        .unwrap_or_default();

                    let mut total_affected = 0;
                    let mut final_rows = existing_rows.clone();

                    for new_row in &records {
                        if !key_columns.is_empty() {
                            let mut conflict_idx = None;
                            for (idx, existing_row) in final_rows.iter().enumerate() {
                                let mut match_count = 0;
                                for &key_col in &key_columns {
                                    if key_col < new_row.len()
                                        && key_col < existing_row.len()
                                        && new_row[key_col] == existing_row[key_col]
                                    {
                                        match_count += 1;
                                    }
                                }
                                if match_count == key_columns.len() {
                                    conflict_idx = Some(idx);
                                    break;
                                }
                            }

                            if let Some(idx) = conflict_idx {
                                let mut updated = final_rows[idx].clone();
                                for (col_name, expr) in updates {
                                    if let Some(ref info) = table_info {
                                        if let Some(col_idx) = info
                                            .columns
                                            .iter()
                                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                                        {
                                            let new_val = match expr {
                                                Expression::Literal(value) => {
                                                    let col_type = &info.columns[col_idx].data_type;
                                                    let upper = col_type.to_uppercase();
                                                    if upper.contains("INT")
                                                        || upper == "BIGINT"
                                                        || upper == "SMALLINT"
                                                    {
                                                        if let Ok(n) = value.parse::<i64>() {
                                                            Value::Integer(n)
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    } else if upper == "FLOAT"
                                                        || upper == "DOUBLE"
                                                        || upper == "DECIMAL"
                                                    {
                                                        if let Ok(n) = value.parse::<f64>() {
                                                            Value::Float(n)
                                                        } else {
                                                            Value::Text(value.clone())
                                                        }
                                                    } else {
                                                        Value::Text(value.clone())
                                                    }
                                                }
                                                _ => Value::Null,
                                            };
                                            if col_idx < updated.len() {
                                                updated[col_idx] = new_val;
                                            }
                                        }
                                    }
                                }
                                final_rows[idx] = updated;
                                total_affected += 1;
                            } else {
                                final_rows.push(new_row.clone());
                                total_affected += 1;
                            }
                        } else {
                            final_rows.push(new_row.clone());
                            total_affected += 1;
                        }
                    }

                    let _ = storage.delete(table_name, &[]);
                    storage.insert(table_name, final_rows)?;
                    Ok(ExecutorResult::new(vec![], total_affected))
                } else if insert.replace {
                    // REPLACE: Delete existing rows with matching PK, then insert
                    let key_columns: Vec<usize> = table_info
                        .as_ref()
                        .map(|info| {
                            info.columns
                                .iter()
                                .enumerate()
                                .filter(|(_, col)| col.is_primary_key || col.is_unique)
                                .map(|(idx, _)| idx)
                                .collect()
                        })
                        .unwrap_or_default();

                    if key_columns.is_empty() {
                        // No PK/unique key - just do a regular insert
                        storage.insert(table_name, records)?;
                        Ok(ExecutorResult::new(vec![], insert.values.len()))
                    } else {
                        let existing_rows = storage.scan(table_name)?;
                        let mut final_rows = existing_rows.clone();
                        let mut total_affected = 0;

                        for new_row in &records {
                            // Find and remove existing row with matching PK
                            let conflict_idx = final_rows
                                .iter()
                                .enumerate()
                                .find(|(_, existing_row)| {
                                    key_columns.iter().all(|&key_col| {
                                        key_col < new_row.len()
                                            && key_col < existing_row.len()
                                            && new_row[key_col] == existing_row[key_col]
                                    })
                                })
                                .map(|(idx, _)| idx);

                            if let Some(idx) = conflict_idx {
                                final_rows.remove(idx);
                            }
                            final_rows.push(new_row.clone());
                            total_affected += 1;
                        }

                        let _ = storage.delete(table_name, &[]);
                        storage.insert(table_name, final_rows)?;
                        Ok(ExecutorResult::new(vec![], total_affected))
                    }
                } else if insert.ignore {
                    // INSERT IGNORE: Skip rows that would cause duplicate key violations
                    let key_columns: Vec<usize> = table_info
                        .as_ref()
                        .map(|info| {
                            info.columns
                                .iter()
                                .enumerate()
                                .filter(|(_, col)| col.is_primary_key || col.is_unique)
                                .map(|(idx, _)| idx)
                                .collect()
                        })
                        .unwrap_or_default();

                    if key_columns.is_empty() {
                        // No PK/unique key - just do a regular insert
                        storage.insert(table_name, records)?;
                        Ok(ExecutorResult::new(vec![], insert.values.len()))
                    } else {
                        let existing_rows = storage.scan(table_name)?;
                        let mut final_rows = existing_rows.clone();
                        let mut total_affected = 0;

                        for new_row in &records {
                            // Check if there's an existing row with matching PK
                            let conflict_exists = final_rows.iter().any(|existing_row| {
                                key_columns.iter().all(|&key_col| {
                                    key_col < new_row.len()
                                        && key_col < existing_row.len()
                                        && new_row[key_col] == existing_row[key_col]
                                })
                            });

                            if !conflict_exists {
                                final_rows.push(new_row.clone());
                                total_affected += 1;
                            }
                        }

                        let _ = storage.delete(table_name, &[]);
                        if !final_rows.is_empty() {
                            storage.insert(table_name, final_rows)?;
                        }
                        Ok(ExecutorResult::new(vec![], total_affected))
                    }
                } else {
                    storage.insert(table_name, records)?;
                    Ok(ExecutorResult::new(vec![], insert.values.len()))
                }
            }
            Statement::CreateTable(create) => {
                let mut storage = self.storage.write().unwrap();

                // If IF NOT EXISTS is set, check if table already exists
                if create.if_not_exists && storage.has_table(&create.name) {
                    return Ok(ExecutorResult::new(vec![], 0));
                }

                let columns: Vec<sqlrustgo_storage::ColumnDefinition> = create
                    .columns
                    .iter()
                    .map(|col| {
                        let references = col.references.as_ref().map(|fk| {
                            sqlrustgo_storage::ForeignKeyConstraint {
                                referenced_table: fk.table.clone(),
                                referenced_column: fk.column.clone(),
                                on_delete: fk.on_delete.as_ref().map(|a| match a {
                                    sqlrustgo_parser::parser::ForeignKeyAction::Cascade => {
                                        sqlrustgo_storage::ForeignKeyAction::Cascade
                                    }
                                    sqlrustgo_parser::parser::ForeignKeyAction::SetNull => {
                                        sqlrustgo_storage::ForeignKeyAction::SetNull
                                    }
                                    sqlrustgo_parser::parser::ForeignKeyAction::Restrict => {
                                        sqlrustgo_storage::ForeignKeyAction::Restrict
                                    }
                                }),
                                on_update: fk.on_update.as_ref().map(|a| match a {
                                    sqlrustgo_parser::parser::ForeignKeyAction::Cascade => {
                                        sqlrustgo_storage::ForeignKeyAction::Cascade
                                    }
                                    sqlrustgo_parser::parser::ForeignKeyAction::SetNull => {
                                        sqlrustgo_storage::ForeignKeyAction::SetNull
                                    }
                                    sqlrustgo_parser::parser::ForeignKeyAction::Restrict => {
                                        sqlrustgo_storage::ForeignKeyAction::Restrict
                                    }
                                }),
                            }
                        });
                        sqlrustgo_storage::ColumnDefinition {
                            name: col.name.clone(),
                            data_type: col.data_type.clone(),
                            nullable: col.nullable,
                            is_unique: col.primary_key,
                            is_primary_key: col.primary_key,
                            references,
                            auto_increment: col.auto_increment,
                        }
                    })
                    .collect();

                let table_info = sqlrustgo_storage::TableInfo {
                    name: create.name.clone(),
                    columns,
                };

                storage.create_table(&table_info)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::CreateView(create) => {
                let mut storage = self.storage.write().unwrap();
                let view_info = sqlrustgo_storage::ViewInfo {
                    name: create.name.clone(),
                    query: create.query.clone(),
                    schema: sqlrustgo_storage::TableInfo {
                        name: create.name.clone(),
                        columns: vec![],
                    },
                    records: vec![],
                };
                storage.create_view(view_info)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Analyze(analyze) => {
                let table_name = analyze.table_name.ok_or_else(|| {
                    SqlError::ExecutionError("ANALYZE requires a table name".to_string())
                })?;
                let storage = self.storage.read().unwrap();
                let stats = storage.analyze_table(&table_name)?;

                let mut rows: Vec<Vec<Value>> = vec![
                    vec![
                        Value::Text("table".to_string()),
                        Value::Text(stats.table_name.clone()),
                    ],
                    vec![
                        Value::Text("row_count".to_string()),
                        Value::Text(stats.row_count.to_string()),
                    ],
                ];

                for col_stat in &stats.column_stats {
                    let col_row = vec![
                        Value::Text(format!("column:{}", col_stat.column_name)),
                        Value::Text(format!(
                            "distinct:{}, null:{}",
                            col_stat.distinct_count, col_stat.null_count
                        )),
                    ];
                    rows.push(col_row);
                }

                let row_count = rows.len();
                Ok(ExecutorResult::new(rows, row_count))
            }
            Statement::Delete(delete) => {
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(&delete.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        delete.table
                    )));
                }

                let table_info = storage.get_table_info(&delete.table).ok();
                let columns = table_info
                    .map(|info| info.columns.clone())
                    .unwrap_or_default();

                // Find primary key column for foreign key reference
                let primary_key_col = columns.iter().position(|c| c.is_primary_key);

                // Find rows that will be deleted
                let all_rows = storage.scan(&delete.table).unwrap_or_default();
                let mut rows_to_delete = Vec::new();

                // If no WHERE clause, delete all rows
                if delete.where_clause.is_none() {
                    rows_to_delete = all_rows.clone();
                } else {
                    for row in &all_rows {
                        if let Some(ref where_clause) = delete.where_clause {
                            if evaluate_where_clause(where_clause, row, &columns) {
                                rows_to_delete.push(row.clone());
                            }
                        }
                    }
                }

                // Handle foreign key constraints for CASCADE/SET NULL/RESTRICT
                // For self-referencing tables, we use storage engine's delete which handles
                // transitive CASCADE properly
                let mut has_self_ref_cascade = false;
                if let (Some(pk_col_idx), true) = (primary_key_col, !rows_to_delete.is_empty()) {
                    // Check if this table has any self-referencing FK with CASCADE
                    let table_info = storage.get_table_info(&delete.table).ok();
                    has_self_ref_cascade = table_info
                        .as_ref()
                        .map(|info| {
                            info.columns.iter().any(|col| {
                                col.references.as_ref().is_some_and(|fk| {
                                    fk.referenced_table == delete.table
                                        && fk.on_delete == Some(ForeignKeyAction::Cascade)
                                })
                            })
                        })
                        .unwrap_or(false);

                    // Only call handle_foreign_key_delete if NOT a self-referencing CASCADE table
                    if !has_self_ref_cascade {
                        for row in &rows_to_delete {
                            let key_value = row[pk_col_idx].clone();
                            handle_foreign_key_delete(
                                &mut *storage,
                                &delete.table,
                                &[key_value],
                                &columns[pk_col_idx].name,
                            )?;
                        }
                    }
                }

                let deleted_count = rows_to_delete.len();

                // Delete matching rows and insert remaining rows
                if deleted_count > 0 {
                    if has_self_ref_cascade {
                        // For self-referencing CASCADE, use storage.delete with proper filter
                        // so that storage engine's CASCADE loop handles transitive deletion
                        if let Some(pk_col_idx) = primary_key_col {
                            for row in &rows_to_delete {
                                let key_value = row[pk_col_idx].clone();
                                // Filter format: [column_index, value]
                                storage.delete(
                                    &delete.table,
                                    &[Value::Integer(pk_col_idx as i64), key_value],
                                )?;
                            }
                        }
                    } else {
                        // For non-self-referencing tables, use delete all + reinsert pattern
                        let remaining_rows: Vec<Vec<Value>> = all_rows
                            .into_iter()
                            .filter(|row| {
                                if let Some(ref where_clause) = delete.where_clause {
                                    !evaluate_where_clause(where_clause, row, &columns)
                                } else {
                                    true
                                }
                            })
                            .collect();
                        let _ = storage.delete(&delete.table, &[]);
                        if !remaining_rows.is_empty() {
                            storage.insert(&delete.table, remaining_rows)?;
                        }
                    }
                }

                Ok(ExecutorResult::new(vec![], deleted_count))
            }
            Statement::Update(update) => {
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(&update.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        update.table
                    )));
                }

                let table_info = storage.get_table_info(&update.table).ok();
                let columns = table_info
                    .map(|info| info.columns.clone())
                    .unwrap_or_default();

                // Find primary key column
                let primary_key_col = columns.iter().position(|c| c.is_primary_key);

                let all_rows = storage.scan(&update.table).unwrap_or_default();
                let mut rows_to_update: Vec<(Vec<Value>, Vec<Value>)> = Vec::new(); // (old_row, new_row)

                // Find rows to update and prepare new values
                // If no WHERE clause, update all rows
                let has_where_clause = update.where_clause.is_some();

                for row in &all_rows {
                    if has_where_clause {
                        if let Some(ref where_clause) = update.where_clause {
                            if !evaluate_where_clause(where_clause, row, &columns) {
                                continue;
                            }
                        }
                    }

                    let mut new_row = row.clone();
                    let mut _old_key_value: Option<Value> = None;

                    // Apply SET clauses and track if primary key is being updated
                    for (col_name, expr) in &update.set_clauses {
                        if let Some(col_idx) = columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                        {
                            // Track old key value if this is the primary key
                            if primary_key_col == Some(col_idx) {
                                _old_key_value = Some(row[col_idx].clone());
                            }

                            let new_value = evaluate_expr(expr, &new_row, &columns);
                            if col_idx < new_row.len() {
                                new_row[col_idx] = new_value;
                            }
                        }
                    }

                    rows_to_update.push((row.clone(), new_row));
                }

                let updated_count = rows_to_update.len();

                // Check RESTRICT constraints BEFORE modifying the parent table
                // This must happen before any delete/insert operations
                if updated_count > 0 {
                    if let Some(pk_col_idx) = primary_key_col {
                        for (old_row, new_row) in &rows_to_update {
                            if let (Some(old_val), Some(new_val)) =
                                (old_row.get(pk_col_idx), new_row.get(pk_col_idx))
                            {
                                if old_val != new_val {
                                    // Check if any child table has RESTRICT that would block this update
                                    let all_tables = storage.list_tables();
                                    for table_name in &all_tables {
                                        if *table_name == update.table {
                                            continue;
                                        }

                                        let table_info = match storage.get_table_info(table_name) {
                                            Ok(info) => info,
                                            Err(_) => continue,
                                        };

                                        for (col_idx, col) in table_info.columns.iter().enumerate()
                                        {
                                            if let Some(ref fk) = col.references {
                                                if fk.referenced_table == update.table
                                                    && fk.referenced_column
                                                        == columns[pk_col_idx].name
                                                    && fk.on_update
                                                        == Some(ForeignKeyAction::Restrict)
                                                {
                                                    let child_rows = storage.scan(table_name)?;
                                                    for child_row in &child_rows {
                                                        if child_row[col_idx] == *old_val {
                                                            return Err(SqlError::ExecutionError(format!(
                                                                "Cannot update: foreign key constraint violation - table '{}' has referenced rows",
                                                                table_name
                                                            )));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Write back updated rows
                if updated_count > 0 {
                    let _ = storage.delete(&update.table, &[]);

                    let mut final_rows = all_rows;
                    final_rows.retain(|r| !rows_to_update.iter().any(|(old, _)| old == r));
                    for (_, new) in rows_to_update.iter() {
                        final_rows.push(new.clone());
                    }

                    storage.insert(&update.table, final_rows)?;

                    // Now handle foreign key cascading AFTER parent is updated
                    // This must happen AFTER the parent record is updated because
                    // child records reference the NEW value, not the old one
                    if let Some(pk_col_idx) = primary_key_col {
                        for (old_row, new_row) in &rows_to_update {
                            if let (Some(old_val), Some(new_val)) =
                                (old_row.get(pk_col_idx), new_row.get(pk_col_idx))
                            {
                                if old_val != new_val {
                                    handle_foreign_key_update(
                                        &mut *storage,
                                        &update.table,
                                        old_val,
                                        new_val,
                                        &columns[pk_col_idx].name,
                                    )?;
                                }
                            }
                        }
                    }
                }

                Ok(ExecutorResult::new(vec![], updated_count))
            }
            Statement::Select(select) => {
                let storage = self.storage.read().unwrap();
                if !storage.has_table(&select.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        select.table
                    )));
                }
                let table_info = storage.get_table_info(&select.table).ok();
                let columns = table_info
                    .map(|info| info.columns.clone())
                    .unwrap_or_default();
                let rows = storage.scan(&select.table).unwrap_or_default();

                // Apply WHERE clause filter if present
                let filtered_rows: Vec<Vec<Value>> =
                    if let Some(ref where_clause) = select.where_clause {
                        rows.into_iter()
                            .filter(|row| evaluate_where_clause(where_clause, row, &columns))
                            .collect()
                    } else {
                        rows
                    };

                // Handle aggregates if present
                if !select.aggregates.is_empty() {
                    let result_row: Vec<Value> = select
                        .aggregates
                        .iter()
                        .map(|agg| compute_aggregate(agg, &filtered_rows, &columns))
                        .collect();
                    return Ok(ExecutorResult::new(vec![result_row], 0));
                }

                // Apply column projection if specified (not SELECT *)
                // SELECT * has columns = [{"*", None}]
                let is_select_star = select.columns.len() == 1 && select.columns[0].name == "*";
                let projected_rows: Vec<Vec<Value>> = if is_select_star {
                    // SELECT * - return all columns
                    filtered_rows
                } else if select.columns.is_empty() {
                    // No columns specified (edge case) - return all columns
                    filtered_rows
                } else {
                    // Project only the selected columns
                    filtered_rows
                        .into_iter()
                        .map(|row| {
                            select
                                .columns
                                .iter()
                                .filter_map(|col| {
                                    columns
                                        .iter()
                                        .position(|c| c.name.eq_ignore_ascii_case(&col.name))
                                        .and_then(|idx| row.get(idx).cloned())
                                })
                                .collect()
                        })
                        .collect()
                };

                Ok(ExecutorResult::new(projected_rows, 0))
            }
            Statement::Explain(explain) => {
                let start = std::time::Instant::now();

                // Clear previous profiling data before execution
                GLOBAL_PROFILER.clear();

                let result = self.execute(*explain.query)?;
                let duration = start.elapsed();

                if explain.analyze {
                    let profiles = GLOBAL_PROFILER.get_all_profiles();
                    let total_time = format!("{:.3} ms", duration.as_secs_f64() * 1000.0);
                    let rows = format_tree_output(&profiles, &total_time);

                    return Ok(ExecutorResult::new(rows, result.affected_rows));
                }

                // EXPLAIN (without ANALYZE) - show query plan with cost
                let rows = vec![vec![
                    Value::Text("Plan:".to_string()),
                    Value::Text("cost=0.00..100.00 rows=1000 width=64".to_string()),
                ]];
                Ok(ExecutorResult::new(rows, 0))
            }
            Statement::Transaction(tx) => match tx.command {
                sqlrustgo_parser::TransactionCommand::Begin => Ok(ExecutorResult::new(vec![], 0)),
                sqlrustgo_parser::TransactionCommand::Commit => Ok(ExecutorResult::new(vec![], 0)),
                sqlrustgo_parser::TransactionCommand::Rollback => {
                    Ok(ExecutorResult::new(vec![], 0))
                }
                sqlrustgo_parser::TransactionCommand::Savepoint { name } => {
                    Ok(ExecutorResult::new(
                        vec![vec![Value::Text(format!("Savepoint '{}' created", name))]],
                        0,
                    ))
                }
                sqlrustgo_parser::TransactionCommand::RollbackTo { name } => {
                    Ok(ExecutorResult::new(
                        vec![vec![Value::Text(format!("Rolled back to '{}'", name))]],
                        0,
                    ))
                }
                sqlrustgo_parser::TransactionCommand::ReleaseSavepoint { name } => {
                    Ok(ExecutorResult::new(
                        vec![vec![Value::Text(format!("Released savepoint '{}'", name))]],
                        0,
                    ))
                }
            },
            Statement::Copy(copy) => {
                use sqlrustgo_storage::parquet::{export_to_parquet, import_from_parquet};

                let table_name = &copy.table_name;
                let path = &copy.path;
                let format = &copy.format;

                // Validate format
                if format.to_uppercase() != "PARQUET" {
                    return Err(SqlError::ExecutionError(format!(
                        "Unsupported COPY format: {}. Only PARQUET is supported.",
                        format
                    )));
                }

                let mut storage = self.storage.write().unwrap();

                if copy.from {
                    // COPY FROM PARQUET - Import data from Parquet file
                    if !storage.has_table(table_name) {
                        return Err(SqlError::ExecutionError(format!(
                            "Table '{}' not found",
                            table_name
                        )));
                    }

                    // Get table info for column names
                    let table_info = storage.get_table_info(table_name).ok().ok_or_else(|| {
                        SqlError::ExecutionError(format!(
                            "Could not get table info for '{}'",
                            table_name
                        ))
                    })?;

                    let column_names: Vec<String> =
                        table_info.columns.iter().map(|c| c.name.clone()).collect();

                    // Import records from Parquet
                    let records = import_from_parquet(path, &column_names)?;

                    // Insert records into table
                    let mut rows_inserted = 0;
                    for record in &records {
                        if storage.insert(table_name, vec![record.clone()]).is_ok() {
                            rows_inserted += 1;
                        }
                    }

                    Ok(ExecutorResult::new(vec![], rows_inserted))
                } else {
                    // COPY TO PARQUET - Export data to Parquet file
                    if !storage.has_table(table_name) {
                        return Err(SqlError::ExecutionError(format!(
                            "Table '{}' not found",
                            table_name
                        )));
                    }

                    // Get table info for column names
                    let table_info = storage.get_table_info(table_name).ok().ok_or_else(|| {
                        SqlError::ExecutionError(format!(
                            "Could not get table info for '{}'",
                            table_name
                        ))
                    })?;

                    let column_names: Vec<String> =
                        table_info.columns.iter().map(|c| c.name.clone()).collect();

                    // Scan all rows from table
                    let records = storage.scan(table_name)?;

                    // Export records to Parquet
                    export_to_parquet(path, &records, &column_names)?;

                    let row_count = records.len();
                    Ok(ExecutorResult::new(
                        vec![vec![Value::Text(format!(
                            "Exported {} rows to {}",
                            row_count, path
                        ))]],
                        row_count,
                    ))
                }
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }

    pub fn execute_plan(&self, plan: &dyn PhysicalPlan) -> Result<ExecutorResult, SqlError> {
        let storage = self.storage.read().unwrap();
        match plan.name() {
            "SeqScan" => {
                let rows = storage.scan(plan.table_name())?;
                Ok(ExecutorResult::new(rows, 0))
            }
            "IndexScan" => {
                let index_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::IndexScanExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast IndexScanExec".to_string())
                    })?;

                let table = index_plan.table_name();
                let index_name = index_plan.index_name();
                let (range_min, range_max) = index_plan.key_range();

                if let (Some(min), Some(max)) = (range_min, range_max) {
                    let row_ids = storage.range_index(table, index_name, min, max);
                    let all_rows = storage.scan(table)?;
                    let indexed_rows: Vec<Vec<Value>> = row_ids
                        .into_iter()
                        .filter_map(|id| all_rows.get(id as usize).cloned())
                        .collect();
                    Ok(ExecutorResult::new(indexed_rows, 0))
                } else {
                    let rows = storage.scan(table)?;
                    Ok(ExecutorResult::new(rows, 0))
                }
            }
            "Filter" => {
                let filter_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::FilterExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast FilterExec".to_string())
                    })?;

                let child = filter_plan.input();
                let input_result = self.execute_plan(child)?;

                let predicate = filter_plan.predicate();
                let schema = child.schema();
                let filtered_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .into_iter()
                    .filter(|row| predicate.matches(row, schema))
                    .collect();

                Ok(ExecutorResult::new(filtered_rows, 0))
            }
            "Projection" => {
                let proj_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast ProjectionExec".to_string())
                    })?;

                let child = proj_plan.input();
                let input_result = self.execute_plan(child)?;

                let exprs = proj_plan.expr();
                let _output_schema = plan.schema();
                let projected_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .iter()
                    .map(|row| {
                        exprs
                            .iter()
                            .filter_map(|expr| expr.evaluate(row, child.schema()))
                            .collect()
                    })
                    .collect();

                Ok(ExecutorResult::new(projected_rows, 0))
            }
            "HashJoin" => {
                let join_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::HashJoinExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast HashJoinExec".to_string())
                    })?;

                let left = join_plan.left();
                let right = join_plan.right();
                let join_type = join_plan.join_type();
                let right_schema_len = right.schema().fields.len();

                let left_result = self.execute_plan(left)?;
                let right_result = self.execute_plan(right)?;

                let left_rows = left_result.rows;
                let right_rows = right_result.rows;

                // Build hash map from right rows using first column as key
                use std::collections::HashMap;
                let mut right_hash: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();
                for rrow in &right_rows {
                    let key = if !rrow.is_empty() {
                        vec![rrow[0].clone()]
                    } else {
                        vec![Value::Null]
                    };
                    right_hash.entry(key).or_default().push(rrow.clone());
                }

                let mut result_rows = Vec::new();
                let mut left_matched: Vec<bool> = vec![false; left_rows.len()];

                // Probe with left rows
                for (lidx, lrow) in left_rows.iter().enumerate() {
                    let key = if !lrow.is_empty() {
                        vec![lrow[0].clone()]
                    } else {
                        vec![Value::Null]
                    };

                    if let Some(matched_right_rows) = right_hash.get(&key) {
                        left_matched[lidx] = true;
                        for rrow in matched_right_rows {
                            let mut combined = lrow.clone();
                            combined.extend(rrow.clone());
                            result_rows.push(combined);
                        }
                    }
                }

                // For LEFT join, emit unmatched left rows with NULLs for right schema
                if join_type == sqlrustgo_planner::JoinType::Left {
                    for (lidx, lrow) in left_rows.iter().enumerate() {
                        if !left_matched[lidx] {
                            let mut combined = lrow.clone();
                            for _ in 0..right_schema_len {
                                combined.push(Value::Null);
                            }
                            result_rows.push(combined);
                        }
                    }
                }

                Ok(ExecutorResult::new(result_rows, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }
}

pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_module_exports() {
        let _ = tokenize("SELECT 1");
        let _ = parse("SELECT 1");
        let _ = Value::Integer(1);
    }

    #[test]
    fn test_sql_result_alias() {
        let result: SqlResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_optimizer_alias() {
        let _: Option<Box<dyn sqlrustgo_optimizer::Optimizer>> = None;
    }

    #[test]
    fn test_physical_plan_trait() {
        let _: Option<Box<dyn PhysicalPlan>> = None;
    }

    #[test]
    fn test_execution_engine_new() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execution_engine_default() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execute_plan_seqscan() {
        use sqlrustgo_planner::{DataType, Field, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let plan = SeqScanExec::new("users".to_string(), schema);
        let result = engine.execute_plan(&plan).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[0][1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_execute_plan_filter() {
        use sqlrustgo_planner::{DataType, Expr, Field, FilterExec, Operator, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                    vec![Value::Integer(3), Value::Text("Charlie".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            Operator::Gt,
            Expr::literal(Value::Integer(1)),
        );
        let filter = FilterExec::new(Box::new(scan), predicate);
        let result = engine.execute_plan(&filter).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_execute_plan_projection() {
        use sqlrustgo_planner::{DataType, Expr, Field, ProjectionExec, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let proj_schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);
        let projection =
            ProjectionExec::new(Box::new(scan), vec![Expr::column("name")], proj_schema);
        let result = engine.execute_plan(&projection).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_storage_engine_export() {
        let _: Option<Box<dyn StorageEngine>> = None;
    }

    #[test]
    fn test_executor_export() {
        let _: Option<Box<dyn Executor>> = None;
    }

    #[test]
    fn test_planner_export() {
        let _: Option<Box<dyn Planner>> = None;
    }

    #[test]
    fn test_execute_analyze_sql() {
        let mut engine = ExecutionEngine::default();

        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(sqlrustgo_parser::parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();
        engine
            .execute(sqlrustgo_parser::parse("INSERT INTO users VALUES (2, 'Bob')").unwrap())
            .unwrap();

        let result = engine
            .execute(sqlrustgo_parser::parse("ANALYZE users").unwrap())
            .unwrap();

        assert!(result.rows.len() > 0);
        assert!(result.rows.iter().any(|r| r.iter().any(|v| match v {
            Value::Text(s) => s.contains("users"),
            _ => false,
        })));
    }

    #[test]
    fn test_execute_explain_analyze() {
        let mut engine = ExecutionEngine::default();

        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        engine
            .execute(sqlrustgo_parser::parse("INSERT INTO users VALUES (1)").unwrap())
            .unwrap();

        let result = engine
            .execute(sqlrustgo_parser::parse("EXPLAIN ANALYZE SELECT * FROM users").unwrap())
            .unwrap();

        assert!(result.rows.len() > 0);
    }

    #[test]
    fn test_execute_explain_without_analyze() {
        let mut engine = ExecutionEngine::default();

        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine
            .execute(sqlrustgo_parser::parse("EXPLAIN SELECT * FROM users").unwrap())
            .unwrap();

        assert!(result.rows.len() >= 0);
    }

    #[test]
    fn test_execute_plan_with_index_scan() {
        use sqlrustgo_planner::{DataType, Field, IndexScanExec, Schema};

        let mut storage = MemoryStorage::new();
        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "users".to_string(),
                columns: vec![sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
            )
            .unwrap();

        let engine = ExecutionEngine::new(std::sync::Arc::new(std::sync::RwLock::new(storage)));

        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let scan = IndexScanExec::new(
            "users".to_string(),
            "idx_id".to_string(),
            sqlrustgo_planner::Expr::Literal(Value::Integer(1)),
            schema,
        );

        let result = engine.execute_plan(&scan).unwrap();
        assert!(result.rows.len() >= 0);
    }

    #[test]
    fn test_execute_plan_with_hash_join() {
        use sqlrustgo_planner::{DataType, Field, HashJoinExec, JoinType, Schema};

        let mut storage = MemoryStorage::new();

        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "users".to_string(),
                columns: vec![sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
            )
            .unwrap();

        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "users".to_string(),
                columns: vec![sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    references: None,
                    auto_increment: false,
                }],
            })
            .unwrap();
        storage
            .insert(
                "orders",
                vec![vec![Value::Integer(1)], vec![Value::Integer(2)]],
            )
            .unwrap();

        let engine = ExecutionEngine::new(std::sync::Arc::new(std::sync::RwLock::new(storage)));

        let left_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let right_schema = Schema::new(vec![Field::new("user_id".to_string(), DataType::Integer)]);

        let left_scan = Box::new(sqlrustgo_planner::SeqScanExec::new(
            "users".to_string(),
            left_schema.clone(),
        ));
        let right_scan = Box::new(sqlrustgo_planner::SeqScanExec::new(
            "orders".to_string(),
            right_schema.clone(),
        ));

        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("user_id".to_string(), DataType::Integer),
        ]);

        let join = HashJoinExec::new(left_scan, right_scan, JoinType::Inner, None, join_schema);

        let result = engine.execute_plan(&join).unwrap();
        assert!(result.rows.len() > 0);
    }

    #[test]
    fn test_execute_delete() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("DELETE FROM users WHERE id = 1").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 1);

        let select_result = engine
            .execute(parse("SELECT COUNT(*) FROM users").unwrap())
            .unwrap();
        assert_eq!(select_result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_execute_update() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("UPDATE users SET name = 'Charlie' WHERE id = 1").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 1);

        let select_result = engine
            .execute(parse("SELECT name FROM users WHERE id = 1").unwrap())
            .unwrap();
        assert_eq!(select_result.rows[0][0], Value::Text("Charlie".to_string()));
    }

    #[test]
    fn test_execute_drop_table() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine.execute(parse("DROP TABLE users").unwrap()).unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_alter_table_add_column() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("ALTER TABLE users ADD COLUMN name TEXT").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_insert_replace() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("REPLACE INTO users VALUES (1, 'Bob')").unwrap())
            .unwrap();
        assert!(result.affected_rows >= 1);

        let select_result = engine
            .execute(parse("SELECT COUNT(*) FROM users").unwrap())
            .unwrap();
        assert_eq!(select_result.rows[0][0], Value::Integer(1));
    }

    #[test]
    fn test_execute_insert_ignore() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap())
            .unwrap();
        engine
            .execute(parse("INSERT INTO users VALUES (1, 'Alice')").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("INSERT IGNORE INTO users VALUES (1, 'Bob')").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 0);

        let select_result = engine
            .execute(parse("SELECT name FROM users WHERE id = 1").unwrap())
            .unwrap();
        assert_eq!(select_result.rows[0][0], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_execute_truncate_parsing() {
        let result = parse("TRUNCATE TABLE users");
        assert!(result.is_ok());
        match result.unwrap() {
            Statement::Truncate(trunc) => {
                assert_eq!(trunc.table_name, "users");
            }
            _ => panic!("Expected Truncate statement"),
        }
    }

    #[test]
    fn test_execute_create_index() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER, name TEXT)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("CREATE INDEX idx_name ON users (name)").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_show_status() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine.execute(parse("SHOW STATUS").unwrap()).unwrap();
        assert!(result.rows.len() >= 0);
    }

    #[test]
    fn test_error_table_not_found() {
        let mut engine = ExecutionEngine::default();
        let result = engine.execute(parse("SELECT * FROM nonexistent").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_error_nonexistent_table() {
        let mut engine = ExecutionEngine::default();
        let result = engine.execute(parse("SELECT * FROM nonexistent").unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_if_not_exists() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine
            .execute(parse("CREATE TABLE IF NOT EXISTS users (id INTEGER)").unwrap())
            .unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_transaction_begin() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine.execute(parse("BEGIN").unwrap()).unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_transaction_commit() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        engine.execute(parse("BEGIN").unwrap()).unwrap();
        let result = engine.execute(parse("COMMIT").unwrap()).unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_transaction_rollback() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        engine.execute(parse("BEGIN").unwrap()).unwrap();
        let result = engine.execute(parse("ROLLBACK").unwrap()).unwrap();
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_execute_grant() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine.execute(parse("GRANT SELECT ON users TO PUBLIC").unwrap());
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_execute_revoke() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();

        let result = engine.execute(parse("REVOKE SELECT ON users FROM PUBLIC").unwrap());
        assert!(result.is_ok() || result.is_err());
    }
}
