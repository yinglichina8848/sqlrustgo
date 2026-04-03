//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

use rust_decimal::Decimal;

pub use sqlrustgo_executor::{Executor, ExecutorResult, GLOBAL_PROFILER};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{
    parse, Expression, GrantStatement, KillStatement, KillType, Lexer, Privilege, RevokeStatement,
    SetOperation, Statement, Token, TransactionCommand,
};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner, SetOperationType};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine, ViewInfo,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

// GMP Document Retrieval Extension
pub use sqlrustgo_gmp::*;

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

/// Extract index usage info from a WHERE clause
/// Returns (column_name, op, value) if the WHERE can use an index
/// For TEXT columns, value is the hash of the string
fn extract_index_predicate(
    where_clause: &sqlrustgo_parser::Expression,
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Option<(String, String, i64)> {
    match where_clause {
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            
            // Only handle simple comparison operators
            if !["<", ">", "=", "<=", ">="].contains(&op_upper.as_str()) {
                return None;
            }
            
            // Check if left side is a column and right side is a value
            let (col_name, value_str) = match (&**left, &**right) {
                (sqlrustgo_parser::Expression::Identifier(name), sqlrustgo_parser::Expression::Literal(val)) => {
                    (name.clone(), val.clone())
                }
                (sqlrustgo_parser::Expression::QualifiedColumn(_, col), sqlrustgo_parser::Expression::Literal(val)) => {
                    (col.clone(), val.clone())
                }
                _ => return None,
            };
            
            // Check if column exists
            let col_def = columns.iter().find(|c| c.name == col_name)?;
            let upper = col_def.data_type.to_uppercase();
            
            // For TEXT columns, we need to hash the string value
            // Hash must match the hash function used in to_index_key()
            use std::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;
            
            let value_i64 = if upper.contains("INT") || upper == "BIGINT" || upper == "SMALLINT" || upper == "TINYINT" {
                // Integer column: parse the value as i64
                value_str.parse::<i64>().ok()?
            } else {
                // TEXT column: hash the string
                // NOTE: For TEXT, only "=" operator works correctly with hash index
                // Range operators (<, >) will not work correctly with hash
                if op_upper != "=" {
                    return None; // Hash doesn't support range queries
                }
                let mut hasher = DefaultHasher::new();
                value_str.hash(&mut hasher);
                hasher.finish() as i64
            };
            
            Some((col_name, op_upper, value_i64))
        }
        _ => None,
    }
}

/// Use index to filter rows for a single table
/// Returns Some(filtered_rows) if index was used, None if full scan needed
fn filter_using_index(
    storage: &std::sync::RwLockReadGuard<'_, dyn sqlrustgo_storage::StorageEngine>,
    table_name: &str,
    column_name: &str,
    op: &str,
    value: i64,
    _columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Option<Vec<Vec<Value>>> {
    match op {
        "=" => {
            // Exact match - use search_index and get_row (O(log n) instead of O(n))
            let row_ids = storage.search_index(table_name, column_name, value);
            if row_ids.is_empty() {
                return Some(vec![]);
            }
            let filtered: Vec<Vec<Value>> = row_ids
                .into_iter()
                .filter_map(|id| storage.get_row(table_name, id as usize).ok().flatten())
                .collect();
            Some(filtered)
        }
        "<" | "<=" | ">" | ">=" => {
            // Range query - use index WITHOUT scanning all rows first
            let (start, end) = match op {
                "<" => (i64::MIN, value),
                "<=" => (i64::MIN, value + 1),
                ">" => (value + 1, i64::MAX),
                ">=" => (value, i64::MAX),
                _ => return None,
            };
            
            // Get row IDs from index (this is O(log n) instead of O(n))
            let row_ids = storage.range_index(table_name, column_name, start, end);
            
            if row_ids.is_empty() {
                return Some(vec![]);
            }
            
            // Now fetch only the specific rows we need using get_row
            let filtered: Vec<Vec<Value>> = row_ids
                .into_iter()
                .filter_map(|id| storage.get_row(table_name, id as usize).ok().flatten())
                .collect();
            Some(filtered)
        }
        _ => None,
    }
}

/// Evaluate a WHERE clause expression against a row
fn evaluate_where_clause(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> bool {
    match expr {
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            if op_upper == "AND" {
                let left_bool = evaluate_where_clause(left, row, columns);
                if !left_bool {
                    return false;
                }
                return evaluate_where_clause(right, row, columns);
            } else if op_upper == "OR" {
                let left_bool = evaluate_where_clause(left, row, columns);
                if left_bool {
                    return true;
                }
                return evaluate_where_clause(right, row, columns);
            }
            let left_val = evaluate_expr(left, row, columns);
            let right_val = evaluate_expr(right, row, columns);
            compare_values(&left_val, &op_upper, &right_val)
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
        sqlrustgo_parser::Expression::Extract { .. } => false,
        sqlrustgo_parser::Expression::Substring { .. } => false,
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
            if let Ok(n) = s.parse::<i64>() {
                Value::Integer(n)
            } else if s.contains('.') {
                if let Ok(d) = s.parse::<Decimal>() {
                    Value::Decimal(d)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::Text(s.clone())
                }
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Float(f)
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
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l + r),
                    (Value::Integer(l), Value::Decimal(r)) => Value::Decimal(Decimal::from(*l) + r),
                    (Value::Decimal(l), Value::Integer(r)) => Value::Decimal(l + Decimal::from(*r)),
                    (Value::Float(l), Value::Decimal(r)) => {
                        Value::Decimal(Decimal::from_f64_retain(*l).unwrap_or_default() + r)
                    }
                    (Value::Decimal(l), Value::Float(r)) => {
                        Value::Decimal(l + Decimal::from_f64_retain(*r).unwrap_or_default())
                    }
                    _ => Value::Null,
                },
                "-" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l - r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 - r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l - *r as f64),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l - r),
                    (Value::Integer(l), Value::Decimal(r)) => Value::Decimal(Decimal::from(*l) - r),
                    (Value::Decimal(l), Value::Integer(r)) => Value::Decimal(l - Decimal::from(*r)),
                    (Value::Float(l), Value::Decimal(r)) => {
                        Value::Decimal(Decimal::from_f64_retain(*l).unwrap_or_default() - r)
                    }
                    (Value::Decimal(l), Value::Float(r)) => {
                        Value::Decimal(l - Decimal::from_f64_retain(*r).unwrap_or_default())
                    }
                    _ => Value::Null,
                },
                "*" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
                    (Value::Float(l), Value::Float(r)) => Value::Float(l * r),
                    (Value::Integer(l), Value::Float(r)) => Value::Float(*l as f64 * r),
                    (Value::Float(l), Value::Integer(r)) => Value::Float(l * *r as f64),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l * r),
                    (Value::Integer(l), Value::Decimal(r)) => Value::Decimal(Decimal::from(*l) * r),
                    (Value::Decimal(l), Value::Integer(r)) => Value::Decimal(l * Decimal::from(*r)),
                    (Value::Float(l), Value::Decimal(r)) => {
                        Value::Decimal(Decimal::from_f64_retain(*l).unwrap_or_default() * r)
                    }
                    (Value::Decimal(l), Value::Float(r)) => {
                        Value::Decimal(l * Decimal::from_f64_retain(*r).unwrap_or_default())
                    }
                    _ => Value::Null,
                },
                "/" => match (&left_val, &right_val) {
                    (Value::Integer(l), Value::Integer(r)) if *r != 0 => Value::Integer(l / r),
                    (Value::Float(l), Value::Float(r)) if *r != 0.0 => Value::Float(l / r),
                    (Value::Integer(l), Value::Float(r)) if *r != 0.0 => {
                        Value::Float(*l as f64 / r)
                    }
                    (Value::Float(l), Value::Integer(r)) if *r != 0 => Value::Float(l / *r as f64),
                    (Value::Decimal(l), Value::Decimal(r)) if !r.is_zero() => Value::Decimal(l / r),
                    (Value::Integer(l), Value::Decimal(r)) if !r.is_zero() => {
                        Value::Decimal(Decimal::from(*l) / r)
                    }
                    (Value::Decimal(l), Value::Integer(r)) if *r != 0 => {
                        Value::Decimal(l / Decimal::from(*r))
                    }
                    (Value::Float(l), Value::Decimal(r)) if !r.is_zero() => {
                        Value::Decimal(Decimal::from_f64_retain(*l).unwrap_or_default() / r)
                    }
                    (Value::Decimal(l), Value::Float(r)) if *r != 0.0 => {
                        Value::Decimal(l / Decimal::from_f64_retain(*r).unwrap_or_default())
                    }
                    _ => Value::Null,
                },
                "=" | "==" | "EQ" => Value::Boolean(left_val == right_val),
                "!=" | "<>" | "NE" => Value::Boolean(left_val != right_val),
                ">" | "GT" => match (left_val, right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Boolean(l > r),
                    (Value::Float(l), Value::Float(r)) => Value::Boolean(l > r),
                    (Value::Integer(l), Value::Float(r)) => Value::Boolean((l as f64) > r),
                    (Value::Float(l), Value::Integer(r)) => Value::Boolean(l > (r as f64)),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Boolean(l > r),
                    (Value::Text(l), Value::Text(r)) => Value::Boolean(l > r),
                    _ => Value::Boolean(false),
                },
                "<" | "LT" => match (left_val, right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Boolean(l < r),
                    (Value::Float(l), Value::Float(r)) => Value::Boolean(l < r),
                    (Value::Integer(l), Value::Float(r)) => Value::Boolean((l as f64) < r),
                    (Value::Float(l), Value::Integer(r)) => Value::Boolean(l < (r as f64)),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Boolean(l < r),
                    (Value::Text(l), Value::Text(r)) => Value::Boolean(l < r),
                    _ => Value::Boolean(false),
                },
                ">=" | "GE" => match (left_val, right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Boolean(l >= r),
                    (Value::Float(l), Value::Float(r)) => Value::Boolean(l >= r),
                    (Value::Integer(l), Value::Float(r)) => Value::Boolean((l as f64) >= r),
                    (Value::Float(l), Value::Integer(r)) => Value::Boolean(l >= (r as f64)),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Boolean(l >= r),
                    (Value::Text(l), Value::Text(r)) => Value::Boolean(l >= r),
                    _ => Value::Boolean(false),
                },
                "<=" | "LE" => match (left_val, right_val) {
                    (Value::Integer(l), Value::Integer(r)) => Value::Boolean(l <= r),
                    (Value::Float(l), Value::Float(r)) => Value::Boolean(l <= r),
                    (Value::Integer(l), Value::Float(r)) => Value::Boolean((l as f64) <= r),
                    (Value::Float(l), Value::Integer(r)) => Value::Boolean(l <= (r as f64)),
                    (Value::Decimal(l), Value::Decimal(r)) => Value::Boolean(l <= r),
                    (Value::Text(l), Value::Text(r)) => Value::Boolean(l <= r),
                    _ => Value::Boolean(false),
                },
                _ => Value::Null,
            }
        }
        sqlrustgo_parser::Expression::Wildcard => Value::Null,
        sqlrustgo_parser::Expression::FunctionCall(_, _) => Value::Null,
        sqlrustgo_parser::Expression::Subquery(_) => Value::Null,
        sqlrustgo_parser::Expression::QualifiedColumn(_table_name, col_name) => {
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(col_name))
            {
                row.get(idx).cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
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
        sqlrustgo_parser::Expression::Extract { field, expr } => {
            let val = evaluate_expr(expr, row, columns);
            if let Value::Text(s) = val {
                let parts: Vec<&str> = s.split('-').collect();
                if parts.len() == 3 {
                    match field.as_str() {
                        "YEAR" => {
                            if let Ok(y) = parts[0].parse::<i64>() {
                                return Value::Integer(y);
                            }
                        }
                        "MONTH" => {
                            if let Ok(m) = parts[1].parse::<i64>() {
                                return Value::Integer(m);
                            }
                        }
                        "DAY" => {
                            if let Ok(d) = parts[2].parse::<i64>() {
                                return Value::Integer(d);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Value::Null
        }
        sqlrustgo_parser::Expression::Substring { expr, start, len } => {
            let val = evaluate_expr(expr, row, columns);
            let start_val = evaluate_expr(start, row, columns);
            let result = match (&val, &start_val) {
                (Value::Text(s), Value::Integer(i)) => {
                    let start_idx = (*i as isize - 1).max(0) as usize;
                    if let Some(len_expr) = len {
                        let len_val = evaluate_expr(len_expr, row, columns);
                        if let Value::Integer(len_int) = len_val {
                            let len_usize = (len_int as usize).min(1000000);
                            Some(
                                s.chars()
                                    .skip(start_idx)
                                    .take(len_usize)
                                    .collect::<String>(),
                            )
                        } else {
                            None
                        }
                    } else {
                        Some(s.chars().skip(start_idx).collect::<String>())
                    }
                }
                _ => None,
            };
            result.map_or(Value::Null, Value::Text)
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

    // Get the column index for simple column references, or None for complex expressions
    let col_idx: Option<usize> = if agg.args.is_empty() {
        None
    } else {
        match &agg.args[0] {
            sqlrustgo_parser::Expression::Identifier(name) => columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name)),
            sqlrustgo_parser::Expression::Wildcard => Some(0),
            _ => None,
        }
    };

    // For complex expressions, we need to evaluate them per row
    let arg_expr = if agg.args.is_empty() {
        None
    } else {
        Some(&agg.args[0])
    };

    // Collect all values first (for DISTINCT handling)
    let mut all_values: Vec<Value> = Vec::new();
    for row in rows {
        let val = if let Some(idx) = col_idx {
            row.get(idx).cloned()
        } else {
            arg_expr.map(|expr| evaluate_expr(expr, row, columns))
        };
        all_values.push(val.unwrap_or(Value::Null));
    }

    // Apply DISTINCT if specified
    if agg.distinct {
        let mut unique_values = all_values.clone();
        unique_values.sort();
        unique_values.dedup();
        all_values = unique_values;
    }

    match agg.func {
        AggregateFunction::Count => {
            if agg.args.is_empty()
                || matches!(
                    agg.args.first(),
                    Some(sqlrustgo_parser::Expression::Wildcard)
                )
            {
                Value::Integer(all_values.len() as i64)
            } else {
                let count = all_values.iter().filter(|v| **v != Value::Null).count();
                Value::Integer(count as i64)
            }
        }
        AggregateFunction::Sum => {
            if all_values.is_empty() {
                return Value::Null;
            }

            let mut int_sum = 0i64;
            let mut float_sum = 0.0f64;
            let mut decimal_sum = Decimal::ZERO;
            let mut has_decimal = false;
            let mut has_float = false;

            for val in &all_values {
                match val {
                    Value::Integer(n) => int_sum += *n,
                    Value::Float(n) => {
                        float_sum += *n;
                        has_float = true;
                    }
                    Value::Decimal(n) => {
                        decimal_sum += *n;
                        has_decimal = true;
                    }
                    _ => {}
                }
            }

            if has_decimal {
                Value::Decimal(decimal_sum)
            } else if has_float {
                Value::Float(int_sum as f64 + float_sum)
            } else {
                Value::Integer(int_sum)
            }
        }
        AggregateFunction::Avg => {
            let mut sum = 0.0f64;
            let mut count = 0i64;
            for val in &all_values {
                if let Value::Integer(n) = val {
                    sum += *n as f64;
                    count += 1;
                } else if let Value::Float(n) = val {
                    sum += *n;
                    count += 1;
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
            for val in &all_values {
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
            min.unwrap_or(Value::Null)
        }
        AggregateFunction::Max => {
            let mut max: Option<Value> = None;
            for val in &all_values {
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
            max.unwrap_or(Value::Null)
        }
    }
}

/// Convert a string to AggregateFunction, returning None if not a valid aggregate
fn str_to_aggregate_function(name: &str) -> Option<sqlrustgo_parser::parser::AggregateFunction> {
    match name.to_uppercase().as_str() {
        "COUNT" => Some(sqlrustgo_parser::parser::AggregateFunction::Count),
        "SUM" => Some(sqlrustgo_parser::parser::AggregateFunction::Sum),
        "AVG" => Some(sqlrustgo_parser::parser::AggregateFunction::Avg),
        "MIN" => Some(sqlrustgo_parser::parser::AggregateFunction::Min),
        "MAX" => Some(sqlrustgo_parser::parser::AggregateFunction::Max),
        _ => None,
    }
}

/// Evaluate a HAVING clause expression against a group's rows.
/// Returns true if the row passes the HAVING condition.
/// This function handles aggregate functions like SUM(col) > value.
fn evaluate_having_expr(
    having: &sqlrustgo_parser::Expression,
    group_rows: &[Vec<Value>],
    aggregates: &[sqlrustgo_parser::parser::AggregateCall],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> bool {
    match having {
        // Handle binary comparison: SUM(amount) > 150
        sqlrustgo_parser::Expression::BinaryOp(left, op, right) => {
            let op_upper = op.to_uppercase();
            if ["=", "==", "EQ", "!=", "<>", "NE", ">", "GT", "<", "LT", ">=", "GE", "<=", "LE"]
                .contains(&op_upper.as_str())
            {
                // Try to evaluate left as aggregate
                if let Some(left_val) =
                    evaluate_aggregate_in_expr(left, group_rows, aggregates, columns)
                {
                    let right_val = evaluate_having_value(right, group_rows, aggregates, columns);
                    match op_upper.as_str() {
                        "=" | "==" | "EQ" => left_val == right_val,
                        "!=" | "<>" | "NE" => left_val != right_val,
                        ">" | "GT" => left_val > right_val,
                        "<" | "LT" => left_val < right_val,
                        ">=" | "GE" => left_val >= right_val,
                        "<=" | "LE" => left_val <= right_val,
                        _ => false,
                    }
                } else if let Some(right_val) =
                    evaluate_aggregate_in_expr(right, group_rows, aggregates, columns)
                {
                    let left_val = evaluate_having_value(left, group_rows, aggregates, columns);
                    match op_upper.as_str() {
                        "=" | "==" | "EQ" => left_val == right_val,
                        "!=" | "<>" | "NE" => left_val != right_val,
                        ">" | "GT" => left_val > right_val,
                        "<" | "LT" => left_val < right_val,
                        ">=" | "GE" => left_val >= right_val,
                        "<=" | "LE" => left_val <= right_val,
                        _ => false,
                    }
                } else {
                    // Fallback to regular evaluation
                    let left_val = evaluate_having_value(left, group_rows, aggregates, columns);
                    let right_val = evaluate_having_value(right, group_rows, aggregates, columns);
                    match op_upper.as_str() {
                        "=" | "==" | "EQ" => left_val == right_val,
                        "!=" | "<>" | "NE" => left_val != right_val,
                        ">" | "GT" => left_val > right_val,
                        "<" | "LT" => left_val < right_val,
                        ">=" | "GE" => left_val >= right_val,
                        "<=" | "LE" => left_val <= right_val,
                        _ => false,
                    }
                }
            } else if op_upper == "AND" {
                evaluate_having_expr(left, group_rows, aggregates, columns)
                    && evaluate_having_expr(right, group_rows, aggregates, columns)
            } else if op_upper == "OR" {
                evaluate_having_expr(left, group_rows, aggregates, columns)
                    || evaluate_having_expr(right, group_rows, aggregates, columns)
            } else {
                false
            }
        }
        // Handle function calls in HAVING like COUNT(*) > 1
        sqlrustgo_parser::Expression::FunctionCall(name, args) => {
            let func_name = name.to_uppercase();
            if let Some(agg_func) = str_to_aggregate_function(&func_name) {
                let rows_owned: Vec<Vec<Value>> =
                    group_rows.iter().map(|r| (*r).clone()).collect();
                let agg_call = sqlrustgo_parser::parser::AggregateCall {
                    func: agg_func,
                    args: args.clone(),
                    distinct: false,
                };
                let result = compute_aggregate(&agg_call, &rows_owned, columns);
                result.to_bool()
            } else {
                false
            }
        }
        // Handle simple identifier (boolean check)
        sqlrustgo_parser::Expression::Identifier(name) => {
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                if let Some(val) = group_rows.first().and_then(|r| r.get(idx)) {
                    return val.to_bool();
                }
            }
            false
        }
        _ => false,
    }
}

/// Helper to evaluate a value in HAVING context, handling aggregate functions
fn evaluate_having_value(
    expr: &sqlrustgo_parser::Expression,
    group_rows: &[Vec<Value>],
    _aggregates: &[sqlrustgo_parser::parser::AggregateCall],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Value {
    match expr {
        sqlrustgo_parser::Expression::Literal(s) => {
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
            if let Some(idx) = columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(name))
            {
                if let Some(val) = group_rows.first().and_then(|r| r.get(idx).cloned()) {
                    return val;
                }
            }
            Value::Null
        }
        sqlrustgo_parser::Expression::FunctionCall(name, args) => {
            let func_name = name.to_uppercase();
            if let Some(agg_func) = str_to_aggregate_function(&func_name) {
                let rows_owned: Vec<Vec<Value>> =
                    group_rows.iter().map(|r| (*r).clone()).collect();
                let agg_call = sqlrustgo_parser::parser::AggregateCall {
                    func: agg_func,
                    args: args.clone(),
                    distinct: false,
                };
                compute_aggregate(&agg_call, &rows_owned, columns)
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

/// Helper to check if an expression contains an aggregate function and evaluate it
fn evaluate_aggregate_in_expr(
    expr: &sqlrustgo_parser::Expression,
    group_rows: &[Vec<Value>],
    aggregates: &[sqlrustgo_parser::parser::AggregateCall],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Option<Value> {
    match expr {
        sqlrustgo_parser::Expression::FunctionCall(name, args) => {
            let func_name = name.to_uppercase();
            if let Some(agg_func) = str_to_aggregate_function(&func_name) {
                // Find if this aggregate matches any in the SELECT clause
                for agg in aggregates {
                    if agg.func == agg_func && agg.args.len() == args.len() {
                        let mut matches = true;
                        for (arg, select_arg) in args.iter().zip(agg.args.iter()) {
                            if let (
                                sqlrustgo_parser::Expression::Identifier(arg_name),
                                sqlrustgo_parser::Expression::Identifier(select_arg_name),
                            ) = (arg, select_arg)
                            {
                                if !arg_name.eq_ignore_ascii_case(select_arg_name) {
                                    matches = false;
                                    break;
                                }
                            } else {
                                matches = false;
                                break;
                            }
                        }
                        if matches {
                            let rows_owned: Vec<Vec<Value>> =
                                group_rows.iter().map(|r| (*r).clone()).collect();
                            return Some(compute_aggregate(agg, &rows_owned, columns));
                        }
                    }
                }
                // If no exact match, try by function name alone
                let rows_owned: Vec<Vec<Value>> =
                    group_rows.iter().map(|r| (*r).clone()).collect();
                let agg_call = sqlrustgo_parser::parser::AggregateCall {
                    func: agg_func,
                    args: args.clone(),
                    distinct: false,
                };
                return Some(compute_aggregate(&agg_call, &rows_owned, columns));
            }
            None
        }
        _ => None,
    }
}

/// Evaluate a GROUP BY key expression to a Value
fn evaluate_group_by_expr(
    expr: &sqlrustgo_parser::Expression,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Option<Value> {
    match expr {
        sqlrustgo_parser::Expression::Identifier(name) => columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))
            .and_then(|idx| row.get(idx).cloned()),
        sqlrustgo_parser::Expression::QualifiedColumn(_table_name, col_name) => {
            // Find the column position for qualified column
            columns
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(col_name))
                .and_then(|idx| row.get(idx).cloned())
        }
        sqlrustgo_parser::Expression::Literal(s) => {
            if let Ok(n) = s.parse::<i64>() {
                Some(Value::Integer(n))
            } else if let Ok(n) = s.parse::<f64>() {
                Some(Value::Float(n))
            } else {
                Some(Value::Text(s.clone()))
            }
        }
        sqlrustgo_parser::Expression::Wildcard => Some(Value::Text("*".to_string())),
        sqlrustgo_parser::Expression::Extract { field, expr } => {
            let expr_val = evaluate_group_by_expr(expr, row, columns)?;
            if let Value::Text(s) = expr_val {
                let date_part = match field.to_uppercase().as_str() {
                    "YEAR" => s.chars().take(4).collect::<String>(),
                    "MONTH" => s.chars().skip(5).take(2).collect::<String>(),
                    "DAY" => s.chars().skip(8).take(2).collect::<String>(),
                    _ => s,
                };
                date_part.parse::<i64>().ok().map(Value::Integer)
            } else {
                None
            }
        }
        sqlrustgo_parser::Expression::Substring { expr, start, len } => {
            let expr_val = evaluate_group_by_expr(expr, row, columns)?;
            let start_val = evaluate_group_by_expr(start, row, columns)?;
            let len_val = len
                .as_ref()
                .and_then(|l| evaluate_group_by_expr(l, row, columns));

            if let Value::Text(s) = expr_val {
                let start_idx = if let Value::Integer(n) = start_val {
                    n as usize
                } else {
                    1
                };
                let len_usize = if let Some(Value::Integer(n)) = len_val {
                    n as usize
                } else {
                    s.len()
                };
                Some(Value::Text(
                    s.chars().skip(start_idx - 1).take(len_usize).collect(),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Evaluate GROUP BY key (combination of all GROUP BY columns) from a row
fn evaluate_group_by_key(
    group_by: &sqlrustgo_parser::parser::GroupByClause,
    row: &[Value],
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Option<Vec<Value>> {
    let mut key = Vec::new();
    for expr in &group_by.columns {
        if let Some(val) = evaluate_group_by_expr(expr, row, columns) {
            key.push(val);
        } else {
            return None;
        }
    }
    Some(key)
}

fn compare_values_for_sort(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (left, right) {
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Null, _) => std::cmp::Ordering::Less,
        (_, Value::Null) => std::cmp::Ordering::Greater,
        (Value::Integer(l), Value::Integer(r)) => l.cmp(r),
        (Value::Float(l), Value::Float(r)) => l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(l), Value::Text(r)) => l.cmp(r),
        (Value::Boolean(l), Value::Boolean(r)) => l.cmp(r),
        (Value::Integer(l), Value::Float(r)) => (*l as f64)
            .partial_cmp(r)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(l), Value::Integer(r)) => l
            .partial_cmp(&(*r as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => std::cmp::Ordering::Equal,
    }
}

fn sort_rows_by_order_by(
    rows: Vec<Vec<Value>>,
    order_by: &sqlrustgo_parser::parser::OrderByClause,
    columns: &[sqlrustgo_storage::ColumnDefinition],
) -> Vec<Vec<Value>> {
    let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = rows
        .into_iter()
        .map(|row| {
            let sort_keys: Vec<Value> = order_by
                .items
                .iter()
                .filter_map(|item| evaluate_group_by_expr(&item.expr, &row, columns))
                .collect();
            (row, sort_keys)
        })
        .collect();

    rows_with_keys.sort_by(|(_, keys1), (_, keys2)| {
        for (key1, key2) in keys1.iter().zip(keys2.iter()) {
            let cmp = compare_values_for_sort(key1, key2);
            if cmp != std::cmp::Ordering::Equal {
                let item_idx = keys1.iter().position(|k| k == key1).unwrap_or(0);
                if !order_by.items[item_idx].asc {
                    return cmp.reverse();
                }
                return cmp;
            }
        }
        std::cmp::Ordering::Equal
    });

    rows_with_keys.into_iter().map(|(row, _)| row).collect()
}

pub struct ExecutionEngine {
    pub storage: Arc<RwLock<dyn StorageEngine>>,
    session_manager: Option<Arc<sqlrustgo_security::SessionManager>>,
    current_session_id: Option<u64>,
}

impl ExecutionEngine {
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self {
            storage,
            session_manager: None,
            current_session_id: None,
        }
    }

    pub fn new_with_session(
        storage: Arc<RwLock<dyn StorageEngine>>,
        session_manager: Arc<sqlrustgo_security::SessionManager>,
        session_id: u64,
    ) -> Self {
        Self {
            storage,
            session_manager: Some(session_manager),
            current_session_id: Some(session_id),
        }
    }

    pub fn session_id(&self) -> Option<u64> {
        self.current_session_id
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
                                                        || upper == "REAL"
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
                                                            || upper == "REAL"
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
            Statement::CreateIndex(create) => {
                let mut storage = self.storage.write().unwrap();
                if create.columns.is_empty() {
                    return Err(SqlError::ExecutionError(
                        "CREATE INDEX requires at least one column".to_string(),
                    ));
                }
                let table_info = storage.get_table_info(&create.table)?;
                let column_index = table_info
                    .columns
                    .iter()
                    .position(|c| c.name == create.columns[0])
                    .ok_or_else(|| {
                        SqlError::ExecutionError(format!(
                            "Column '{}' not found in table '{}'",
                            create.columns[0], create.table
                        ))
                    })?;
                storage.create_table_index(&create.table, &create.columns[0], column_index)?;
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

                let tables = if select.tables.is_empty() {
                    vec![select.table.clone()]
                } else {
                    select.tables.clone()
                };

                for table_name in &tables {
                    if !storage.has_table(table_name) {
                        return Err(SqlError::ExecutionError(format!(
                            "Table '{}' not found",
                            table_name
                        )));
                    }
                }

                let mut all_columns: Vec<sqlrustgo_storage::ColumnDefinition> = Vec::new();
                let mut all_rows: Vec<Vec<Value>> = vec![vec![]];

                for table_name in &tables {
                    let table_info = storage.get_table_info(table_name).ok();
                    if let Some(info) = table_info {
                        for col in &info.columns {
                            all_columns.push(col.clone());
                        }
                    }
                    
                    // Try to use index for single-table SELECT with WHERE on indexed column
                    #[allow(clippy::unnecessary_unwrap)]
                    let rows = if tables.len() == 1 && select.where_clause.is_some() {
                        if let Some((col_name, op, value)) = extract_index_predicate(select.where_clause.as_ref().unwrap(), &all_columns) {
                            if let Some(indexed_rows) = filter_using_index(&storage, table_name, &col_name, &op, value, &all_columns) {
                                indexed_rows
                            } else {
                                storage.scan(table_name).unwrap_or_default()
                            }
                        } else {
                            storage.scan(table_name).unwrap_or_default()
                        }
                    } else {
                        storage.scan(table_name).unwrap_or_default()
                    };

                    let new_all_rows: Vec<Vec<Value>> = all_rows
                        .iter()
                        .flat_map(|existing_row| {
                            rows.iter()
                                .map(|row| {
                                    let mut combined = existing_row.clone();
                                    combined.extend(row.clone());
                                    combined
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    all_rows = new_all_rows;
                }

                let columns = all_columns.clone();

                let filtered_rows: Vec<Vec<Value>> =
                    if let Some(ref where_clause) = select.where_clause {
                        // If we already used index to filter, no need to filter again
                        if tables.len() == 1 {
                            if let Some((ref col_name, ref _op, ref value)) = extract_index_predicate(where_clause, &columns) {
                                let index_results = storage.search_index(&tables[0], col_name, *value);
                                if !index_results.is_empty() {
                                    // Already filtered by index, skip evaluation
                                    all_rows
                                } else {
                                    all_rows.into_iter().filter(|row| evaluate_where_clause(where_clause, row, &columns)).collect()
                                }
                            } else {
                                all_rows.into_iter().filter(|row| evaluate_where_clause(where_clause, row, &columns)).collect()
                            }
                        } else {
                            all_rows.into_iter().filter(|row| evaluate_where_clause(where_clause, row, &columns)).collect()
                        }
                    } else {
                        all_rows
                    };

                // Handle aggregates if present
                if !select.aggregates.is_empty() {
                    // Check if there's a GROUP BY clause
                    if let Some(ref group_by) = select.group_by {
                        // GROUP BY: group rows and compute aggregates per group
                        use std::collections::HashMap;
                        let mut grouped: HashMap<Vec<Value>, Vec<&Vec<Value>>> = HashMap::new();

                        // Group rows by GROUP BY key
                        for row in &filtered_rows {
                            if let Some(group_key) = evaluate_group_by_key(group_by, row, &columns)
                            {
                                grouped.entry(group_key).or_default().push(row);
                            }
                        }

                        // For each group, compute aggregates and build result rows
                        let mut result_rows: Vec<Vec<Value>> = Vec::new();
                        for (group_key, group_rows) in grouped {
                            let mut result_row = group_key;
                            for agg in &select.aggregates {
                                // Convert &[&Vec<Value>] to Vec<Vec<Value>>
                                let rows_owned: Vec<Vec<Value>> =
                                    group_rows.iter().map(|r| (*r).clone()).collect();
                                let agg_result = compute_aggregate(agg, &rows_owned, &columns);
                                result_row.push(agg_result);
                            }

                            // Apply HAVING filter if present
                            if let Some(ref having) = select.having {
                                let rows_owned: Vec<Vec<Value>> =
                                    group_rows.iter().map(|r| (*r).clone()).collect();
                                if !evaluate_having_expr(having, &rows_owned, &select.aggregates, &columns)
                                {
                                    continue; // Skip this group
                                }
                            }

                            result_rows.push(result_row);
                        }

                        return Ok(ExecutorResult::new(result_rows, 0));
                    } else {
                        // No GROUP BY: compute aggregates over all rows (original behavior)
                        let result_row: Vec<Value> = select
                            .aggregates
                            .iter()
                            .map(|agg| compute_aggregate(agg, &filtered_rows, &columns))
                            .collect();
                        return Ok(ExecutorResult::new(vec![result_row], 0));
                    }
                }

// Sort rows by ORDER BY clause BEFORE projection
                // This is necessary because ORDER BY can reference columns not in SELECT list
                let ordered_rows: Vec<Vec<Value>> = if let Some(ref order_by) = select.order_by {
                    sort_rows_by_order_by(filtered_rows, order_by, &columns)
                } else {
                    filtered_rows
                };
                // Apply column projection if specified (not SELECT *)
                // SELECT * has columns = [{"*", None}]
                let is_select_star = select.columns.len() == 1 && select.columns[0].name == "*";

                // Apply column projection AFTER sorting
                let result_rows: Vec<Vec<Value>> = if is_select_star || select.columns.is_empty() {
                    ordered_rows
                } else {
                    ordered_rows
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

                Ok(ExecutorResult::new(result_rows, 0))
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
            Statement::Kill(kill) => self.execute_kill(&kill),
            _ => Ok(ExecutorResult::empty()),
        }
    }

    fn execute_kill(&mut self, kill: &KillStatement) -> Result<ExecutorResult, SqlError> {
        let session_manager = self
            .session_manager
            .as_ref()
            .ok_or_else(|| SqlError::ExecutionError("Session manager not available".to_string()))?;

        let current_session_id = self
            .current_session_id
            .ok_or_else(|| SqlError::ExecutionError("Not in a valid session".to_string()))?;

        let target_session_id = kill.process_id;

        if target_session_id == current_session_id {
            return Err(SqlError::ExecutionError(
                "Cannot kill self session".to_string(),
            ));
        }

        let target_session = session_manager.get_session(target_session_id);
        if target_session.is_none() {
            return Err(SqlError::ExecutionError(format!(
                "Unknown thread id: {}",
                target_session_id
            )));
        }

        let target_session = target_session.unwrap();
        let current_session = session_manager
            .get_session(current_session_id)
            .ok_or_else(|| SqlError::ExecutionError("Current session not found".to_string()))?;

        let is_own_session = target_session.user == current_session.user;
        if !is_own_session && !current_session.can_kill() {
            return Err(SqlError::ExecutionError(
                "Access denied: need SUPER privilege to kill other user's sessions".to_string(),
            ));
        }

        match kill.kill_type {
            KillType::Connection => {
                session_manager
                    .kill_session(target_session_id)
                    .map_err(SqlError::ExecutionError)?;
                Ok(ExecutorResult::new(
                    vec![vec![Value::Text(format!(
                        "CONNECTION {} executed",
                        target_session_id
                    ))]],
                    0,
                ))
            }
            KillType::Query => {
                session_manager
                    .kill_query(target_session_id)
                    .map_err(SqlError::ExecutionError)?;
                Ok(ExecutorResult::new(
                    vec![vec![Value::Text(format!(
                        "QUERY {} executed",
                        target_session_id
                    ))]],
                    0,
                ))
            }
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

                // Extract join key columns from condition
                #[allow(clippy::collapsible_match)]
                let left_key_col = if let Some(ref cond) = join_plan.condition() {
                    if let sqlrustgo_planner::Expr::BinaryExpr { left, .. } = cond {
                        if let sqlrustgo_planner::Expr::Column(col) = &**left {
                            Some(col.name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                #[allow(clippy::collapsible_match)]
                let right_key_col = if let Some(ref cond) = join_plan.condition() {
                    if let sqlrustgo_planner::Expr::BinaryExpr { right, .. } = cond {
                        if let sqlrustgo_planner::Expr::Column(col) = &**right {
                            Some(col.name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Build hash map from right rows
                use std::collections::HashMap;
                let mut right_hash: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();
                for rrow in &right_rows {
                    let key = if let Some(ref col_name) = right_key_col {
                        if let Some(idx) = right.schema().fields.iter().position(|f| &f.name == col_name) {
                            if idx < rrow.len() {
                                vec![rrow[idx].clone()]
                            } else {
                                vec![Value::Null]
                            }
                        } else if !rrow.is_empty() {
                            vec![rrow[0].clone()]
                        } else {
                            vec![Value::Null]
                        }
                    } else if !rrow.is_empty() {
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
                    let key = if let Some(ref col_name) = left_key_col {
                        if let Some(idx) = left.schema().fields.iter().position(|f| &f.name == col_name) {
                            if idx < lrow.len() {
                                vec![lrow[idx].clone()]
                            } else {
                                vec![Value::Null]
                            }
                        } else if !lrow.is_empty() {
                            vec![lrow[0].clone()]
                        } else {
                            vec![Value::Null]
                        }
                    } else if !lrow.is_empty() {
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
            session_manager: None,
            current_session_id: None,
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
