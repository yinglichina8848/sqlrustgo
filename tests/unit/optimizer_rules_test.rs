// Tests for optimizer rules
use sqlrustgo_optimizer::rules::{
    Expr, JoinType, Operator, Plan, PredicatePushdown, ProjectionPruning, RuleContext,
    SimpleColumnSet, Value,
};
use sqlrustgo_optimizer::Rule;
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
    assert_eq!(format!("{:?}", Operator::Eq), "Eq");
    assert_eq!(format!("{:?}", Operator::NotEq), "NotEq");
    assert_eq!(format!("{:?}", Operator::Gt), "Gt");
    assert_eq!(format!("{:?}", Operator::Lt), "Lt");
    assert_eq!(format!("{:?}", Operator::And), "And");
    assert_eq!(format!("{:?}", Operator::Or), "Or");
    assert_eq!(format!("{:?}", Operator::Not), "Not");
    assert_eq!(format!("{:?}", Operator::Like), "Like");
}

#[test]
fn test_plan_get_children() {
    let table_scan = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    assert!(table_scan.get_children().is_empty());

    let filter = Plan::Filter {
        predicate: Expr::Column("id".to_string()),
        input: Box::new(table_scan),
    };
    assert_eq!(filter.get_children().len(), 1);

    let left = Plan::TableScan {
        table_name: "users".to_string(),
        projection: None,
    };
    let right = Plan::TableScan {
        table_name: "orders".to_string(),
        projection: None,
    };
    let join = Plan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinType::Inner,
        condition: None,
    };
    assert_eq!(join.get_children().len(), 2);
}

#[test]
fn test_plan_get_child_mut() {
    let mut plan = Plan::Filter {
        predicate: Expr::Column("id".to_string()),
        input: Box::new(Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        }),
    };
    assert!(plan.get_child_mut().is_some());

    let mut empty = Plan::EmptyRelation;
    assert!(empty.get_child_mut().is_none());
}

#[test]
fn test_plan_index_scan() {
    let plan = Plan::IndexScan {
        table_name: "users".to_string(),
        index_name: "idx_id".to_string(),
        predicate: Some(Expr::Column("id".to_string())),
    };
    assert_eq!(plan.type_name(), "IndexScan");
}

#[test]
fn test_expr_column() {
    let expr = Expr::Column("name".to_string());
    match expr {
        Expr::Column(s) => assert_eq!(s, "name"),
        _ => panic!("Expected Column variant"),
    }
}

#[test]
fn test_expr_literal() {
    let expr = Expr::Literal(Value::Integer(42));
    match expr {
        Expr::Literal(Value::Integer(i)) => assert_eq!(i, 42),
        _ => panic!("Expected Literal(Integer) variant"),
    }

    let expr2 = Expr::Literal(Value::String("hello".to_string()));
    match expr2 {
        Expr::Literal(Value::String(s)) => assert_eq!(s, "hello"),
        _ => panic!("Expected Literal(String) variant"),
    }
}

#[test]
fn test_expr_binary_expr() {
    let left = Box::new(Expr::Literal(Value::Integer(1)));
    let right = Box::new(Expr::Literal(Value::Integer(2)));
    let expr = Expr::BinaryExpr {
        left,
        op: Operator::Plus,
        right,
    };
    match expr {
        Expr::BinaryExpr { left, op, right } => {
            assert!(matches!(*left, Expr::Literal(Value::Integer(1))));
            assert!(matches!(op, Operator::Plus));
            assert!(matches!(*right, Expr::Literal(Value::Integer(2))));
        }
        _ => panic!("Expected BinaryExpr variant"),
    }
}

#[test]
fn test_expr_unary_expr() {
    let expr = Box::new(Expr::Literal(Value::Boolean(true)));
    let unary = Expr::UnaryExpr {
        op: Operator::Not,
        expr,
    };
    match unary {
        Expr::UnaryExpr { op, expr } => {
            assert!(matches!(op, Operator::Not));
            assert!(matches!(*expr, Expr::Literal(Value::Boolean(true))));
        }
        _ => panic!("Expected UnaryExpr variant"),
    }
}

#[test]
fn test_simple_column_set() {
    let mut set = SimpleColumnSet::new();
    assert!(set.is_all);
    assert!(set.indices.is_empty());

    set.add("name");
    assert!(!set.is_all);
}

#[test]
fn test_predicate_pushdown_new() {
    let rule = PredicatePushdown::new();
    assert_eq!(rule.name(), "PredicatePushdown");
}

#[test]
fn test_predicate_pushdown_apply_table_scan() {
    let mut plan = Plan::Filter {
        predicate: Expr::Column("id".to_string()),
        input: Box::new(Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        }),
    };

    let rule = PredicatePushdown::new();
    let mut ctx = RuleContext::new();
    let changed = rule.apply(&mut plan, &mut ctx);
    assert!(!changed);
}

#[test]
fn test_projection_pruning_new() {
    let rule = ProjectionPruning::new();
    assert_eq!(rule.name(), "ProjectionPruning");
}

#[test]
fn test_rule_context_new() {
    let ctx = RuleContext::new();
    assert_eq!(ctx.depth, 0);
    assert_eq!(ctx.rules_applied, 0);
    assert!(ctx.continue_optimization);
    assert!(ctx.index_hints.is_empty());
    assert!(ctx.session_vars.is_empty());
}

#[test]
fn test_rule_context_with_index_hints() {
    let ctx = RuleContext::with_index_hints(vec![]);
    assert!(ctx.index_hints.is_empty());
    assert_eq!(ctx.depth, 0);
}

#[test]
fn test_rule_context_increment_depth() {
    let mut ctx = RuleContext::new();
    ctx.increment_depth();
    assert_eq!(ctx.depth, 1);
    ctx.increment_depth();
    assert_eq!(ctx.depth, 2);
}

#[test]
fn test_rule_context_decrement_depth() {
    let mut ctx = RuleContext::new();
    ctx.depth = 5;
    ctx.decrement_depth();
    assert_eq!(ctx.depth, 4);
    ctx.decrement_depth();
    assert_eq!(ctx.depth, 3);
}

#[test]
fn test_rule_context_record_rule_applied() {
    let mut ctx = RuleContext::new();
    ctx.record_rule_applied();
    assert_eq!(ctx.rules_applied, 1);
    ctx.record_rule_applied();
    assert_eq!(ctx.rules_applied, 2);
}

#[test]
fn test_value_equality() {
    assert_eq!(Value::Integer(42), Value::Integer(42));
    assert_eq!(
        Value::String("hello".to_string()),
        Value::String("hello".to_string())
    );
    assert_eq!(Value::Boolean(true), Value::Boolean(true));
    assert_ne!(Value::Integer(42), Value::Integer(43));
}

#[test]
fn test_operator_equality() {
    assert_eq!(Operator::Plus, Operator::Plus);
    assert_eq!(Operator::Eq, Operator::Eq);
    assert_ne!(Operator::Plus, Operator::Minus);
}

#[test]
fn test_join_type_equality() {
    assert_eq!(JoinType::Inner, JoinType::Inner);
    assert_eq!(JoinType::Left, JoinType::Left);
    assert_eq!(JoinType::Right, JoinType::Right);
    assert_eq!(JoinType::Full, JoinType::Full);
}
