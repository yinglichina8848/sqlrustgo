//! SQL Fuzz Testing (SQLancer) Implementation
//!
//! This module provides SQL fuzz testing capabilities to automatically
//! generate and validate SQL queries, detecting semantic bugs.

pub mod generator;
pub mod oracle;

use generator::{DdlGenerator, DmlGenerator};
use oracle::{OracleResult, TlpOracle};
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FuzzerConfig {
    pub max_iterations: u64,
    pub max_table_size: usize,
    pub thread_count: usize,
    pub timeout_ms: u64,
}

impl Default for FuzzerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            max_table_size: 10,
            thread_count: 3,
            timeout_ms: 5000,
        }
    }
}

pub struct Fuzzer {
    config: FuzzerConfig,
    ddl_gen: DdlGenerator,
    dml_gen: DmlGenerator,
    oracle: TlpOracle,
}

impl Fuzzer {
    pub fn new(config: FuzzerConfig) -> Self {
        Self {
            config: config.clone(),
            ddl_gen: DdlGenerator::new(),
            dml_gen: DmlGenerator::new(0),
            oracle: TlpOracle::new(config.thread_count),
        }
    }

    pub fn run<E, R>(&mut self, mut executor: E) -> FuzzerResult
    where
        E: FnMut(&str) -> Result<R, String>,
        R: Clone,
    {
        let mut result = FuzzerResult::default();
        let start = Instant::now();

        for i in 0..self.config.max_iterations {
            if start.elapsed() > Duration::from_millis(self.config.timeout_ms) {
                result.timeout = true;
                break;
            }

            let sql = self.generate_random_sql();

            match executor(&sql) {
                Ok(_) => {
                    result.successful_queries += 1;
                }
                Err(e) => {
                    result.failed_queries += 1;
                    if result.errors.len() < 10 {
                        result
                            .errors
                            .push(format!("Iteration {}: {} - {}", i, sql, e));
                    }
                }
            }
        }

        result
    }

    fn generate_random_sql(&mut self) -> String {
        let choice = rand::thread_rng().gen_range(0..10);

        match choice {
            0 | 1 => self.ddl_gen.generate_create_table(),
            2 if self.ddl_gen.get_table_count() > 0 => self.ddl_gen.generate_drop_table(),
            _ => {
                self.dml_gen
                    .update_table_count(self.ddl_gen.get_table_count());

                let dml_choice = rand::thread_rng().gen_range(0..4);
                match dml_choice {
                    0 => self
                        .dml_gen
                        .generate_insert()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                    1 => self
                        .dml_gen
                        .generate_select()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                    2 => self
                        .dml_gen
                        .generate_update()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                    _ => self
                        .dml_gen
                        .generate_delete()
                        .unwrap_or_else(|| "SELECT 1".to_string()),
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FuzzerResult {
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub timeout: bool,
    pub errors: Vec<String>,
}

impl FuzzerResult {
    pub fn total_queries(&self) -> u64 {
        self.successful_queries + self.failed_queries
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries() as f64;
        if total == 0.0 {
            return 0.0;
        }
        self.successful_queries as f64 / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzer_basic() {
        let config = FuzzerConfig {
            max_iterations: 10,
            ..Default::default()
        };

        let mut fuzzer = Fuzzer::new(config);
        let executor = |sql: &str| -> Result<i32, String> {
            if sql.contains("DROP TABLE nonexistent") {
                Err("Table does not exist".to_string())
            } else {
                Ok(1)
            }
        };

        let result = fuzzer.run(executor);
        assert!(result.total_queries() > 0);
    }

    #[test]
    fn test_fuzzer_result() {
        let result = FuzzerResult {
            successful_queries: 80,
            failed_queries: 20,
            timeout: false,
            errors: vec![],
        };

        assert_eq!(result.total_queries(), 100);
        assert_eq!(result.success_rate(), 0.8);
    }
}
