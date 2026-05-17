//! Flaky Test Detector
//!
//! Statistical detection of flaky tests based on historical pass rates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyResult {
    pub test_name: String,
    pub pass_rate: f64,
    pub is_flaky: bool,
    pub confidence: f64,
    pub recommended_action: FlakyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlakyAction {
    Retry,
    MarkKnownFlaky,
    RequiresInvestigation,
    NeedsFix,
}

pub struct FlakyTestDetector {
    history: HashMap<String, (usize, usize)>,
    significance_threshold: f64,
    min_runs: u32,
}

impl FlakyTestDetector {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            significance_threshold: 0.05,
            min_runs: 10,
        }
    }

    pub fn analyze(&self, test_name: &str) -> Option<FlakyResult> {
        let entry = self.history.get(test_name)?;
        let (pass, fail) = *entry;
        let total = pass + fail;
        if total < self.min_runs as usize {
            return None;
        }
        let pass_rate = pass as f64 / total as f64;
        let is_flaky = pass_rate < 0.95;

        let action = if is_flaky {
            if pass_rate >= 0.80 {
                FlakyAction::Retry
            } else {
                FlakyAction::NeedsFix
            }
        } else {
            FlakyAction::RequiresInvestigation
        };

        Some(FlakyResult {
            test_name: test_name.to_string(),
            pass_rate,
            is_flaky,
            confidence: (total as f64 / 100.0).min(1.0),
            recommended_action: action,
        })
    }

    pub fn record(&mut self, test_name: String, passed: bool) {
        let entry = self.history.entry(test_name).or_insert((0, 0));
        if passed {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
}

impl Default for FlakyTestDetector {
    fn default() -> Self {
        Self::new()
    }
}
