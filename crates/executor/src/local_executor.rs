//! LocalExecutor - Local query execution implementation
//!
//! Implements the Executor trait for local execution using StorageEngine.

use sqlrustgo_planner::{AggregateExec, AggregateFunction, PhysicalPlan, ProjectionExec};
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
        let table_name = plan.table_name();
        if table_name.is_empty() {
            return Ok(ExecutorResult::empty());
        }

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

        let child_result = self.execute(children[0])?;

        let aggregate = plan.as_any().downcast_ref::<AggregateExec>();

        let (group_expr, aggregate_expr) = match aggregate {
            Some(p) => (p.group_expr(), p.aggregate_expr()),
            None => return Ok(ExecutorResult::empty()),
        };

        if group_expr.is_empty() {
            let mut agg_results = vec![];

            for agg_expr in aggregate_expr {
                if let sqlrustgo_planner::Expr::AggregateFunction {
                    func,
                    args,
                    distinct: _,
                } = agg_expr
                {
                    let values: Vec<Value> = child_result
                        .rows
                        .iter()
                        .map(|row| {
                            if let Some(arg) = args.first() {
                                arg.evaluate(row, children[0].schema())
                                    .unwrap_or(Value::Null)
                            } else {
                                Value::Integer(child_result.rows.len() as i64)
                            }
                        })
                        .collect();

                    let result = self.compute_aggregate(func, &values);
                    agg_results.push(result);
                }
            }

            if !agg_results.is_empty() {
                return Ok(ExecutorResult::new(vec![agg_results], 0));
            }

            Ok(ExecutorResult::empty())
        } else {
            let mut groups: std::collections::HashMap<Vec<Value>, Vec<Vec<Value>>> =
                std::collections::HashMap::new();

            for row in &child_result.rows {
                let key: Vec<Value> = group_expr
                    .iter()
                    .map(|expr| {
                        expr.evaluate(row, children[0].schema())
                            .unwrap_or(Value::Null)
                    })
                    .collect();
                groups.entry(key).or_insert_with(Vec::new).push(row.clone());
            }

            let mut results = vec![];
            for (key, group_rows) in groups {
                let mut row = key;
                for agg_expr in aggregate_expr {
                    if let sqlrustgo_planner::Expr::AggregateFunction {
                        func,
                        args,
                        distinct: _,
                    } = agg_expr
                    {
                        let values: Vec<Value> = group_rows
                            .iter()
                            .map(|r| {
                                if let Some(arg) = args.first() {
                                    arg.evaluate(r, children[0].schema()).unwrap_or(Value::Null)
                                } else {
                                    Value::Integer(group_rows.len() as i64)
                                }
                            })
                            .collect();

                        let result = self.compute_aggregate(func, &values);
                        row.push(result);
                    }
                }
                results.push(row);
            }

            Ok(ExecutorResult::new(results, 0))
        }
    }

    fn compute_aggregate(&self, func: &AggregateFunction, values: &[Value]) -> Value {
        match func {
            AggregateFunction::Count => Value::Integer(values.len() as i64),
            AggregateFunction::Sum => {
                let mut sum: i64 = 0;
                for v in values {
                    if let Value::Integer(n) = v {
                        sum += n;
                    }
                }
                Value::Integer(sum)
            }
            AggregateFunction::Avg => {
                let mut sum: i64 = 0;
                let mut count = 0;
                for v in values {
                    if let Value::Integer(n) = v {
                        sum += n;
                        count += 1;
                    }
                }
                if count > 0 {
                    Value::Integer(sum / count as i64)
                } else {
                    Value::Null
                }
            }
            AggregateFunction::Min => {
                let mut min_val: Option<i64> = None;
                for v in values {
                    if let Value::Integer(n) = v {
                        match min_val {
                            Some(m) if *n < m => min_val = Some(*n),
                            None => min_val = Some(*n),
                            _ => {}
                        }
                    }
                }
                min_val.map(Value::Integer).unwrap_or(Value::Null)
            }
            AggregateFunction::Max => {
                let mut max_val: Option<i64> = None;
                for v in values {
                    if let Value::Integer(n) = v {
                        match max_val {
                            Some(m) if *n > m => max_val = Some(*n),
                            None => max_val = Some(*n),
                            _ => {}
                        }
                    }
                }
                max_val.map(Value::Integer).unwrap_or(Value::Null)
            }
        }
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

    #[test]
    fn test_execute_aggregate_count() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1)],
                    vec![Value::Integer(2)],
                    vec![Value::Integer(3)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "count".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Count,
                args: vec![],
                distinct: false,
            }],
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(3)]);
    }

    #[test]
    fn test_execute_aggregate_sum() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "sum".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(600)]);
    }

    #[test]
    fn test_execute_aggregate_avg() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "avg".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Avg,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(200)]);
    }

    #[test]
    fn test_execute_aggregate_with_group_by() {
        use sqlrustgo_planner::{AggregateExec, AggregateFunction, Expr};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Text("A".to_string()), Value::Integer(100)],
                    vec![Value::Text("A".to_string()), Value::Integer(200)],
                    vec![Value::Text("B".to_string()), Value::Integer(300)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![
            Field::new("category".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("category".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("sum".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let seq_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![Expr::column("category")],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Sum,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 2);
    }
}
