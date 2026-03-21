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

pub use logical_plan::{LogicalPlan, SetOperationType};
pub use optimizer::{DefaultOptimizer, NoOpOptimizer, Optimizer, OptimizerRule};
pub use physical_plan::{
    AggregateExec, ExplainExec, FilterExec, HashJoinExec, IndexScanExec, LimitExec,
    OperatorMetrics, PhysicalPlan, ProjectionExec, SeqScanExec, SetOperationExec,
    SortMergeJoinExec,
};
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
    Concatenate,
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

    pub fn evaluate(&self, row: &[Value], schema: &Schema) -> Option<Value> {
        match self {
            Expr::Column(col) => {
                let idx = schema.field_index(&col.name)?;
                row.get(idx).cloned()
            }
            Expr::Literal(val) => Some(val.clone()),
            Expr::BinaryExpr { left, op, right } => {
                let lv = left.evaluate(row, schema)?;
                let rv = right.evaluate(row, schema)?;
                evaluate_binary_op(&lv, op, &rv)
            }
            Expr::UnaryExpr { op, expr } => {
                let v = expr.evaluate(row, schema)?;
                evaluate_unary_op(&v, op)
            }
            Expr::AggregateFunction { .. } => None,
            Expr::Alias { expr, .. } => expr.evaluate(row, schema),
            Expr::Wildcard => None,
            Expr::QualifiedWildcard { .. } => None,
        }
    }

    pub fn matches(&self, row: &[Value], schema: &Schema) -> bool {
        if let Some(value) = self.evaluate(row, schema) {
            value.to_bool()
        } else {
            false
        }
    }
}

fn evaluate_binary_op(left: &Value, op: &Operator, right: &Value) -> Option<Value> {
    use sqlrustgo_types::Value::*;
    use Operator::*;

    match (left, op, right) {
        (Integer(l), Eq, Integer(r)) => Some(Boolean(l == r)),
        (Integer(l), NotEq, Integer(r)) => Some(Boolean(l != r)),
        (Integer(l), Lt, Integer(r)) => Some(Boolean(l < r)),
        (Integer(l), LtEq, Integer(r)) => Some(Boolean(l <= r)),
        (Integer(l), Gt, Integer(r)) => Some(Boolean(l > r)),
        (Integer(l), GtEq, Integer(r)) => Some(Boolean(l >= r)),
        (Text(l), Eq, Text(r)) => Some(Boolean(l == r)),
        (Text(l), NotEq, Text(r)) => Some(Boolean(l != r)),
        (Text(l), Lt, Text(r)) => Some(Boolean(l < r)),
        (Text(l), LtEq, Text(r)) => Some(Boolean(l <= r)),
        (Text(l), Gt, Text(r)) => Some(Boolean(l > r)),
        (Text(l), GtEq, Text(r)) => Some(Boolean(l >= r)),
        (Integer(l), Plus, Integer(r)) => Some(Integer(l + r)),
        (Integer(l), Minus, Integer(r)) => Some(Integer(l - r)),
        (Integer(l), Multiply, Integer(r)) => Some(Integer(l * r)),
        (Integer(l), Divide, Integer(r)) => {
            if *r != 0 {
                Some(Integer(l / r))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn evaluate_unary_op(value: &Value, op: &Operator) -> Option<Value> {
    use sqlrustgo_types::Value::*;

    match (value, op) {
        (Integer(v), Operator::Not) => Some(Boolean(*v == 0)),
        (Boolean(v), Operator::Not) => Some(Boolean(!v)),
        (Integer(v), Operator::Minus) => Some(Integer(-v)),
        _ => None,
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
            Operator::Concatenate => write!(f, "||"),
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

    #[test]
    fn test_expr_display_unary() {
        let expr = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::literal(Value::Integer(1))),
        };
        assert!(!expr.to_string().is_empty());
    }

    #[test]
    fn test_expr_display_aggregate() {
        let expr = Expr::AggregateFunction {
            func: AggregateFunction::Count,
            args: vec![Expr::column("id")],
            distinct: false,
        };
        assert!(expr.to_string().contains("COUNT"));

        let expr_distinct = Expr::AggregateFunction {
            func: AggregateFunction::Sum,
            args: vec![Expr::column("amount")],
            distinct: true,
        };
        assert!(expr_distinct.to_string().contains("DISTINCT"));
    }

    #[test]
    fn test_expr_display_alias() {
        let expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "user_id".to_string(),
        };
        assert!(expr.to_string().contains("AS"));
    }

    #[test]
    fn test_expr_display_wildcard() {
        let expr = Expr::Wildcard;
        assert_eq!(expr.to_string(), "*");

        let qualified = Expr::QualifiedWildcard {
            qualifier: "users".to_string(),
        };
        assert_eq!(qualified.to_string(), "users.*");
    }

    #[test]
    fn test_operator_display_all() {
        assert_eq!(Operator::And.to_string(), "AND");
        assert_eq!(Operator::Or.to_string(), "OR");
        assert_eq!(Operator::Like.to_string(), "LIKE");
        assert_eq!(Operator::Plus.to_string(), "+");
        assert_eq!(Operator::Minus.to_string(), "-");
    }

    #[test]
    fn test_expr_evaluate_column() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = Expr::column("id");
        let row = vec![Value::Integer(42)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_expr_evaluate_literal() {
        let schema = Schema::new(vec![]);
        let expr = Expr::literal(Value::Integer(42));
        let row = vec![];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_expr_evaluate_binary_expr() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let expr = Expr::binary_expr(Expr::column("a"), Operator::Plus, Expr::column("b"));
        let row = vec![Value::Integer(10), Value::Integer(5)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Integer(15)));
    }

    #[test]
    fn test_expr_evaluate_binary_expr_integer_comparison() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let expr = Expr::binary_expr(Expr::column("a"), Operator::Gt, Expr::column("b"));
        let row = vec![Value::Integer(10), Value::Integer(5)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Boolean(true)));
    }

    #[test]
    fn test_expr_evaluate_binary_expr_text_comparison() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Text),
            Field::new("b".to_string(), DataType::Text),
        ]);
        let expr = Expr::binary_expr(Expr::column("a"), Operator::Lt, Expr::column("b"));
        let row = vec![
            Value::Text("apple".to_string()),
            Value::Text("banana".to_string()),
        ];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Boolean(true)));
    }

    #[test]
    fn test_expr_evaluate_binary_expr_divide_by_zero() {
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let expr = Expr::binary_expr(Expr::column("a"), Operator::Divide, Expr::column("b"));
        let row = vec![Value::Integer(10), Value::Integer(0)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, None);
    }

    #[test]
    fn test_expr_evaluate_unary_expr_not() {
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let expr = Expr::UnaryExpr {
            op: Operator::Not,
            expr: Box::new(Expr::column("a")),
        };
        let row = vec![Value::Integer(0)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Boolean(true)));
    }

    #[test]
    fn test_expr_evaluate_unary_expr_minus() {
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let expr = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::column("a")),
        };
        let row = vec![Value::Integer(10)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Integer(-10)));
    }

    #[test]
    fn test_expr_evaluate_alias() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = Expr::Alias {
            expr: Box::new(Expr::column("id")),
            name: "user_id".to_string(),
        };
        let row = vec![Value::Integer(42)];
        let result = expr.evaluate(&row, &schema);
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_expr_matches() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = Expr::binary_expr(
            Expr::column("id"),
            Operator::Gt,
            Expr::literal(Value::Integer(10)),
        );
        let row = vec![Value::Integer(20)];
        assert!(expr.matches(&row, &schema));
    }

    #[test]
    fn test_expr_matches_returns_false_for_null() {
        let schema = Schema::new(vec![Field::new("id".to_string(), DataType::Integer)]);
        let expr = Expr::Wildcard;
        let row = vec![Value::Integer(20)];
        assert!(!expr.matches(&row, &schema));
    }

    #[test]
    fn test_evaluate_binary_op_integer() {
        use sqlrustgo_types::Value::*;
        // Test multiplication
        let result = evaluate_binary_op(&Integer(3), &Operator::Multiply, &Integer(4));
        assert_eq!(result, Some(Integer(12)));
        // Test subtraction
        let result = evaluate_binary_op(&Integer(10), &Operator::Minus, &Integer(3));
        assert_eq!(result, Some(Integer(7)));
    }

    #[test]
    fn test_evaluate_unary_op_not_boolean() {
        use sqlrustgo_types::Value::*;
        let result = evaluate_unary_op(&Boolean(true), &Operator::Not);
        assert_eq!(result, Some(Boolean(false)));
    }

    #[test]
    fn test_schema_default() {
        let schema = Schema::default();
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_join_type_variants_left_semi() {
        assert!(matches!(JoinType::LeftSemi, JoinType::LeftSemi));
        assert!(matches!(JoinType::LeftAnti, JoinType::LeftAnti));
        assert!(matches!(JoinType::RightSemi, JoinType::RightSemi));
        assert!(matches!(JoinType::RightAnti, JoinType::RightAnti));
    }

    #[test]
    fn test_aggregate_function_all_variants() {
        // Test all variants are accessible
        let funcs = vec![
            AggregateFunction::Count,
            AggregateFunction::Sum,
            AggregateFunction::Avg,
            AggregateFunction::Min,
            AggregateFunction::Max,
        ];
        assert_eq!(funcs.len(), 5);
    }

    #[test]
    fn test_operator_all_variants() {
        // Test all operator variants
        let ops = vec![
            Operator::Eq,
            Operator::NotEq,
            Operator::Lt,
            Operator::LtEq,
            Operator::Gt,
            Operator::GtEq,
            Operator::Plus,
            Operator::Minus,
            Operator::Multiply,
            Operator::Divide,
            Operator::Modulo,
            Operator::And,
            Operator::Or,
            Operator::Not,
            Operator::Like,
        ];
        assert_eq!(ops.len(), 15);
    }

    #[test]
    fn test_datatype_all_variants() {
        let types = vec![
            DataType::Boolean,
            DataType::Integer,
            DataType::Float,
            DataType::Text,
            DataType::Blob,
            DataType::Null,
        ];
        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_field_with_nullable_default() {
        let field = Field::new("id".to_string(), DataType::Integer);
        assert!(field.nullable);

        let field_not_null = Field::new_not_null("id".to_string(), DataType::Integer);
        assert!(!field_not_null.nullable);
    }

    // === Tests for expression evaluation ===

    #[test]
    fn test_expr_evaluate_binary_op_unsupported_types() {
        // Test binary operations on unsupported type combinations
        // This tests the _ => None case at line 209
        let schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Integer),
            Field::new("b".to_string(), DataType::Integer),
        ]);
        let row = vec![Value::Integer(5), Value::Integer(3)];

        // Test Integer modulo (not supported in evaluate_binary_op)
        let expr = Expr::binary_expr(Expr::column("a"), Operator::Modulo, Expr::column("b"));
        let result = expr.evaluate(&row, &schema);
        assert!(result.is_none()); // Modulo on integers returns None

        // Test Float arithmetic (not supported in evaluate_binary_op)
        let float_schema = Schema::new(vec![
            Field::new("a".to_string(), DataType::Float),
            Field::new("b".to_string(), DataType::Float),
        ]);
        let float_row = vec![Value::Float(5.0), Value::Float(3.0)];
        let expr2 = Expr::binary_expr(Expr::column("a"), Operator::Plus, Expr::column("b"));
        let result2 = expr2.evaluate(&float_row, &float_schema);
        assert!(result2.is_none()); // Float arithmetic returns None
    }

    #[test]
    fn test_expr_evaluate_unary_op_unsupported() {
        // Test unary operations on unsupported types
        // This tests the _ => None case at line 220
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Text)]);
        let row = vec![Value::Text("hello".to_string())];

        // Test NOT operator on Text (not supported)
        let expr = Expr::UnaryExpr {
            op: Operator::Not,
            expr: Box::new(Expr::column("a")),
        };
        let result = expr.evaluate(&row, &schema);
        assert!(result.is_none());

        // Test Minus operator on Text (not supported)
        let expr2 = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::column("a")),
        };
        let result2 = expr2.evaluate(&row, &schema);
        assert!(result2.is_none());
    }

    #[test]
    fn test_expr_evaluate_qualified_wildcard() {
        // Test QualifiedWildcard evaluate returns None
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let row = vec![Value::Integer(5)];

        let expr = Expr::QualifiedWildcard {
            qualifier: "table".to_string(),
        };
        let result = expr.evaluate(&row, &schema);
        assert!(result.is_none());
    }

    #[test]
    fn test_expr_evaluate_aggregate_function() {
        // Test AggregateFunction evaluate returns None
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let row = vec![Value::Integer(5)];

        let expr = Expr::AggregateFunction {
            func: AggregateFunction::Count,
            args: vec![Expr::column("a")],
            distinct: false,
        };
        let result = expr.evaluate(&row, &schema);
        assert!(result.is_none());
    }

    #[test]
    fn test_expr_matches_returns_false_for_none() {
        // Test matches returns false when evaluate returns None
        let schema = Schema::new(vec![Field::new("a".to_string(), DataType::Integer)]);
        let row = vec![Value::Integer(5)];

        // AggregateFunction evaluate returns None, so matches should return false
        let expr = Expr::AggregateFunction {
            func: AggregateFunction::Count,
            args: vec![Expr::column("a")],
            distinct: false,
        };
        let result = expr.matches(&row, &schema);
        assert!(!result);
    }

    #[test]
    fn test_evaluate_binary_op_integer_comparisons() {
        use sqlrustgo_types::Value::*;

        // Test integer comparisons
        assert_eq!(
            evaluate_binary_op(&Integer(1), &Operator::Lt, &Integer(2)),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(2), &Operator::Lt, &Integer(1)),
            Some(Boolean(false))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(1), &Operator::LtEq, &Integer(1)),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(2), &Operator::Gt, &Integer(1)),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(1), &Operator::GtEq, &Integer(1)),
            Some(Boolean(true))
        );
    }

    #[test]
    fn test_evaluate_binary_op_text_comparisons() {
        use sqlrustgo_types::Value::*;

        // Test text comparisons
        assert_eq!(
            evaluate_binary_op(
                &Text("a".to_string()),
                &Operator::Lt,
                &Text("b".to_string())
            ),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(
                &Text("b".to_string()),
                &Operator::Gt,
                &Text("a".to_string())
            ),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(
                &Text("a".to_string()),
                &Operator::Eq,
                &Text("a".to_string())
            ),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(
                &Text("a".to_string()),
                &Operator::NotEq,
                &Text("b".to_string())
            ),
            Some(Boolean(true))
        );
    }

    #[test]
    fn test_evaluate_binary_op_text_lt_eq() {
        use sqlrustgo_types::Value::*;

        assert_eq!(
            evaluate_binary_op(
                &Text("a".to_string()),
                &Operator::LtEq,
                &Text("a".to_string())
            ),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(
                &Text("a".to_string()),
                &Operator::LtEq,
                &Text("b".to_string())
            ),
            Some(Boolean(true))
        );
    }

    #[test]
    fn test_evaluate_binary_op_text_gt_eq() {
        use sqlrustgo_types::Value::*;

        assert_eq!(
            evaluate_binary_op(
                &Text("b".to_string()),
                &Operator::GtEq,
                &Text("b".to_string())
            ),
            Some(Boolean(true))
        );
        assert_eq!(
            evaluate_binary_op(
                &Text("b".to_string()),
                &Operator::GtEq,
                &Text("a".to_string())
            ),
            Some(Boolean(true))
        );
    }

    #[test]
    fn test_evaluate_binary_op_arithmetic() {
        use sqlrustgo_types::Value::*;

        assert_eq!(
            evaluate_binary_op(&Integer(5), &Operator::Plus, &Integer(3)),
            Some(Integer(8))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(10), &Operator::Minus, &Integer(3)),
            Some(Integer(7))
        );
        assert_eq!(
            evaluate_binary_op(&Integer(4), &Operator::Multiply, &Integer(3)),
            Some(Integer(12))
        );
    }

    #[test]
    fn test_evaluate_binary_op_division_by_zero() {
        use sqlrustgo_types::Value::*;

        let result = evaluate_binary_op(&Integer(10), &Operator::Divide, &Integer(0));
        assert!(result.is_none());
    }
}
