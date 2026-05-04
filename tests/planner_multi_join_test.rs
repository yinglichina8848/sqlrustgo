use sqlrustgo_planner::{LogicalPlan, Schema, DataType, Field, JoinType, Expr, Operator};

fn make_table_scan(table_name: &str, fields: Vec<(&str, DataType)>) -> LogicalPlan {
    LogicalPlan::TableScan {
        table_name: table_name.to_string(),
        schema: Schema::new(
            fields.into_iter().map(|(n, t)| Field::new(n.to_string(), t)).collect()
        ),
        projection: None,
    }
}

fn eq_expr(left: Expr, right: Expr) -> Expr {
    Expr::BinaryExpr {
        left: Box::new(left),
        op: Operator::Eq,
        right: Box::new(right),
    }
}

#[test]
fn test_three_table_join() {
    let t1 = make_table_scan("t1", vec![("a".into(), DataType::Integer)]);
    let t2 = make_table_scan("t2", vec![("b".into(), DataType::Integer)]);
    let t3 = make_table_scan("t3", vec![("c".into(), DataType::Integer)]);

    let join_ab = LogicalPlan::Join {
        left: Box::new(t1),
        right: Box::new(t2),
        join_type: JoinType::Inner,
        condition: Some(eq_expr(Expr::column("a"), Expr::column("b"))),
    };

    let join_abc = LogicalPlan::Join {
        left: Box::new(join_ab),
        right: Box::new(t3),
        join_type: JoinType::Inner,
        condition: Some(eq_expr(Expr::column("b"), Expr::column("c"))),
    };

    match join_abc {
        LogicalPlan::Join { left, right, join_type, .. } => {
            assert_eq!(join_type, JoinType::Inner);
            match (*left, *right) {
                (LogicalPlan::Join { .. }, LogicalPlan::TableScan { table_name, .. }) => {
                    assert_eq!(table_name, "t3");
                }
                _ => panic!("Expected nested join with t3 as right side"),
            }
        }
        _ => panic!("Expected Join plan"),
    }
}

#[test]
fn test_left_join() {
    let t1 = make_table_scan("t1", vec![("a".into(), DataType::Integer)]);
    let t2 = make_table_scan("t2", vec![("b".into(), DataType::Integer)]);

    let join = LogicalPlan::Join {
        left: Box::new(t1),
        right: Box::new(t2),
        join_type: JoinType::Left,
        condition: Some(eq_expr(Expr::column("a"), Expr::column("b"))),
    };

    match join {
        LogicalPlan::Join { join_type, .. } => {
            assert_eq!(join_type, JoinType::Left);
        }
        _ => panic!("Expected Join plan"),
    }
}

#[test]
fn test_cross_join() {
    let t1 = make_table_scan("t1", vec![("a".into(), DataType::Integer)]);
    let t2 = make_table_scan("t2", vec![("b".into(), DataType::Integer)]);

    let join = LogicalPlan::Join {
        left: Box::new(t1),
        right: Box::new(t2),
        join_type: JoinType::Cross,
        condition: None,
    };

    match join {
        LogicalPlan::Join { join_type, condition, .. } => {
            assert_eq!(join_type, JoinType::Cross);
            assert!(condition.is_none());
        }
        _ => panic!("Expected Join plan"),
    }
}

#[test]
fn test_right_join() {
    let t1 = make_table_scan("t1", vec![("a".into(), DataType::Integer)]);
    let t2 = make_table_scan("t2", vec![("b".into(), DataType::Integer)]);

    let join = LogicalPlan::Join {
        left: Box::new(t1),
        right: Box::new(t2),
        join_type: JoinType::Right,
        condition: Some(eq_expr(Expr::column("a"), Expr::column("b"))),
    };

    match join {
        LogicalPlan::Join { join_type, .. } => {
            assert_eq!(join_type, JoinType::Right);
        }
        _ => panic!("Expected Join plan"),
    }
}
