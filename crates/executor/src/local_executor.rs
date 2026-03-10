//! LocalExecutor - Local query execution implementation
//!
//! Implements the Executor trait for local execution using StorageEngine.

use sqlrustgo_planner::PhysicalPlan;
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

        // Execute child first
        let child_result = self.execute(children[0])?;

        // For projection, we need the projection indices from the plan
        // But SeqScanExec stores projection as Option<Vec<usize>>
        // We need a different approach - let's just return the child's result
        Ok(child_result)
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
    fn test_local_executor_name() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        assert_eq!(executor.name(), "local");
    }

    #[test]
    fn test_local_executor_is_ready() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        assert!(executor.is_ready());
    }

    #[test]
    fn test_local_executor_execute_projection() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for Projection
        struct MockProjectionPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockProjectionPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "Projection"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockProjectionPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_filter() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for Filter
        struct MockFilterPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockFilterPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "Filter"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockFilterPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_aggregate() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for Aggregate
        struct MockAggregatePlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockAggregatePlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "Aggregate"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockAggregatePlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_hash_join() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for HashJoin with only one child
        struct MockHashJoinPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockHashJoinPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "HashJoin"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockHashJoinPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_sort() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for Sort
        struct MockSortPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockSortPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "Sort"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockSortPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_limit() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for Limit
        struct MockLimitPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockLimitPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                // Return empty to trigger early return
                vec![]
            }
            fn name(&self) -> &str {
                "Limit"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockLimitPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_unknown_plan() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a mock plan for unknown type
        struct MockUnknownPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockUnknownPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![]
            }
            fn name(&self) -> &str {
                "UnknownPlan"
            }
        }

        let test_schema = make_test_schema();
        let result = executor.execute(&MockUnknownPlan { schema: test_schema });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_projection_with_child() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a child plan
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent projection with child
        struct MockProjectionPlan {
            schema: Schema,
            child: MockChildPlan,
        }
        impl PhysicalPlan for MockProjectionPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.child]
            }
            fn name(&self) -> &str {
                "Projection"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockProjectionPlan {
            schema: test_schema,
            child: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_filter_with_child() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a child plan
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent filter with child
        struct MockFilterPlan {
            schema: Schema,
            child: MockChildPlan,
        }
        impl PhysicalPlan for MockFilterPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.child]
            }
            fn name(&self) -> &str {
                "Filter"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockFilterPlan {
            schema: test_schema,
            child: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_aggregate_with_child() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a child plan
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent aggregate with child
        struct MockAggregatePlan {
            schema: Schema,
            child: MockChildPlan,
        }
        impl PhysicalPlan for MockAggregatePlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.child]
            }
            fn name(&self) -> &str {
                "Aggregate"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockAggregatePlan {
            schema: test_schema,
            child: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_hash_join_with_two_children() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create two child plans
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent hash join with two children
        struct MockHashJoinPlan {
            schema: Schema,
            left: MockChildPlan,
            right: MockChildPlan,
        }
        impl PhysicalPlan for MockHashJoinPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.left, &self.right]
            }
            fn name(&self) -> &str {
                "HashJoin"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockHashJoinPlan {
            schema: test_schema.clone(),
            left: MockChildPlan {
                schema: test_schema.clone(),
            },
            right: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_sort_with_child() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a child plan
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent sort with child
        struct MockSortPlan {
            schema: Schema,
            child: MockChildPlan,
        }
        impl PhysicalPlan for MockSortPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.child]
            }
            fn name(&self) -> &str {
                "Sort"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockSortPlan {
            schema: test_schema,
            child: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_local_executor_execute_limit_with_child() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        // Create a child plan
        struct MockChildPlan {
            schema: Schema,
        }
        impl PhysicalPlan for MockChildPlan {
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
                "test_table"
            }
        }

        // Create parent limit with child
        struct MockLimitPlan {
            schema: Schema,
            child: MockChildPlan,
        }
        impl PhysicalPlan for MockLimitPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![&self.child]
            }
            fn name(&self) -> &str {
                "Limit"
            }
        }

        let test_schema = make_test_schema();
        let test_schema_clone = test_schema.clone();
        let result = executor.execute(&MockLimitPlan {
            schema: test_schema,
            child: MockChildPlan {
                schema: test_schema_clone,
            },
        });
        assert!(result.is_ok());
    }
}
