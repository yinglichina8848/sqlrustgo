//! Statement to LogicalPlan Converter
//!
//! Converts parser's Statement into planner's LogicalPlan.

use crate::{Column, DataType, Expr, Field, LogicalPlan, Operator, Schema};
use sqlrustgo_parser::parser::SelectStatement;
use sqlrustgo_parser::{Expression, Statement};
use sqlrustgo_types::Value;

/// Error type for conversion failures
#[derive(Debug, Clone)]
pub struct ConversionError(pub String);

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Conversion error: {}", self.0)
    }
}

impl std::error::Error for ConversionError {}

/// Converter from parser Statement to planner LogicalPlan
pub struct StatementConverter {
    // Table name to schema mapping would be needed for full conversion
}

impl StatementConverter {
    pub fn new() -> Self {
        Self {}
    }

    /// Convert a parser Statement to a planner LogicalPlan
    pub fn convert(&self, statement: &Statement) -> Result<LogicalPlan, ConversionError> {
        match statement {
            Statement::Select(select) => self.convert_select(select),
            _ => Err(ConversionError(format!(
                "Unsupported statement type for conversion: {:?}",
                std::mem::discriminant(statement)
            ))),
        }
    }

    /// Convert a SELECT statement
    fn convert_select(&self, select: &SelectStatement) -> Result<LogicalPlan, ConversionError> {
        // For now, create a simple TableScan and wrap with Filter/Projection if needed
        let table_name = if select.tables.is_empty() {
            select.table.clone()
        } else {
            select.tables[0].clone()
        };

        // Create schema for the table (placeholder - would need catalog lookup)
        let schema = Schema::new(vec![Field::new(
            "placeholder".to_string(),
            DataType::Integer,
        )]);

        // Build the logical plan bottom-up
        let mut plan: LogicalPlan = LogicalPlan::TableScan {
            table_name,
            schema: schema.clone(),
            projection: None,
        };

        // Add WHERE clause as Filter if present
        if let Some(ref where_clause) = select.where_clause {
            let predicate = self.convert_expression(where_clause)?;
            plan = LogicalPlan::Filter {
                predicate,
                input: Box::new(plan),
            };
        }

        // Add projection if columns are specified
        if !select.columns.is_empty() {
            let proj_exprs: Result<Vec<Expr>, _> = select
                .columns
                .iter()
                .map(|col| Ok(Expr::Column(Column::new(col.name.clone()))))
                .collect();
            plan = LogicalPlan::Projection {
                input: Box::new(plan),
                expr: proj_exprs?,
                schema,
            };
        }

        Ok(plan)
    }

    /// Convert a parser Expression to a planner Expr
    pub fn convert_expression(&self, expr: &Expression) -> Result<Expr, ConversionError> {
        match expr {
            Expression::Literal(s) => {
                // Try to parse as different types
                if let Ok(n) = s.parse::<i64>() {
                    Ok(Expr::Literal(Value::Integer(n)))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(Expr::Literal(Value::Float(f)))
                } else if s.eq_ignore_ascii_case("true") {
                    Ok(Expr::Literal(Value::Boolean(true)))
                } else if s.eq_ignore_ascii_case("false") {
                    Ok(Expr::Literal(Value::Boolean(false)))
                } else {
                    Ok(Expr::Literal(Value::Text(s.clone())))
                }
            }
            Expression::Identifier(name) => Ok(Expr::Column(Column::new(name.clone()))),
            Expression::QualifiedColumn(table, col) => Ok(Expr::Column(Column::new_qualified(
                table.clone(),
                col.clone(),
            ))),
            Expression::BinaryOp(left, op, right) => {
                let left_expr = self.convert_expression(left)?;
                let right_expr = self.convert_expression(right)?;
                let operator = self.convert_operator(op)?;
                Ok(Expr::binary_expr(left_expr, operator, right_expr))
            }
            Expression::Wildcard => Ok(Expr::Wildcard),
            Expression::InList { expr, values } => {
                let expr_box = Box::new(self.convert_expression(expr)?);
                let values_exprs: Result<Vec<Expr>, _> =
                    values.iter().map(|v| self.convert_expression(v)).collect();
                Ok(Expr::InList {
                    expr: expr_box,
                    values: values_exprs?,
                })
            }
            Expression::InSubquery { expr, subquery } => {
                let expr_box = Box::new(self.convert_expression(expr)?);
                let subquery_plan = self.convert(subquery)?;
                Ok(Expr::InSubquery {
                    expr: expr_box,
                    subquery: Box::new(subquery_plan),
                })
            }
            Expression::Exists(subquery) => {
                let subquery_plan = self.convert(subquery)?;
                Ok(Expr::Exists(Box::new(subquery_plan)))
            }
            Expression::AnyAll {
                expr,
                op,
                subquery,
                is_any,
            } => {
                let expr_box = Box::new(self.convert_expression(expr)?);
                let subquery_plan = self.convert(subquery)?;
                let operator = self.convert_operator(op)?;
                Ok(Expr::AnyAll {
                    expr: expr_box,
                    op: operator,
                    subquery: Box::new(subquery_plan),
                    any_all: if *is_any {
                        crate::SubqueryType::Any
                    } else {
                        crate::SubqueryType::All
                    },
                })
            }
            Expression::Subquery(stmt) => {
                let subquery_plan = self.convert(stmt)?;
                Ok(Expr::ScalarSubquery(Box::new(subquery_plan)))
            }
            Expression::Between { expr, low, high } => {
                let expr_box = Box::new(self.convert_expression(expr)?);
                let low_box = Box::new(self.convert_expression(low)?);
                let high_box = Box::new(self.convert_expression(high)?);
                Ok(Expr::Between {
                    expr: expr_box,
                    low: low_box,
                    high: high_box,
                })
            }
            Expression::Placeholder => Ok(Expr::Parameter { index: 0 }),
            Expression::FunctionCall(name, args) => {
                let args_exprs: Result<Vec<Expr>, _> =
                    args.iter().map(|a| self.convert_expression(a)).collect();
                let func = self.convert_aggregate_function(name)?;
                Ok(Expr::AggregateFunction {
                    func,
                    args: args_exprs?,
                    distinct: false,
                })
            }
            _ => Err(ConversionError(format!(
                "Unsupported expression type: {:?}",
                std::mem::discriminant(expr)
            ))),
        }
    }

    /// Convert a string operator to Operator enum
    fn convert_operator(&self, op: &str) -> Result<Operator, ConversionError> {
        match op.to_uppercase().as_str() {
            "=" | "==" | "EQ" => Ok(Operator::Eq),
            "!=" | "<>" | "NE" => Ok(Operator::NotEq),
            ">" | "GT" => Ok(Operator::Gt),
            ">=" | "GE" => Ok(Operator::GtEq),
            "<" | "LT" => Ok(Operator::Lt),
            "<=" | "LE" => Ok(Operator::LtEq),
            "+" | "PLUS" => Ok(Operator::Plus),
            "-" | "MINUS" => Ok(Operator::Minus),
            "*" | "MULTIPLY" => Ok(Operator::Multiply),
            "/" | "DIVIDE" => Ok(Operator::Divide),
            "%" | "MODULO" => Ok(Operator::Modulo),
            "AND" | "&&" => Ok(Operator::And),
            "OR" | "||" => Ok(Operator::Or),
            "NOT" | "!" => Ok(Operator::Not),
            "LIKE" => Ok(Operator::Like),
            _ => Err(ConversionError(format!("Unknown operator: {}", op))),
        }
    }

    /// Convert aggregate function name to AggregateFunction enum
    fn convert_aggregate_function(
        &self,
        name: &str,
    ) -> Result<crate::AggregateFunction, ConversionError> {
        match name.to_uppercase().as_str() {
            "COUNT" => Ok(crate::AggregateFunction::Count),
            "SUM" => Ok(crate::AggregateFunction::Sum),
            "AVG" => Ok(crate::AggregateFunction::Avg),
            "MIN" => Ok(crate::AggregateFunction::Min),
            "MAX" => Ok(crate::AggregateFunction::Max),
            _ => Err(ConversionError(format!(
                "Unknown aggregate function: {}",
                name
            ))),
        }
    }
}

impl Default for StatementConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_parser::parse;

    #[test]
    fn test_convert_simple_select() {
        let sql = "SELECT * FROM users";
        let stmt = parse(sql).unwrap();
        let converter = StatementConverter::new();
        let result = converter.convert(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_select_with_where() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let stmt = parse(sql).unwrap();
        let converter = StatementConverter::new();
        let result = converter.convert(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_expression_literal() {
        let expr = Expression::Literal("42".to_string());
        let converter = StatementConverter::new();
        let result = converter.convert_expression(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Literal(Value::Integer(42)) => {}
            _ => panic!("Expected Integer(42)"),
        }
    }

    #[test]
    fn test_convert_expression_identifier() {
        let expr = Expression::Identifier("name".to_string());
        let converter = StatementConverter::new();
        let result = converter.convert_expression(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Column(col) => assert_eq!(col.name, "name"),
            _ => panic!("Expected Column"),
        }
    }

    #[test]
    fn test_convert_exists_subquery() {
        let sql = "SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders)";
        let stmt = parse(sql).unwrap();
        let converter = StatementConverter::new();
        let result = converter.convert(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_any_all_subquery() {
        let sql = "SELECT * FROM products WHERE price > ANY (SELECT price FROM competitor)";
        let stmt = parse(sql).unwrap();
        let converter = StatementConverter::new();
        let result = converter.convert(&stmt);
        assert!(result.is_ok());

        let sql2 = "SELECT * FROM products WHERE price > ALL (SELECT price FROM competitor)";
        let stmt2 = parse(sql2).unwrap();
        let result2 = converter.convert(&stmt2);
        assert!(result2.is_ok());
    }
}
