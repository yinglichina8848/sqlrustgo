//! Test Reporter Module
//!
//! Provides report generation in various formats (HTML, JSON, Markdown).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use test_results::{CollectorStatistics, TestRunSession, TrendAnalysis};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub title: String,
    pub output_path: PathBuf,
    pub include_output: bool,
    pub max_failed_display: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            title: "Test Report".to_string(),
            output_path: PathBuf::from("test_report.html"),
            include_output: false,
            max_failed_display: 20,
        }
    }
}

pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(ReportConfig::default())
    }

    pub fn generate_html_report(&self, session: &TestRunSession) -> String {
        let pass_rate = session.pass_rate();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ background: #2c3e50; color: white; padding: 20px; border-radius: 8px 8px 0 0; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; padding: 20px; }}
        .stat-card {{ background: #ecf0f1; padding: 20px; border-radius: 8px; text-align: center; }}
        .stat-value {{ font-size: 32px; font-weight: bold; }}
        .stat-label {{ color: #7f8c8d; margin-top: 5px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Test Results</h1>
            <p>Generated: {}</p>
        </div>
        <div class="summary">
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Tests</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Passed</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}</div>
                <div class="stat-label">Failed</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{}%</div>
                <div class="stat-label">Pass Rate</div>
            </div>
        </div>
    </div>
</body>
</html>"#,
            self.config.title,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            session.total_tests,
            session.passed,
            session.failed,
            pass_rate as i32
        )
    }

    pub fn generate_markdown_report(&self, session: &TestRunSession) -> String {
        let pass_rate = session.pass_rate();

        format!(
            r#"# Test Results

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | {} |
| Passed | {} |
| Failed | {} |
| Skipped | {} |
| Pass Rate | {:.1}% |
| Duration | {}ms |

## Metadata

- **Session ID:** {}
- **Timestamp:** {}
"#,
            session.total_tests,
            session.passed,
            session.failed,
            session.skipped,
            pass_rate,
            session.total_duration_ms,
            session.id,
            session.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    pub fn generate_json_report(&self, session: &TestRunSession) -> String {
        serde_json::to_string_pretty(session).unwrap_or_default()
    }

    pub fn save_html_report(&self, session: &TestRunSession) -> Result<(), String> {
        let content = self.generate_html_report(session);
        fs::write(&self.config.output_path, content).map_err(|e| e.to_string())
    }

    pub fn save_markdown_report(&self, session: &TestRunSession) -> Result<(), String> {
        let content = self.generate_markdown_report(session);
        let md_path = self.config.output_path.with_extension("md");
        fs::write(&md_path, content).map_err(|e| e.to_string())
    }

    pub fn save_json_report(&self, session: &TestRunSession) -> Result<(), String> {
        let content = self.generate_json_report(session);
        let json_path = self.config.output_path.with_extension("json");
        fs::write(&json_path, content).map_err(|e| e.to_string())
    }

    pub fn generate_trend_report(
        &self,
        sessions: &[TestRunSession],
        stats: &CollectorStatistics,
    ) -> String {
        let trend = TrendAnalysis::analyze(sessions);

        format!(
            r#"# Test Trend Analysis

## Overall Statistics

- **Total Runs:** {}
- **Total Tests:** {}
- **Average Pass Rate:** {:.1}%
- **Total Failures:** {}

## Trend

- Pass Rate Trend entries: {}

## Recommendations

- Review any regression patterns
- Check flaky tests
"#,
            stats.total_runs,
            stats.total_tests,
            stats.avg_pass_rate,
            stats.total_failed,
            trend.pass_rate_trend.len()
        )
    }
}

pub struct CiReportGenerator;

impl CiReportGenerator {
    pub fn generate_github_summary(session: &TestRunSession) -> String {
        if session.is_success() {
            format!(
                "## Test Results\n\n- Total: {} tests\n- Passed: {}\n- Duration: {}ms\n",
                session.total_tests, session.passed, session.total_duration_ms
            )
        } else {
            format!(
                "## Test Results\n\n- Total: {} tests\n- Passed: {}\n- Failed: {}\n- Duration: {}ms\n",
                session.total_tests, session.passed, session.failed, session.total_duration_ms
            )
        }
    }

    pub fn generate_junit_xml(session: &TestRunSession) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="{}" tests="{}" failures="{}" errors="{}" skipped="{}" time="{}">
</testsuite>"#,
            session.id,
            session.total_tests,
            session.failed,
            session.crashed,
            session.skipped,
            session.total_duration_ms as f64 / 1000.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator_creation() {
        let config = ReportConfig::default();
        let generator = ReportGenerator::new(config);
        assert_eq!(generator.config.title, "Test Report");
    }

    #[test]
    fn test_html_report_generation() {
        let session = TestRunSession::new("test_session");
        let generator = ReportGenerator::with_default_config();
        let html = generator.generate_html_report(&session);
        assert!(html.contains("Test Results"));
    }

    #[test]
    fn test_markdown_report_generation() {
        let session = TestRunSession::new("test_session");
        let generator = ReportGenerator::with_default_config();
        let md = generator.generate_markdown_report(&session);
        assert!(md.contains("test_session"));
    }

    #[test]
    fn test_json_report_generation() {
        let session = TestRunSession::new("test_session");
        let generator = ReportGenerator::with_default_config();
        let json = generator.generate_json_report(&session);
        assert!(json.contains("test_session"));
    }

    #[test]
    fn test_github_summary() {
        let session = TestRunSession::new("test_session");
        let summary = CiReportGenerator::generate_github_summary(&session);
        assert!(summary.contains("Test Results"));
    }

    #[test]
    fn test_junit_xml() {
        let session = TestRunSession::new("test_session");
        let xml = CiReportGenerator::generate_junit_xml(&session);
        assert!(xml.contains("testsuite"));
    }
}
