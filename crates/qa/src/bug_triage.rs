//! Bug Triage AI
//!
//! Automated bug classification and routing using heuristics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugReport {
    pub title: String,
    pub description: String,
    pub stack_trace: Option<String>,
    pub module: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    pub predicted_severity: Severity,
    pub predicted_module: String,
    pub confidence: f32,
    pub similar_bugs: Vec<String>,
    pub suggested_fix: Option<String>,
}

pub struct BugTriageAI;

impl BugTriageAI {
    pub fn new() -> Self {
        Self
    }

    pub fn triage(&self, report: &BugReport) -> TriageResult {
        let severity = Self::predict_severity(&report.title, &report.description);
        let module = Self::predict_module(&report.title, &report.description);
        let similar = Self::find_similar_bugs(&report.description);
        let fix = Self::suggest_fix(&report.description);

        TriageResult {
            predicted_severity: severity,
            predicted_module: module,
            confidence: 0.75,
            similar_bugs: similar,
            suggested_fix: fix,
        }
    }

    fn predict_severity(title: &str, _desc: &str) -> Severity {
        let title_lower = title.to_lowercase();
        if title_lower.contains("crash")
            || title_lower.contains("deadlock")
            || title_lower.contains("data loss")
        {
            Severity::Critical
        } else if title_lower.contains("memory leak") || title_lower.contains("oom") {
            Severity::High
        } else if title_lower.contains("incorrect") || title_lower.contains("wrong") {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    fn predict_module(title: &str, desc: &str) -> String {
        let text = format!("{} {}", title, desc).to_lowercase();
        if text.contains("executor") || text.contains("query execution") {
            "executor".to_string()
        } else if text.contains("storage") || text.contains("buffer pool") {
            "storage".to_string()
        } else if text.contains("planner") || text.contains("optimizer") {
            "optimizer".to_string()
        } else if text.contains("parser") {
            "parser".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn find_similar_bugs(_desc: &str) -> Vec<String> {
        vec![]
    }

    fn suggest_fix(_desc: &str) -> Option<String> {
        None
    }
}

impl Default for BugTriageAI {
    fn default() -> Self {
        Self::new()
    }
}
