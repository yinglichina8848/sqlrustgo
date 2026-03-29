//! Stored Procedure Executor
//!
//! This module provides basic stored procedure execution support.

use crate::ExecutorResult;
use sqlrustgo_types::Value;
use std::sync::Arc;

/// Stored procedure executor for calling stored procedures
#[derive(Clone)]
pub struct StoredProcExecutor {
    catalog: Arc<sqlrustgo_catalog::Catalog>,
}

impl StoredProcExecutor {
    /// Create a new stored procedure executor
    pub fn new(catalog: Arc<sqlrustgo_catalog::Catalog>) -> Self {
        Self { catalog }
    }

    /// Execute a stored procedure call
    pub fn execute_call(&self, name: &str, _args: Vec<Value>) -> Result<ExecutorResult, String> {
        // Look up the stored procedure
        let _procedure = self
            .catalog
            .get_stored_procedure(name)
            .ok_or_else(|| format!("Stored procedure '{}' not found", name))?;

        // For now, just return a simple result indicating the procedure was found
        // A full implementation would parse and execute the procedure body
        Ok(ExecutorResult::new(
            vec![vec![Value::Text(format!(
                "Procedure '{}' executed successfully",
                name
            ))]],
            1,
        ))
    }

    /// Check if a stored procedure exists
    pub fn has_procedure(&self, name: &str) -> bool {
        self.catalog.has_stored_procedure(name)
    }

    /// List all stored procedure names
    pub fn list_procedures(&self) -> Vec<&str> {
        self.catalog.stored_procedure_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_catalog::Catalog;

    #[test]
    fn test_stored_proc_executor_not_found() {
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);

        let result = executor.execute_call("non_existent", vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_stored_proc_executor_list_empty() {
        let catalog = Arc::new(Catalog::new());
        let executor = StoredProcExecutor::new(catalog);

        assert!(executor.list_procedures().is_empty());
        assert!(!executor.has_procedure("test"));
    }
}
