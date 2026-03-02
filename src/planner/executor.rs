//! Execution Engine Module
//!
//! Provides the ExecutionEngine trait and implementations for query execution.

use crate::types::{SqlError, SqlResult, Value};

pub trait ExecutionEngine: Send + Sync {
    fn execute(&self, plan: &dyn super::PhysicalPlan) -> SqlResult<Vec<Vec<Value>>>;
    fn name(&self) -> &str;
}

pub struct DefaultExecutionEngine;

impl DefaultExecutionEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine for DefaultExecutionEngine {
    fn execute(&self, plan: &dyn super::PhysicalPlan) -> SqlResult<Vec<Vec<Value>>> {
        plan.execute()
    }

    fn name(&self) -> &str {
        "default"
    }
}

pub struct EngineRegistry {
    engines: std::collections::HashMap<String, Box<dyn ExecutionEngine>>,
    default: String,
}

impl EngineRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            engines: std::collections::HashMap::new(),
            default: "default".to_string(),
        };
        registry.register("default", Box::new(DefaultExecutionEngine::new()));
        registry
    }

    pub fn register(&mut self, name: &str, engine: Box<dyn ExecutionEngine>) {
        self.engines.insert(name.to_string(), engine);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ExecutionEngine> {
        self.engines.get(name).map(|e| e.as_ref())
    }

    pub fn execute(&self, plan: &dyn super::PhysicalPlan) -> SqlResult<Vec<Vec<Value>>> {
        let engine = self.engines.get(&self.default).ok_or_else(|| {
            SqlError::ExecutionError("No execution engine registered".to_string())
        })?;
        engine.execute(plan)
    }

    pub fn set_default(&mut self, name: &str) -> SqlResult<()> {
        if self.engines.contains_key(name) {
            self.default = name.to_string();
            Ok(())
        } else {
            Err(SqlError::ExecutionError(format!(
                "Engine '{}' not found",
                name
            )))
        }
    }

    pub fn names(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
