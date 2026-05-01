//! Predicate evaluation types for storage-level query pushdown

use serde::{Deserialize, Serialize};
pub use sqlrustgo_types::Value;

/// Index operation types for index-assisted scans
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IndexOp {
    /// Equality predicate: column = value
    Eq(i64),
    /// Range predicate: start <= column <= end
    Range(i64, i64),
}

/// Expression types for predicate evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Column reference by name
    Column(String),
    /// Constant value
    Value(Value),
    /// Parameter placeholder (for prepared statements)
    Parameter(usize),
}

impl Expr {
    /// Evaluate this expression against a record
    pub fn evaluate(&self, _record: &[Value]) -> Option<Value> {
        match self {
            Expr::Column(_name) => {
                // Column evaluation is handled by the caller with schema info
                None
            }
            Expr::Value(v) => Some(v.clone()),
            Expr::Parameter(_) => None,
        }
    }
}

/// Predicate for filtering records at the storage layer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    // Comparison predicates
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),

    // Membership predicate
    In(Box<Expr>, Vec<Expr>),

    // Logical predicates
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),

    // Null checks
    IsNull(Box<Expr>),
    IsNotNull(Box<Expr>),
}

impl Predicate {
    /// Create an equality predicate: column = value
    pub fn eq(column: impl Into<String>, value: Value) -> Self {
        Predicate::Eq(
            Box::new(Expr::Column(column.into())),
            Box::new(Expr::Value(value)),
        )
    }

    /// Create a less-than predicate: column < value
    pub fn lt(column: impl Into<String>, value: Value) -> Self {
        Predicate::Lt(
            Box::new(Expr::Column(column.into())),
            Box::new(Expr::Value(value)),
        )
    }

    /// Create a less-than-or-equal predicate: column <= value
    pub fn lte(column: impl Into<String>, value: Value) -> Self {
        Predicate::Lte(
            Box::new(Expr::Column(column.into())),
            Box::new(Expr::Value(value)),
        )
    }

    /// Create a greater-than predicate: column > value
    pub fn gt(column: impl Into<String>, value: Value) -> Self {
        Predicate::Gt(
            Box::new(Expr::Column(column.into())),
            Box::new(Expr::Value(value)),
        )
    }

    /// Create a greater-than-or-equal predicate: column >= value
    pub fn gte(column: impl Into<String>, value: Value) -> Self {
        Predicate::Gte(
            Box::new(Expr::Column(column.into())),
            Box::new(Expr::Value(value)),
        )
    }

    /// Create an AND predicate
    pub fn and(left: Predicate, right: Predicate) -> Self {
        Predicate::And(Box::new(left), Box::new(right))
    }

    /// Create an OR predicate
    pub fn or(left: Predicate, right: Predicate) -> Self {
        Predicate::Or(Box::new(left), Box::new(right))
    }

    /// Create a NOT predicate
    #[allow(clippy::should_implement_trait)]
    pub fn not(predicate: Predicate) -> Self {
        Predicate::Not(Box::new(predicate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_eq() {
        let pred = Predicate::eq("id", Value::Integer(42));
        assert!(matches!(pred, Predicate::Eq(_, _)));
    }

    #[test]
    fn test_predicate_and() {
        let pred = Predicate::and(
            Predicate::eq("a", Value::Integer(1)),
            Predicate::eq("b", Value::Integer(2)),
        );
        assert!(matches!(pred, Predicate::And(_, _)));
    }

    #[test]
    fn test_predicate_or() {
        let pred = Predicate::or(
            Predicate::gt("age", Value::Integer(18)),
            Predicate::lt("age", Value::Integer(65)),
        );
        assert!(matches!(pred, Predicate::Or(_, _)));
    }

    #[test]
    fn test_predicate_lt() {
        let pred = Predicate::lt("age", Value::Integer(18));
        assert!(matches!(pred, Predicate::Lt(_, _)));
    }

    #[test]
    fn test_predicate_lte() {
        let pred = Predicate::lte("age", Value::Integer(18));
        assert!(matches!(pred, Predicate::Lte(_, _)));
    }

    #[test]
    fn test_predicate_gt() {
        let pred = Predicate::gt("age", Value::Integer(65));
        assert!(matches!(pred, Predicate::Gt(_, _)));
    }

    #[test]
    fn test_predicate_gte() {
        let pred = Predicate::gte("age", Value::Integer(65));
        assert!(matches!(pred, Predicate::Gte(_, _)));
    }

    #[test]
    fn test_predicate_not() {
        let pred = Predicate::not(Predicate::eq("active", Value::Boolean(true)));
        assert!(matches!(pred, Predicate::Not(_)));
    }

    #[test]
    fn test_predicate_is_null() {
        let pred = Predicate::IsNull(Box::new(Expr::Column("email".to_string())));
        assert!(matches!(pred, Predicate::IsNull(_)));
    }

    #[test]
    fn test_predicate_is_not_null() {
        let pred = Predicate::IsNotNull(Box::new(Expr::Column("email".to_string())));
        assert!(matches!(pred, Predicate::IsNotNull(_)));
    }

    #[test]
    fn test_predicate_in() {
        let pred = Predicate::In(
            Box::new(Expr::Column("status".to_string())),
            vec![
                Expr::Value(Value::Text("active".to_string())),
                Expr::Value(Value::Text("pending".to_string())),
            ],
        );
        assert!(matches!(pred, Predicate::In(_, _)));
    }

    #[test]
    fn test_expr_value() {
        let expr = Expr::Value(Value::Integer(42));
        assert!(matches!(expr, Expr::Value(_)));
    }

    #[test]
    fn test_expr_column() {
        let expr = Expr::Column("name".to_string());
        assert!(matches!(expr, Expr::Column(_)));
    }

    #[test]
    fn test_expr_parameter() {
        let expr = Expr::Parameter(1);
        assert!(matches!(expr, Expr::Parameter(1)));
    }

    #[test]
    fn test_index_op_eq() {
        let op = IndexOp::Eq(42);
        assert!(matches!(op, IndexOp::Eq(42)));
    }

    #[test]
    fn test_index_op_range() {
        let op = IndexOp::Range(1, 100);
        assert!(matches!(op, IndexOp::Range(1, 100)));
    }

    #[test]
    fn test_predicate_debug() {
        let pred = Predicate::eq("id", Value::Integer(42));
        let debug = format!("{:?}", pred);
        assert!(debug.contains("Eq"));
    }
}
