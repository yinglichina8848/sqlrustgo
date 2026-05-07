//! Differential Testing Framework
//!
//! Compares SQL execution results between SQLRustGo and MySQL 5.7

use serde::{Deserialize, Serialize};
use sqlrustgo_executor::ExecutorResult;
use sqlrustgo_types::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Result from a SQL engine execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResult {
    pub success: bool,
    pub rows: Vec<Vec<Value>>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Differential test result comparing two engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub case_name: String,
    pub sql: String,
    pub status: DiffStatus,
    pub sqlrustgo_result: Option<EngineResult>,
    pub mysql_result: Option<EngineResult>,
    pub mismatch_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffStatus {
    /// Both engines returned matching results
    Match,
    /// Both engines failed (possibly same error)
    BothError,
    /// Results differ between engines
   Mismatch,
    /// One engine succeeded, other failed
    OneFailed,
    /// Test skipped
    Skipped,
}

/// SQL Engine trait - abstract interface for SQL engines
pub trait SqlEngine: Send + Sync {
    /// Execute SQL and return results
    fn execute(&self, sql: &str) -> EngineResult;

    /// Check if engine is available
    fn is_available(&self) -> bool;

    /// Get engine name
    fn name(&self) -> &str;
}

/// SQLRustGo Engine Runner
pub struct SqlRustGoRunner {
    executor: std::sync::Mutex<crate::SimpleExecutor>,
}

impl SqlRustGoRunner {
    pub fn new() -> Self {
        Self {
            executor: std::sync::Mutex::new(crate::SimpleExecutor::new()),
        }
    }

    pub fn reset(&self) {
        *self.executor.lock().unwrap() = crate::SimpleExecutor::new();
    }
}

impl Default for SqlRustGoRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlEngine for SqlRustGoRunner {
    fn execute(&self, sql: &str) -> EngineResult {
        let start = std::time::Instant::now();
        match self.executor.lock().unwrap().execute(sql) {
            Ok(result) => EngineResult {
                success: true,
                rows: result.rows,
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => EngineResult {
                success: false,
                rows: vec![],
                error: Some(e),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "SQLRustGo"
    }
}

/// MySQL 5.7 Engine Runner
/// Note: Requires MySQL 5.7 to be installed and accessible
pub struct MySqlRunner {
    connection_string: String,
}

impl MySqlRunner {
    pub fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
        }
    }
}

impl SqlEngine for MySqlRunner {
    fn execute(&self, sql: &str) -> EngineResult {
        // MySQL execution not implemented yet
        // Would use mysql crate or subprocess call to mysql CLI
        EngineResult {
            success: false,
            rows: vec![],
            error: Some("MySQL runner not yet implemented".to_string()),
            execution_time_ms: 0,
        }
    }

    fn is_available(&self) -> bool {
        // Check if mysql client is available
        std::process::Command::new("mysql")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn name(&self) -> &str {
        "MySQL 5.7"
    }
}

/// Result Comparator - compares results from two engines
pub struct ResultComparator;

impl ResultComparator {
    /// Compare two engine results and determine if they match
    pub fn compare(
        sqlrustgo: &EngineResult,
        mysql: &EngineResult,
        sql: &str,
        case_name: &str,
    ) -> DiffResult {
        // If both failed, consider it a match (same error behavior)
        if !sqlrustgo.success && !mysql.success {
            return DiffResult {
                case_name: case_name.to_string(),
                sql: sql.to_string(),
                status: DiffStatus::BothError,
                sqlrustgo_result: Some(sqlrustgo.clone()),
                mysql_result: Some(mysql.clone()),
                mismatch_reason: None,
            };
        }

        // If one succeeded and one failed
        if sqlrustgo.success != mysql.success {
            return DiffResult {
                case_name: case_name.to_string(),
                sql: sql.to_string(),
                status: DiffStatus::OneFailed,
                sqlrustgo_result: Some(sqlrustgo.clone()),
                mysql_result: Some(mysql.clone()),
                mismatch_reason: Some(format!(
                    "SQLRustGo: {}, MySQL: {}",
                    if sqlrustgo.success {
                        "success".to_string()
                    } else {
                        sqlrustgo.error.clone().unwrap_or_default()
                    },
                    if mysql.success {
                        "success".to_string()
                    } else {
                        mysql.error.clone().unwrap_or_default()
                    }
                )),
            };
        }

        // Both succeeded - compare rows
        let sorted_sqlrustgo = Self::sort_rows(&sqlrustgo.rows);
        let sorted_mysql = Self::sort_rows(&mysql.rows);

        if Self::rows_match(&sorted_sqlrustgo, &sorted_mysql) {
            DiffResult {
                case_name: case_name.to_string(),
                sql: sql.to_string(),
                status: DiffStatus::Match,
                sqlrustgo_result: Some(sqlrustgo.clone()),
                mysql_result: Some(mysql.clone()),
                mismatch_reason: None,
            }
        } else {
            DiffResult {
                case_name: case_name.to_string(),
                sql: sql.to_string(),
                status: DiffStatus::Mismatch,
                sqlrustgo_result: Some(sqlrustgo.clone()),
                mysql_result: Some(mysql.clone()),
                mismatch_reason: Some(Self::format_mismatch(&sorted_sqlrustgo, &sorted_mysql)),
            }
        }
    }

    /// Sort rows for comparison (handles ORDER BY differences)
    fn sort_rows(rows: &[Vec<Value>]) -> Vec<Vec<Value>> {
        let mut sorted = rows.to_vec();
        sorted.sort_by(|a, b| {
            let a_strs: Vec<String> = a.iter().map(Self::value_to_string).collect();
            let b_strs: Vec<String> = b.iter().map(Self::value_to_string).collect();
            a_strs.cmp(&b_strs)
        });
        sorted
    }

    fn value_to_string(v: &Value) -> String {
        match v {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Blob(b) => format!("BLOB({})", b.len()),
        }
    }

    /// Check if two row sets match (with NULL handling)
    fn rows_match(a: &[Vec<Value>], b: &[Vec<Value>]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for (row_a, row_b) in a.iter().zip(b.iter()) {
            if row_a.len() != row_b.len() {
                return false;
            }
            for (val_a, val_b) in row_a.iter().zip(row_b.iter()) {
                if !Self::values_match(val_a, val_b) {
                    return false;
                }
            }
        }
        true
    }

    /// Compare two values with proper NULL handling
    fn values_match(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Null, _) | (_, Value::Null) => false,
            (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
            (Value::Float(f1), Value::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
            (Value::Text(s1), Value::Text(s2)) => s1 == s2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Blob(a), Value::Blob(b)) => a == b,
            _ => false,
        }
    }

    fn format_mismatch(a: &[Vec<Value>], b: &[Vec<Value>]) -> String {
        if a.len() != b.len() {
            return format!("Row count mismatch: SQLRustGo={}, MySQL={}", a.len(), b.len());
        }
        format!(
            "Row content mismatch:\n  SQLRustGo: {:?}\n  MySQL: {:?}",
            a.first(),
            b.first()
        )
    }
}

/// Differential Corpus - runs differential tests
pub struct DifferentialCorpus<E1: SqlEngine, E2: SqlEngine> {
    sqlrustgo: E1,
    mysql: E2,
}

impl<E1: SqlEngine, E2: SqlEngine> DifferentialCorpus<E1, E2> {
    pub fn new(sqlrustgo: E1, mysql: E2) -> Self {
        Self { sqlrustgo, mysql }
    }
}

impl DifferentialCorpus<SqlRustGoRunner, MySqlRunner> {
    /// Create a new differential corpus with SQLRustGo and MySQL
    pub fn with_engines(
        sqlrustgo: SqlRustGoRunner,
        mysql: MySqlRunner,
    ) -> Self {
        Self::new(sqlrustgo, mysql)
    }
}

impl<E1: SqlEngine + 'static, E2: SqlEngine + 'static> DifferentialCorpus<E1, E2> {
    /// Execute differential test on a single SQL case
    pub fn execute_case(&self, case_name: &str, sql: &str) -> DiffResult {
        let sqlrustgo_result = self.sqlrustgo.execute(sql);
        let mysql_result = self.mysql.execute(sql);

        ResultComparator::compare(&sqlrustgo_result, &mysql_result, sql, case_name)
    }

    /// Execute differential tests on multiple cases
    pub fn execute_batch(&self, cases: &[(&str, &str)]) -> Vec<DiffResult> {
        cases
            .iter()
            .map(|(name, sql)| self.execute_case(name, sql))
            .collect()
    }

    /// Check if MySQL is available
    pub fn is_mysql_available(&self) -> bool {
        self.mysql.is_available()
    }

    /// Get summary statistics
    pub fn summarize(results: &[DiffResult]) -> DifferentialSummary {
        let total = results.len();
        let matches = results.iter().filter(|r| r.status == DiffStatus::Match).count();
        let mismatches = results.iter().filter(|r| r.status == DiffStatus::Mismatch).count();
        let both_error = results.iter().filter(|r| r.status == DiffStatus::BothError).count();
        let one_failed = results.iter().filter(|r| r.status == DiffStatus::OneFailed).count();
        let skipped = results.iter().filter(|r| r.status == DiffStatus::Skipped).count();

        DifferentialSummary {
            total_cases: total,
            matches,
            mismatches,
            both_error,
            one_failed,
            skipped,
            match_rate: if total > 0 {
                (matches as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Summary of differential test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialSummary {
    pub total_cases: usize,
    pub matches: usize,
    pub mismatches: usize,
    pub both_error: usize,
    pub one_failed: usize,
    pub skipped: usize,
    pub match_rate: f64,
}

/// File result for differential corpus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialFileResult {
    pub file_path: String,
    pub total_cases: usize,
    pub matches: usize,
    pub mismatches: usize,
    pub results: Vec<DiffResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_comparator_match() {
        let sqlrustgo = EngineResult {
            success: true,
            rows: vec![
                vec![Value::Integer(1), Value::Text("hello".to_string())],
                vec![Value::Integer(2), Value::Text("world".to_string())],
            ],
            error: None,
            execution_time_ms: 10,
        };

        let mysql = EngineResult {
            success: true,
            rows: vec![
                vec![Value::Integer(2), Value::Text("world".to_string())],
                vec![Value::Integer(1), Value::Text("hello".to_string())],
            ],
            error: None,
            execution_time_ms: 15,
        };

        let result = ResultComparator::compare(&sqlrustgo, &mysql, "SELECT * FROM t", "test");
        assert_eq!(result.status, DiffStatus::Match);
    }

    #[test]
    fn test_result_comparator_mismatch() {
        let sqlrustgo = EngineResult {
            success: true,
            rows: vec![vec![Value::Integer(1)]],
            error: None,
            execution_time_ms: 10,
        };

        let mysql = EngineResult {
            success: true,
            rows: vec![vec![Value::Integer(2)]],
            error: None,
            execution_time_ms: 15,
        };

        let result = ResultComparator::compare(&sqlrustgo, &mysql, "SELECT * FROM t", "test");
        assert_eq!(result.status, DiffStatus::Mismatch);
    }

    #[test]
    fn test_result_comparator_one_failed() {
        let sqlrustgo = EngineResult {
            success: false,
            rows: vec![],
            error: Some("Parse error".to_string()),
            execution_time_ms: 10,
        };

        let mysql = EngineResult {
            success: true,
            rows: vec![vec![Value::Integer(1)]],
            error: None,
            execution_time_ms: 15,
        };

        let result = ResultComparator::compare(&sqlrustgo, &mysql, "SELECT * FROM t", "test");
        assert_eq!(result.status, DiffStatus::OneFailed);
    }
}
