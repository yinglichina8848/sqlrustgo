//! Test Registry Module
//!
//! Provides test metadata management and test discovery for regression testing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestCategory {
    Unit,
    Integration,
    Anomaly,
    Stress,
    E2E,
    CI,
}

impl TestCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TestCategory::Unit => "unit",
            TestCategory::Integration => "integration",
            TestCategory::Anomaly => "anomaly",
            TestCategory::Stress => "stress",
            TestCategory::E2E => "e2e",
            TestCategory::CI => "ci",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestPriority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl TestPriority {
    pub fn as_u8(&self) -> u8 {
        match self {
            TestPriority::P0 => 0,
            TestPriority::P1 => 1,
            TestPriority::P2 => 2,
            TestPriority::P3 => 3,
            TestPriority::P4 => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    pub id: String,
    pub name: String,
    pub category: TestCategory,
    pub module: String,
    pub tags: Vec<String>,
    pub priority: TestPriority,
    pub timeout_ms: u64,
    pub flaky: bool,
    pub file_path: String,
}

impl TestMetadata {
    pub fn new(id: &str, name: &str, category: TestCategory, module: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            module: module.to_string(),
            tags: Vec::new(),
            priority: TestPriority::P2,
            timeout_ms: 60000,
            flaky: false,
            file_path: String::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_priority(mut self, priority: TestPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn with_file_path(mut self, path: &str) -> Self {
        self.file_path = path.to_string();
        self
    }
}

#[derive(Debug, Default)]
pub struct TestRegistry {
    tests: HashMap<String, TestMetadata>,
    modules: HashMap<String, Vec<String>>,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, metadata: TestMetadata) {
        self.tests.insert(metadata.id.clone(), metadata.clone());

        self.modules
            .entry(metadata.module.clone())
            .or_insert_with(Vec::new)
            .push(metadata.id.clone());
    }

    pub fn get(&self, id: &str) -> Option<&TestMetadata> {
        self.tests.get(id)
    }

    pub fn get_by_category(&self, category: TestCategory) -> Vec<&TestMetadata> {
        self.tests
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn get_by_module(&self, module: &str) -> Vec<&TestMetadata> {
        self.tests.values().filter(|t| t.module == module).collect()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<&TestMetadata> {
        self.tests
            .values()
            .filter(|t| t.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_by_priority(&self, priority: TestPriority) -> Vec<&TestMetadata> {
        self.tests
            .values()
            .filter(|t| t.priority == priority)
            .collect()
    }

    pub fn get_all(&self) -> Vec<&TestMetadata> {
        self.tests.values().collect()
    }

    pub fn get_module_dependencies(&self, module: &str) -> Vec<String> {
        let mut deps = Vec::new();

        match module {
            "executor" => {
                deps.push("planner".to_string());
                deps.push("optimizer".to_string());
            }
            "planner" => {
                deps.push("parser".to_string());
            }
            "optimizer" => {
                deps.push("planner".to_string());
                deps.push("storage".to_string());
            }
            "storage" => {
                deps.push("types".to_string());
            }
            "transaction" => {
                deps.push("storage".to_string());
            }
            _ => {}
        }

        deps
    }

    pub fn get_affected_modules(&self, changed_files: &[String]) -> Vec<String> {
        let mut affected = Vec::new();

        for file in changed_files {
            let module = self.file_to_module(file);
            if !affected.contains(&module) {
                affected.push(module.clone());
            }

            let deps = self.get_module_dependencies(&module);
            for dep in deps {
                if !affected.contains(&dep) {
                    affected.push(dep);
                }
            }
        }

        affected
    }

    fn file_to_module(&self, file: &str) -> String {
        let path = PathBuf::from(file);

        if let Some(path_str) = path.to_str() {
            if path_str.contains("crates/parser") {
                return "parser".to_string();
            } else if path_str.contains("crates/planner") {
                return "planner".to_string();
            } else if path_str.contains("crates/optimizer") {
                return "optimizer".to_string();
            } else if path_str.contains("crates/executor") {
                return "executor".to_string();
            } else if path_str.contains("crates/storage") {
                return "storage".to_string();
            } else if path_str.contains("crates/transaction") {
                return "transaction".to_string();
            } else if path_str.contains("crates/server") {
                return "server".to_string();
            }
        }

        "unknown".to_string()
    }

    pub fn get_affected_tests(&self, changed_files: &[String]) -> Vec<&TestMetadata> {
        let affected_modules = self.get_affected_modules(changed_files);

        self.tests
            .values()
            .filter(|t| affected_modules.contains(&t.module))
            .collect()
    }

    pub fn total_count(&self) -> usize {
        self.tests.len()
    }

    pub fn count_by_category(&self) -> HashMap<TestCategory, usize> {
        let mut counts = HashMap::new();

        for test in self.tests.values() {
            *counts.entry(test.category).or_insert(0) += 1;
        }

        counts
    }
}

pub struct TestRegistryBuilder {
    registry: TestRegistry,
}

impl TestRegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: TestRegistry::new(),
        }
    }

    pub fn register_from_tests_dir(mut self, tests_dir: &PathBuf) -> Self {
        if let Ok(entries) = std::fs::read_dir(tests_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        let category = match dir_name {
                            "unit" => TestCategory::Unit,
                            "integration" => TestCategory::Integration,
                            "anomaly" => TestCategory::Anomaly,
                            "stress" => TestCategory::Stress,
                            "e2e" => TestCategory::E2E,
                            "ci" => TestCategory::CI,
                            _ => continue,
                        };

                        if let Ok(files) = std::fs::read_dir(&path) {
                            for file in files.flatten() {
                                let file_path = file.path();
                                if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                                    if let Some(file_name) =
                                        file_path.file_name().and_then(|n| n.to_str())
                                    {
                                        let test_name = file_name.trim_end_matches(".rs");
                                        let module = self.infer_module(&path);

                                        let metadata = TestMetadata::new(
                                            test_name, test_name, category, &module,
                                        )
                                        .with_file_path(file_path.to_str().unwrap_or(""))
                                        .with_timeout(60000);

                                        self.registry.register(metadata);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self
    }

    fn infer_module(&self, path: &PathBuf) -> String {
        let path_str = path.to_string_lossy().to_string();

        if path_str.contains("parser") {
            "parser".to_string()
        } else if path_str.contains("planner") {
            "planner".to_string()
        } else if path_str.contains("optimizer") {
            "optimizer".to_string()
        } else if path_str.contains("executor") {
            "executor".to_string()
        } else if path_str.contains("storage") {
            "storage".to_string()
        } else if path_str.contains("transaction") {
            "transaction".to_string()
        } else if path_str.contains("server") {
            "server".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn build(self) -> TestRegistry {
        self.registry
    }
}

impl Default for TestRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register() {
        let mut registry = TestRegistry::new();

        let metadata = TestMetadata::new(
            "test_001",
            "test_select_parser",
            TestCategory::Unit,
            "parser",
        );

        registry.register(metadata);

        assert_eq!(registry.total_count(), 1);
    }

    #[test]
    fn test_query_by_category() {
        let mut registry = TestRegistry::new();

        registry.register(TestMetadata::new("t1", "t1", TestCategory::Unit, "parser"));
        registry.register(TestMetadata::new("t2", "t2", TestCategory::Unit, "planner"));
        registry.register(TestMetadata::new(
            "t3",
            "t3",
            TestCategory::Integration,
            "parser",
        ));

        let unit_tests = registry.get_by_category(TestCategory::Unit);
        assert_eq!(unit_tests.len(), 2);
    }

    #[test]
    fn test_query_by_module() {
        let mut registry = TestRegistry::new();

        registry.register(TestMetadata::new("t1", "t1", TestCategory::Unit, "parser"));
        registry.register(TestMetadata::new("t2", "t2", TestCategory::Unit, "parser"));
        registry.register(TestMetadata::new("t3", "t3", TestCategory::Unit, "planner"));

        let parser_tests = registry.get_by_module("parser");
        assert_eq!(parser_tests.len(), 2);
    }

    #[test]
    fn test_module_dependencies() {
        let registry = TestRegistry::new();

        let deps = registry.get_module_dependencies("executor");
        assert!(deps.contains(&"planner".to_string()));
        assert!(deps.contains(&"optimizer".to_string()));
    }

    #[test]
    fn test_affected_tests() {
        let mut registry = TestRegistry::new();

        registry.register(TestMetadata::new("t1", "t1", TestCategory::Unit, "parser"));
        registry.register(TestMetadata::new("t2", "t2", TestCategory::Unit, "planner"));
        registry.register(TestMetadata::new(
            "t3",
            "t3",
            TestCategory::Unit,
            "executor",
        ));

        let changed = vec!["crates/planner/src/lib.rs".to_string()];
        let affected = registry.get_affected_tests(&changed);

        assert!(affected.iter().any(|t| t.module == "planner"));
    }
}
