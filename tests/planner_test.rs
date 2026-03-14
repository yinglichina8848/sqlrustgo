// Planner Tests
use sqlrustgo_planner::{
    AggregateFunction, Column, DataType, Expr, Field, JoinType, LogicalPlan, Operator, Schema,
};
use sqlrustgo_types::Value;

#[test]
fn test_schema_new() {
    let fields = vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ];
    let schema = Schema::new(fields);
    assert_eq!(schema.fields.len(), 2);
}

#[test]
fn test_schema_field() {
    let fields = vec![
        Field::new("id".to_string(), DataType::Integer),
        Field::new("name".to_string(), DataType::Text),
    ];
    let schema = Schema::new(fields);

    let field = schema.field("id");
    assert!(field.is_some());

    let field = schema.field("name");
    assert!(field.is_some());

    let field = schema.field("not_exist");
    assert!(field.is_none());
}

#[test]
fn test_field_new() {
    let field = Field::new("test".to_string(), DataType::Integer);
    assert_eq!(field.name, "test");
    assert_eq!(field.data_type, DataType::Integer);
    assert!(field.nullable); // default is true
}

#[test]
fn test_field_new_not_null() {
    let field = Field::new_not_null("test".to_string(), DataType::Integer);
    assert_eq!(field.name, "test");
    assert_eq!(field.data_type, DataType::Integer);
    assert!(!field.nullable);
}

#[test]
fn test_datatype_variants() {
    // Test all DataType variants
    let _ = DataType::Boolean;
    let _ = DataType::Integer;
    let _ = DataType::Float;
    let _ = DataType::Text;
    let _ = DataType::Blob;
    let _ = DataType::Null;
}

#[test]
fn test_datatype_from_sql_type() {
    assert_eq!(DataType::from_sql_type("INTEGER"), DataType::Integer);
    assert_eq!(DataType::from_sql_type("INT"), DataType::Integer);
    assert_eq!(DataType::from_sql_type("FLOAT"), DataType::Float);
    assert_eq!(DataType::from_sql_type("TEXT"), DataType::Text);
    assert_eq!(DataType::from_sql_type("BLOB"), DataType::Blob);
    assert_eq!(DataType::from_sql_type("BOOLEAN"), DataType::Boolean);
}

#[test]
fn test_join_type_variants() {
    assert_eq!(format!("{:?}", JoinType::Inner), "Inner");
    assert_eq!(format!("{:?}", JoinType::Left), "Left");
    assert_eq!(format!("{:?}", JoinType::Right), "Right");
    assert_eq!(format!("{:?}", JoinType::Full), "Full");
    assert_eq!(format!("{:?}", JoinType::Cross), "Cross");
}

#[test]
fn test_aggregate_function_variants() {
    assert_eq!(format!("{:?}", AggregateFunction::Count), "Count");
    assert_eq!(format!("{:?}", AggregateFunction::Sum), "Sum");
    assert_eq!(format!("{:?}", AggregateFunction::Avg), "Avg");
    assert_eq!(format!("{:?}", AggregateFunction::Min), "Min");
    assert_eq!(format!("{:?}", AggregateFunction::Max), "Max");
}

#[test]
fn test_operator_variants() {
    assert_eq!(format!("{:?}", Operator::Eq), "Eq");
    assert_eq!(format!("{:?}", Operator::Lt), "Lt");
    assert_eq!(format!("{:?}", Operator::LtEq), "LtEq");
    assert_eq!(format!("{:?}", Operator::Gt), "Gt");
    assert_eq!(format!("{:?}", Operator::GtEq), "GtEq");
    assert_eq!(format!("{:?}", Operator::Plus), "Plus");
    assert_eq!(format!("{:?}", Operator::Minus), "Minus");
    assert_eq!(format!("{:?}", Operator::Multiply), "Multiply");
    assert_eq!(format!("{:?}", Operator::Divide), "Divide");
}

#[test]
fn test_expr_column() {
    let col = Column::new("id".to_string());
    let _expr = Expr::Column(col);
}

#[test]
fn test_expr_literal() {
    let _expr = Expr::Literal(Value::Integer(42));
}

#[test]
fn test_expr_binary() {
    let col = Column::new("a".to_string());
    let left = Expr::Column(col);
    let right = Expr::Literal(Value::Integer(1));
    let _expr = Expr::BinaryExpr {
        left: Box::new(left),
        op: Operator::Plus,
        right: Box::new(right),
    };
}
