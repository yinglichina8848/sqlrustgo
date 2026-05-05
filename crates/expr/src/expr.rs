//! Unified expression AST for SQLRustGo
//!
//! This is the single source of truth for expressions across:
//! - Parser (raw SQL AST)
//! - Planner / CBO (logical plan)
//! - Executor (physical execution)
//!
//! All expression evaluation flows through this type.

use serde::{Deserialize, Serialize};
use sqlrustgo_types::Value;

use crate::op::BinaryOp;

/// Unified expression AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Column reference: `table.column` or unqualified `column`
    Column { table: Option<String>, name: String },
    /// Literal value: `1`, `'hello'`, `NULL`
    Literal(Value),
    /// Binary expression: `a + b`, `x > 10`
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    /// Logical NOT: `NOT expr`
    Unary { op: UnaryOp, expr: Box<Expr> },
    /// Aggregate function call: `COUNT(*)`, `SUM(x)`
    Aggregate {
        func: AggregateFunc,
        args: Vec<Expr>,
        distinct: bool,
    },
    /// Alias: `expr AS name`
    Alias { expr: Box<Expr>, alias: String },
    /// Wildcard: `*`
    Wildcard,
    /// Qualified wildcard: `table.*`
    QualifiedWildcard { table: String },
    /// Function call: `LOWER(name)`
    Function { name: String, args: Vec<Expr> },
    /// Subquery (used in IN / EXISTS / scalar contexts)
    Subquery(Box<SubqueryExpr>),
}

/// Aggregate functions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateFunc {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Minus,
}

/// Subquery expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubqueryExpr {
    pub query: Subquery,
}

/// Subquery type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Subquery {
    /// Single-row subquery (scalar context)
    Scalar,
    /// Existence check
    Exists,
    /// IN list check
    In,
}

impl Expr {
    /// Create a simple unqualified column reference
    pub fn column(name: &str) -> Self {
        Expr::Column {
            table: None,
            name: name.to_string(),
        }
    }

    /// Create a qualified column reference
    pub fn qualified_column(table: &str, name: &str) -> Self {
        Expr::Column {
            table: Some(table.to_string()),
            name: name.to_string(),
        }
    }

    /// Create a literal value
    pub fn literal(value: Value) -> Self {
        Expr::Literal(value)
    }

    /// Create a binary expression
    pub fn binary(left: Expr, op: BinaryOp, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// Returns true if this expression contains a subquery
    pub fn contains_subquery(&self) -> bool {
        match self {
            Expr::Subquery(_) => true,
            Expr::Binary { left, right, .. } => {
                left.contains_subquery() || right.contains_subquery()
            }
            Expr::Unary { expr, .. } => expr.contains_subquery(),
            Expr::Aggregate { args, .. } => args.iter().any(|a| a.contains_subquery()),
            Expr::Alias { expr, .. } => expr.contains_subquery(),
            Expr::Function { args, .. } => args.iter().any(|a| a.contains_subquery()),
            _ => false,
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Column { table, name } => {
                if let Some(t) = table {
                    write!(f, "{}.{}", t, name)
                } else {
                    write!(f, "{}", name)
                }
            }
            Expr::Literal(v) => write!(f, "{}", v),
            Expr::Binary { left, op, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Unary { op, expr } => write!(f, "({} {})", op, expr),
            Expr::Aggregate {
                func,
                args,
                distinct,
            } => {
                let d = if *distinct { "DISTINCT " } else { "" };
                let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({}{})", func, d, args_str.join(", "))
            }
            Expr::Alias { expr, alias } => write!(f, "{} AS {}", expr, alias),
            Expr::Wildcard => write!(f, "*"),
            Expr::QualifiedWildcard { table } => write!(f, "{}.*", table),
            Expr::Function { name, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", name, args_str.join(", "))
            }
            Expr::Subquery(_) => write!(f, "(SUBQUERY)"),
        }
    }
}

impl std::fmt::Display for AggregateFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateFunc::Count => write!(f, "COUNT"),
            AggregateFunc::Sum => write!(f, "SUM"),
            AggregateFunc::Avg => write!(f, "AVG"),
            AggregateFunc::Min => write!(f, "MIN"),
            AggregateFunc::Max => write!(f, "MAX"),
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "NOT"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}
