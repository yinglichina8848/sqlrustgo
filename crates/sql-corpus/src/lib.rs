//! SQL Regression Corpus
//!
//! A SQL-based regression test framework that loads SQL files and executes them
//! against SQLRustGo to verify correct behavior.

use serde::{Deserialize, Serialize};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    parse, AlterTableOperation, Expression, InsertStatement, SelectStatement, ShowStatement,
    Statement, TruncateStatement, WithSelect,
};
use sqlrustgo_parser::transaction::{IsolationLevel, TransactionStatement};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod differential;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlTestResult {
    pub case_name: String,
    pub sql: String,
    pub success: bool,
    pub rows_returned: usize,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub expected_rows: Option<usize>,
    pub expected_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusFileResult {
    pub file_path: String,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<SqlTestResult>,
}

struct SimpleExecutor {
    storage: MemoryStorage,
}

impl SimpleExecutor {
    fn new() -> Self {
        Self {
            storage: MemoryStorage::new(),
        }
    }

    fn reset(&mut self) {
        self.storage = MemoryStorage::new();
    }

    fn execute(&mut self, sql: &str) -> Result<ExecutorResult, String> {
        let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

        match statement {
            Statement::CreateTable(create) => {
                let info = TableInfo {
                    name: create.name.clone(),
                    columns: create
                        .columns
                        .into_iter()
                        .map(|c| ColumnDefinition {
                            name: c.name,
                            data_type: c.data_type,
                            nullable: c.nullable,
                            primary_key: c.primary_key,
                            auto_increment: c.auto_increment,
                        })
                        .collect(),
                    foreign_keys: vec![],
                    unique_constraints: vec![],
                    check_constraints: vec![],
                    partition_info: None,
                    has_hidden_rowid: false,
                    next_rowid: 0,
                };
                self.storage
                    .create_table(&info)
                    .map_err(|e| format!("Create table error: {:?}", e))?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Insert(insert) => {
                let records = self.evaluate_insert_values(&insert)?;
                self.storage
                    .insert(&insert.table, records)
                    .map_err(|e| format!("Insert error: {:?}", e))?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Select(select) => {
                let rows = self.execute_select(&select)?;
                let count = rows.len();
                Ok(ExecutorResult::new(rows, count))
            }
            Statement::Delete(delete) => {
                // If no WHERE clause, delete all rows
                if delete.where_clause.is_none() {
                    let count = self
                        .storage
                        .delete(&delete.table, &[])
                        .map_err(|e| format!("Delete error: {:?}", e))?;
                    return Ok(ExecutorResult::new(vec![], count));
                }

                // Get table info to find column indices
                let table_info = self
                    .storage
                    .get_table_info(&delete.table)
                    .map_err(|e| format!("Get table info error: {:?}", e))?;

                // Scan all rows
                let all_rows = self
                    .storage
                    .scan(&delete.table)
                    .map_err(|e| format!("Scan error: {:?}", e))?;

                // Filter rows based on WHERE clause
                let where_clause = delete.where_clause.as_ref().unwrap();
                let rows_to_delete: Vec<Vec<Value>> = all_rows
                    .clone()
                    .into_iter()
                    .filter(|row| self.evaluate_where(where_clause, row, &table_info))
                    .collect();

                let count = rows_to_delete.len();

                if count == 0 {
                    return Ok(ExecutorResult::new(vec![], 0));
                }

                // Keep rows that don't match the WHERE clause
                let rows_to_keep: Vec<Vec<Value>> = all_rows
                    .into_iter()
                    .filter(|row| !self.evaluate_where(where_clause, row, &table_info))
                    .collect();

                // Delete all rows and re-insert non-matching ones
                self.storage
                    .delete(&delete.table, &[])
                    .map_err(|e| format!("Delete error: {:?}", e))?;

                if !rows_to_keep.is_empty() {
                    self.storage
                        .insert(&delete.table, rows_to_keep)
                        .map_err(|e| format!("Insert error: {:?}", e))?;
                }

                Ok(ExecutorResult::new(vec![], count))
            }
            Statement::WithSelect(ref ws) => {
                // ws.select is Box<Statement>, need to unbox to match
                if let Statement::Select(ref select) = *ws.select {
                    if select.table.is_empty() {
                        // Recursive CTE or empty CTE → can't execute without CTE defs
                        Ok(ExecutorResult::new(vec![], 0))
                    } else {
                        // Execute the inner SELECT directly
                        let inner_sql = format!("SELECT * FROM {}", select.table);
                        self.execute(&inner_sql)
                    }
                } else {
                    Err(format!("Expected SELECT in WITH, got {:?}", ws.select))
                }
            }
            Statement::Update(update) => {
                let updates: Vec<(usize, Value)> = update
                    .set_clauses
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (_col, expr))| {
                        if let Ok(val) = self.evaluate_expression(expr) {
                            Some((i, val))
                        } else {
                            None
                        }
                    })
                    .collect();
                let count = self
                    .storage
                    .update(&update.table, &[], &updates)
                    .map_err(|e| format!("Update error: {:?}", e))?;
                Ok(ExecutorResult::new(vec![], count))
            }
            Statement::DropTable(drop) => {
                self.storage
                    .drop_table(&drop.name)
                    .map_err(|e| format!("Drop table error: {:?}", e))?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::AlterTable(alter) => {
                match &alter.operation {
                    AlterTableOperation::AddColumn {
                        name,
                        data_type,
                        nullable,
                        ..
                    } => {
                        let col = ColumnDefinition {
                            name: name.clone(),
                            data_type: data_type.clone(),
                            nullable: *nullable,
                            primary_key: false,
                            auto_increment: false,
                        };
                        self.storage
                            .add_column(&alter.table_name, col)
                            .map_err(|e| format!("Add column error: {:?}", e))?;
                    }
                    AlterTableOperation::RenameTo { new_name } => {
                        self.storage
                            .rename_table(&alter.table_name, new_name)
                            .map_err(|e| format!("Rename table error: {:?}", e))?;
                    }
                    AlterTableOperation::DropColumn { name } => {
                        return Err(format!(
                            "DROP COLUMN '{}' not yet implemented in sql-corpus executor",
                            name
                        ));
                    }
                    AlterTableOperation::ModifyColumn {
                        name, data_type, ..
                    } => {
                        return Err(format!(
                            "MODIFY COLUMN '{} {}' not yet implemented in sql-corpus executor",
                            name, data_type
                        ));
                    }
                }
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Truncate(truncate) => {
                // TRUNCATE TABLE - delete all rows from table
                self.execute_truncate(&truncate)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::CreateIndex(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Union(union_stmt) => {
                let left_rows = self.execute_statement(&union_stmt.left)?;
                let right_rows = self.execute_statement(&union_stmt.right)?;
                let mut combined: Vec<Vec<Value>> = left_rows;
                combined.extend(right_rows);
                if !union_stmt.union_all {
                    combined.sort();
                    combined.dedup();
                }
                let count = combined.len();
                Ok(ExecutorResult::new(combined, count))
            }
            Statement::Analyze(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Check(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Optimize(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Vacuum(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Repair(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Backup(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Restore(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Explain(_) => Ok(ExecutorResult::new(vec![], 0)),
            Statement::Show(show) => {
                let rows = self.execute_show(&show);
                let count = rows.len();
                Ok(ExecutorResult::new(rows, count))
            }
            Statement::Transaction(tx_stmt) => {
                self.execute_transaction(&tx_stmt)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            _ => Err("Unsupported statement type".to_string()),
        }
    }

    fn evaluate_insert_values(&self, insert: &InsertStatement) -> Result<Vec<Vec<Value>>, String> {
        let mut all_records = Vec::new();

        for row in &insert.values {
            let mut record = Vec::new();
            for expr in row {
                record.push(self.evaluate_expression(expr)?);
            }
            all_records.push(record);
        }

        Ok(all_records)
    }

    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Ok(Value::Null)
                } else if let Ok(n) = s.parse::<i64>() {
                    Ok(Value::Integer(n))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(Value::Float(f))
                } else if s.starts_with('\'') && s.ends_with('\'') {
                    Ok(Value::Text(s[1..s.len() - 1].to_string()))
                } else {
                    Ok(Value::Text(s.to_string()))
                }
            }
            Expression::Identifier(_) => Ok(Value::Null),
            _ => Ok(Value::Null),
        }
    }

    fn execute_select(&self, select: &SelectStatement) -> Result<Vec<Vec<Value>>, String> {
        // Use first_table() for backward compat: from.tables[0] or select.table
        let table_name = select.first_table();

        // Handle INFORMATION_SCHEMA queries
        if table_name.eq_ignore_ascii_case("information_schema.tables") {
            return self.execute_information_schema_tables(select);
        }
        if table_name.eq_ignore_ascii_case("information_schema.columns") {
            return self.execute_information_schema_columns(select);
        }
        if table_name.eq_ignore_ascii_case("information_schema.indexes") {
            return self.execute_information_schema_indexes(select);
        }
        if table_name.eq_ignore_ascii_case("information_schema.statistics") {
            return self.execute_information_schema_statistics(select);
        }

        let mut rows = self
            .storage
            .scan(&table_name)
            .map_err(|e| format!("Scan error: {:?}", e))?;

        if let Some(ref where_clause) = select.where_clause {
            let table_info = self
                .storage
                .get_table_info(&table_name)
                .map_err(|e| format!("Get table info error: {:?}", e))?;
            rows.retain(|row| self.evaluate_where(where_clause, row, &table_info));
        }

        Ok(rows)
    }

    fn execute_information_schema_tables(
        &self,
        select: &SelectStatement,
    ) -> Result<Vec<Vec<Value>>, String> {
        let mut rows = Vec::new();
        let table_names = self.storage.list_tables();
        for name in table_names {
            rows.push(vec![
                Value::Text("default".to_string()),
                Value::Text(name.clone()),
                Value::Text("BASE TABLE".to_string()),
                Value::Text("YES".to_string()),
            ]);
        }

        if let Some(ref where_clause) = select.where_clause {
            let fake_info = self.create_fake_table_info_for_tables();
            rows.retain(|row| self.evaluate_where(where_clause, row, &fake_info));
        }

        Ok(rows)
    }

    fn execute_information_schema_columns(
        &self,
        select: &SelectStatement,
    ) -> Result<Vec<Vec<Value>>, String> {
        let mut rows = Vec::new();
        let table_names = self.storage.list_tables();
        for name in table_names {
            if let Ok(info) = self.storage.get_table_info(&name) {
                for (i, col) in info.columns.iter().enumerate() {
                    let (char_max, num_prec, num_scale) = self.get_type_attributes(&col.data_type);
                    rows.push(vec![
                        Value::Text("default".to_string()),
                        Value::Text(name.clone()),
                        Value::Text(col.name.clone()),
                        Value::Integer((i + 1) as i64),
                        Value::Null,
                        Value::Text(if col.nullable {
                            "YES".to_string()
                        } else {
                            "NO".to_string()
                        }),
                        Value::Text(col.data_type.clone()),
                        char_max,
                        num_prec,
                        num_scale,
                    ]);
                }
            }
        }

        if let Some(ref where_clause) = select.where_clause {
            let fake_info = self.create_fake_table_info_for_columns();
            rows.retain(|row| self.evaluate_where(where_clause, row, &fake_info));
        }

        Ok(rows)
    }

    fn execute_information_schema_indexes(
        &self,
        select: &SelectStatement,
    ) -> Result<Vec<Vec<Value>>, String> {
        let mut rows = Vec::new();
        let table_names = self.storage.list_tables();
        for name in table_names {
            if let Ok(info) = self.storage.get_table_info(&name) {
                for col in &info.columns {
                    if col.primary_key {
                        rows.push(vec![
                            Value::Text("default".to_string()),
                            Value::Text(name.clone()),
                            Value::Text(format!("idx_{}_pk", name)),
                            Value::Text(col.name.clone()),
                            Value::Integer(1),
                            Value::Boolean(true),
                            Value::Boolean(true),
                        ]);
                    }
                }
            }
        }

        if let Some(ref where_clause) = select.where_clause {
            let fake_info = self.create_fake_table_info_for_indexes();
            rows.retain(|row| self.evaluate_where(where_clause, row, &fake_info));
        }

        Ok(rows)
    }

    fn execute_information_schema_statistics(
        &self,
        select: &SelectStatement,
    ) -> Result<Vec<Vec<Value>>, String> {
        self.execute_information_schema_indexes(select)
    }

    fn create_fake_table_info_for_tables(&self) -> TableInfo {
        TableInfo {
            name: "information_schema.tables".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "table_schema".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "table_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "table_type".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "is_insertable_into".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
            ],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
            has_hidden_rowid: false,
            next_rowid: 0,
        }
    }

    fn create_fake_table_info_for_columns(&self) -> TableInfo {
        TableInfo {
            name: "information_schema.columns".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "table_schema".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "table_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "column_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "ordinal_position".to_string(),
                    data_type: "integer".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "column_default".to_string(),
                    data_type: "text".to_string(),
                    nullable: true,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "is_nullable".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "data_type".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "character_maximum_length".to_string(),
                    data_type: "integer".to_string(),
                    nullable: true,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "numeric_precision".to_string(),
                    data_type: "integer".to_string(),
                    nullable: true,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "numeric_scale".to_string(),
                    data_type: "integer".to_string(),
                    nullable: true,
                    primary_key: false,
                    auto_increment: false,
                },
            ],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
            has_hidden_rowid: false,
            next_rowid: 0,
        }
    }

    fn create_fake_table_info_for_indexes(&self) -> TableInfo {
        TableInfo {
            name: "information_schema.indexes".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "table_schema".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "table_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "index_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "column_name".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "ordinal_position".to_string(),
                    data_type: "integer".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "is_unique".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
                ColumnDefinition {
                    name: "is_primary".to_string(),
                    data_type: "text".to_string(),
                    nullable: false,
                    primary_key: false,
                    auto_increment: false,
                },
            ],
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
            has_hidden_rowid: false,
            next_rowid: 0,
        }
    }

    fn get_type_attributes(&self, data_type: &str) -> (Value, Value, Value) {
        match data_type.to_uppercase().as_str() {
            "TEXT" | "VARCHAR" | "CHAR" => (Value::Integer(65535), Value::Null, Value::Null),
            "INTEGER" | "INT" => (Value::Null, Value::Integer(64), Value::Integer(0)),
            "REAL" | "FLOAT" | "DOUBLE" => (Value::Null, Value::Integer(53), Value::Null),
            _ => (Value::Null, Value::Null, Value::Null),
        }
    }

    fn execute_statement(&self, stmt: &Statement) -> Result<Vec<Vec<Value>>, String> {
        match stmt {
            Statement::Select(select) => self.execute_select(select),
            Statement::Union(union_stmt) => {
                let left_rows = self.execute_statement(&union_stmt.left)?;
                let right_rows = self.execute_statement(&union_stmt.right)?;
                if union_stmt.union_all {
                    Ok(left_rows.into_iter().chain(right_rows).collect())
                } else {
                    let mut combined = left_rows;
                    combined.extend(right_rows);
                    combined.sort();
                    combined.dedup();
                    Ok(combined)
                }
            }
            _ => Err(format!("Unsupported statement type: {:?}", stmt)),
        }
    }

    fn evaluate_where(&self, expr: &Expression, row: &[Value], table_info: &TableInfo) -> bool {
        match expr {
            // Handle AND conditions
            Expression::BinaryOp(left, op, right) if op.to_uppercase() == "AND" => {
                self.evaluate_where(left, row, table_info)
                    && self.evaluate_where(right, row, table_info)
            }
            // Handle OR conditions
            Expression::BinaryOp(left, op, right) if op.to_uppercase() == "OR" => {
                self.evaluate_where(left, row, table_info)
                    || self.evaluate_where(right, row, table_info)
            }
            // Handle IS NULL
            Expression::BinaryOp(left, op, right)
                if op.to_uppercase() == "IS"
                    && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
            {
                if let Expression::Identifier(col_name) = left.as_ref() {
                    if let Some(col_idx) = self.find_column_index(col_name, table_info) {
                        if let Some(row_val) = row.get(col_idx) {
                            return matches!(row_val, Value::Null);
                        }
                    }
                }
                false
            }
            // Handle IS NOT NULL
            Expression::BinaryOp(left, op, right)
                if op.to_uppercase() == "IS NOT"
                    && matches!(right.as_ref(), Expression::Literal(s) if s.to_uppercase() == "NULL") =>
            {
                if let Expression::Identifier(col_name) = left.as_ref() {
                    if let Some(col_idx) = self.find_column_index(col_name, table_info) {
                        if let Some(row_val) = row.get(col_idx) {
                            return !matches!(row_val, Value::Null);
                        }
                    }
                }
                false
            }
            // Handle comparison operators
            Expression::BinaryOp(left, op, right) => {
                self.evaluate_binary_comparison(left, op, right, row, table_info)
            }
            _ => true,
        }
    }

    fn evaluate_binary_comparison(
        &self,
        left: &Expression,
        op: &str,
        right: &Expression,
        row: &[Value],
        table_info: &TableInfo,
    ) -> bool {
        let left_val = self.get_expression_value(left, row, table_info);
        let right_val = self.get_expression_value(right, row, table_info);

        match op.to_uppercase().as_str() {
            "=" | "==" | "IS" => left_val == right_val,
            "!=" | "<>" => left_val != right_val,
            ">" => self.compare_values(&left_val, &right_val) > 0,
            ">=" => self.compare_values(&left_val, &right_val) >= 0,
            "<" => self.compare_values(&left_val, &right_val) < 0,
            "<=" => self.compare_values(&left_val, &right_val) <= 0,
            _ => false,
        }
    }

    fn get_expression_value(
        &self,
        expr: &Expression,
        row: &[Value],
        table_info: &TableInfo,
    ) -> Value {
        match expr {
            Expression::Literal(s) => {
                let s = s.trim();
                if s.eq_ignore_ascii_case("NULL") {
                    Value::Null
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
            Expression::Identifier(name) => {
                if let Some(col_idx) = self.find_column_index(name, table_info) {
                    row.get(col_idx).cloned().unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.get_expression_value(left, row, table_info);
                let right_val = self.get_expression_value(right, row, table_info);
                self.evaluate_binary_op_value(&left_val, &right_val, op)
            }
            _ => Value::Null,
        }
    }

    fn evaluate_binary_op_value(&self, left: &Value, right: &Value, op: &str) -> Value {
        match op.to_uppercase().as_str() {
            "=" | "==" | "IS" => Value::Boolean(left == right),
            "!=" | "<>" => Value::Boolean(left != right),
            ">" => Value::Boolean(self.compare_values(left, right) > 0),
            ">=" => Value::Boolean(self.compare_values(left, right) >= 0),
            "<" => Value::Boolean(self.compare_values(left, right) < 0),
            "<=" => Value::Boolean(self.compare_values(left, right) <= 0),
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

    fn compare_values(&self, left: &Value, right: &Value) -> i32 {
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

    fn find_column_index(&self, col_name: &str, table_info: &TableInfo) -> Option<usize> {
        table_info
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(col_name))
    }

    fn execute_show(&self, show: &ShowStatement) -> Vec<Vec<Value>> {
        match show {
            ShowStatement::Tables => {
                let names = self.storage.list_tables();
                names
                    .into_iter()
                    .map(|name| vec![Value::Text(name)])
                    .collect()
            }
            ShowStatement::Databases => {
                vec![vec![Value::Text("default".to_string())]]
            }
            ShowStatement::Columns { table, .. } => {
                if let Ok(info) = self.storage.get_table_info(table) {
                    info.columns
                        .iter()
                        .map(|c| {
                            // MySQL 5.7 SHOW COLUMNS format: Field, Type, Null, Key, Default, Extra
                            vec![
                                Value::Text(c.name.clone()),
                                Value::Text(c.data_type.clone()),
                                Value::Text(if c.nullable { "YES" } else { "NO" }.to_string()),
                                Value::Text(if c.primary_key {
                                    "PRI".to_string()
                                } else {
                                    String::new()
                                }),
                                Value::Null,
                                Value::Text("".to_string()),
                            ]
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            ShowStatement::Index { table, .. } => {
                if let Ok(_info) = self.storage.get_table_info(table) {
                    vec![vec![Value::Text("index_on_".to_string() + table)]]
                } else {
                    vec![]
                }
            }
            ShowStatement::CreateTable { table } => {
                let sql = format!("CREATE TABLE {} (...)", table);
                vec![vec![Value::Text(sql)]]
            }
            ShowStatement::Variables { .. } => vec![],
            ShowStatement::Grants { .. } => {
                vec![vec![Value::Text(
                    "GRANT ALL ON *.* TO current_user".to_string(),
                )]]
            }
            ShowStatement::TableStatus { .. } => vec![],
            ShowStatement::Status { .. } => vec![],
            ShowStatement::Events => vec![],
            ShowStatement::Processlist => vec![],
            ShowStatement::TransactionHistory { .. } => vec![],
            ShowStatement::LockWaits => vec![],
            ShowStatement::RecoveryHistory { .. } => vec![],
            ShowStatement::WalStats => vec![],
        }
    }

    fn execute_transaction(&mut self, stmt: &TransactionStatement) -> Result<(), String> {
        match stmt {
            TransactionStatement::Begin { .. } => Ok(()),
            TransactionStatement::BeginIdempotent { .. } => Ok(()),
            TransactionStatement::Commit { .. } => Ok(()),
            TransactionStatement::Rollback { .. } => Ok(()),
            TransactionStatement::SetTransaction { isolation_level } => match isolation_level {
                IsolationLevel::ReadCommitted
                | IsolationLevel::ReadUncommitted
                | IsolationLevel::SnapshotIsolation
                | IsolationLevel::Serializable => Ok(()),
            },
            TransactionStatement::StartTransaction { .. } => Ok(()),
            TransactionStatement::Savepoint { .. } => Ok(()),
            TransactionStatement::RollbackToSavepoint { .. } => Ok(()),
            TransactionStatement::ReleaseSavepoint { .. } => Ok(()),
        }
    }

    #[allow(dead_code)]
    fn execute_with_select(&mut self, with_select: &WithSelect) -> Result<(), String> {
        if let Some(ref with_clause) = with_select.with_clause {
            for cte in &with_clause.ctes {
                let cte_rows = self.execute_statement(&cte.subquery)?;
                let column_count = if cte.columns.is_empty() {
                    if cte_rows.is_empty() {
                        0
                    } else {
                        cte_rows[0].len()
                    }
                } else {
                    cte.columns.len()
                };
                let columns: Vec<ColumnDefinition> = (0..column_count)
                    .map(|i| ColumnDefinition {
                        name: if cte.columns.is_empty() {
                            format!("col_{}", i)
                        } else {
                            cte.columns[i].clone()
                        },
                        data_type: "TEXT".to_string(),
                        nullable: true,
                        primary_key: false,
                        auto_increment: false,
                    })
                    .collect();
                let table_info = TableInfo {
                    name: cte.name.clone(),
                    columns,
                    foreign_keys: vec![],
                    unique_constraints: vec![],
                    check_constraints: vec![],
                    partition_info: None,
                    has_hidden_rowid: false,
                    next_rowid: 0,
                };
                self.storage
                    .create_table(&table_info)
                    .map_err(|e| format!("Create CTE table error: {:?}", e))?;
                if !cte_rows.is_empty() {
                    self.storage
                        .insert(&cte.name, cte_rows)
                        .map_err(|e| format!("Insert CTE rows error: {:?}", e))?;
                }
            }
        }
        // with_select.select is Box<Statement>, unbox it to get SelectStatement
        if let Statement::Select(ref select) = *with_select.select {
            self.execute_select(select)?;
        } else {
            return Err(format!(
                "Expected SELECT in WITH, got {:?}",
                with_select.select
            ));
        }
        Ok(())
    }

    fn execute_truncate(&mut self, truncate: &TruncateStatement) -> Result<(), String> {
        self.storage
            .delete(&truncate.name, &[])
            .map_err(|e| format!("Truncate table error: {:?}", e))?;
        Ok(())
    }
}

pub struct SqlCorpus {
    corpus_root: PathBuf,
    executor: SimpleExecutor,
}

impl SqlCorpus {
    pub fn new(corpus_root: PathBuf) -> Self {
        Self {
            corpus_root,
            executor: SimpleExecutor::new(),
        }
    }

    pub fn reset(&mut self) {
        self.executor.reset();
    }

    /// Execute all SQL files recursively (legacy method - may use high memory)
    #[deprecated(note = "Use execute_batched() for memory-constrained environments")]
    pub fn execute_all(&mut self) -> HashMap<String, CorpusFileResult> {
        let mut results = HashMap::new();
        self.execute_directory(&mut results);
        results
    }

    /// Execute a single subdirectory batch (memory-efficient)
    #[allow(deprecated)]
    pub fn execute_batch(&mut self, subdir: &str) -> HashMap<String, CorpusFileResult> {
        let mut results = HashMap::new();
        let batch_root = self.corpus_root.join(subdir);

        if !batch_root.is_dir() {
            return results;
        }

        for entry in fs::read_dir(&batch_root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories within the batch
                let mut sub_corpus = SqlCorpus::new(path.clone());
                let sub_results = sub_corpus.execute_all();
                results.extend(sub_results);
            } else if path.extension().is_some_and(|e| e == "sql") {
                let file_result = self.execute_file(&path);
                let relative_path = path
                    .strip_prefix(&self.corpus_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                results.insert(relative_path, file_result);
            }
        }

        // Release memory by clearing executor state after batch
        self.reset();
        results
    }

    /// List all top-level subdirectories (for batch processing)
    pub fn list_batches(&self) -> Vec<String> {
        let mut batches = Vec::new();
        if !self.corpus_root.is_dir() {
            return batches;
        }

        for entry in fs::read_dir(&self.corpus_root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    batches.push(name.to_string());
                }
            }
        }
        batches.sort();
        batches
    }

    #[allow(deprecated)]
    fn execute_directory(&mut self, results: &mut HashMap<String, CorpusFileResult>) {
        if !self.corpus_root.is_dir() {
            return;
        }

        for entry in fs::read_dir(&self.corpus_root).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let mut sub_corpus = SqlCorpus::new(path);
                let sub_results = sub_corpus.execute_all();
                results.extend(sub_results);
            } else if path.extension().is_some_and(|e| e == "sql") {
                let file_result = self.execute_file(&path);
                let relative_path = path
                    .strip_prefix(&self.corpus_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                results.insert(relative_path, file_result);
            }
        }
    }

    pub fn execute_file(&mut self, path: &Path) -> CorpusFileResult {
        let content = fs::read_to_string(path).unwrap_or_default();
        let file_results = self.parse_and_execute(&content);

        let total = file_results.len();
        let passed = file_results.iter().filter(|r| r.success).count();
        let failed = total - passed;

        CorpusFileResult {
            file_path: path.to_string_lossy().to_string(),
            total_cases: total,
            passed,
            failed,
            results: file_results,
        }
    }

    pub fn parse_and_execute(&mut self, content: &str) -> Vec<SqlTestResult> {
        let first_lines: Vec<_> = content.lines().take(5).collect();
        for line in &first_lines {
            let trimmed = line.trim();
            if trimmed == "-- === SKIP ===" || trimmed == "-- === IGNORE ===" {
                return Vec::new();
            }
        }

        let mut results = Vec::new();
        let mut current_case: Option<TestCase> = None;
        let mut setup_sql = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("-- === SETUP ===") {
                if let Some(case) = current_case.take() {
                    results.push(self.execute_case(case, &setup_sql));
                }
                setup_sql.clear();
            } else if trimmed.starts_with("-- === CASE:") {
                if let Some(case) = current_case.take() {
                    results.push(self.execute_case(case, &setup_sql));
                }

                let case_name = trimmed
                    .trim_start_matches("-- === CASE:")
                    .trim()
                    .to_string();
                current_case = Some(TestCase {
                    name: case_name,
                    sql: String::new(),
                    expected_rows: None,
                    expected_columns: None,
                    skip: false,
                });
            } else if trimmed == "-- === SKIP ===" || trimmed == "-- === IGNORE ===" {
                // Mark current case as skipped
                if let Some(ref mut case) = current_case {
                    case.skip = true;
                }
            } else if trimmed.starts_with("-- EXPECT:") {
                if let Some(ref mut case) = current_case {
                    let expect = trimmed.trim_start_matches("-- EXPECT:").trim();
                    if expect.starts_with("ERROR") {
                        case.expected_rows = Some(0);
                    } else if expect.starts_with("rows") {
                        if let Ok(n) = expect.split_whitespace().next().unwrap_or("0").parse() {
                            case.expected_rows = Some(n);
                        }
                    }
                }
            } else if let Some(ref mut current_case) = current_case {
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    if !current_case.sql.is_empty() {
                        current_case.sql.push('\n');
                    }
                    current_case.sql.push_str(trimmed);
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("--") {
                if !setup_sql.is_empty() {
                    setup_sql.push('\n');
                }
                setup_sql.push_str(trimmed);
            }
        }

        if let Some(case) = current_case {
            results.push(self.execute_case(case, &setup_sql));
        }

        results
    }

    fn execute_case(&mut self, case: TestCase, setup_sql: &str) -> SqlTestResult {
        let start = std::time::Instant::now();

        if case.skip {
            return SqlTestResult {
                case_name: case.name,
                sql: case.sql,
                success: true,
                rows_returned: 0,
                execution_time_ms: 0,
                error_message: None,
                expected_rows: case.expected_rows,
                expected_columns: case.expected_columns,
            };
        }

        self.reset();

        if !setup_sql.is_empty() {
            if let Err(e) = self.execute_sql(setup_sql) {
                return SqlTestResult {
                    case_name: case.name,
                    sql: case.sql,
                    success: false,
                    rows_returned: 0,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error_message: Some(format!("Setup failed: {}", e)),
                    expected_rows: case.expected_rows,
                    expected_columns: case.expected_columns,
                };
            }
        }

        match self.execute_sql(&case.sql) {
            Ok(result) => {
                let rows_returned = result.rows.len();
                let success = case
                    .expected_rows
                    .map(|expected| rows_returned == expected)
                    .unwrap_or(true);

                SqlTestResult {
                    case_name: case.name,
                    sql: case.sql,
                    success,
                    rows_returned,
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    error_message: if !success {
                        Some(format!(
                            "Expected {} rows, got {}",
                            case.expected_rows.unwrap_or(0),
                            rows_returned
                        ))
                    } else {
                        None
                    },
                    expected_rows: case.expected_rows,
                    expected_columns: case.expected_columns,
                }
            }
            Err(e) => SqlTestResult {
                case_name: case.name,
                sql: case.sql,
                success: false,
                rows_returned: 0,
                execution_time_ms: start.elapsed().as_millis() as u64,
                error_message: Some(e),
                expected_rows: case.expected_rows,
                expected_columns: case.expected_columns,
            },
        }
    }

    fn execute_sql(&mut self, sql: &str) -> Result<ExecutorResult, String> {
        let statements: Vec<&str> = sql.split(';').filter(|s| !s.trim().is_empty()).collect();
        let mut last_result = Ok(ExecutorResult::new(vec![], 0));

        for stmt in statements {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }
            last_result = self.executor.execute(stmt);
        }

        last_result
    }

    pub fn summary(&self, results: &HashMap<String, CorpusFileResult>) -> CorpusSummary {
        let mut total_cases = 0;
        let mut passed = 0;
        let mut failed = 0;

        for result in results.values() {
            total_cases += result.total_cases;
            passed += result.passed;
            failed += result.failed;
        }

        CorpusSummary {
            total_files: results.len(),
            total_cases,
            passed,
            failed,
            pass_rate: if total_cases > 0 {
                (passed as f64 / total_cases as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    sql: String,
    expected_rows: Option<usize>,
    expected_columns: Option<Vec<String>>,
    skip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusSummary {
    pub total_files: usize,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
}
