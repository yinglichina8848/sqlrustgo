// Tests for optimizer rules
use sqlrustgo_optimizer::rules::{Expr, JoinType, Operator, Plan, Value};
use std::f64::consts::PI;

#[test]
fn test_plan_table_scan() {
    let plan = Plan::TableScan {
        table_name: "users".to_string(),
        projection: Some(vec![0, 1, 2]),
    };
    assert_eq!(plan.type_name(), "TableScan");
}

#[test]
fn test_plan_filter() {
    let input = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let expr = Expr::Column("age".to_string());
    let plan = Plan::Filter {
        predicate: expr,
        input: Box::new(input),
    };
    assert_eq!(plan.type_name(), "Filter");
}

#[test]
fn test_plan_projection() {
    let input = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let expr = Expr::Column("name".to_string());
    let plan = Plan::Projection {
        expr: vec![expr],
        input: Box::new(input),
    };
    assert_eq!(plan.type_name(), "Projection");
}

#[test]
fn test_plan_join() {
    let left = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let right = Plan::TableScan {
        table_name: "orders".to_string(),
        projection: None,
    };
    let plan = Plan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Inner,
        condition: None,
    };
    assert_eq!(plan.type_name(), "Join");
}

#[test]
fn test_plan_aggregate() {
    let input = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let plan = Plan::Aggregate {
        group_by: vec![Expr::Column("dept".to_string())],
        aggregates: vec![Expr::Column("salary".to_string())],
        input: Box::new(input),
    };
    assert_eq!(plan.type_name(), "Aggregate");
}

#[test]
fn test_plan_sort() {
    let input = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let plan = Plan::Sort {
        expr: vec![Expr::Column("name".to_string())],
        input: Box::new(input),
    };
    assert_eq!(plan.type_name(), "Sort");
}

#[test]
fn test_plan_limit() {
    let input = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let plan = Plan::Limit {
        limit: 100,
        input: Box::new(input),
    };
    assert_eq!(plan.type_name(), "Limit");
}

#[test]
fn test_plan_empty_relation() {
    let plan = Plan::EmptyRelation;
    assert_eq!(plan.type_name(), "EmptyRelation");
}

#[test]
fn test_value_debug() {
    let v = Value::Integer(42);
    assert!(format!("{:?}", v).contains("Integer"));

    let v2 = Value::Float(PI);
    assert!(format!("{:?}", v2).contains("Float"));

    let v3 = Value::Boolean(true);
    assert!(format!("{:?}", v3).contains("Boolean"));

    let v4 = Value::String("hello".to_string());
    assert!(format!("{:?}", v4).contains("String"));

    let v5 = Value::Null;
    assert!(format!("{:?}", v5).contains("Null"));
}

#[test]
fn test_join_type_debug() {
    assert_eq!(format!("{:?}", JoinType::Inner), "Inner");
    assert_eq!(format!("{:?}", JoinType::Left), "Left");
    assert_eq!(format!("{:?}", JoinType::Right), "Right");
    assert_eq!(format!("{:?}", JoinType::Full), "Full");
}

#[test]
fn test_operator_debug() {
    assert_eq!(format!("{:?}", Operator::Plus), "Plus");
    assert_eq!(format!("{:?}", Operator::Minus), "Minus");
    assert_eq!(format!("{:?}", Operator::Multiply), "Multiply");
    assert_eq!(format!("{:?}", Operator::Divide), "Divide");
}
