//! Test Runner Module
//!
//! Core test execution engine for regression testing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    TimedOut,
    Crashed,
}

impl TestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestStatus::Pending => "pending",
            TestStatus::Running => "running",
            TestStatus::Passed => "passed",
            TestStatus::Failed => "failed",
            TestStatus::Skipped => "skipped",
            TestStatus::TimedOut => "timed_out",
            TestStatus::Crashed => "crashed",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, TestStatus::Passed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub output: String,
    pub error_message: Option<String>,
    pub retries: u32,
}

impl TestResult {
    pub fn new(test_id: &str, name: &str) -> Self {
        let now = Utc::now();
        Self {
            test_id: test_id.to_string(),
            name: name.to_string(),
            status: TestStatus::Pending,
            duration_ms: 0,
            started_at: now,
            finished_at: now,
            output: String::new(),
            error_message: None,
            retries: 0,
        }
    }

    pub fn passed(duration_ms: u64) -> Self {
        let now = Utc::now();
        Self {
            test_id: String::new(),
            name: String::new(),
            status: TestStatus::Passed,
            duration_ms,
            started_at: now,
            finished_at: now,
            output: String::new(),
            error_message: None,
            retries: 0,
        }
    }

    pub fn failed(duration_ms: u64, error: &str) -> Self {
        let now = Utc::now();
        Self {
            test_id: String::new(),
            name: String::new(),
            status: TestStatus::Failed,
            duration_ms,
            started_at: now,
            finished_at: now,
            output: String::new(),
            error_message: Some(error.to_string()),
            retries: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunConfig {
    pub max_parallel: usize,
    pub retry_count: u32,
    pub timeout_per_test_ms: u64,
    pub working_dir: PathBuf,
    pub cargo_binary: String,
    pub test_flags: Vec<String>,
}

impl Default for TestRunConfig {
    fn default() -> Self {
        Self {
            max_parallel: num_cpus::get(),
            retry_count: 0,
            timeout_per_test_ms: 120000,
            working_dir: PathBuf::from("."),
            cargo_binary: "cargo".to_string(),
            test_flags: vec![],
        }
    }
}

pub struct TestRunner {
    config: TestRunConfig,
    results: HashMap<String, TestResult>,
}

impl TestRunner {
    pub fn new(config: TestRunConfig) -> Self {
        Self {
            config,
            results: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(TestRunConfig::default())
    }

    pub async fn run_test(&self, test_id: &str, name: &str) -> TestResult {
        let start_time = Instant::now();
        let result = Self::execute_cargo_test(&self.config, test_id, name).await;
        let duration_ms = start_time.elapsed().as_millis() as u64;

        let mut final_result = result;
        final_result.duration_ms = duration_ms;
        final_result.finished_at = Utc::now();

        final_result
    }

    async fn execute_cargo_test(config: &TestRunConfig, test_id: &str, name: &str) -> TestResult {
        let started_at = Utc::now();
        let mut cmd = Command::new(&config.cargo_binary);
        cmd.arg("test")
            .arg(test_id)
            .arg("--")
            .args(&config.test_flags)
            .current_dir(&config.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = match cmd.output().await {
            Ok(o) => o,
            Err(e) => {
                return TestResult {
                    test_id: test_id.to_string(),
                    name: name.to_string(),
                    status: TestStatus::Crashed,
                    duration_ms: 0,
                    started_at,
                    finished_at: Utc::now(),
                    output: String::new(),
                    error_message: Some(format!("Failed to execute test: {}", e)),
                    retries: 0,
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined_output = format!("{}\n{}", stdout, stderr);

        let status = if output.status.success() {
            TestStatus::Passed
        } else if output.status.code() == Some(101) {
            TestStatus::Failed
        } else {
            TestStatus::Crashed
        };

        TestResult {
            test_id: test_id.to_string(),
            name: name.to_string(),
            status,
            duration_ms: 0,
            started_at,
            finished_at: Utc::now(),
            output: combined_output,
            error_message: None,
            retries: 0,
        }
    }

    pub async fn run_tests(&mut self, test_ids: Vec<String>) -> Vec<TestResult> {
        let mut results = Vec::new();

        for test_id in test_ids {
            let result = self.run_test(&test_id, &test_id).await;
            self.results.insert(test_id, result.clone());
            results.push(result);
        }

        results
    }

    pub fn get_result(&self, test_id: &str) -> Option<TestResult> {
        self.results.get(test_id).cloned()
    }

    pub fn get_all_results(&self) -> Vec<TestResult> {
        self.results.values().cloned().collect()
    }

    pub fn summary(&self) -> TestRunSummary {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut timed_out = 0;
        let mut crashed = 0;
        let mut total_duration_ms = 0u64;

        for result in self.results.values() {
            match result.status {
                TestStatus::Passed => passed += 1,
                TestStatus::Failed => failed += 1,
                TestStatus::Skipped => skipped += 1,
                TestStatus::TimedOut => timed_out += 1,
                TestStatus::Crashed => crashed += 1,
                _ => {}
            }
            total_duration_ms += result.duration_ms;
        }

        TestRunSummary {
            total: self.results.len(),
            passed,
            failed,
            skipped,
            timed_out,
            crashed,
            total_duration_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub timed_out: usize,
    pub crashed: usize,
    pub total_duration_ms: u64,
}

impl TestRunSummary {
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.timed_out == 0 && self.crashed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_creation() {
        let result = TestResult::new("test_001", "test_select");
        assert_eq!(result.status, TestStatus::Pending);
    }

    #[test]
    fn test_result_passed() {
        let result = TestResult::passed(100);
        assert_eq!(result.status, TestStatus::Passed);
    }

    #[test]
    fn test_result_failed() {
        let result = TestResult::failed(50, "Assertion failed");
        assert_eq!(result.status, TestStatus::Failed);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_run_summary() {
        let summary = TestRunSummary {
            total: 10,
            passed: 8,
            failed: 1,
            skipped: 0,
            timed_out: 1,
            crashed: 0,
            total_duration_ms: 5000,
        };

        assert_eq!(summary.pass_rate(), 80.0);
        assert!(!summary.all_passed());
    }

    #[test]
    fn test_run_summary_all_passed() {
        let summary = TestRunSummary {
            total: 10,
            passed: 10,
            failed: 0,
            skipped: 0,
            timed_out: 0,
            crashed: 0,
            total_duration_ms: 5000,
        };

        assert!(summary.all_passed());
    }

    #[test]
    fn test_config_defaults() {
        let config = TestRunConfig::default();
        assert!(config.max_parallel > 0);
        assert_eq!(config.retry_count, 0);
    }
}
