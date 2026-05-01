//! SQL Regression Corpus
//!
//! A SQL-based regression test framework that loads SQL files and executes them
//! against SQLRustGo to verify correct behavior.

use serde::{Deserialize, Serialize};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_parser::parser::{
    parse, AlterTableOperation, Expression, InsertStatement, SelectStatement, Statement,
};
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
                        })
                        .collect(),
                    foreign_keys: vec![],
                    unique_constraints: vec![],
                    check_constraints: vec![],
                    partition_info: None,
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
                }
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::CreateIndex(_) => Ok(ExecutorResult::new(vec![], 0)),
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
        let mut rows = self
            .storage
            .scan(&select.table)
            .map_err(|e| format!("Scan error: {:?}", e))?;

        if let Some(ref where_clause) = select.where_clause {
            let table_info = self
                .storage
                .get_table_info(&select.table)
                .map_err(|e| format!("Get table info error: {:?}", e))?;
            rows.retain(|row| self.evaluate_where(where_clause, row, &table_info));
        }

        Ok(rows)
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

    pub fn execute_all(&mut self) -> HashMap<String, CorpusFileResult> {
        let mut results = HashMap::new();
        self.execute_directory(&mut results);
        results
    }

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
                });
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
            } else if current_case.is_some() {
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    if !current_case.as_ref().unwrap().sql.is_empty() {
                        current_case.as_mut().unwrap().sql.push('\n');
                    }
                    current_case.as_mut().unwrap().sql.push_str(trimmed);
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusSummary {
    pub total_files: usize,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
}
