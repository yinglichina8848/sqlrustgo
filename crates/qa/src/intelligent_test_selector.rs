//! Intelligent Test Selector
//!
//! ML-based test selection that predicts which tests are most likely
//! to fail based on code changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChanges {
    pub files_changed: Vec<String>,
    pub lines_changed: usize,
    pub modules_affected: Vec<String>,
}

pub struct IntelligentTestSelector {
    /// Historical test results: test_name -> pass_count, fail_count
    history: HashMap<String, (usize, usize)>,
}

impl IntelligentTestSelector {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn predict_failure_probability(&self, changes: &CodeChanges) -> HashMap<String, f64> {
        let mut predictions = HashMap::new();
        for (test_name, &(pass, fail)) in &self.history {
            let total = pass + fail;
            if total == 0 {
                predictions.insert(test_name.clone(), 0.5);
                continue;
            }
            // Base failure rate
            let base_rate = fail as f64 / total as f64;
            // Adjust based on affected modules
            let module_factor = if changes.modules_affected.iter().any(|m| test_name.contains(m)) {
                1.5
            } else {
                1.0
            };
            predictions.insert(test_name.clone(), (base_rate * module_factor).min(1.0));
        }
        predictions
    }

    pub fn select_tests(&self, changes: &CodeChanges, max_tests: usize) -> Vec<String> {
        let predictions = self.predict_failure_probability(changes);
        let mut sorted: Vec<_> = predictions.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(max_tests).map(|(name, _)| name).collect()
    }

    pub fn record_result(&mut self, test_name: String, passed: bool) {
        let entry = self.history.entry(test_name).or_insert((0, 0));
        if passed {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
}

impl Default for IntelligentTestSelector {
    fn default() -> Self {
        Self::new()
    }
}
