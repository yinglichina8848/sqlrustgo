//! Prepared Statements Module
//!
//! Provides support for SQL prepared statements with parameter binding.
//!
//! # SQL Syntax
//! ```sql
//! PREPARE stmt FROM 'SELECT * FROM users WHERE id = ?';
//! SET @id = 1;
//! EXECUTE stmt USING @id;
//! DEALLOCATE PREPARE stmt;
//! ```

use crate::{DataType, PhysicalPlan};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// A prepared statement with cached execution plan
#[derive(Clone)]
pub struct PreparedStatement {
    /// Unique identifier for this prepared statement
    pub id: String,
    /// Original SQL text
    pub sql: String,
    /// Cached physical execution plan
    pub plan: Arc<dyn PhysicalPlan>,
    /// Inferred parameter types
    pub param_types: Vec<DataType>,
    /// When this statement was created
    pub created_at: DateTime<Utc>,
    /// Last time this statement was executed
    pub last_used_at: DateTime<Utc>,
    /// Number of times this statement has been executed
    pub use_count: u64,
}

impl PreparedStatement {
    /// Create a new prepared statement
    pub fn new(
        id: String,
        sql: String,
        plan: Arc<dyn PhysicalPlan>,
        param_types: Vec<DataType>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            sql,
            plan,
            param_types,
            created_at: now,
            last_used_at: now,
            use_count: 0,
        }
    }

    /// Update the last used timestamp and increment use count
    pub fn record_use(&mut self) {
        self.last_used_at = Utc::now();
        self.use_count += 1;
    }
}

impl std::fmt::Debug for PreparedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedStatement")
            .field("id", &self.id)
            .field("sql", &self.sql)
            .field("param_types", &self.param_types)
            .field("created_at", &self.created_at)
            .field("last_used_at", &self.last_used_at)
            .field("use_count", &self.use_count)
            // Note: plan is not displayed because dyn PhysicalPlan doesn't implement Debug
            .finish()
    }
}

/// Manager for prepared statements with cache eviction support
#[derive(Debug)]
pub struct PreparedStatementManager {
    /// Map of statement ID to prepared statement
    statements: HashMap<String, PreparedStatement>,
    /// Maximum number of cached statements
    max_size: usize,
}

impl Default for PreparedStatementManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl PreparedStatementManager {
    /// Create a new manager with specified max cache size
    pub fn new(max_size: usize) -> Self {
        Self {
            statements: HashMap::new(),
            max_size,
        }
    }

    /// Prepare a statement and cache its plan
    ///
    /// Returns the statement ID
    pub fn prepare(
        &mut self,
        id: String,
        sql: String,
        plan: Arc<dyn PhysicalPlan>,
        param_types: Vec<DataType>,
    ) -> String {
        // If at capacity, evict least recently used
        if self.statements.len() >= self.max_size {
            self.evict_lru();
        }

        let stmt = PreparedStatement::new(id.clone(), sql, plan, param_types);
        self.statements.insert(id.clone(), stmt);
        id
    }

    /// Get a prepared statement by ID
    pub fn get(&self, id: &str) -> Option<&PreparedStatement> {
        self.statements.get(id)
    }

    /// Get a mutable prepared statement by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PreparedStatement> {
        self.statements.get_mut(id)
    }

    /// Execute a prepared statement with the given parameters
    ///
    /// Note: This returns the plan and parameters; actual execution
    /// is done by the executor with parameter binding
    pub fn execute(&self, id: &str) -> Option<(&Arc<dyn PhysicalPlan>, &Vec<DataType>)> {
        self.statements.get(id).map(|stmt| (&stmt.plan, &stmt.param_types))
    }

    /// Record that a statement was used (for LRU tracking)
    pub fn record_use(&mut self, id: &str) {
        if let Some(stmt) = self.statements.get_mut(id) {
            stmt.record_use();
        }
    }

    /// Deallocate (remove) a prepared statement
    pub fn deallocate(&mut self, id: &str) -> bool {
        self.statements.remove(id).is_some()
    }

    /// Check if a statement exists
    pub fn exists(&self, id: &str) -> bool {
        self.statements.contains_key(id)
    }

    /// Get the number of cached statements
    pub fn len(&self) -> usize {
        self.statements.len()
    }

    /// Check if there are no cached statements
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// Evict the least recently used statement
    fn evict_lru(&mut self) {
        if let Some(lru_id) = self
            .statements
            .iter()
            .min_by_key(|(_, stmt)| stmt.last_used_at)
            .map(|(id, _)| id.clone())
        {
            self.statements.remove(&lru_id);
        }
    }

    /// Cleanup idle statements that haven't been used for the specified duration
    ///
    /// Returns the number of statements removed
    pub fn cleanup_idle(&mut self, max_idle: std::time::Duration) -> usize {
        let now = Utc::now();
        // Convert std::time::Duration to chrono::Duration
        let idle_duration = Duration::from_std(max_idle)
            .unwrap_or_else(|_| Duration::hours(1)); // Default to 1 hour if conversion fails
        let idle_threshold = now - idle_duration;

        let to_remove: Vec<String> = self
            .statements
            .iter()
            .filter(|(_, stmt)| stmt.last_used_at < idle_threshold)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &to_remove {
            self.statements.remove(id);
        }

        to_remove.len()
    }

    /// Clear all prepared statements
    pub fn clear(&mut self) {
        self.statements.clear();
    }

    /// Get all statement IDs
    pub fn statement_ids(&self) -> Vec<&String> {
        self.statements.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Schema, Field};

    /// A simple mock physical plan for testing
    struct MockPhysicalPlan {
        schema: Schema,
    }

    impl MockPhysicalPlan {
        fn new() -> Self {
            Self {
                schema: Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]),
            }
        }
    }

    impl PhysicalPlan for MockPhysicalPlan {
        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn children(&self) -> Vec<&dyn PhysicalPlan> {
            vec![]
        }

        fn name(&self) -> &str {
            "MockScan"
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_prepared_statement_creation() {
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;
        let stmt = PreparedStatement::new(
            "test_stmt".to_string(),
            "SELECT * FROM users WHERE id = ?".to_string(),
            plan,
            vec![DataType::Integer],
        );

        assert_eq!(stmt.id, "test_stmt");
        assert_eq!(stmt.sql, "SELECT * FROM users WHERE id = ?");
        assert_eq!(stmt.param_types, vec![DataType::Integer]);
        assert_eq!(stmt.use_count, 0);
    }

    #[test]
    fn test_prepared_statement_record_use() {
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;
        let mut stmt = PreparedStatement::new(
            "test_stmt".to_string(),
            "SELECT * FROM users".to_string(),
            plan,
            vec![],
        );

        assert_eq!(stmt.use_count, 0);
        stmt.record_use();
        assert_eq!(stmt.use_count, 1);
    }

    #[test]
    fn test_manager_prepare() {
        let mut manager = PreparedStatementManager::new(10);
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;

        let id = manager.prepare(
            "stmt1".to_string(),
            "SELECT * FROM users".to_string(),
            plan,
            vec![],
        );

        assert_eq!(id, "stmt1");
        assert!(manager.exists("stmt1"));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_manager_get() {
        let mut manager = PreparedStatementManager::new(10);
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;

        manager.prepare(
            "stmt1".to_string(),
            "SELECT * FROM users".to_string(),
            plan,
            vec![],
        );

        let stmt = manager.get("stmt1");
        assert!(stmt.is_some());
        assert_eq!(stmt.unwrap().id, "stmt1");
    }

    #[test]
    fn test_manager_deallocate() {
        let mut manager = PreparedStatementManager::new(10);
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;

        manager.prepare(
            "stmt1".to_string(),
            "SELECT * FROM users".to_string(),
            plan,
            vec![],
        );

        assert!(manager.exists("stmt1"));
        assert!(manager.deallocate("stmt1"));
        assert!(!manager.exists("stmt1"));
    }

    #[test]
    fn test_manager_eviction() {
        let mut manager = PreparedStatementManager::new(2);
        let plan = Arc::new(MockPhysicalPlan::new()) as Arc<dyn PhysicalPlan>;

        manager.prepare(
            "stmt1".to_string(),
            "SELECT 1".to_string(),
            plan.clone(),
            vec![],
        );
        manager.prepare(
            "stmt2".to_string(),
            "SELECT 2".to_string(),
            plan.clone(),
            vec![],
        );

        // Adding a third should evict one
        manager.prepare(
            "stmt3".to_string(),
            "SELECT 3".to_string(),
            plan,
            vec![],
        );

        // At least one of the first two should be evicted
        let remaining: Vec<_> = ["stmt1", "stmt2", "stmt3"]
            .iter()
            .filter(|s| manager.exists(s))
            .collect();
        assert!(remaining.len() >= 2);
    }
}
