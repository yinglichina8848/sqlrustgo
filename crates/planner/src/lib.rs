//! SQLRustGo Planner Module
//!
//! This module provides query planning and optimization interfaces.

pub mod logical_plan;
pub mod optimizer;
pub mod physical_plan;
#[allow(clippy::module_inception)]
pub mod planner;

// TODO: Add these modules after migration
// pub mod analyzer;
// pub mod cost;
// pub mod executor;

pub use logical_plan::LogicalPlan;
pub use optimizer::{DefaultOptimizer, NoOpOptimizer, Optimizer, OptimizerRule};
pub use physical_plan::PhysicalPlan;
pub use planner::{DefaultPlanner, NoOpPlanner, Planner};

use sqlrustgo_types::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    LeftSemi,
    LeftAnti,
    RightSemi,
    RightAnti,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Not,
    Like,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortExpr {
    pub expr: Expr,
    pub asc: bool,
    pub nulls_first: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub relation: Option<String>,
    pub name: String,
}

impl Column {
    pub fn new(name: String) -> Self {
        Self {
            relation: None,
            name,
        }
    }

    pub fn new_qualified(relation: String, name: String) -> Self {
        Self {
            relation: Some(relation),
            name,
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.relation {
            Some(rel) => write!(f, "{}.{}", rel, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(Column),
    Literal(Value),
    BinaryExpr {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    UnaryExpr {
        op: Operator,
        expr: Box<Expr>,
    },
    AggregateFunction {
        func: AggregateFunction,
        args: Vec<Expr>,
        distinct: bool,
    },
    Alias {
        expr: Box<Expr>,
        name: String,
    },
    Wildcard,
    QualifiedWildcard {
        qualifier: String,
    },
}

impl Expr {
    pub fn column(name: &str) -> Self {
        Expr::Column(Column::new(name.to_string()))
    }

    pub fn literal(value: Value) -> Self {
        Expr::Literal(value)
    }

    pub fn binary_expr(left: Expr, op: Operator, right: Expr) -> Self {
        Expr::BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Column(col) => write!(f, "{}", col),
            Expr::Literal(val) => write!(f, "{}", val),
            Expr::BinaryExpr { left, op, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::UnaryExpr { op, expr } => write!(f, "({} {})", op, expr),
            Expr::AggregateFunction {
                func,
                args,
                distinct,
            } => {
                let func_name = match func {
                    AggregateFunction::Count => "COUNT",
                    AggregateFunction::Sum => "SUM",
                    AggregateFunction::Avg => "AVG",
                    AggregateFunction::Min => "MIN",
                    AggregateFunction::Max => "MAX",
                };
                let distinct_str = if *distinct { "DISTINCT " } else { "" };
                let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({}{})", func_name, distinct_str, args_str.join(", "))
            }
            Expr::Alias { expr, name } => write!(f, "{} AS {}", expr, name),
            Expr::Wildcard => write!(f, "*"),
            Expr::QualifiedWildcard { qualifier } => write!(f, "{}.*", qualifier),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Eq => write!(f, "="),
            Operator::NotEq => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::LtEq => write!(f, "<="),
            Operator::Gt => write!(f, ">"),
            Operator::GtEq => write!(f, ">="),
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::Modulo => write!(f, "%"),
            Operator::And => write!(f, "AND"),
            Operator::Or => write!(f, "OR"),
            Operator::Not => write!(f, "NOT"),
            Operator::Like => write!(f, "LIKE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Schema {
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn empty() -> Self {
        Self { fields: vec![] }
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name == name)
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
        }
    }

    pub fn new_not_null(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    Text,
    Blob,
    Null,
}

impl DataType {
    pub fn from_sql_type(type_name: &str) -> Self {
        match type_name.to_uppercase().as_str() {
            "INTEGER" | "INT" => DataType::Integer,
            "FLOAT" | "DOUBLE" | "REAL" => DataType::Float,
            "TEXT" | "VARCHAR" | "CHAR" => DataType::Text,
            "BLOB" | "BINARY" => DataType::Blob,
            "BOOLEAN" | "BOOL" => DataType::Boolean,
            _ => DataType::Null,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Null => write!(f, "NULL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_column_new() {
        let col = Column::new("id".to_string());
        assert_eq!(col.name, "id");
        assert!(col.relation.is_none());
    }

    #[test]
    fn test_column_new_qualified() {
        let col = Column::new_qualified("users".to_string(), "id".to_string());
        assert_eq!(col.name, "id");
        assert_eq!(col.relation, Some("users".to_string()));
    }

    #[test]
    fn test_column_display() {
        let col = Column::new("id".to_string());
        assert_eq!(col.to_string(), "id");

        let qualified = Column::new_qualified("users".to_string(), "id".to_string());
        assert_eq!(qualified.to_string(), "users.id");
    }

    #[test]
    fn test_expr_column() {
        let expr = Expr::column("id");
        assert!(matches!(expr, Expr::Column(_)));
    }

    #[test]
    fn test_expr_literal() {
        let expr = Expr::literal(Value::Integer(42));
        assert!(matches!(expr, Expr::Literal(Value::Integer(42))));
    }

    #[test]
    fn test_expr_binary_expr() {
        let expr = Expr::binary_expr(
            Expr::column("a"),
            Operator::Eq,
            Expr::literal(Value::Integer(1)),
        );
        assert!(matches!(expr, Expr::BinaryExpr { .. }));
    }

    #[test]
    fn test_expr_display() {
        let expr = Expr::column("id");
        assert_eq!(expr.to_string(), "id");
    }

    #[test]
    fn test_operator_display() {
        assert_eq!(Operator::Eq.to_string(), "=");
        assert_eq!(Operator::NotEq.to_string(), "!=");
        assert_eq!(Operator::Lt.to_string(), "<");
        assert_eq!(Operator::LtEq.to_string(), "<=");
        assert_eq!(Operator::Gt.to_string(), ">");
        assert_eq!(Operator::GtEq.to_string(), ">=");
        assert_eq!(Operator::Plus.to_string(), "+");
        assert_eq!(Operator::Minus.to_string(), "-");
        assert_eq!(Operator::Multiply.to_string(), "*");
        assert_eq!(Operator::Divide.to_string(), "/");
        assert_eq!(Operator::Modulo.to_string(), "%");
        assert_eq!(Operator::And.to_string(), "AND");
        assert_eq!(Operator::Or.to_string(), "OR");
        assert_eq!(Operator::Not.to_string(), "NOT");
        assert_eq!(Operator::Like.to_string(), "LIKE");
    }

    #[test]
    fn test_schema_new() {
        let schema = Schema::new(vec![]);
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_schema_empty() {
        let schema = Schema::empty();
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_schema_field() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        assert!(schema.field("id").is_some());
        assert!(schema.field("nonexistent").is_none());
    }

    #[test]
    fn test_schema_field_index() {
        let schema = Schema::new(vec![
            Field::new("id".to_string(), DataType::Integer),
            Field::new("name".to_string(), DataType::Text),
        ]);
        assert_eq!(schema.field_index("id"), Some(0));
        assert_eq!(schema.field_index("name"), Some(1));
        assert_eq!(schema.field_index("nonexistent"), None);
    }

    #[test]
    fn test_field_new() {
        let field = Field::new("id".to_string(), DataType::Integer);
        assert_eq!(field.name, "id");
        assert_eq!(field.data_type, DataType::Integer);
        assert!(field.nullable);
    }

    #[test]
    fn test_field_new_not_null() {
        let field = Field::new_not_null("id".to_string(), DataType::Integer);
        assert_eq!(field.name, "id");
        assert!(!field.nullable);
    }

    #[test]
    fn test_datatype_from_sql_type() {
        assert_eq!(DataType::from_sql_type("INTEGER"), DataType::Integer);
        assert_eq!(DataType::from_sql_type("INT"), DataType::Integer);
        assert_eq!(DataType::from_sql_type("FLOAT"), DataType::Float);
        assert_eq!(DataType::from_sql_type("DOUBLE"), DataType::Float);
        assert_eq!(DataType::from_sql_type("TEXT"), DataType::Text);
        assert_eq!(DataType::from_sql_type("VARCHAR"), DataType::Text);
        assert_eq!(DataType::from_sql_type("BLOB"), DataType::Blob);
        assert_eq!(DataType::from_sql_type("BOOLEAN"), DataType::Boolean);
        assert_eq!(DataType::from_sql_type("UNKNOWN"), DataType::Null);
    }

    #[test]
    fn test_datatype_display() {
        assert_eq!(DataType::Boolean.to_string(), "BOOLEAN");
        assert_eq!(DataType::Integer.to_string(), "INTEGER");
        assert_eq!(DataType::Float.to_string(), "FLOAT");
        assert_eq!(DataType::Text.to_string(), "TEXT");
        assert_eq!(DataType::Blob.to_string(), "BLOB");
        assert_eq!(DataType::Null.to_string(), "NULL");
    }

    #[test]
    fn test_join_type_variants() {
        assert!(matches!(JoinType::Inner, JoinType::Inner));
        assert!(matches!(JoinType::Left, JoinType::Left));
        assert!(matches!(JoinType::Right, JoinType::Right));
        assert!(matches!(JoinType::Full, JoinType::Full));
        assert!(matches!(JoinType::Cross, JoinType::Cross));
    }

    #[test]
    fn test_aggregate_function_variants() {
        assert!(matches!(AggregateFunction::Count, AggregateFunction::Count));
        assert!(matches!(AggregateFunction::Sum, AggregateFunction::Sum));
        assert!(matches!(AggregateFunction::Avg, AggregateFunction::Avg));
        assert!(matches!(AggregateFunction::Min, AggregateFunction::Min));
        assert!(matches!(AggregateFunction::Max, AggregateFunction::Max));
    }

    #[test]
    fn test_sort_expr() {
        let expr = SortExpr {
            expr: Expr::column("id"),
            asc: true,
            nulls_first: false,
        };
        assert!(expr.asc);
        assert!(!expr.nulls_first);
    }
}
