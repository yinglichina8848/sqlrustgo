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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{DataType, Field, PhysicalPlan, Schema};
    use std::sync::Arc;

    struct MockPlan {
        schema: Schema,
        rows: Vec<Vec<Value>>,
    }

    impl MockPlan {
        fn new(schema: Schema, rows: Vec<Vec<Value>>) -> Self {
            Self { schema, rows }
        }
    }

    impl PhysicalPlan for MockPlan {
        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
            vec![]
        }

        fn execute(&self) -> SqlResult<Vec<Vec<Value>>> {
            Ok(self.rows.clone())
        }
    }

    #[test]
    fn test_default_execution_engine_new() {
        let engine = DefaultExecutionEngine::new();
        assert_eq!(engine.name(), "default");
    }

    #[test]
    fn test_default_execution_engine_execute() {
        let engine = DefaultExecutionEngine::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);
        let rows = vec![vec![Value::Integer(1)]];
        let plan = MockPlan::new(schema, rows);

        let result = engine.execute(&plan).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_engine_registry_new() {
        let registry = EngineRegistry::new();
        let names = registry.names();
        assert!(names.contains(&"default".to_string()));
    }

    #[test]
    fn test_engine_registry_register() {
        let mut registry = EngineRegistry::new();
        registry.register("custom", Box::new(DefaultExecutionEngine::new()));

        let names = registry.names();
        assert!(names.contains(&"custom".to_string()));
        assert!(names.contains(&"default".to_string()));
    }

    #[test]
    fn test_engine_registry_get() {
        let mut registry = EngineRegistry::new();
        registry.register("custom", Box::new(DefaultExecutionEngine::new()));

        let engine = registry.get("custom");
        assert!(engine.is_some());
        assert_eq!(engine.unwrap().name(), "default");
    }

    #[test]
    fn test_engine_registry_execute() {
        let registry = EngineRegistry::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);
        let rows = vec![vec![Value::Integer(1)]];
        let plan = MockPlan::new(schema, rows);

        let result = registry.execute(&plan).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_engine_registry_set_default() {
        let mut registry = EngineRegistry::new();
        registry.register("custom", Box::new(DefaultExecutionEngine::new()));

        let result = registry.set_default("custom");
        assert!(result.is_ok());
    }

    #[test]
    fn test_engine_registry_set_default_not_found() {
        let mut registry = EngineRegistry::new();
        let result = registry.set_default("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_registry_names() {
        let mut registry = EngineRegistry::new();
        registry.register("engine1", Box::new(DefaultExecutionEngine::new()));
        registry.register("engine2", Box::new(DefaultExecutionEngine::new()));

        let names = registry.names();
        assert_eq!(names.len(), 3);  // default + engine1 + engine2
    }

    #[test]
    fn test_engine_registry_set_default_change() {
        let mut registry = EngineRegistry::new();
        registry.register("custom", Box::new(DefaultExecutionEngine::new()));

        // Set default to custom
        let result = registry.set_default("custom");
        assert!(result.is_ok());

        // Verify default changed
        let engine = registry.get("custom");
        assert!(engine.is_some());
    }
}
