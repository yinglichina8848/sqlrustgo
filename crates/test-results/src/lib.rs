//! Test Results Module
//!
//! Provides result collection, statistics, and trend analysis for regression testing.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultRecord {
    pub test_id: String,
    pub name: String,
    pub status: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub category: Option<String>,
    pub module: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunSession {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub timed_out: usize,
    pub crashed: usize,
    pub total_duration_ms: u64,
    pub results: Vec<TestResultRecord>,
}

impl TestRunSession {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            timestamp: Utc::now(),
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            timed_out: 0,
            crashed: 0,
            total_duration_ms: 0,
            results: Vec::new(),
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total_tests as f64) * 100.0
    }

    pub fn is_success(&self) -> bool {
        self.failed == 0 && self.timed_out == 0 && self.crashed == 0
    }
}

pub struct ResultCollector {
    current_session: Option<TestRunSession>,
    history: RwLock<VecDeque<TestRunSession>>,
    max_history: usize,
    storage_path: Option<PathBuf>,
}

impl ResultCollector {
    pub fn new() -> Self {
        Self {
            current_session: None,
            history: RwLock::new(VecDeque::new()),
            max_history: 100,
            storage_path: None,
        }
    }

    pub fn with_storage(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    pub fn start_session(&mut self, session_id: &str) {
        self.current_session = Some(TestRunSession::new(session_id));
    }

    pub fn add_result(&mut self, record: TestResultRecord) {
        if let Some(ref mut session) = self.current_session {
            session.total_tests += 1;
            session.total_duration_ms += record.duration_ms;

            match record.status.as_str() {
                "passed" => session.passed += 1,
                "failed" => session.failed += 1,
                "skipped" => session.skipped += 1,
                "timed_out" => session.timed_out += 1,
                "crashed" => session.crashed += 1,
                _ => {}
            }

            session.results.push(record);
        }
    }

    pub fn finish_session(&mut self) -> Option<TestRunSession> {
        if let Some(session) = self.current_session.take() {
            let mut history = self.history.write();

            if history.len() >= self.max_history {
                history.pop_front();
            }

            history.push_back(session.clone());

            if let Some(ref path) = self.storage_path {
                self.save_session_to_disk(&session, path);
            }

            return Some(session);
        }
        None
    }

    fn save_session_to_disk(&self, session: &TestRunSession, base_path: &PathBuf) {
        let file_name = format!("session_{}.json", session.timestamp.format("%Y%m%d_%H%M%S"));
        let file_path = base_path.join(&file_name);

        if let Ok(json) = serde_json::to_string_pretty(session) {
            let _ = fs::write(&file_path, json);
        }
    }

    pub fn get_current_session(&self) -> Option<&TestRunSession> {
        self.current_session.as_ref()
    }

    pub fn get_history(&self) -> Vec<TestRunSession> {
        self.history.read().iter().cloned().collect()
    }

    pub fn get_latest_sessions(&self, count: usize) -> Vec<TestRunSession> {
        let history = self.history.read();
        history.iter().rev().take(count).cloned().collect()
    }

    pub fn get_statistics(&self) -> CollectorStatistics {
        let history = self.history.read();

        if history.is_empty() {
            return CollectorStatistics::default();
        }

        let total_runs = history.len();
        let total_tests: usize = history.iter().map(|s| s.total_tests).sum();
        let total_passed: usize = history.iter().map(|s| s.passed).sum();
        let total_failed: usize = history.iter().map(|s| s.failed).sum();
        let avg_pass_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let mut fail_rates_by_category: HashMap<String, f64> = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        for session in history.iter() {
            for result in &session.results {
                if let Some(ref category) = result.category {
                    let count = category_counts.entry(category.clone()).or_insert(0);
                    *count += 1;

                    if result.status == "failed" {
                        let rate = fail_rates_by_category
                            .entry(category.clone())
                            .or_insert(0.0);
                        *rate += 1.0;
                    }
                }
            }
        }

        for (category, fail_count) in fail_rates_by_category.iter_mut() {
            if let Some(&total) = category_counts.get(category) {
                *fail_count = (*fail_count / total as f64) * 100.0;
            }
        }

        let mut flaky_tests: HashMap<String, u32> = HashMap::new();
        for session in history.iter() {
            for result in &session.results {
                if result.status == "failed" {
                    let count = flaky_tests.entry(result.test_id.clone()).or_insert(0);
                    *count += 1;
                }
            }
        }

        CollectorStatistics {
            total_runs,
            total_tests,
            total_passed,
            total_failed,
            avg_pass_rate,
            fail_rates_by_category,
            flaky_tests,
        }
    }

    pub fn load_from_disk(&mut self, path: &PathBuf) -> Result<(), String> {
        if !path.is_dir() {
            return Err("Path must be a directory".to_string());
        }

        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;

        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    if let Ok(session) = serde_json::from_str::<TestRunSession>(&content) {
                        let mut history = self.history.write();
                        if history.len() < self.max_history {
                            history.push_back(session);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.current_session = None;
        self.history.write().clear();
    }
}

impl Default for ResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectorStatistics {
    pub total_runs: usize,
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub avg_pass_rate: f64,
    pub fail_rates_by_category: HashMap<String, f64>,
    pub flaky_tests: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub pass_rate_trend: Vec<f64>,
    pub avg_duration_trend: Vec<u64>,
    pub failure_count_trend: Vec<usize>,
    pub regression_detected: bool,
    pub improvement_detected: bool,
}

impl TrendAnalysis {
    pub fn analyze(history: &[TestRunSession]) -> Self {
        if history.is_empty() {
            return Self {
                pass_rate_trend: Vec::new(),
                avg_duration_trend: Vec::new(),
                failure_count_trend: Vec::new(),
                regression_detected: false,
                improvement_detected: false,
            };
        }

        let pass_rate_trend: Vec<f64> = history.iter().map(|s| s.pass_rate()).collect();
        let avg_duration_trend: Vec<u64> = history
            .iter()
            .map(|s| {
                if s.total_tests > 0 {
                    s.total_duration_ms / s.total_tests as u64
                } else {
                    0
                }
            })
            .collect();
        let failure_count_trend: Vec<usize> = history
            .iter()
            .map(|s| s.failed + s.timed_out + s.crashed)
            .collect();

        let recent_runs = pass_rate_trend.len().min(5);
        let regression_detected = if recent_runs >= 2 {
            let recent_avg: f64 =
                pass_rate_trend.iter().rev().take(recent_runs).sum::<f64>() / recent_runs as f64;
            let older_avg: f64 = if recent_runs < pass_rate_trend.len() {
                pass_rate_trend
                    .iter()
                    .rev()
                    .skip(recent_runs)
                    .take(recent_runs)
                    .sum::<f64>()
                    / recent_runs as f64
            } else {
                recent_avg
            };
            recent_avg < older_avg - 5.0
        } else {
            false
        };

        let improvement_detected = if recent_runs >= 2 {
            let recent_avg: f64 =
                pass_rate_trend.iter().rev().take(recent_runs).sum::<f64>() / recent_runs as f64;
            let older_avg: f64 = if recent_runs < pass_rate_trend.len() {
                pass_rate_trend
                    .iter()
                    .rev()
                    .skip(recent_runs)
                    .take(recent_runs)
                    .sum::<f64>()
                    / recent_runs as f64
            } else {
                recent_avg
            };
            recent_avg > older_avg + 5.0
        } else {
            false
        };

        Self {
            pass_rate_trend,
            avg_duration_trend,
            failure_count_trend,
            regression_detected,
            improvement_detected,
        }
    }
}

pub struct ResultAnalyzer;

impl ResultAnalyzer {
    pub fn analyze_failures(sessions: &[TestRunSession]) -> FailureAnalysis {
        let mut failure_patterns: HashMap<String, usize> = HashMap::new();
        let mut failed_tests: HashMap<String, Vec<String>> = HashMap::new();

        for session in sessions {
            for result in &session.results {
                if result.status == "failed" {
                    let error_key = result
                        .error_message
                        .clone()
                        .unwrap_or_else(|| "Unknown error".to_string());

                    let short_error = if error_key.len() > 50 {
                        format!("{}...", &error_key[..50])
                    } else {
                        error_key
                    };

                    *failure_patterns.entry(short_error).or_insert(0) += 1;

                    failed_tests
                        .entry(result.test_id.clone())
                        .or_insert_with(Vec::new)
                        .push(session.id.clone());
                }
            }
        }

        let common_failures: Vec<(String, usize)> = failure_patterns
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .collect();

        let flaky: Vec<(String, usize)> = failed_tests
            .into_iter()
            .filter(|(_, sessions)| sessions.len() > 1)
            .map(|(test, sessions)| (test, sessions.len()))
            .collect();

        FailureAnalysis {
            common_failures,
            flaky_tests: flaky,
        }
    }

    pub fn generate_recommendations(stats: &CollectorStatistics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if stats.avg_pass_rate < 80.0 {
            recommendations
                .push("Pass rate is below 80%. Consider running more tests in CI.".to_string());
        }

        for (category, rate) in &stats.fail_rates_by_category {
            if *rate > 10.0 {
                recommendations.push(format!(
                    "Category '{}' has {:.1}% failure rate. Investigate root cause.",
                    category, rate
                ));
            }
        }

        let flaky_threshold = (stats.total_runs / 4) as u32;
        for (test, count) in &stats.flaky_tests {
            if *count > flaky_threshold {
                recommendations.push(format!(
                    "Test '{}' failed {} times. Consider marking as flaky or fixing.",
                    test, count
                ));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All tests are passing at healthy levels.".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    pub common_failures: Vec<(String, usize)>,
    pub flaky_tests: Vec<(String, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = TestRunSession::new("test_session_001");
        assert_eq!(session.total_tests, 0);
        assert_eq!(session.pass_rate(), 0.0);
    }

    #[test]
    fn test_collector_basic() {
        let mut collector = ResultCollector::new();
        collector.start_session("session_001");

        collector.add_result(TestResultRecord {
            test_id: "test_001".to_string(),
            name: "test_select".to_string(),
            status: "passed".to_string(),
            duration_ms: 100,
            timestamp: Utc::now(),
            category: Some("unit".to_string()),
            module: Some("parser".to_string()),
            error_message: None,
        });

        collector.add_result(TestResultRecord {
            test_id: "test_002".to_string(),
            name: "test_insert".to_string(),
            status: "failed".to_string(),
            duration_ms: 50,
            timestamp: Utc::now(),
            category: Some("unit".to_string()),
            module: Some("executor".to_string()),
            error_message: Some("Assertion failed".to_string()),
        });

        let session = collector.finish_session().unwrap();
        assert_eq!(session.total_tests, 2);
        assert_eq!(session.passed, 1);
        assert_eq!(session.failed, 1);
        assert!((session.pass_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_trend_analysis() {
        let mut sessions = Vec::new();

        for i in 0..5 {
            let mut session = TestRunSession::new(&format!("session_{}", i));
            session.passed = 90 + i;
            session.total_tests = 100;
            sessions.push(session);
        }

        let trend = TrendAnalysis::analyze(&sessions);
        assert!(trend.pass_rate_trend.len() == 5);
        assert!(!trend.regression_detected);
    }

    #[test]
    fn test_statistics() {
        let mut collector = ResultCollector::new();
        collector.start_session("session_001");

        for _ in 0..5 {
            collector.add_result(TestResultRecord {
                test_id: "test_001".to_string(),
                name: "test".to_string(),
                status: "passed".to_string(),
                duration_ms: 100,
                timestamp: Utc::now(),
                category: Some("unit".to_string()),
                module: None,
                error_message: None,
            });
        }

        collector.finish_session();
        let stats = collector.get_statistics();
        assert_eq!(stats.total_runs, 1);
        assert_eq!(stats.total_tests, 5);
    }

    #[test]
    fn test_recommendations() {
        let stats = CollectorStatistics {
            total_runs: 10,
            total_tests: 100,
            total_passed: 75,
            total_failed: 25,
            avg_pass_rate: 75.0,
            fail_rates_by_category: HashMap::new(),
            flaky_tests: HashMap::new(),
        };

        let recommendations = ResultAnalyzer::generate_recommendations(&stats);
        assert!(!recommendations.is_empty());
    }
}
