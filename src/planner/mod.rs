//! Logical Plan Module
//!
//! Defines the logical execution plan representation.

use crate::types::Value;
use std::fmt;

pub mod analyzer;
pub mod executor;
pub mod logical_plan;
pub mod optimizer;
pub mod physical_plan;

pub use analyzer::Analyzer;
pub use logical_plan::*;
pub use optimizer::*;
pub use physical_plan::*;

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

#[derive(Debug, Clone, PartialEq)]
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
