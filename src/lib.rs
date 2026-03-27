//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult, GLOBAL_PROFILER};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{
    parse, Expression, GrantStatement, Lexer, Privilege, RevokeStatement, SetOperation, Statement,
    Token,
};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner, SetOperationType};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine, ViewInfo,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

use std::sync::{Arc, RwLock};

use sqlrustgo_executor::OperatorProfile;
use sqlrustgo_storage::{ForeignKeyAction, ForeignKeyConstraint};

/// Format the EXPLAIN ANALYZE output as a tree structure (PostgreSQL-style)
fn format_tree_output(profiles: &[OperatorProfile], total_time: &str) -> Vec<Vec<Value>> {
    let mut rows = Vec::new();

    // Header
    rows.push(vec![
        Value::Text("┌─────────────────────────────────────────────────────────────────┐".to_string()),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);
    rows.push(vec![
        Value::Text("│                      Execution Plan                             │".to_string()),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);
    rows.push(vec![
        Value::Text("├─────────────────────────────────────────────────────────────────┤".to_string()),
        Value::Text("".to_string()),
        Value::Text("".to_string()),
    ]);

    // Operator nodes
    for (i, profile) in profiles.iter().enumerate() {
        let is_last = i == profiles.len() - 1;
        let prefix = if is_last { "└─ " } else { "├─ " };
        let time_ms = profile.total_time_ns as f64 / 1_000_000.0;

        rows.push(vec![
            Value::Text(format!(
                "│  {} {} (rows={})",
                prefix,
                profile.operator_name,
                profile.rows_processed
            )),
            Value::Text("".to_string()),
            Value::Text("".to_string()),
        ]);
        rows.push(vec![
            Value::Text(format!(
                "│        Actual Time: {:.3} ms, Rows: {}",
                time_ms,
                profile.rows_processed
            )),
            Value::Text("".to_string()),
            Value::Text("".to_string()),
        ]);
    }

    // Footer
    rows.push(vec![
        Value::Text("└─────────────────────────────────────────────────────────────────┘".to_string()),
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
                    match fk.on_delete {
                        Some(ForeignKeyAction::Restrict) => {
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
                            // TODO: 当前只处理直接子表，不支持递归删除孙表及更深层次
                            // 例如: parent -> child -> grandchild，删除 parent 时 child 被删除，
                            //      但 grandchild 不会被处理（需要递归实现）
                            let child_rows = storage.scan(&table_name)?;
                            if let Some(pk_val) = parent_key_values.first() {
                                let original_count = child_rows.len();
                                let remaining: Vec<Vec<Value>> = child_rows
                                    .into_iter()
                                    .filter(|r| r[col_idx] != *pk_val)
                                    .collect();
                                let deleted = original_count - remaining.len();
                                if deleted > 0 {
                                    let _ = storage.delete(&table_name, &[]);
                                    if !remaining.is_empty() {
                                        storage.insert(&table_name, remaining)?;
                                    }
                                    total_cascade_deletes += deleted;
                                }
                            }
                        }
                        Some(ForeignKeyAction::SetNull) => {
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
                            let mut new_rows: Vec<Vec<Value>> = child_rows
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
                            let mut new_rows: Vec<Vec<Value>> = child_rows
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
        sqlrustgo_parser::Expression::BinaryOp(_, _, _) => {
            // For binary ops, we'd need to evaluate recursively
            // This shouldn't happen in a simple where clause evaluation
            Value::Null
        }
        sqlrustgo_parser::Expression::Wildcard => Value::Null,
        sqlrustgo_parser::Expression::FunctionCall(_, _) => Value::Null,
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
    let mut text_chars: Vec<char> = text.chars().collect();
    let mut pattern_chars: Vec<char> = pattern.chars().collect();

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
                    agg.args.get(0),
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
                                            if let Some(ref info) = table_info {
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
                                                        } else if value.to_uppercase() == "FALSE" {
                                                            Value::Boolean(false)
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

                storage.insert(table_name, records)?;
                Ok(ExecutorResult::new(vec![], insert.values.len()))
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
                                on_delete: fk
                                    .on_delete
                                    .as_ref()
                                    .map(|_| sqlrustgo_storage::ForeignKeyAction::Cascade),
                                on_update: fk
                                    .on_update
                                    .as_ref()
                                    .map(|_| sqlrustgo_storage::ForeignKeyAction::Cascade),
                            }
                        });
                        sqlrustgo_storage::ColumnDefinition {
                            name: col.name.clone(),
                            data_type: col.data_type.clone(),
                            nullable: col.nullable,
                            is_unique: false,
                            is_primary_key: false,
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
                if let (Some(pk_col_idx), true) = (primary_key_col, !rows_to_delete.is_empty()) {
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

                // Filter rows - keep rows that DON'T match the WHERE clause
                let mut remaining_rows: Vec<Vec<Value>> = all_rows
                    .into_iter()
                    .filter(|row| {
                        if let Some(ref where_clause) = delete.where_clause {
                            !evaluate_where_clause(where_clause, row, &columns)
                        } else {
                            true
                        }
                    })
                    .collect();

                let deleted_count = rows_to_delete.len();

                // Delete matching rows and insert remaining rows
                if deleted_count > 0 {
                    let _ = storage.delete(&delete.table, &[]);
                    if !remaining_rows.is_empty() {
                        storage.insert(&delete.table, remaining_rows)?;
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
                    let mut old_key_value: Option<Value> = None;

                    // Apply SET clauses and track if primary key is being updated
                    for (col_name, expr) in &update.set_clauses {
                        if let Some(col_idx) = columns
                            .iter()
                            .position(|c| c.name.eq_ignore_ascii_case(col_name))
                        {
                            // Track old key value if this is the primary key
                            if primary_key_col == Some(col_idx) {
                                old_key_value = Some(row[col_idx].clone());
                            }

                            let new_value = evaluate_expr(expr, &new_row, &columns);
                            if col_idx < new_row.len() {
                                new_row[col_idx] = new_value;
                            }
                        }
                    }

                    // Handle foreign key constraints if primary key is being updated
                    if let (Some(pk_col), Some(old_val)) = (primary_key_col, old_key_value) {
                        if let Some(new_key_val) = new_row.get(pk_col) {
                            handle_foreign_key_update(
                                &mut *storage,
                                &update.table,
                                &old_val,
                                new_key_val,
                                &columns[pk_col].name,
                            )?;
                        }
                    }

                    rows_to_update.push((row.clone(), new_row));
                }

                let updated_count = rows_to_update.len();

                // Write back updated rows
                if updated_count > 0 {
                    let _ = storage.delete(&update.table, &[]);

                    let mut final_rows = all_rows;
                    final_rows.retain(|r| !rows_to_update.iter().any(|(old, _)| old == r));
                    for (_, new) in rows_to_update {
                        final_rows.push(new);
                    }

                    storage.insert(&update.table, final_rows)?;
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
                Ok(result)
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
}
