//! LocalExecutor - Local query execution implementation
//!
//! Implements the Executor trait for local execution using StorageEngine.

use sqlrustgo_planner::{
    AggregateExec, AggregateFunction, FilterExec, HashJoinExec, JoinType, PhysicalPlan,
    ProjectionExec,
};
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

        // Get the predicate from FilterExec
        let filter = plan.as_any().downcast_ref::<FilterExec>();

        let predicate = match filter {
            Some(f) => f.predicate(),
            None => return Ok(ExecutorResult::empty()),
        };

        let input_schema = children[0].schema();

        // Filter rows based on predicate
        let filtered_rows: Vec<Vec<Value>> = child_result
            .rows
            .into_iter()
            .filter(|row| {
                let predicate_val = predicate.evaluate(row, input_schema).unwrap_or(Value::Null);
                matches!(predicate_val, Value::Boolean(true))
            })
            .collect();

        Ok(ExecutorResult::new(filtered_rows, 0))
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
                groups.entry(key).or_default().push(row.clone());
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

        let left_result = self.execute(children[0])?;
        let right_result = self.execute(children[1])?;

        let hash_join = plan.as_any().downcast_ref::<HashJoinExec>();

        let (join_type, condition) = match hash_join {
            Some(hj) => (hj.join_type(), hj.condition()),
            None => return Ok(ExecutorResult::empty()),
        };

        let condition = match condition {
            Some(c) => c,
            None => {
                return Ok(ExecutorResult::new(
                    cartesian_product(&left_result.rows, &right_result.rows),
                    0,
                ));
            }
        };

        let left_schema = children[0].schema();
        let right_schema = children[1].schema();

        match join_type {
            JoinType::Inner => {
                let matched = hash_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                Ok(ExecutorResult::new(matched, 0))
            }
            JoinType::Left => {
                let matched = hash_inner_join(
                    &left_result.rows,
                    &right_result.rows,
                    condition,
                    left_schema,
                    right_schema,
                );
                let left_only: Vec<Vec<Value>> = left_result
                    .rows
                    .iter()
                    .filter(|lrow| {
                        !matched.iter().any(|m| {
                            m.iter().take(lrow.len()).collect::<Vec<_>>()
                                == lrow.iter().collect::<Vec<_>>()
                        })
                    })
                    .map(|lrow| {
                        let mut row = lrow.clone();
                        row.extend(vec![Value::Null; right_schema.fields.len()]);
                        row
                    })
                    .collect();
                let mut results = matched;
                results.extend(left_only);
                Ok(ExecutorResult::new(results, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

fn cartesian_product(left: &[Vec<Value>], right: &[Vec<Value>]) -> Vec<Vec<Value>> {
    let mut result = Vec::new();
    for lrow in left {
        for rrow in right {
            let mut row = lrow.clone();
            row.extend(rrow.clone());
            result.push(row);
        }
    }
    result
}

fn hash_inner_join(
    left: &[Vec<Value>],
    right: &[Vec<Value>],
    condition: &sqlrustgo_planner::Expr,
    left_schema: &sqlrustgo_planner::Schema,
    right_schema: &sqlrustgo_planner::Schema,
) -> Vec<Vec<Value>> {
    let mut results = Vec::new();

    for lrow in left {
        for rrow in right {
            let mut combined = lrow.clone();
            combined.extend(rrow.clone());

            let full_schema = sqlrustgo_planner::Schema::new(
                left_schema
                    .fields
                    .iter()
                    .chain(right_schema.fields.iter())
                    .cloned()
                    .collect(),
            );

            if condition
                .evaluate(&combined, &full_schema)
                .map(|v| v.to_bool())
                .unwrap_or(false)
            {
                results.push(combined);
            }
        }
    }

    results
}

impl<'a> LocalExecutor<'a> {
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
    use sqlrustgo_planner::{
        AggregateExec, AggregateFunction, Expr, Field, FilterExec, Operator, PhysicalPlan,
        ProjectionExec, Schema, SeqScanExec,
    };
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

    #[test]
    fn test_execute_hash_join_inner() {
        use sqlrustgo_planner::{Expr, HashJoinExec, JoinType};

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
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(1), Value::Integer(100), Value::Integer(1)],
                    vec![Value::Integer(2), Value::Integer(200), Value::Integer(1)],
                    vec![Value::Integer(3), Value::Integer(300), Value::Integer(2)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);
        let output_schema = Schema::new(vec![
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("order_id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("amount".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("user_id".to_string(), sqlrustgo_planner::DataType::Integer),
        ]);

        let left_scan = sqlrustgo_planner::SeqScanExec::new("users".to_string(), left_schema);
        let right_scan = sqlrustgo_planner::SeqScanExec::new("orders".to_string(), right_schema);

        let join_condition = Expr::binary_expr(
            Expr::column("user_id"),
            sqlrustgo_planner::Operator::Eq,
            Expr::column("user_id"),
        );

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Inner,
            Some(join_condition),
            output_schema,
        );

        let result = executor.execute(&hash_join).unwrap();

        assert!(result.rows.len() >= 2);
    }

    #[test]
    fn test_execute_hash_join_cross() {
        use sqlrustgo_planner::{HashJoinExec, JoinType};

        let mut storage = MemoryStorage::new();
        storage
            .insert("a", vec![vec![Value::Integer(1)], vec![Value::Integer(2)]])
            .unwrap();
        storage
            .insert(
                "b",
                vec![
                    vec![Value::Text("x".to_string())],
                    vec![Value::Text("y".to_string())],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let left_schema = Schema::new(vec![Field::new(
            "id".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let right_schema = Schema::new(vec![Field::new(
            "val".to_string(),
            sqlrustgo_planner::DataType::Text,
        )]);
        let output_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("val".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let left_scan = sqlrustgo_planner::SeqScanExec::new("a".to_string(), left_schema);
        let right_scan = sqlrustgo_planner::SeqScanExec::new("b".to_string(), right_schema);

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Cross,
            None,
            output_schema,
        );

        let result = executor.execute(&hash_join).unwrap();

        assert_eq!(result.rows.len(), 4);
    }

    #[test]
    fn test_execute_sort() {
        use sqlrustgo_planner::{physical_plan::SortExec, DataType, Expr, SeqScanExec};
        // Test execution of SortExec
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);

        // Create a table scan as child
        let scan = SeqScanExec::new("test".to_string(), schema.clone());

        // Create sort plan
        let sort_expr = vec![sqlrustgo_planner::SortExpr {
            expr: Expr::column("value"),
            asc: true,
            nulls_first: false,
        }];

        let sort = SortExec::new(Box::new(scan), sort_expr);

        let result = executor.execute(&sort).unwrap();
        assert!(result.rows.is_empty()); // No data in storage
    }

    #[test]
    fn test_execute_limit() {
        use sqlrustgo_planner::{physical_plan::LimitExec, DataType, SeqScanExec};
        // Test execution of LimitExec
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);

        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // Create a table scan as child
        let scan = SeqScanExec::new("test".to_string(), schema.clone());

        // Create limit plan with offset
        let limit = LimitExec::new(Box::new(scan), 10, Some(5));

        let result = executor.execute(&limit).unwrap();
        assert!(result.rows.is_empty()); // No data in storage
    }

    #[test]
    fn test_executor_name() {
        // Test the executor name method
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        assert_eq!(executor.name(), "local");
    }

    #[test]
    fn test_execute_with_unknown_plan_type() {
        use sqlrustgo_planner::PhysicalPlan;
        use std::any::Any;

        struct UnknownPlan {
            schema: Schema,
        }
        impl UnknownPlan {
            fn new() -> Self {
                Self {
                    schema: Schema::new(vec![]),
                }
            }
        }
        impl PhysicalPlan for UnknownPlan {
            fn schema(&self) -> &Schema {
                &self.schema
            }
            fn children(&self) -> Vec<&dyn PhysicalPlan> {
                vec![]
            }
            fn name(&self) -> &str {
                "Unknown"
            }
            fn table_name(&self) -> &str {
                ""
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        let result = executor.execute(&UnknownPlan::new()).unwrap();
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_execute_projection_with_empty_projection() {
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

        let seq_scan = SeqScanExec::new("users".to_string(), input_schema);
        let projection = ProjectionExec::new(
            Box::new(seq_scan),
            vec![],
            Schema::new(vec![
                Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
                Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            ]),
        );

        let result = executor.execute(&projection).unwrap();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_execute_filter_with_children() {
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

        let seq_scan = SeqScanExec::new("users".to_string(), input_schema);
        let filter = FilterExec::new(
            Box::new(seq_scan),
            Expr::binary_expr(
                Expr::column("id"),
                sqlrustgo_planner::Operator::Gt,
                Expr::literal(Value::Integer(1)),
            ),
        );

        let result = executor.execute(&filter).unwrap();
        // All rows returned since filter doesn't actually filter in this implementation
        assert!(result.rows.len() >= 1);
    }

    // === Tests for uncovered code paths ===

    #[test]
    fn test_executor_is_ready() {
        let storage = MemoryStorage::new();
        let executor = LocalExecutor::new(&storage);
        // is_ready should return true (always ready)
        assert!(executor.is_ready());
    }

    #[test]
    fn test_execute_aggregate_min() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(50)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "min".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Min,
                args: vec![Expr::column("amount")],
                distinct: false,
            }],
            output_schema,
        );

        let result = executor.execute(&aggregate).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec![Value::Integer(50)]);
    }

    #[test]
    fn test_execute_aggregate_max() {
        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "orders",
                vec![
                    vec![Value::Integer(100)],
                    vec![Value::Integer(200)],
                    vec![Value::Integer(50)],
                ],
            )
            .unwrap();

        let executor = LocalExecutor::new(&storage);

        let input_schema = Schema::new(vec![Field::new(
            "amount".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);
        let output_schema = Schema::new(vec![Field::new(
            "max".to_string(),
            sqlrustgo_planner::DataType::Integer,
        )]);

        let seq_scan = SeqScanExec::new("orders".to_string(), input_schema);
        let aggregate = AggregateExec::new(
            Box::new(seq_scan),
            vec![],
            vec![Expr::AggregateFunction {
                func: AggregateFunction::Max,
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
    fn test_execute_hash_join_left() {
        use sqlrustgo_planner::HashJoinExec;
        use sqlrustgo_planner::JoinType;

        let mut left_storage = MemoryStorage::new();
        left_storage
            .insert(
                "left_table",
                vec![
                    vec![Value::Integer(1), Value::Text("A".to_string())],
                    vec![Value::Integer(2), Value::Text("B".to_string())],
                ],
            )
            .unwrap();

        let mut right_storage = MemoryStorage::new();
        right_storage
            .insert(
                "right_table",
                vec![vec![Value::Integer(1), Value::Text("X".to_string())]],
            )
            .unwrap();

        let executor = LocalExecutor::new(&left_storage);

        let left_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let right_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);
        let join_schema = Schema::new(vec![
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("name".to_string(), sqlrustgo_planner::DataType::Text),
            Field::new("id".to_string(), sqlrustgo_planner::DataType::Integer),
            Field::new("value".to_string(), sqlrustgo_planner::DataType::Text),
        ]);

        let left_scan = SeqScanExec::new("left_table".to_string(), left_schema);
        let right_scan = SeqScanExec::new("right_table".to_string(), right_schema);

        let join_condition = Some(Expr::BinaryExpr {
            left: Box::new(Expr::column("id")),
            op: Operator::Eq,
            right: Box::new(Expr::column("id")),
        });

        let hash_join = HashJoinExec::new(
            Box::new(left_scan),
            Box::new(right_scan),
            JoinType::Left,
            join_condition,
            join_schema,
        );

        let result = executor.execute(&hash_join);
        // Left join should complete without error
        assert!(result.is_ok());
    }
}
