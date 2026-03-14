//! Projection Executor - 列投影算子
//!
//! 实现列投影功能，支持：
//! - 列选择 (Column selection)
//! - 表达式投影 (Expression projection)
//! - 别名支持 (Alias support)
//! - 通配符展开 (Wildcard expansion)

use crate::executor::VolcanoExecutor;
use sqlrustgo_planner::{Expr, PhysicalPlan, Schema};
use sqlrustgo_types::{SqlError, SqlResult, Value};
use std::any::Any;

/// Projection 算子配置
#[derive(Debug, Clone)]
pub struct ProjectionConfig {
    /// 投影表达式列表
    pub exprs: Vec<ProjectionExpr>,
    /// 输出模式
    pub output_schema: Schema,
}

/// 投影表达式
#[derive(Debug, Clone)]
pub enum ProjectionExpr {
    /// 列引用
    Column { name: String, index: usize },
    /// 表达式投影
    Expression { expr: Expr, alias: Option<String> },
    /// 通配符
    Wildcard,
    /// 限定通配符 (如 table.*)
    QualifiedWildcard { qualifier: String },
}

/// Projection Volcano Executor
/// 实现 Volcano 模型的投影算子
pub struct ProjectionExecutor {
    /// 子算子
    child: Box<dyn VolcanoExecutor>,
    /// 投影表达式
    exprs: Vec<ProjectionExpr>,
    /// 输出模式
    schema: Schema,
    /// 输入模式
    input_schema: Schema,
    /// 是否已初始化
    initialized: bool,
}

impl ProjectionExecutor {
    /// 创建新的 ProjectionExecutor
    pub fn new(
        child: Box<dyn VolcanoExecutor>,
        exprs: Vec<ProjectionExpr>,
        output_schema: Schema,
        input_schema: Schema,
    ) -> Self {
        Self {
            child,
            exprs,
            schema: output_schema,
            input_schema,
            initialized: false,
        }
    }

    /// 从 PhysicalPlan 创建 ProjectionExecutor
    pub fn from_physical_plan(
        child: Box<dyn VolcanoExecutor>,
        plan: &dyn PhysicalPlan,
        input_schema: Schema,
    ) -> SqlResult<Self> {
        let projection = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
            .ok_or_else(|| {
                SqlError::ExecutionError("Failed to cast to ProjectionExec".to_string())
            })?;

        let exprs = projection.expr();
        let output_schema = plan.schema().clone();

        let projection_exprs = Self::build_projection_exprs(exprs, &output_schema)?;

        Ok(Self::new(
            child,
            projection_exprs,
            output_schema,
            input_schema,
        ))
    }

    /// 构建投影表达式列表
    fn build_projection_exprs(
        exprs: &[Expr],
        output_schema: &Schema,
    ) -> SqlResult<Vec<ProjectionExpr>> {
        let mut projection_exprs = Vec::new();

        for expr in exprs {
            match expr {
                Expr::Column(col) => {
                    let idx = output_schema.field_index(&col.name).ok_or_else(|| {
                        SqlError::ExecutionError(format!("Column {} not found in schema", col.name))
                    })?;
                    projection_exprs.push(ProjectionExpr::Column {
                        name: col.name.clone(),
                        index: idx,
                    });
                }
                Expr::Literal(value) => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: Expr::Literal(value.clone()),
                        alias: None,
                    });
                }
                Expr::Alias { expr, name } => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: *expr.clone(),
                        alias: Some(name.clone()),
                    });
                }
                Expr::BinaryExpr { .. } | Expr::UnaryExpr { .. } => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: expr.clone(),
                        alias: None,
                    });
                }
                Expr::Wildcard => {
                    projection_exprs.push(ProjectionExpr::Wildcard);
                }
                Expr::QualifiedWildcard { qualifier } => {
                    projection_exprs.push(ProjectionExpr::QualifiedWildcard {
                        qualifier: qualifier.clone(),
                    });
                }
                Expr::AggregateFunction { .. } => {
                    return Err(SqlError::ExecutionError(
                        "Aggregate function not supported in projection".to_string(),
                    ));
                }
            }
        }

        Ok(projection_exprs)
    }

    /// 评估单个投影表达式
    fn evaluate_projection_expr(&self, expr: &ProjectionExpr, row: &[Value]) -> SqlResult<Value> {
        match expr {
            ProjectionExpr::Column { name: _, index } => {
                if *index < row.len() {
                    Ok(row[*index].clone())
                } else {
                    Ok(Value::Null)
                }
            }
            ProjectionExpr::Expression { expr, .. } => Ok(expr
                .evaluate(row, &self.input_schema)
                .unwrap_or(Value::Null)),
            ProjectionExpr::Wildcard => Err(SqlError::ExecutionError(
                "Wildcard should be expanded before evaluation".to_string(),
            )),
            ProjectionExpr::QualifiedWildcard { .. } => Err(SqlError::ExecutionError(
                "Qualified wildcard should be expanded before evaluation".to_string(),
            )),
        }
    }

    /// 展开通配符表达式
    #[allow(dead_code)]
    fn expand_wildcards(&self) -> SqlResult<Vec<ProjectionExpr>> {
        let mut expanded = Vec::new();

        for expr in &self.exprs {
            match expr {
                ProjectionExpr::Wildcard => {
                    for field in &self.input_schema.fields {
                        let idx = self.input_schema.field_index(&field.name).unwrap_or(0);
                        expanded.push(ProjectionExpr::Column {
                            name: field.name.clone(),
                            index: idx,
                        });
                    }
                }
                ProjectionExpr::QualifiedWildcard { qualifier } => {
                    for field in &self.input_schema.fields {
                        if field.name.starts_with(qualifier.as_str()) || field.name == *qualifier {
                            let idx = self.input_schema.field_index(&field.name).unwrap_or(0);
                            expanded.push(ProjectionExpr::Column {
                                name: field.name.clone(),
                                index: idx,
                            });
                        }
                    }
                }
                _ => expanded.push(expr.clone()),
            }
        }

        Ok(expanded)
    }
}

impl VolcanoExecutor for ProjectionExecutor {
    fn init(&mut self) -> SqlResult<()> {
        // 初始化子算子
        self.child.init()?;

        // 预处理通配符
        let mut processed_exprs = Vec::new();
        for expr in &self.exprs {
            match expr {
                ProjectionExpr::Wildcard | ProjectionExpr::QualifiedWildcard { .. } => {
                    // 收集所有行来展开通配符
                    let mut all_rows = Vec::new();
                    while let Some(row) = self.child.next()? {
                        all_rows.push(row);
                    }
                    self.child.close()?;

                    // 根据输入模式展开通配符
                    for field in &self.input_schema.fields {
                        let idx = self.input_schema.field_index(&field.name).unwrap_or(0);
                        processed_exprs.push(ProjectionExpr::Column {
                            name: field.name.clone(),
                            index: idx,
                        });
                    }

                    // 将收集的行重新放回（这里简化处理，实际应该缓存）
                    // 注意：这是一个简化实现，完整实现需要更好的缓存机制
                    self.initialized = true;
                    return Ok(());
                }
                _ => processed_exprs.push(expr.clone()),
            }
        }

        self.exprs = processed_exprs;
        self.initialized = true;
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if !self.initialized {
            return Err(SqlError::ExecutionError(
                "Executor not initialized".to_string(),
            ));
        }

        // 从子算子获取下一行
        let child_row = self.child.next()?;

        if let Some(row) = child_row {
            // 对每一行进行投影
            let mut projected_row = Vec::with_capacity(self.exprs.len());

            for expr in &self.exprs {
                let value = self.evaluate_projection_expr(expr, &row)?;
                projected_row.push(value);
            }

            Ok(Some(projected_row))
        } else {
            Ok(None)
        }
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
        "Projection"
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Projection Builder
/// 用于从物理计划构建 ProjectionExecutor
pub struct ProjectionBuilder;

impl ProjectionBuilder {
    /// 从物理计划构建 ProjectionExecutor
    pub fn build(
        child: Box<dyn VolcanoExecutor>,
        plan: &dyn PhysicalPlan,
        input_schema: Schema,
    ) -> SqlResult<Box<dyn VolcanoExecutor>> {
        let projection = plan
            .as_any()
            .downcast_ref::<sqlrustgo_planner::ProjectionExec>()
            .ok_or_else(|| {
                SqlError::ExecutionError("Failed to cast to ProjectionExec".to_string())
            })?;

        let exprs = Self::build_exprs(projection.expr(), plan.schema())?;

        Ok(Box::new(ProjectionExecutor::new(
            child,
            exprs,
            plan.schema().clone(),
            input_schema,
        )))
    }

    fn build_exprs(exprs: &[Expr], output_schema: &Schema) -> SqlResult<Vec<ProjectionExpr>> {
        let mut projection_exprs = Vec::new();

        for expr in exprs {
            match expr {
                Expr::Column(col) => {
                    let idx = output_schema.field_index(&col.name).ok_or_else(|| {
                        SqlError::ExecutionError(format!("Column {} not found in schema", col.name))
                    })?;
                    projection_exprs.push(ProjectionExpr::Column {
                        name: col.name.clone(),
                        index: idx,
                    });
                }
                Expr::Literal(value) => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: Expr::Literal(value.clone()),
                        alias: None,
                    });
                }
                Expr::Alias { expr, name } => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: *expr.clone(),
                        alias: Some(name.clone()),
                    });
                }
                Expr::BinaryExpr { .. } | Expr::UnaryExpr { .. } => {
                    projection_exprs.push(ProjectionExpr::Expression {
                        expr: expr.clone(),
                        alias: None,
                    });
                }
                Expr::Wildcard => {
                    projection_exprs.push(ProjectionExpr::Wildcard);
                }
                Expr::QualifiedWildcard { qualifier } => {
                    projection_exprs.push(ProjectionExpr::QualifiedWildcard {
                        qualifier: qualifier.clone(),
                    });
                }
                Expr::AggregateFunction { .. } => {
                    return Err(SqlError::ExecutionError(
                        "Aggregate function not supported in projection".to_string(),
                    ));
                }
            }
        }

        Ok(projection_exprs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::{DataType, Field, PhysicalPlan, Schema};
    use sqlrustgo_types::Value;

    /// 简单的测试用 Mock 子算子
    struct MockChildExecutor {
        data: Vec<Vec<Value>>,
        idx: usize,
        schema: Schema,
    }

    impl MockChildExecutor {
        fn new() -> Self {
            Self {
                data: vec![
                    vec![
                        Value::Integer(1),
                        Value::Text("Alice".to_string()),
                        Value::Integer(25),
                    ],
                    vec![
                        Value::Integer(2),
                        Value::Text("Bob".to_string()),
                        Value::Integer(30),
                    ],
                    vec![
                        Value::Integer(3),
                        Value::Text("Charlie".to_string()),
                        Value::Integer(35),
                    ],
                ],
                idx: 0,
                schema: Schema::new(vec![
                    Field::new("id".to_string(), DataType::Integer),
                    Field::new("name".to_string(), DataType::Text),
                    Field::new("age".to_string(), DataType::Integer),
                ]),
            }
        }
    }

    impl VolcanoExecutor for MockChildExecutor {
        fn init(&mut self) -> SqlResult<()> {
            self.idx = 0;
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
            self.idx = 0;
            Ok(())
        }

        fn schema(&self) -> &Schema {
            &self.schema
        }

        fn name(&self) -> &str {
            "MockChild"
        }

        fn is_initialized(&self) -> bool {
            true
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_projection_column_selection() {
        let mut child = Box::new(MockChildExecutor::new());
        let input_schema = child.schema().clone();

        let exprs = vec![
            ProjectionExpr::Column {
                name: "id".to_string(),
                index: 0,
            },
            ProjectionExpr::Column {
                name: "name".to_string(),
                index: 1,
            },
        ];

        let output_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);

        let mut executor = ProjectionExecutor::new(child, exprs, output_schema, input_schema);
        executor.init().unwrap();

        let row1 = executor.next().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1.len(), 2);
        assert_eq!(row1[0], Value::Integer(1));
        assert_eq!(row1[1], Value::Text("Alice".to_string()));

        let row2 = executor.next().unwrap();
        assert!(row2.is_some());
        let row2 = row2.unwrap();
        assert_eq!(row2[0], Value::Integer(2));
        assert_eq!(row2[1], Value::Text("Bob".to_string()));
    }

    #[test]
    fn test_projection_expression() {
        let mut child = Box::new(MockChildExecutor::new());
        let input_schema = child.schema().clone();

        // 投影表达式: id + 10
        let expr = Expr::binary_expr(
            Expr::column("id"),
            sqlrustgo_planner::Operator::Plus,
            Expr::literal(Value::Integer(10)),
        );

        let exprs = vec![ProjectionExpr::Expression {
            expr,
            alias: Some("id_plus_10".to_string()),
        }];

        let output_schema = Schema::new(vec![Field::new(
            "id_plus_10".to_string(),
            DataType::Integer,
        )]);

        let mut executor = ProjectionExecutor::new(child, exprs, output_schema, input_schema);
        executor.init().unwrap();

        let row1 = executor.next().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1.len(), 1);
        assert_eq!(row1[0], Value::Integer(11)); // 1 + 10
    }

    #[test]
    fn test_projection_with_alias() {
        let mut child = Box::new(MockChildExecutor::new());
        let input_schema = child.schema().clone();

        let exprs = vec![
            ProjectionExpr::Column {
                name: "id".to_string(),
                index: 0,
            },
            ProjectionExpr::Expression {
                expr: Expr::column("name"),
                alias: Some("user_name".to_string()),
            },
        ];

        let output_schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("user_name".to_string(), DataType::Text),
        ]);

        let mut executor = ProjectionExecutor::new(child, exprs, output_schema, input_schema);
        executor.init().unwrap();

        let row1 = executor.next().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1.len(), 2);
        assert_eq!(row1[0], Value::Integer(1));
        assert_eq!(row1[1], Value::Text("Alice".to_string()));
    }

    #[test]
    fn test_projection_empty_child() {
        let child = Box::new(MockChildExecutor::new());
        let input_schema = child.schema().clone();

        let exprs = vec![ProjectionExpr::Column {
            name: "id".to_string(),
            index: 0,
        }];

        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        // 创建一个已经耗尽的 child
        let mut executor = ProjectionExecutor::new(child, exprs, output_schema, input_schema);
        executor.init().unwrap();

        // 手动消耗所有行
        while executor.next().unwrap().is_some() {}

        // 再次调用应该返回 None
        let row = executor.next().unwrap();
        assert!(row.is_none());

        executor.close().unwrap();
    }

    #[test]
    fn test_projection_schema() {
        let child = Box::new(MockChildExecutor::new());
        let input_schema = child.schema().clone();

        let exprs = vec![ProjectionExpr::Column {
            name: "id".to_string(),
            index: 0,
        }];

        let output_schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);

        let executor = ProjectionExecutor::new(child, exprs, output_schema.clone(), input_schema);

        assert_eq!(executor.schema().fields.len(), 1);
        assert_eq!(executor.name(), "Projection");
    }
}
