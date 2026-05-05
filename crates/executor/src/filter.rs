//! Filter Executor - Volcano Model implementation for filter operations
//!
//! This module provides the FilterVolcanoExecutor which implements the VolcanoExecutor trait
//! for filtering rows based on a predicate expression.

use sqlrustgo_planner::{Expr, Schema};
use sqlrustgo_types::{SqlResult, Value};
use std::any::Any;

use super::VolcanoExecutor;

/// FilterVolcanoExecutor - executes filter (WHERE) operations
///
/// This executor takes a child executor and a predicate expression.
/// It only returns rows where the predicate evaluates to true.
pub struct FilterVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    predicate: Expr,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
}

impl FilterVolcanoExecutor {
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        predicate: Expr,
        schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            child,
            predicate,
            schema,
            input_schema,
            initialized: false,
        }
    }

    pub fn predicate(&self) -> &Expr {
        &self.predicate
    }

    pub fn input_schema(&self) -> &Schema {
        &self.input_schema
    }
}

impl VolcanoExecutor for FilterVolcanoExecutor {
    fn init(&mut self) -> SqlResult<()> {
        self.child.init()?;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        while let Some(row) = self.child.next()? {
            let predicate_val = self.predicate.evaluate(&row, &self.input_schema);

            match predicate_val {
                Some(Value::Boolean(true)) => return Ok(Some(row)),
                Some(Value::Null) => {
                    if self.predicate.contains_subquery() {
                        return Ok(Some(row));
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> SqlResult<()> {
        self.child.close()?;
        self.initialized = false;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "Filter"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{Column, DataType, Field};
    use sqlrustgo_types::Value;

    struct MockExecutor {
        schema: Schema,
        initialized: bool,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                schema: Schema::empty(),
                initialized: false,
            }
        }
    }

    impl VolcanoExecutor for MockExecutor {
        fn init(&mut self) -> SqlResult<()> {
            self.initialized = true;
            Ok(())
        }

        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            Ok(None)
        }

        fn close(&mut self) -> SqlResult<()> {
            self.initialized = false;
            Ok(())
        }

        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn name(&self) -> &str {
            "Mock"
        }

        fn is_initialized(&self) -> bool {
            self.initialized
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct MockExecutorWithData {
        data: Vec<Vec<Value>>,
        idx: usize,
        initialized: bool,
        schema: Schema,
    }

    impl MockExecutorWithData {
        fn new(data: Vec<Vec<Value>>) -> Self {
            Self {
                data,
                idx: 0,
                initialized: false,
                schema: Schema::empty(),
            }
        }
    }

    impl VolcanoExecutor for MockExecutorWithData {
        fn init(&mut self) -> SqlResult<()> {
            self.initialized = true;
            Ok(())
        }

        fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
            if self.idx >= self.data.len() {
                return Ok(None);
            }
            let row = self.data[self.idx].clone();
            self.idx += 1;
            Ok(Some(row))
        }

        fn close(&mut self) -> SqlResult<()> {
            self.initialized = false;
            self.idx = 0;
            Ok(())
        }

        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn name(&self) -> &str {
            "MockWithData"
        }

        fn is_initialized(&self) -> bool {
            self.initialized
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_filter_executor_name() {
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            Schema::empty(),
            Schema::empty(),
        );
        assert_eq!(executor.name(), "Filter");
    }

    #[test]
    fn test_filter_executor_schema() {
        let fields = vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ];
        let schema = Schema::new(fields);
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            schema.clone(),
            Schema::empty(),
        );
        assert_eq!(executor.schema().fields.len(), 2);
    }

    #[test]
    fn test_filter_executor_init() {
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            Schema::empty(),
            Schema::empty(),
        );
        assert!(!executor.is_initialized());
    }

    #[test]
    fn test_filter_executor_next_filters_false() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::Literal(Value::Boolean(false)),
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_executor_next_filters_matching() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::Gt,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()[0], Value::Integer(2));
    }

    #[test]
    fn test_filter_executor_close() {
        let child = Box::new(MockExecutorWithData::new(vec![]));
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::Literal(Value::Boolean(true)),
            schema,
            input_schema,
        );

        executor.init().unwrap();
        executor.close().unwrap();
        assert!(!executor.is_initialized());
    }

    #[test]
    fn test_filter_executor_as_any() {
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            Schema::empty(),
            Schema::empty(),
        );
        let any = executor.as_any();
        assert!(any.is::<FilterVolcanoExecutor>());
    }

    #[test]
    fn test_filter_executor_predicate_and_input_schema() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            Schema::empty(),
            input_schema.clone(),
        );
        assert_eq!(executor.input_schema().fields.len(), 1);
    }

    #[test]
    fn test_filter_with_empty_child() {
        let child = Box::new(MockExecutorWithData::new(vec![]));
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::Literal(Value::Boolean(true)),
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let result = executor.next().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_filter_all_rows_match() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::Literal(Value::Boolean(true)),
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let r1 = executor.next().unwrap();
        let r2 = executor.next().unwrap();
        let r3 = executor.next().unwrap();
        let r4 = executor.next().unwrap();

        assert!(r1.is_some() && r2.is_some() && r3.is_some());
        assert!(r4.is_none());
    }

    #[test]
    fn test_filter_multiple_rows_some_match() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
            vec![Value::Integer(4)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::Gt,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0][0], Value::Integer(3));
        assert_eq!(results[1][0], Value::Integer(4));
    }

    #[test]
    fn test_filter_less_than() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::Lt,
                right: Box::new(Expr::Literal(Value::Integer(3))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_equals() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::Eq,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_not_equals() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::NotEq,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_and_expression() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1), Value::Integer(10)],
            vec![Value::Integer(2), Value::Integer(20)],
            vec![Value::Integer(3), Value::Integer(30)],
        ]));

        let input_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("value".to_string(), DataType::Integer),
        ]);
        let schema = input_schema.clone();

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Column(Column::new("id".to_string()))),
                    op: sqlrustgo_planner::Operator::Gt,
                    right: Box::new(Expr::Literal(Value::Integer(1))),
                }),
                op: sqlrustgo_planner::Operator::And,
                right: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Column(Column::new("value".to_string()))),
                    op: sqlrustgo_planner::Operator::Lt,
                    right: Box::new(Expr::Literal(Value::Integer(30))),
                }),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0][0], Value::Integer(2));
    }

    #[test]
    fn test_filter_or_expression() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Column(Column::new("id".to_string()))),
                    op: sqlrustgo_planner::Operator::Eq,
                    right: Box::new(Expr::Literal(Value::Integer(1))),
                }),
                op: sqlrustgo_planner::Operator::Or,
                right: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Column(Column::new("id".to_string()))),
                    op: sqlrustgo_planner::Operator::Eq,
                    right: Box::new(Expr::Literal(Value::Integer(3))),
                }),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_greater_than_or_equal() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::GtEq,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_less_than_or_equal() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::LtEq,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_with_text_column() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Text("apple".to_string())],
            vec![Value::Text("banana".to_string())],
            vec![Value::Text("cherry".to_string())],
        ]));

        let input_schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);
        let schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);

        let mut executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("name".to_string()))),
                op: sqlrustgo_planner::Operator::Like,
                right: Box::new(Expr::Literal(Value::Text("b%".to_string()))),
            },
            schema,
            input_schema,
        );

        executor.init().unwrap();
        let results: Vec<_> = std::iter::from_fn(|| executor.next().unwrap())
            .collect();

        assert_eq!(results.len(), 1);
    }
}
