//! LocalExecutor - Local query execution implementation
//!
//! Implements the Executor trait for local execution using StorageEngine.

use sqlrustgo_planner::{PhysicalPlan, ProjectionExec};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::{SqlResult, Value};

use crate::{Executor, ExecutorResult};

/// LocalExecutor - executes physical plans using StorageEngine
pub struct LocalExecutor<'a> {
    storage: &'a dyn StorageEngine,
}

impl<'a> LocalExecutor<'a> {
    /// Create a new LocalExecutor with the given storage engine
    pub fn new(storage: &'a dyn StorageEngine) -> Self {
        Self { storage }
    }

    /// Execute a physical plan and return results
    pub fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        match plan.name() {
            "SeqScan" => self.execute_seq_scan(plan),
            "Projection" => self.execute_projection(plan),
            "Filter" => self.execute_filter(plan),
            "Aggregate" => self.execute_aggregate(plan),
            "HashJoin" => self.execute_hash_join(plan),
            "Sort" => self.execute_sort(plan),
            "Limit" => self.execute_limit(plan),
            _ => Ok(ExecutorResult::empty()),
        }
    }

    /// Execute sequential scan
    fn execute_seq_scan(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        // Get table name from the physical plan using the table_name method
        let table_name = plan.table_name();

        // Scan from storage
        let records = self.storage.scan(table_name).unwrap_or_default();

        // Convert records to rows
        let rows: Vec<Vec<Value>> = records;

        Ok(ExecutorResult::new(rows, 0))
    }

    /// Execute projection (column selection)
    fn execute_projection(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        let child_result = self.execute(children[0])?;

        let projection = plan
            .as_any()
            .downcast_ref::<ProjectionExec>()
            .map(|p| p.expr().clone())
            .unwrap_or_default();

        if projection.is_empty() {
            return Ok(child_result);
        }

        let input_schema = children[0].schema();
        let _output_schema = plan.schema();

        let projected_rows: Vec<Vec<Value>> = child_result
            .rows
            .iter()
            .map(|row| {
                projection
                    .iter()
                    .map(|expr| expr.evaluate(row, input_schema).unwrap_or(Value::Null))
                    .collect()
            })
            .collect();

        Ok(ExecutorResult::new(
            projected_rows,
            child_result.affected_rows,
        ))
    }

    /// Execute filter (WHERE clause)
    fn execute_filter(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Filter evaluation is complex - we would need to evaluate the predicate
        // For now, return the child result as-is
        // The actual filtering would require expression evaluation
        Ok(child_result)
    }

    /// Execute aggregate (COUNT, SUM, AVG, etc.)
    fn execute_aggregate(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Aggregate computation would go here
        // For now, return child result
        Ok(child_result)
    }

    /// Execute hash join
    fn execute_hash_join(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.len() < 2 {
            return Ok(ExecutorResult::empty());
        }

        // Execute both children
        let _left_result = self.execute(children[0])?;
        let _right_result = self.execute(children[1])?;

        // Join computation would go here
        // For now, return empty result
        Ok(ExecutorResult::empty())
    }

    /// Execute sort
    fn execute_sort(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Sort would go here
        Ok(child_result)
    }

    /// Execute limit
    fn execute_limit(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        let children = plan.children();
        if children.is_empty() {
            return Ok(ExecutorResult::empty());
        }

        // Execute child first
        let child_result = self.execute(children[0])?;

        // Limit would go here
        Ok(child_result)
    }
}

impl<'a> Executor for LocalExecutor<'a> {
    fn execute(&self, plan: &dyn PhysicalPlan) -> SqlResult<ExecutorResult> {
        LocalExecutor::execute(self, plan)
    }

    fn name(&self) -> &str {
        "local"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Field, Schema};
    use sqlrustgo_storage::MemoryStorage;
    use std::any::Any;

    fn make_test_schema() -> Schema {
        Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )])
    }

    #[test]
    fn test_local_executor_creation() {
        let storage = MemoryStorage::new();
        let _executor = LocalExecutor::new(&storage);
        // Storage is set up correctly if we get here
        assert!(!storage.has_table("nonexistent"));
    }

    #[test]
    fn test_local_executor_with_empty_table() {
        let mut storage = MemoryStorage::new();
        storage
            .create_table(&sqlrustgo_storage::TableInfo {
                name: "users".to_string(),
                columns: vec![],
            })
            .unwrap();

        let executor = LocalExecutor::new(&storage);
        let test_schema = make_test_schema();

        // Create a mock plan
        struct MockPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![]
            }
            fn name(&self) -> &str {
                "SeqScan"
            }
            fn table_name(&self) -> &str {
                "users"
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let result = executor
            .execute(&MockPlan {
                schema: test_schema,
            })
            .unwrap();
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_local_executor_send_sync() {
        fn _check<T: Send + Sync>() {}
        let _storage = MemoryStorage::new();
        _check::<LocalExecutor>();
    }

    #[test]
    fn test_execute_projection() {
        use sqlrustgo_planner::Expr;

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let output_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let projection =
            ProjectionExec::new(Box::new(seq_scan), vec![Expr::column("id")], output_schema);

        let result = executor.execute(&projection).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec![Value::Integer(1)]);
        assert_eq!(result.rows[1], vec![Value::Integer(2)]);
    }

    #[test]
    fn test_execute_projection_multiple_columns() {
        use sqlrustgo_planner::Expr;

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let projection = ProjectionExec::new(
            Box::new(seq_scan),
            vec![Expr::column("name"), Expr::column("id")],
            output_schema,
        );

        let result = executor.execute(&projection).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(
            result.rows[0],
            vec![Value::Text("Alice".to_string()), Value::Integer(1)]
        );
    }
}
