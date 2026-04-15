//! SQL Regression Corpus
//!
//! A SQL-based regression test framework that loads SQL files and executes them
//! against SQLRustGo to verify correct behavior.
//!
//! # Directory Structure
//!
//! - sql_corpus/
//!   - DML/SELECT/ (predicates, joins, subqueries, aggregates)
//!   - DML/INSERT/, DML/UPDATE/, DML/DELETE/
//!   - DDL/FOREIGN_KEY/ (cascade, restrict, set_null)
//!   - DDL/TABLE/, TRANSACTION/, SPECIAL/
//!
//! # File Format
//!
//! SQL files contain test cases with SETUP and CASE markers:
//!
//! ```sql
//! -- === SETUP ===
//! CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
//! INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');
//!
//! -- === CASE: select_all ===
//! SELECT * FROM users;
//! -- EXPECT: 2 rows
//!
//! -- === CASE: select_with_where ===
//! SELECT * FROM users WHERE id = 1;
//! -- EXPECT: 1 row
//! ```

use serde::{Deserialize, Serialize};
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use sqlrustgo_executor::ExecutorResult;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Result of a SQL test case execution
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

/// Result of loading and executing a SQL corpus file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusFileResult {
    pub file_path: String,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<SqlTestResult>,
}

/// SQL Corpus loader and executor
pub struct SqlCorpus {
    corpus_root: PathBuf,
    engine: Arc<RwLock<ExecutionEngine>>,
}

impl SqlCorpus {
    /// Create a new SQL corpus executor with a fresh in-memory database
    pub fn new(corpus_root: PathBuf) -> Self {
        Self {
            corpus_root,
            engine: Arc::new(RwLock::new(ExecutionEngine::new(Arc::new(RwLock::new(
                MemoryStorage::new(),
            ))))),
        }
    }

    /// Create with a custom engine (for testing with different storage backends)
    pub fn with_engine(corpus_root: PathBuf, engine: ExecutionEngine) -> Self {
        Self {
            corpus_root,
            engine: Arc::new(RwLock::new(engine)),
        }
    }

    /// Reset the database to a clean state
    pub fn reset(&self) {
        // Create a new engine with fresh storage
        *self.engine.write().unwrap() =
            ExecutionEngine::new(Arc::new(RwLock::new(MemoryStorage::new())));
    }

    /// Execute all SQL files in the corpus recursively
    pub fn execute_all(&self) -> HashMap<String, CorpusFileResult> {
        let mut results = HashMap::new();
        self.execute_directory(&self.corpus_root, &mut results);
        results
    }

    /// Execute all SQL files in a specific directory
    pub fn execute_directory(&self, dir: &Path, results: &mut HashMap<String, CorpusFileResult>) {
        if !dir.is_dir() {
            return;
        }

        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories
                self.execute_directory(&path, results);
            } else if path.extension().map_or(false, |e| e == "sql") {
                // Process SQL file
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

    /// Execute a single SQL file and return results
    pub fn execute_file(&self, path: &Path) -> CorpusFileResult {
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

    /// Parse and execute SQL content
    pub fn parse_and_execute(&self, content: &str) -> Vec<SqlTestResult> {
        let mut results = Vec::new();
        let mut current_case: Option<TestCase> = None;
        let mut setup_sql = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("-- === SETUP ===") {
                // Execute any pending case first
                if let Some(case) = current_case.take() {
                    results.push(self.execute_case(case, &setup_sql));
                }
                setup_sql.clear();
            } else if trimmed.starts_with("-- === CASE:") {
                // Save current case name and execute previous one
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
                        // Error expected
                        case.expected_rows = Some(0);
                    } else if expect.starts_with("rows") {
                        // Parse "N rows"
                        if let Ok(n) = expect.split_whitespace().next().unwrap_or("0").parse() {
                            case.expected_rows = Some(n);
                        }
                    }
                }
            } else if trimmed.starts_with("-- COLUMNS:") {
                if let Some(ref mut case) = current_case {
                    let cols = trimmed
                        .trim_start_matches("-- COLUMNS:")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    case.expected_columns = Some(cols);
                }
            } else if current_case.is_some() {
                // Add line to current case SQL
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    if !current_case.as_ref().unwrap().sql.is_empty() {
                        current_case.as_mut().unwrap().sql.push('\n');
                    }
                    current_case.as_mut().unwrap().sql.push_str(trimmed);
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("--") {
                // Setup SQL
                if !setup_sql.is_empty() {
                    setup_sql.push('\n');
                }
                setup_sql.push_str(trimmed);
            }
        }

        // Execute last pending case
        if let Some(case) = current_case {
            results.push(self.execute_case(case, &setup_sql));
        }

        results
    }

    fn execute_case(&self, case: TestCase, setup_sql: &str) -> SqlTestResult {
        let start = std::time::Instant::now();

        // Reset database for each case
        self.reset();

        // Execute setup SQL first
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

        // Execute test SQL
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
                error_message: Some(e.to_string()),
                expected_rows: case.expected_rows,
                expected_columns: case.expected_columns,
            },
        }
    }

    fn execute_sql(&self, sql: &str) -> Result<ExecutorResult, sqlrustgo_types::SqlError> {
        // Handle multiple statements separated by semicolons
        let statements: Vec<&str> = sql.split(';').filter(|s| !s.trim().is_empty()).collect();

        let mut engine = self.engine.write().unwrap();
        let mut last_result: Option<ExecutorResult> = None;

        for stmt in statements {
            let stmt = stmt.trim();
            if stmt.is_empty() {
                continue;
            }

            // Parse the statement
            let statement = parse(stmt).map_err(|e| {
                sqlrustgo_types::SqlError::ExecutionError(format!("Parse error: {:?}", e))
            })?;

            // Execute the statement
            last_result = Some(engine.execute(statement)?);
        }

        Ok(last_result.unwrap_or_else(|| ExecutorResult::new(vec![], 0)))
    }

    /// Get a summary of all results
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

/// Internal representation of a test case
#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    sql: String,
    expected_rows: Option<usize>,
    expected_columns: Option<Vec<String>>,
}

/// Summary of corpus execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusSummary {
    pub total_files: usize,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_execute_simple() {
        let corpus = SqlCorpus::new(PathBuf::from("."));
        let sql = r#"
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');

-- === CASE: select_all ===
SELECT * FROM users;
-- EXPECT: 2 rows
"#;
        let results = corpus.parse_and_execute(sql);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].rows_returned, 2);
    }

    #[test]
    fn test_parse_and_execute_with_setup() {
        let corpus = SqlCorpus::new(PathBuf::from("."));
        let sql = r#"
-- === SETUP ===
CREATE TABLE t (x INTEGER);
INSERT INTO t VALUES (1), (2), (3);

-- === CASE: count ===
SELECT COUNT(*) FROM t;
-- EXPECT: 1 rows

-- === CASE: select_all ===
SELECT * FROM t;
-- EXPECT: 3 rows
"#;
        let results = corpus.parse_and_execute(sql);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }
}
