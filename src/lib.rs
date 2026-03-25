//! SQLRustGo Database System Library
//!
//! A Rust implementation of a SQL-92 compliant database system.
//! This crate re-exports functionality from the modular crates/ workspace.

pub use sqlrustgo_executor::{Executor, ExecutorResult};
pub use sqlrustgo_optimizer::Optimizer as QueryOptimizer;
pub use sqlrustgo_parser::lexer::tokenize;
pub use sqlrustgo_parser::{parse, Expression, Lexer, SetOperation, Statement, Token};
pub use sqlrustgo_planner::{LogicalPlan, Optimizer, PhysicalPlan, Planner, SetOperationType};
pub use sqlrustgo_storage::{
    BPlusTree, BufferPool, FileStorage, MemoryStorage, Page, StorageEngine,
};
pub use sqlrustgo_types::{SqlError, SqlResult, Value};

use std::sync::{Arc, RwLock};

pub struct ExecutionEngine {
    storage: Arc<RwLock<dyn StorageEngine>>,
}

impl ExecutionEngine {
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self { storage }
    }

    pub fn execute(&mut self, statement: Statement) -> Result<ExecutorResult, SqlError> {
        match statement {
            Statement::Insert(insert) => {
                let table_name = &insert.table;
                let mut storage = self.storage.write().unwrap();
                if !storage.has_table(table_name) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        table_name
                    )));
                }

                let records: Vec<Vec<Value>> = insert
                    .values
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|expr| match expr {
                                Expression::Literal(value) => Value::Text(value.clone()),
                                _ => Value::Null,
                            })
                            .collect()
                    })
                    .collect();

                storage.insert(table_name, records)?;
                Ok(ExecutorResult::new(vec![], insert.values.len()))
            }
            Statement::CreateTable(create) => {
                let mut storage = self.storage.write().unwrap();
                let columns: Vec<sqlrustgo_storage::ColumnDefinition> = create
                    .columns
                    .iter()
                    .map(|col| sqlrustgo_storage::ColumnDefinition {
                        name: col.name.clone(),
                        data_type: col.data_type.clone(),
                        nullable: col.nullable,
                        is_unique: false,
                    })
                    .collect();

                let table_info = sqlrustgo_storage::TableInfo {
                    name: create.name.clone(),
                    columns,
                };

                storage.create_table(&table_info)?;
                Ok(ExecutorResult::new(vec![], 0))
            }
            Statement::Select(select) => {
                let storage = self.storage.read().unwrap();
                if !storage.has_table(&select.table) {
                    return Err(SqlError::ExecutionError(format!(
                        "Table '{}' not found",
                        select.table
                    )));
                }
                let rows = storage.scan(&select.table).unwrap_or_default();
                Ok(ExecutorResult::new(rows, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }

    pub fn execute_plan(&self, plan: &dyn PhysicalPlan) -> Result<ExecutorResult, SqlError> {
        let storage = self.storage.read().unwrap();
        match plan.name() {
            "SeqScan" => {
                let rows = storage.scan(plan.table_name())?;
                Ok(ExecutorResult::new(rows, 0))
            }
            "Filter" => {
                let filter_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::FilterExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast FilterExec".to_string())
                    })?;

                let child = filter_plan.input();
                let input_result = self.execute_plan(child)?;

                let predicate = filter_plan.predicate();
                let schema = child.schema();
                let filtered_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .into_iter()
                    .filter(|row| predicate.matches(row, schema))
                    .collect();

                Ok(ExecutorResult::new(filtered_rows, 0))
            }
            "Projection" => {
                let proj_plan = plan
                    .as_any()
                    .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
                    .ok_or_else(|| {
                        SqlError::ExecutionError("Failed to downcast ProjectionExec".to_string())
                    })?;

                let child = proj_plan.input();
                let input_result = self.execute_plan(child)?;

                let exprs = proj_plan.expr();
                let _output_schema = plan.schema();
                let projected_rows: Vec<Vec<Value>> = input_result
                    .rows
                    .iter()
                    .map(|row| {
                        exprs
                            .iter()
                            .filter_map(|expr| expr.evaluate(row, child.schema()))
                            .collect()
                    })
                    .collect();

                Ok(ExecutorResult::new(projected_rows, 0))
            }
            _ => Ok(ExecutorResult::empty()),
        }
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(MemoryStorage::new())),
        }
    }
}

pub fn init() {
    println!("SQLRustGo Database System initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
    }

    #[test]
    fn test_module_exports() {
        let _ = tokenize("SELECT 1");
        let _ = parse("SELECT 1");
        let _ = Value::Integer(1);
    }

    #[test]
    fn test_sql_result_alias() {
        let result: SqlResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_optimizer_alias() {
        let _: Option<Box<dyn sqlrustgo_optimizer::Optimizer>> = None;
    }

    #[test]
    fn test_physical_plan_trait() {
        let _: Option<Box<dyn PhysicalPlan>> = None;
    }

    #[test]
    fn test_execution_engine_new() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execution_engine_default() {
        let mut engine = ExecutionEngine::default();
        engine
            .execute(sqlrustgo_parser::parse("CREATE TABLE users (id INTEGER)").unwrap())
            .unwrap();
        let stmt = sqlrustgo_parser::parse("SELECT * FROM users").unwrap();
        assert_eq!(engine.execute(stmt).unwrap().rows.len(), 0);
    }

    #[test]
    fn test_execute_plan_seqscan() {
        use sqlrustgo_planner::{DataType, Field, Schema, SeqScanExec};

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

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        let plan = SeqScanExec::new("users".to_string(), schema);
        let result = engine.execute_plan(&plan).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[0][1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_execute_plan_filter() {
        use sqlrustgo_planner::{DataType, Expr, Field, FilterExec, Operator, Schema, SeqScanExec};

        let mut storage = MemoryStorage::new();
        storage
            .insert(
                "users",
                vec![
                    vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    vec![Value::Integer(2), Value::Text("Bob".to_string())],
                    vec![Value::Integer(3), Value::Text("Charlie".to_string())],
                ],
            )
            .unwrap();

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let predicate = Expr::binary_expr(
            Expr::column("id"),
            Operator::Gt,
            Expr::literal(Value::Integer(1)),
        );
        let filter = FilterExec::new(Box::new(scan), predicate);
        let result = engine.execute_plan(&filter).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Integer(2));
    }

    #[test]
    fn test_execute_plan_projection() {
        use sqlrustgo_planner::{DataType, Expr, Field, ProjectionExec, Schema, SeqScanExec};

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

        let engine = ExecutionEngine::new(Arc::new(RwLock::new(storage)));
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let scan = SeqScanExec::new("users".to_string(), schema.clone());
        let proj_schema = Schema::new(vec![Field::new("name".to_string(), DataType::Text)]);
        let projection =
            ProjectionExec::new(Box::new(scan), vec![Expr::column("name")], proj_schema);
        let result = engine.execute_plan(&projection).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_storage_engine_export() {
        let _: Option<Box<dyn StorageEngine>> = None;
    }

    #[test]
    fn test_executor_export() {
        let _: Option<Box<dyn Executor>> = None;
    }

    #[test]
    fn test_planner_export() {
        let _: Option<Box<dyn Planner>> = None;
    }
}
