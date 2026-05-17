//! SQLRustGo QA Module
//!
//! AI-powered testing tools including:
//! - AI Test Generator: LLM-based test case generation
//! - Intelligent Test Selection: ML-based test selection for PRs
//! - Bug Triage AI: Automated bug classification and routing
//! - Flaky Test Detector: Statistical flaky test detection

pub mod ai_test_generator;
pub mod intelligent_test_selector;
pub mod bug_triage;
pub mod flaky_detector;

pub use ai_test_generator::{AITestGenerator, TestCase, TestContext};
pub use intelligent_test_selector::IntelligentTestSelector;
pub use bug_triage::{BugTriageAI, BugReport, TriageResult, Severity};
pub use flaky_detector::{FlakyTestDetector, FlakyAction};
