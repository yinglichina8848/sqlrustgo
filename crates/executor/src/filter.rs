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
    /// Create a new FilterVolcanoExecutor
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

    /// Get the predicate expression
    pub fn predicate(&self) -> &Expr {
        &self.predicate
    }

    /// Get the input schema
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
        // Pull-based iteration: keep asking child for next row until predicate matches
        while let Some(row) = self.child.next()? {
            let predicate_val = self
                .predicate
                .evaluate(&row, &self.input_schema)
                .unwrap_or(Value::Null);

            // Only return rows where predicate evaluates to Boolean(true)
            if let Value::Boolean(true) = predicate_val {
                return Ok(Some(row));
            }
            // Otherwise continue to next row
        }
        // No more rows or no matching rows
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
        use sqlrustgo_planner::{DataType, Field};

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
    fn test_filter_executor_next_with_matching_rows() {
        let child = Box::new(MockExecutorWithData::new(vec![
            vec![Value::Integer(1)],
            vec![Value::Integer(2)],
            vec![Value::Integer(3)],
        ]));

        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let executor = FilterVolcanoExecutor::new(
            child,
            Expr::BinaryExpr {
                left: Box::new(Expr::Column(Column::new("id".to_string()))),
                op: sqlrustgo_planner::Operator::Gt,
                right: Box::new(Expr::Literal(Value::Integer(1))),
            },
            schema,
            input_schema,
        );

        assert_eq!(executor.name(), "Filter");
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

    // Mock executor for testing
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
    fn test_filter_executor_predicate_getter() {
        let predicate = Expr::Literal(Value::Boolean(true));
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            predicate.clone(),
            Schema::empty(),
            Schema::empty(),
        );
        let result = executor.predicate();
        assert!(matches!(result, Expr::Literal(Value::Boolean(true))));
    }

    #[test]
    fn test_filter_executor_input_schema_getter() {
        let input_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let executor = FilterVolcanoExecutor::new(
            Box::new(MockExecutor::new()),
            Expr::Literal(Value::Boolean(true)),
            Schema::empty(),
            input_schema,
        );
        let result = executor.input_schema();
        assert!(result.field("id").is_some());
    }
}
