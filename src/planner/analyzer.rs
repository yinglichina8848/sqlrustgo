//! Query Analyzer Module
//!
//! Converts Parser's Statement AST into LogicalPlan with:
//! - Table and column binding
//! - Type checking
//! - Schema resolution

use crate::parser::{
    AggregateCall, ColumnDefinition, DeleteStatement, Expression, InsertStatement,
    SelectStatement, Statement, UpdateStatement,
};
use crate::planner::{
    AggregateFunction as PlannerAggFunc, DataType, Expr, Field, LogicalPlan, Operator, Schema,
};
use crate::types::SqlError;
use std::collections::HashMap;

/// Analyzer transforms SQL statements into logical plans
pub struct Analyzer {
    /// Table schemas: table_name -> Schema
    tables: HashMap<String, Schema>,
}

impl Analyzer {
    /// Create a new analyzer with known table schemas
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Create analyzer with pre-loaded schemas
    pub fn with_schemas(schemas: HashMap<String, Schema>) -> Self {
        Self { tables: schemas }
    }

    /// Register a table schema
    pub fn register_table(&mut self, name: String, schema: Schema) {
        self.tables.insert(name, schema);
    }

    /// Analyze a statement and return a logical plan
    pub fn analyze(&self, statement: Statement) -> Result<LogicalPlan, SqlError> {
        match statement {
            Statement::Select(select) => self.analyze_select(select),
            Statement::Insert(insert) => self.analyze_insert(insert),
            Statement::Update(update) => self.analyze_update(update),
            Statement::Delete(delete) => self.analyze_delete(delete),
            Statement::CreateTable(create) => self.analyze_create_table(create),
            Statement::DropTable(drop) => self.analyze_drop_table(drop),
        }
    }

    /// Analyze SELECT statement into LogicalPlan
    fn analyze_select(&self, select: SelectStatement) -> Result<LogicalPlan, SqlError> {
        // Get table schema
        let table_schema = self
            .tables
            .get(&select.table)
            .ok_or_else(|| SqlError::ExecutionError(format!("Unknown table: {}", select.table)))?
            .clone();

        // Build projection expressions
        let mut proj_exprs = Vec::new();
        let mut output_fields = Vec::new();

        for col in &select.columns {
            let expr = self.bind_column(&col.name, &table_schema, col.alias.as_deref())?;

            // Determine output field type
            let field_name = col.alias.clone().unwrap_or_else(|| col.name.clone());
            let data_type = self.infer_expression_type(&expr, &table_schema);
            output_fields.push(Field::new(field_name, data_type));

            proj_exprs.push(expr);
        }

        // Create TableScan as base
        let mut plan = LogicalPlan::TableScan {
            table_name: select.table,
            projection: None,
            filters: vec![],
            limit: None,
            schema: table_schema,
        };

        // Add WHERE filter if present
        if let Some(where_expr) = &select.where_clause {
            let bound_expr = self.bind_expression(where_expr, &plan.schema())?;
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate: bound_expr,
            };
        }

        // Add aggregates if present
        if !select.aggregates.is_empty() {
            let mut aggr_exprs = Vec::new();
            for aggr in &select.aggregates {
                let expr = self.bind_aggregate(aggr, &plan.schema())?;
                aggr_exprs.push(expr);
            }
            plan = LogicalPlan::Aggregate {
                input: Box::new(plan),
                group_expr: vec![],
                aggr_expr: aggr_exprs,
                schema: Schema::new(output_fields.clone()),
            };
        }

        // Add projection
        let schema = Schema::new(output_fields);
        plan = LogicalPlan::Projection {
            input: Box::new(plan),
            expr: proj_exprs,
            schema,
        };

        Ok(plan)
    }

    /// Analyze INSERT statement
    fn analyze_insert(&self, insert: InsertStatement) -> Result<LogicalPlan, SqlError> {
        let table_schema = self
            .tables
            .get(&insert.table)
            .ok_or_else(|| SqlError::ExecutionError(format!("Unknown table: {}", insert.table)))?
            .clone();

        // Convert values to expressions
        let mut values_exprs = Vec::new();
        for row in &insert.values {
            let mut row_exprs = Vec::new();
            for val in row {
                row_exprs.push(self.literal_to_expr(val));
            }
            values_exprs.push(row_exprs);
        }

        Ok(LogicalPlan::Values {
            values: values_exprs,
            schema: table_schema,
        })
    }

    /// Analyze UPDATE statement
    fn analyze_update(&self, update: UpdateStatement) -> Result<LogicalPlan, SqlError> {
        let table_schema = self
            .tables
            .get(&update.table)
            .ok_or_else(|| SqlError::ExecutionError(format!("Unknown table: {}", update.table)))?
            .clone();

        // Build set expressions
        let mut set_exprs = Vec::new();
        for (col, expr) in &update.set_clauses {
            // Verify column exists
            if table_schema.field(col).is_none() {
                return Err(SqlError::ExecutionError(format!("Unknown column: {}", col)));
            }
            let bound_expr = self.bind_expression(expr, &table_schema)?;
            set_exprs.push((col.clone(), bound_expr));
        }

        // Create base table scan
        let mut plan = LogicalPlan::TableScan {
            table_name: update.table,
            projection: None,
            filters: vec![],
            limit: None,
            schema: table_schema,
        };

        // Add WHERE filter if present
        if let Some(where_expr) = &update.where_clause {
            let bound_expr = self.bind_expression(where_expr, &plan.schema())?;
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate: bound_expr,
            };
        }

        let schema = plan.schema().clone();
        Ok(LogicalPlan::Update {
            input: Box::new(plan),
            set_exprs,
            schema,
        })
    }

    /// Analyze DELETE statement
    fn analyze_delete(&self, delete: DeleteStatement) -> Result<LogicalPlan, SqlError> {
        let table_schema = self
            .tables
            .get(&delete.table)
            .ok_or_else(|| SqlError::ExecutionError(format!("Unknown table: {}", delete.table)))?
            .clone();

        // Create base table scan
        let mut plan = LogicalPlan::TableScan {
            table_name: delete.table,
            projection: None,
            filters: vec![],
            limit: None,
            schema: table_schema,
        };

        // Add WHERE filter if present
        if let Some(where_expr) = &delete.where_clause {
            let bound_expr = self.bind_expression(where_expr, &plan.schema())?;
            plan = LogicalPlan::Filter {
                input: Box::new(plan),
                predicate: bound_expr,
            };
        }

        let schema = plan.schema().clone();
        Ok(LogicalPlan::Delete {
            input: Box::new(plan),
            schema,
        })
    }

    /// Analyze CREATE TABLE statement
    fn analyze_create_table(&self, create: crate::parser::CreateTableStatement) -> Result<LogicalPlan, SqlError> {
        let fields: Vec<Field> = create
            .columns
            .iter()
            .map(|col| self.column_def_to_field(col))
            .collect();

        Ok(LogicalPlan::CreateTable {
            name: create.name,
            schema: Schema::new(fields),
        })
    }

    /// Analyze DROP TABLE statement
    fn analyze_drop_table(&self, drop: crate::parser::DropTableStatement) -> Result<LogicalPlan, SqlError> {
        Ok(LogicalPlan::DropTable {
            name: drop.name,
            schema: Schema::empty(),
        })
    }

    /// Bind a column reference to a schema
    fn bind_column(&self, name: &str, schema: &Schema, alias: Option<&str>) -> Result<Expr, SqlError> {
        // Check if it's a wildcard
        if name == "*" {
            return Ok(Expr::Wildcard);
        }

        // Check if it's a qualified wildcard (table.*)
        if name.ends_with(".*") {
            let qualifier = name.trim_end_matches(".*");
            return Ok(Expr::QualifiedWildcard {
                qualifier: qualifier.to_string(),
            });
        }

        // Check if column exists in schema
        if schema.field(name).is_some() {
            Ok(Expr::Column(crate::planner::Column::new(name.to_string())))
        } else {
            Err(SqlError::ExecutionError(format!("Unknown column: {}", name)))
        }
    }

    /// Bind a parser Expression to planner Expr
    fn bind_expression(&self, expr: &Expression, schema: &Schema) -> Result<Expr, SqlError> {
        match expr {
            Expression::Literal(lit) => Ok(Expr::Literal(crate::types::parse_sql_literal(lit))),
            Expression::Identifier(name) => {
                self.bind_column(name, schema, None)
            }
            Expression::BinaryOp(left, op, right) => {
                let left_bound = self.bind_expression(left, schema)?;
                let right_bound = self.bind_expression(right, schema)?;
                let operator = self.bind_operator(op)?;
                Ok(Expr::binary_expr(left_bound, operator, right_bound))
            }
            _ => Err(SqlError::ExecutionError("Unsupported expression".to_string())),
        }
    }

    /// Bind aggregate call
    fn bind_aggregate(&self, aggr: &AggregateCall, schema: &Schema) -> Result<Expr, SqlError> {
        let func = match aggr.func {
            crate::parser::AggregateFunction::Count => PlannerAggFunc::Count,
            crate::parser::AggregateFunction::Sum => PlannerAggFunc::Sum,
            crate::parser::AggregateFunction::Avg => PlannerAggFunc::Avg,
            crate::parser::AggregateFunction::Min => PlannerAggFunc::Min,
            crate::parser::AggregateFunction::Max => PlannerAggFunc::Max,
        };

        let arg = match &aggr.column {
            Some(col) => vec![self.bind_column(col, schema, None)?],
            None => vec![],
        };

        Ok(Expr::AggregateFunction {
            func,
            args: arg,
            distinct: false,
        })
    }

    /// Bind binary operator
    fn bind_operator(&self, op: &str) -> Result<Operator, SqlError> {
        match op.to_uppercase().as_str() {
            "=" => Ok(Operator::Eq),
            "!=" | "<>" => Ok(Operator::NotEq),
            "<" => Ok(Operator::Lt),
            "<=" => Ok(Operator::LtEq),
            ">" => Ok(Operator::Gt),
            ">=" => Ok(Operator::GtEq),
            "AND" => Ok(Operator::And),
            "OR" => Ok(Operator::Or),
            "LIKE" => Ok(Operator::Like),
            "+" => Ok(Operator::Plus),
            "-" => Ok(Operator::Minus),
            "*" => Ok(Operator::Multiply),
            "/" => Ok(Operator::Divide),
            "%" => Ok(Operator::Modulo),
            _ => Err(SqlError::ExecutionError(format!("Unknown operator: {}", op))),
        }
    }

    /// Bind unary operator
    fn bind_unary_operator(&self, op: &str) -> Result<Operator, SqlError> {
        match op.to_uppercase().as_str() {
            "NOT" => Ok(Operator::Not),
            "-" => Ok(Operator::Minus),
            _ => Err(SqlError::ExecutionError(format!("Unknown unary operator: {}", op))),
        }
    }

    /// Convert parser literal to planner literal expression
    fn literal_to_expr(&self, expr: &Expression) -> Expr {
        match expr {
            Expression::Literal(lit) => Expr::Literal(crate::types::parse_sql_literal(lit)),
            Expression::Identifier(name) => Expr::Column(crate::planner::Column::new(name.clone())),
            Expression::BinaryOp(left, op, right) => {
                let left_bound = self.bind_expression(left, &Schema::empty()).unwrap_or_else(|_| Expr::Wildcard);
                let right_bound = self.bind_expression(right, &Schema::empty()).unwrap_or_else(|_| Expr::Wildcard);
                let operator = self.bind_operator(op).unwrap_or(Operator::Eq);
                Expr::binary_expr(left_bound, operator, right_bound)
            }
        }
    }

    /// Infer expression type
    fn infer_expression_type(&self, expr: &Expr, _schema: &Schema) -> DataType {
        match expr {
            Expr::Literal(val) => match val {
                crate::types::Value::Integer(_) => DataType::Integer,
                crate::types::Value::Float(_) => DataType::Float,
                crate::types::Value::Text(_) => DataType::Text,
                crate::types::Value::Boolean(_) => DataType::Boolean,
                crate::types::Value::Null => DataType::Null,
                crate::types::Value::Blob(_) => DataType::Blob,
            },
            Expr::Column(_) => DataType::Text, // Default, would need schema lookup
            Expr::BinaryExpr { .. } => DataType::Text,
            Expr::UnaryExpr { .. } => DataType::Text,
            Expr::AggregateFunction { func, .. } => match func {
                PlannerAggFunc::Count => DataType::Integer,
                PlannerAggFunc::Sum | PlannerAggFunc::Avg => DataType::Float,
                PlannerAggFunc::Min | PlannerAggFunc::Max => DataType::Text,
            },
            Expr::Alias { expr, .. } => self.infer_expression_type(expr, _schema),
            Expr::Wildcard | Expr::QualifiedWildcard { .. } => DataType::Text,
        }
    }

    /// Convert column definition to field
    fn column_def_to_field(&self, col: &ColumnDefinition) -> Field {
        let data_type = DataType::from_sql_type(&col.data_type);
        if col.nullable {
            Field::new(col.name.clone(), data_type)
        } else {
            Field::new_not_null(col.name.clone(), data_type)
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{AggregateCall, AggregateFunction as ParserAggFunc, Expression};

    #[test]
    fn test_analyzer_creation() {
        let analyzer = Analyzer::new();
        assert!(analyzer.tables.is_empty());
    }

    #[test]
    fn test_analyzer_with_schemas() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );

        let analyzer = Analyzer::with_schemas(schemas);
        assert!(analyzer.tables.contains_key("users"));
    }

    #[test]
    fn test_analyzer_register_table() {
        let mut analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);
        analyzer.register_table("users".to_string(), schema);

        assert!(analyzer.tables.contains_key("users"));
    }

    #[test]
    fn test_bind_column_wildcard() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let result = analyzer.bind_column("*", &schema, None);
        assert!(matches!(result, Ok(Expr::Wildcard)));
    }

    #[test]
    fn test_bind_column_qualified_wildcard() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let result = analyzer.bind_column("users.*", &schema, None);
        assert!(matches!(result, Ok(Expr::QualifiedWildcard { .. })));
    }

    #[test]
    fn test_bind_column_unknown() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let result = analyzer.bind_column("unknown", &schema, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_operator() {
        let analyzer = Analyzer::new();

        assert!(analyzer.bind_operator("=").is_ok());
        assert!(analyzer.bind_operator("!=").is_ok());
        assert!(analyzer.bind_operator("<>").is_ok());
        assert!(analyzer.bind_operator("<").is_ok());
        assert!(analyzer.bind_operator("<=").is_ok());
        assert!(analyzer.bind_operator(">").is_ok());
        assert!(analyzer.bind_operator(">=").is_ok());
        assert!(analyzer.bind_operator("AND").is_ok());
        assert!(analyzer.bind_operator("OR").is_ok());
        assert!(analyzer.bind_operator("LIKE").is_ok());
        assert!(analyzer.bind_operator("+").is_ok());
        assert!(analyzer.bind_operator("-").is_ok());
        assert!(analyzer.bind_operator("*").is_ok());
        assert!(analyzer.bind_operator("/").is_ok());
        assert!(analyzer.bind_operator("%").is_ok());

        let result = analyzer.bind_operator("UNKNOWN");
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_unary_operator() {
        let analyzer = Analyzer::new();

        assert!(analyzer.bind_unary_operator("NOT").is_ok());
        assert!(analyzer.bind_unary_operator("-").is_ok());

        let result = analyzer.bind_unary_operator("UNKNOWN");
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_aggregate_count() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let aggr = AggregateCall {
            func: ParserAggFunc::Count,
            column: Some("id".to_string()),
        };

        let result = analyzer.bind_aggregate(&aggr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bind_aggregate_sum() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("amount".to_string(), DataType::Integer),
        ]);

        let aggr = AggregateCall {
            func: ParserAggFunc::Sum,
            column: Some("amount".to_string()),
        };

        let result = analyzer.bind_aggregate(&aggr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bind_aggregate_avg() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("price".to_string(), DataType::Float),
        ]);

        let aggr = AggregateCall {
            func: ParserAggFunc::Avg,
            column: Some("price".to_string()),
        };

        let result = analyzer.bind_aggregate(&aggr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bind_aggregate_min_max() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("name".to_string(), DataType::Text),
        ]);

        let aggr_min = AggregateCall {
            func: ParserAggFunc::Min,
            column: Some("name".to_string()),
        };
        let aggr_max = AggregateCall {
            func: ParserAggFunc::Max,
            column: Some("name".to_string()),
        };

        assert!(analyzer.bind_aggregate(&aggr_min, &schema).is_ok());
        assert!(analyzer.bind_aggregate(&aggr_max, &schema).is_ok());
    }

    #[test]
    fn test_infer_expression_type_literal() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        let int_lit = Expr::Literal(crate::types::Value::Integer(42));
        assert_eq!(analyzer.infer_expression_type(&int_lit, &schema), DataType::Integer);

        let float_lit = Expr::Literal(crate::types::Value::Float(3.14));
        assert_eq!(analyzer.infer_expression_type(&float_lit, &schema), DataType::Float);

        let text_lit = Expr::Literal(crate::types::Value::Text("hello".to_string()));
        assert_eq!(analyzer.infer_expression_type(&text_lit, &schema), DataType::Text);

        let bool_lit = Expr::Literal(crate::types::Value::Boolean(true));
        assert_eq!(analyzer.infer_expression_type(&bool_lit, &schema), DataType::Boolean);

        let null_lit = Expr::Literal(crate::types::Value::Null);
        assert_eq!(analyzer.infer_expression_type(&null_lit, &schema), DataType::Null);

        let blob_lit = Expr::Literal(crate::types::Value::Blob(vec![1, 2, 3]));
        assert_eq!(analyzer.infer_expression_type(&blob_lit, &schema), DataType::Blob);
    }

    #[test]
    fn test_infer_expression_type_column() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        let col = Expr::Column(crate::planner::Column::new("id".to_string()));
        assert_eq!(analyzer.infer_expression_type(&col, &schema), DataType::Text);
    }

    #[test]
    fn test_infer_expression_type_aggregate() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        let count_aggr = Expr::AggregateFunction {
            func: crate::planner::AggregateFunction::Count,
            args: vec![],
            distinct: false,
        };
        assert_eq!(analyzer.infer_expression_type(&count_aggr, &schema), DataType::Integer);

        let sum_aggr = Expr::AggregateFunction {
            func: crate::planner::AggregateFunction::Sum,
            args: vec![],
            distinct: false,
        };
        assert_eq!(analyzer.infer_expression_type(&sum_aggr, &schema), DataType::Float);

        let avg_aggr = Expr::AggregateFunction {
            func: crate::planner::AggregateFunction::Avg,
            args: vec![],
            distinct: false,
        };
        assert_eq!(analyzer.infer_expression_type(&avg_aggr, &schema), DataType::Float);

        let min_aggr = Expr::AggregateFunction {
            func: crate::planner::AggregateFunction::Min,
            args: vec![],
            distinct: false,
        };
        assert_eq!(analyzer.infer_expression_type(&min_aggr, &schema), DataType::Text);

        let max_aggr = Expr::AggregateFunction {
            func: crate::planner::AggregateFunction::Max,
            args: vec![],
            distinct: false,
        };
        assert_eq!(analyzer.infer_expression_type(&max_aggr, &schema), DataType::Text);
    }

    #[test]
    fn test_column_def_to_field() {
        let analyzer = Analyzer::new();

        let col_def = crate::parser::ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
        };
        let field = analyzer.column_def_to_field(&col_def);
        assert_eq!(field.name, "id");
        assert_eq!(field.data_type, DataType::Integer);
        assert!(!field.nullable);

        let col_def_nullable = crate::parser::ColumnDefinition {
            name: "name".to_string(),
            data_type: "TEXT".to_string(),
            nullable: true,
        };
        let field_nullable = analyzer.column_def_to_field(&col_def_nullable);
        assert_eq!(field_nullable.name, "name");
        assert_eq!(field_nullable.data_type, DataType::Text);
        assert!(field_nullable.nullable);
    }

    #[test]
    fn test_bind_expression_literal() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        let expr = Expression::Literal("42".to_string());
        let result = analyzer.bind_expression(&expr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bind_expression_identifier() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let expr = Expression::Identifier("id".to_string());
        let result = analyzer.bind_expression(&expr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bind_expression_binary_op() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("a".to_string(), DataType::Integer),
            Field::new_not_null("b".to_string(), DataType::Integer),
        ]);

        let expr = Expression::BinaryOp(
            Box::new(Expression::Identifier("a".to_string())),
            "+".to_string(),
            Box::new(Expression::Identifier("b".to_string())),
        );
        let result = analyzer.bind_expression(&expr, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn test_literal_to_expr_literal() {
        let analyzer = Analyzer::new();
        let expr = Expression::Literal("42".to_string());
        let result = analyzer.literal_to_expr(&expr);
        assert!(matches!(result, Expr::Literal(_)));
    }

    #[test]
    fn test_literal_to_expr_identifier() {
        let analyzer = Analyzer::new();
        let expr = Expression::Identifier("id".to_string());
        let result = analyzer.literal_to_expr(&expr);
        assert!(matches!(result, Expr::Column(_)));
    }

    #[test]
    fn test_literal_to_expr_binary_op() {
        let analyzer = Analyzer::new();
        let expr = Expression::BinaryOp(
            Box::new(Expression::Identifier("a".to_string())),
            "=".to_string(),
            Box::new(Expression::Literal("1".to_string())),
        );
        let result = analyzer.literal_to_expr(&expr);
        assert!(matches!(result, Expr::BinaryExpr { .. }));
    }

    // ============================================
    // Tests for analyze methods
    // ============================================

    #[test]
    fn test_analyze_select_simple() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Select(SelectStatement {
            table: "users".to_string(),
            columns: vec![crate::parser::SelectColumn {
                name: "id".to_string(),
                alias: None,
            }],
            where_clause: None,
            aggregates: vec![],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::Projection { .. }));
    }

    #[test]
    fn test_analyze_select_with_where() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Select(SelectStatement {
            table: "users".to_string(),
            columns: vec![crate::parser::SelectColumn {
                name: "id".to_string(),
                alias: None,
            }],
            where_clause: Some(Expression::BinaryOp(
                Box::new(Expression::Identifier("id".to_string())),
                ">".to_string(),
                Box::new(Expression::Literal("10".to_string())),
            )),
            aggregates: vec![],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_select_with_aggregate() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "orders".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new_not_null("amount".to_string(), DataType::Integer),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Select(SelectStatement {
            table: "orders".to_string(),
            columns: vec![],
            where_clause: None,
            aggregates: vec![AggregateCall {
                func: ParserAggFunc::Count,
                column: None,
            }],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_insert() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Insert(InsertStatement {
            table: "users".to_string(),
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![
                vec![Expression::Literal("1".to_string()), Expression::Literal("Alice".to_string())],
            ],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::Values { .. }));
    }

    #[test]
    fn test_analyze_update() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Update(UpdateStatement {
            table: "users".to_string(),
            set_clauses: vec![(
                "name".to_string(),
                Expression::Literal("Bob".to_string()),
            )],
            where_clause: None,
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::Update { .. }));
    }

    #[test]
    fn test_analyze_update_with_where() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Update(UpdateStatement {
            table: "users".to_string(),
            set_clauses: vec![(
                "name".to_string(),
                Expression::Literal("Bob".to_string()),
            )],
            where_clause: Some(Expression::BinaryOp(
                Box::new(Expression::Identifier("id".to_string())),
                "=".to_string(),
                Box::new(Expression::Literal("1".to_string())),
            )),
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_delete() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Delete(DeleteStatement {
            table: "users".to_string(),
            where_clause: None,
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::Delete { .. }));
    }

    #[test]
    fn test_analyze_delete_with_where() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Delete(DeleteStatement {
            table: "users".to_string(),
            where_clause: Some(Expression::BinaryOp(
                Box::new(Expression::Identifier("id".to_string())),
                ">".to_string(),
                Box::new(Expression::Literal("10".to_string())),
            )),
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_delete_unknown_table() {
        let analyzer = Analyzer::new();

        let stmt = Statement::Delete(DeleteStatement {
            table: "nonexistent".to_string(),
            where_clause: None,
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_create_table() {
        let analyzer = Analyzer::new();

        let stmt = Statement::CreateTable(crate::parser::CreateTableStatement {
            name: "users".to_string(),
            columns: vec![
                crate::parser::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                },
                crate::parser::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                },
            ],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::CreateTable { .. }));
    }

    #[test]
    fn test_analyze_drop_table() {
        let analyzer = Analyzer::new();

        let stmt = Statement::DropTable(crate::parser::DropTableStatement {
            name: "users".to_string(),
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(matches!(plan, LogicalPlan::DropTable { .. }));
    }

    #[test]
    fn test_analyze_unknown_table() {
        let analyzer = Analyzer::new();

        let stmt = Statement::Select(SelectStatement {
            table: "unknown".to_string(),
            columns: vec![crate::parser::SelectColumn {
                name: "id".to_string(),
                alias: None,
            }],
            where_clause: None,
            aggregates: vec![],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_insert_unknown_table() {
        let analyzer = Analyzer::new();

        let stmt = Statement::Insert(InsertStatement {
            table: "unknown".to_string(),
            columns: vec!["id".to_string()],
            values: vec![vec![Expression::Literal("1".to_string())]],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_update_unknown_column() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Update(UpdateStatement {
            table: "users".to_string(),
            set_clauses: vec![(
                "unknown_column".to_string(),
                Expression::Literal("value".to_string()),
            )],
            where_clause: None,
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_expression_unknown_column() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let expr = Expression::Identifier("unknown".to_string());
        let result = analyzer.bind_expression(&expr, &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_bind_aggregate_unknown_column() {
        let analyzer = Analyzer::new();
        let schema = Schema::new(vec![
            Field::new_not_null("id".to_string(), DataType::Integer),
        ]);

        let aggr = AggregateCall {
            func: ParserAggFunc::Count,
            column: Some("unknown".to_string()),
        };

        let result = analyzer.bind_aggregate(&aggr, &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_column_def_to_field_various_types() {
        let analyzer = Analyzer::new();

        // Test FLOAT type
        let col_def = crate::parser::ColumnDefinition {
            name: "price".to_string(),
            data_type: "FLOAT".to_string(),
            nullable: true,
        };
        let field = analyzer.column_def_to_field(&col_def);
        assert_eq!(field.data_type, DataType::Float);

        // Test BOOLEAN type
        let col_def = crate::parser::ColumnDefinition {
            name: "active".to_string(),
            data_type: "BOOLEAN".to_string(),
            nullable: false,
        };
        let field = analyzer.column_def_to_field(&col_def);
        assert_eq!(field.data_type, DataType::Boolean);

        // Test BLOB type
        let col_def = crate::parser::ColumnDefinition {
            name: "data".to_string(),
            data_type: "BLOB".to_string(),
            nullable: true,
        };
        let field = analyzer.column_def_to_field(&col_def);
        assert_eq!(field.data_type, DataType::Blob);

        // Test unknown type (defaults to Null)
        let col_def = crate::parser::ColumnDefinition {
            name: "extra".to_string(),
            data_type: "UNKNOWN".to_string(),
            nullable: true,
        };
        let field = analyzer.column_def_to_field(&col_def);
        assert_eq!(field.data_type, DataType::Null);
    }

    #[test]
    fn test_infer_expression_type_binary_expr() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        // Binary expr returns Text (current implementation)
        let binary = Expr::BinaryExpr {
            op: Operator::Plus,
            left: Box::new(Expr::Literal(crate::types::Value::Integer(1))),
            right: Box::new(Expr::Literal(crate::types::Value::Integer(2))),
        };
        assert_eq!(analyzer.infer_expression_type(&binary, &schema), DataType::Text);

        // Binary expr with text should return Text
        let binary = Expr::BinaryExpr {
            op: Operator::Like,
            left: Box::new(Expr::Literal(crate::types::Value::Text("a".to_string()))),
            right: Box::new(Expr::Literal(crate::types::Value::Text("b".to_string()))),
        };
        assert_eq!(analyzer.infer_expression_type(&binary, &schema), DataType::Text);
    }

    #[test]
    fn test_infer_expression_type_alias() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        // Test alias expression type inference
        let alias = Expr::Alias {
            expr: Box::new(Expr::Literal(crate::types::Value::Integer(42))),
            name: "my_int".to_string(),
        };
        assert_eq!(analyzer.infer_expression_type(&alias, &schema), DataType::Integer);

        // Test alias with text
        let alias_text = Expr::Alias {
            expr: Box::new(Expr::Literal(crate::types::Value::Text("hello".to_string()))),
            name: "my_text".to_string(),
        };
        assert_eq!(analyzer.infer_expression_type(&alias_text, &schema), DataType::Text);
    }

    #[test]
    fn test_infer_expression_type_wildcard() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        // Test wildcard type inference
        assert_eq!(analyzer.infer_expression_type(&Expr::Wildcard, &schema), DataType::Text);

        // Test qualified wildcard type inference
        let qualified_wildcard = Expr::QualifiedWildcard {
            qualifier: "users".to_string(),
        };
        assert_eq!(analyzer.infer_expression_type(&qualified_wildcard, &schema), DataType::Text);
    }

    #[test]
    fn test_infer_expression_type_unary_expr() {
        let analyzer = Analyzer::new();
        let schema = Schema::empty();

        // Test unary expression type inference
        let unary = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::Literal(crate::types::Value::Integer(5))),
        };
        assert_eq!(analyzer.infer_expression_type(&unary, &schema), DataType::Text);
    }

    #[test]
    fn test_analyze_select_with_alias() {
        let mut schemas = HashMap::new();
        schemas.insert(
            "users".to_string(),
            Schema::new(vec![
                Field::new_not_null("id".to_string(), DataType::Integer),
                Field::new("name".to_string(), DataType::Text),
            ]),
        );
        let analyzer = Analyzer::with_schemas(schemas);

        let stmt = Statement::Select(SelectStatement {
            table: "users".to_string(),
            columns: vec![
                crate::parser::SelectColumn {
                    name: "id".to_string(),
                    alias: Some("user_id".to_string()),
                },
                crate::parser::SelectColumn {
                    name: "name".to_string(),
                    alias: Some("full_name".to_string()),
                },
            ],
            where_clause: None,
            aggregates: vec![],
        });

        let result = analyzer.analyze(stmt);
        assert!(result.is_ok());
    }
}
