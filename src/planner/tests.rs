//! Tests for planner module

use crate::planner::{
    AggregateFunction, Column, DataType, Expr, Field, JoinType, LogicalPlan, Operator, Schema,
    SortExpr,
};
use crate::types::Value;

// ============================================================================
// Column Tests
// ============================================================================

#[test]
fn test_column_new() {
    let col = Column::new("id".to_string());
    assert_eq!(col.name, "id");
    assert_eq!(col.relation, None);
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

// ============================================================================
// Expr Tests
// ============================================================================

#[test]
fn test_expr_column() {
    let expr = Expr::column("name");
    assert!(matches!(expr, Expr::Column(_)));
}

#[test]
fn test_expr_literal() {
    let expr = Expr::literal(Value::Integer(42));
    assert!(matches!(expr, Expr::Literal(Value::Integer(42))));
}

#[test]
fn test_expr_binary() {
    let expr = Expr::binary_expr(
        Expr::column("a"),
        Operator::Plus,
        Expr::column("b"),
    );
    assert!(matches!(expr, Expr::BinaryExpr { .. }));
}

#[test]
fn test_expr_display() {
    let expr = Expr::column("id");
    assert_eq!(expr.to_string(), "id");

    let lit = Expr::literal(Value::Integer(100));
    assert_eq!(lit.to_string(), "100");

    let binary = Expr::binary_expr(
        Expr::column("a"),
        Operator::Plus,
        Expr::column("b"),
    );
    assert_eq!(binary.to_string(), "(a + b)");

    let alias = Expr::Alias {
        expr: Box::new(Expr::column("name")),
        name: "full_name".to_string(),
    };
    assert_eq!(alias.to_string(), "name AS full_name");

    assert_eq!(Expr::Wildcard.to_string(), "*");

    let qualified_wildcard = Expr::QualifiedWildcard {
        qualifier: "users".to_string(),
    };
    assert_eq!(qualified_wildcard.to_string(), "users.*");
}

#[test]
fn test_expr_aggregate_function_display() {
    let count = Expr::AggregateFunction {
        func: AggregateFunction::Count,
        args: vec![Expr::column("id")],
        distinct: false,
    };
    assert_eq!(count.to_string(), "COUNT(id)");

    let sum = Expr::AggregateFunction {
        func: AggregateFunction::Sum,
        args: vec![Expr::column("amount")],
        distinct: false,
    };
    assert_eq!(sum.to_string(), "SUM(amount)");

    let avg = Expr::AggregateFunction {
        func: AggregateFunction::Avg,
        args: vec![Expr::column("price")],
        distinct: true,
    };
    assert_eq!(avg.to_string(), "AVG(DISTINCT price)");
}

// ============================================================================
// Operator Tests
// ============================================================================

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

// ============================================================================
// Schema Tests
// ============================================================================

#[test]
fn test_schema_new() {
    let fields = vec![
        Field::new_not_null("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ];
    let schema = Schema::new(fields);
    assert_eq!(schema.fields.len(), 2);
}

#[test]
fn test_schema_empty() {
    let schema = Schema::empty();
    assert!(schema.fields.is_empty());
}

#[test]
fn test_schema_field() {
    let schema = Schema::new(vec![
        Field::new_not_null("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ]);

    assert!(schema.field("id").is_some());
    assert!(schema.field("name").is_some());
    assert!(schema.field("unknown").is_none());
}

#[test]
fn test_schema_field_index() {
    let schema = Schema::new(vec![
        Field::new_not_null("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
        Field::new("age".to_string(), DataType::Integer),
    ]);

    assert_eq!(schema.field_index("id"), Some(0));
    assert_eq!(schema.field_index("name"), Some(1));
    assert_eq!(schema.field_index("age"), Some(2));
    assert_eq!(schema.field_index("unknown"), None);
}

// ============================================================================
// Field Tests
// ============================================================================

#[test]
fn test_field_new() {
    let field = Field::new("name".to_string(), DataType::Text);
    assert_eq!(field.name, "name");
    assert_eq!(field.data_type, DataType::Text);
    assert!(field.nullable);
}

#[test]
fn test_field_new_not_null() {
    let field = Field::new_not_null("id".to_string(), DataType::Integer);
    assert_eq!(field.name, "id");
    assert_eq!(field.data_type, DataType::Integer);
    assert!(!field.nullable);
}

// ============================================================================
// DataType Tests
// ============================================================================

#[test]
fn test_datatype_from_sql_type() {
    assert_eq!(DataType::from_sql_type("INTEGER"), DataType::Integer);
    assert_eq!(DataType::from_sql_type("INT"), DataType::Integer);
    assert_eq!(DataType::from_sql_type("int"), DataType::Integer);
    assert_eq!(DataType::from_sql_type("FLOAT"), DataType::Float);
    assert_eq!(DataType::from_sql_type("DOUBLE"), DataType::Float);
    assert_eq!(DataType::from_sql_type("REAL"), DataType::Float);
    assert_eq!(DataType::from_sql_type("TEXT"), DataType::Text);
    assert_eq!(DataType::from_sql_type("VARCHAR"), DataType::Text);
    assert_eq!(DataType::from_sql_type("CHAR"), DataType::Text);
    assert_eq!(DataType::from_sql_type("BLOB"), DataType::Blob);
    assert_eq!(DataType::from_sql_type("BINARY"), DataType::Blob);
    assert_eq!(DataType::from_sql_type("BOOLEAN"), DataType::Boolean);
    assert_eq!(DataType::from_sql_type("BOOL"), DataType::Boolean);
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

// ============================================================================
// SortExpr Tests
// ============================================================================

#[test]
fn test_sort_expr() {
    let expr = SortExpr {
        expr: Expr::column("name"),
        asc: true,
        nulls_first: false,
    };
    assert!(expr.asc);
    assert!(!expr.nulls_first);
}

// ============================================================================
// JoinType Tests
// ============================================================================

#[test]
fn test_join_type() {
    assert_eq!(JoinType::Inner, JoinType::Inner);
    assert_eq!(JoinType::Left, JoinType::Left);
    assert_ne!(JoinType::Inner, JoinType::Left);
}

// ============================================================================
// AggregateFunction Tests
// ============================================================================

#[test]
fn test_aggregate_function() {
    assert_eq!(AggregateFunction::Count, AggregateFunction::Count);
    assert_eq!(AggregateFunction::Sum, AggregateFunction::Sum);
    assert_ne!(AggregateFunction::Count, AggregateFunction::Sum);
}

// ============================================================================
// Expr Display Tests
// ============================================================================

#[test]
fn test_expr_unary_display() {
    let expr = Expr::UnaryExpr {
        op: Operator::Minus,
        expr: Box::new(Expr::literal(Value::Integer(5))),
    };
    let s = expr.to_string();
    assert!(s.contains("-"));
}

#[test]
fn test_expr_aggregate_display() {
    let expr = Expr::AggregateFunction {
        func: AggregateFunction::Min,
        args: vec![Expr::column("id")],
        distinct: false,
    };
    assert_eq!(expr.to_string(), "MIN(id)");

    let expr_distinct = Expr::AggregateFunction {
        func: AggregateFunction::Count,
        args: vec![Expr::column("id")],
        distinct: true,
    };
    assert_eq!(expr_distinct.to_string(), "COUNT(DISTINCT id)");
}

#[test]
fn test_expr_aggregate_max_display() {
    // Cover line 164: MAX aggregate function display
    let expr = Expr::AggregateFunction {
        func: AggregateFunction::Max,
        args: vec![Expr::column("value")],
        distinct: false,
    };
    assert_eq!(expr.to_string(), "MAX(value)");
}

#[test]
fn test_expr_aggregate_distinct_min_max() {
    // Test DISTINCT with MIN and MAX
    let min_distinct = Expr::AggregateFunction {
        func: AggregateFunction::Min,
        args: vec![Expr::column("price")],
        distinct: true,
    };
    assert_eq!(min_distinct.to_string(), "MIN(DISTINCT price)");

    let max_distinct = Expr::AggregateFunction {
        func: AggregateFunction::Max,
        args: vec![Expr::column("price")],
        distinct: true,
    };
    assert_eq!(max_distinct.to_string(), "MAX(DISTINCT price)");
}

#[test]
fn test_expr_alias_display() {
    let expr = Expr::Alias {
        expr: Box::new(Expr::column("id")),
        name: "user_id".to_string(),
    };
    assert_eq!(expr.to_string(), "id AS user_id");
}

#[test]
fn test_expr_qualified_wildcard_display() {
    let expr = Expr::QualifiedWildcard {
        qualifier: "users".to_string(),
    };
    assert_eq!(expr.to_string(), "users.*");
}

// ============================================================================
// LogicalPlan Tests
// ============================================================================

#[test]
fn test_logical_plan_sort() {
    let child = LogicalPlan::TableScan {
        table_name: "test".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    let plan = LogicalPlan::Sort {
        input: Box::new(child),
        expr: vec![SortExpr {
            expr: Expr::column("id"),
            asc: false,
            nulls_first: true,
        }],
    };
    let schema = plan.schema();
    assert!(schema.fields.len() > 0);
}

#[test]
fn test_logical_plan_limit() {
    let child = LogicalPlan::TableScan {
        table_name: "test".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    let plan = LogicalPlan::Limit {
        input: Box::new(child),
        n: 10,
    };
    let schema = plan.schema();
    assert!(schema.fields.len() > 0);
}

#[test]
fn test_logical_plan_subquery() {
    let subquery = Box::new(LogicalPlan::TableScan {
        table_name: "inner".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    });
    let plan = LogicalPlan::Subquery {
        subquery,
        alias: Some("sq".to_string()),
    };
    let schema = plan.schema();
    assert!(schema.fields.len() > 0);
}

#[test]
fn test_logical_plan_update() {
    let input = Box::new(LogicalPlan::TableScan {
        table_name: "test".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    });
    let plan = LogicalPlan::Update {
        input,
        set_exprs: vec![("id".to_string(), Expr::literal(Value::Integer(1)))],
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    let schema = plan.schema();
    assert!(schema.fields.len() > 0);
}

#[test]
fn test_logical_plan_delete() {
    let input = Box::new(LogicalPlan::TableScan {
        table_name: "test".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    });
    let plan = LogicalPlan::Delete {
        input,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    let schema = plan.schema();
    assert!(schema.fields.len() > 0);
}

#[test]
fn test_logical_plan_children() {
    // Test children method for different plan types
    let scan = LogicalPlan::TableScan {
        table_name: "test".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    assert_eq!(scan.children().len(), 0);

    // Sort has one child
    let sort = LogicalPlan::Sort {
        input: Box::new(scan.clone()),
        expr: vec![],
    };
    assert_eq!(sort.children().len(), 1);

    // Limit has one child
    let limit = LogicalPlan::Limit {
        input: Box::new(scan.clone()),
        n: 10,
    };
    assert_eq!(limit.children().len(), 1);

    // Subquery has one child
    let subquery = LogicalPlan::Subquery {
        subquery: Box::new(scan.clone()),
        alias: Some("sq".to_string()),
    };
    assert_eq!(subquery.children().len(), 1);

    // Aggregate has one child
    let agg = LogicalPlan::Aggregate {
        input: Box::new(scan.clone()),
        group_expr: vec![],
        aggr_expr: vec![],
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    assert_eq!(agg.children().len(), 1);

    // Join has two children
    let scan2 = LogicalPlan::TableScan {
        table_name: "test2".to_string(),
        filters: vec![],
        projection: None,
        limit: None,
        schema: Schema::new(vec![Field::new_not_null("id".to_string(), DataType::Integer)]),
    };
    let join = LogicalPlan::Join {
        left: Box::new(scan.clone()),
        right: Box::new(scan2),
        on: vec![],
        filter: None,
        join_type: JoinType::Inner,
        schema: Schema::new(vec![]),
    };
    assert_eq!(join.children().len(), 2);

    // Update has one child
    let update = LogicalPlan::Update {
        input: Box::new(scan.clone()),
        set_exprs: vec![],
        schema: Schema::new(vec![]),
    };
    assert_eq!(update.children().len(), 1);

    // Delete has one child
    let delete = LogicalPlan::Delete {
        input: Box::new(scan),
        schema: Schema::new(vec![]),
    };
    assert_eq!(delete.children().len(), 1);
}
