use sqlrustgo_planner::Expr;
use sqlrustgo_planner::{AggregateFunction, JoinType, LogicalPlan, Operator, Schema};
use sqlrustgo_types::Value;

#[cfg(test)]
mod logical_plan_tests {
    use super::*;

    #[test]
    fn test_logical_plan_table_scan() {
        let plan = LogicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        if let LogicalPlan::TableScan { ref table_name, .. } = plan {
            assert_eq!(table_name, "users");
        }
    }

    #[test]
    fn test_logical_plan_projection() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Projection {
            input: Box::new(inner),
            expr: vec![],
            schema: Schema::empty(),
        };
        assert!(matches!(plan, LogicalPlan::Projection { .. }));
    }

    #[test]
    fn test_logical_plan_filter() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Filter {
            predicate: Expr::Literal(Value::Boolean(true)),
            input: Box::new(inner),
        };
        assert!(matches!(plan, LogicalPlan::Filter { .. }));
    }

    #[test]
    fn test_logical_plan_aggregate() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Aggregate {
            input: Box::new(inner),
            group_expr: vec![],
            aggregate_expr: vec![],
            schema: Schema::empty(),
        };
        assert!(matches!(plan, LogicalPlan::Aggregate { .. }));
    }

    #[test]
    fn test_logical_plan_sort() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Sort {
            input: Box::new(inner),
            sort_expr: vec![],
        };
        assert!(matches!(plan, LogicalPlan::Sort { .. }));
    }

    #[test]
    fn test_logical_plan_limit() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Limit {
            input: Box::new(inner),
            limit: 100,
            offset: None,
        };
        assert!(matches!(plan, LogicalPlan::Limit { .. }));
    }

    #[test]
    fn test_logical_plan_limit_with_offset() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Limit {
            input: Box::new(inner),
            limit: 100,
            offset: Some(10),
        };
        assert!(matches!(
            plan,
            LogicalPlan::Limit {
                limit: 100,
                offset: Some(10),
                ..
            }
        ));
    }

    #[test]
    fn test_logical_plan_join_inner() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Inner,
            condition: None,
        };
        assert!(matches!(
            plan,
            LogicalPlan::Join {
                join_type: JoinType::Inner,
                ..
            }
        ));
    }

    #[test]
    fn test_logical_plan_join_left() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Left,
            condition: None,
        };
        assert!(matches!(
            plan,
            LogicalPlan::Join {
                join_type: JoinType::Left,
                ..
            }
        ));
    }

    #[test]
    fn test_logical_plan_join_right() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Right,
            condition: None,
        };
        assert!(matches!(
            plan,
            LogicalPlan::Join {
                join_type: JoinType::Right,
                ..
            }
        ));
    }

    #[test]
    fn test_logical_plan_join_full() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Full,
            condition: None,
        };
        assert!(matches!(
            plan,
            LogicalPlan::Join {
                join_type: JoinType::Full,
                ..
            }
        ));
    }

    #[test]
    fn test_logical_plan_union() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Union {
            left: Box::new(left),
            right: Box::new(right),
        };
        assert!(matches!(plan, LogicalPlan::Union { .. }));
    }

    #[test]
    fn test_logical_plan_delete() {
        let plan = LogicalPlan::Delete {
            table_name: "users".to_string(),
            predicate: None,
        };
        assert!(matches!(plan, LogicalPlan::Delete { .. }));
    }

    #[test]
    fn test_logical_plan_update() {
        let plan = LogicalPlan::Update {
            table_name: "users".to_string(),
            updates: vec![],
            predicate: None,
        };
        assert!(matches!(plan, LogicalPlan::Update { .. }));
    }
}

#[cfg(test)]
mod join_type_tests {
    use super::*;

    #[test]
    fn test_join_type_inner() {
        assert_eq!(JoinType::Inner, JoinType::Inner);
    }

    #[test]
    fn test_join_type_left() {
        assert_eq!(JoinType::Left, JoinType::Left);
    }

    #[test]
    fn test_join_type_right() {
        assert_eq!(JoinType::Right, JoinType::Right);
    }

    #[test]
    fn test_join_type_full() {
        assert_eq!(JoinType::Full, JoinType::Full);
    }

    #[test]
    fn test_join_type_cross() {
        assert_eq!(JoinType::Cross, JoinType::Cross);
    }

    #[test]
    fn test_join_type_left_semi() {
        assert_eq!(JoinType::LeftSemi, JoinType::LeftSemi);
    }

    #[test]
    fn test_join_type_left_anti() {
        assert_eq!(JoinType::LeftAnti, JoinType::LeftAnti);
    }

    #[test]
    fn test_join_type_right_semi() {
        assert_eq!(JoinType::RightSemi, JoinType::RightSemi);
    }

    #[test]
    fn test_join_type_right_anti() {
        assert_eq!(JoinType::RightAnti, JoinType::RightAnti);
    }

    #[test]
    fn test_join_type_all_variants() {
        let variants = [
            JoinType::Inner,
            JoinType::Left,
            JoinType::Right,
            JoinType::Full,
            JoinType::Cross,
            JoinType::LeftSemi,
            JoinType::LeftAnti,
            JoinType::RightSemi,
            JoinType::RightAnti,
        ];
        assert_eq!(variants.len(), 9);
    }
}

#[cfg(test)]
mod aggregate_function_tests {
    use super::*;

    #[test]
    fn test_aggregate_function_count() {
        assert_eq!(AggregateFunction::Count, AggregateFunction::Count);
    }

    #[test]
    fn test_aggregate_function_sum() {
        assert_eq!(AggregateFunction::Sum, AggregateFunction::Sum);
    }

    #[test]
    fn test_aggregate_function_avg() {
        assert_eq!(AggregateFunction::Avg, AggregateFunction::Avg);
    }

    #[test]
    fn test_aggregate_function_min() {
        assert_eq!(AggregateFunction::Min, AggregateFunction::Min);
    }

    #[test]
    fn test_aggregate_function_max() {
        assert_eq!(AggregateFunction::Max, AggregateFunction::Max);
    }

    #[test]
    fn test_aggregate_function_all_variants() {
        let variants = [
            AggregateFunction::Count,
            AggregateFunction::Sum,
            AggregateFunction::Avg,
            AggregateFunction::Min,
            AggregateFunction::Max,
        ];
        assert_eq!(variants.len(), 5);
    }
}

#[cfg(test)]
mod operator_tests {
    use super::*;

    #[test]
    fn test_operator_equality() {
        assert_eq!(Operator::Eq, Operator::Eq);
        assert_eq!(Operator::NotEq, Operator::NotEq);
    }

    #[test]
    fn test_operator_comparison() {
        assert_eq!(Operator::Lt, Operator::Lt);
        assert_eq!(Operator::LtEq, Operator::LtEq);
        assert_eq!(Operator::Gt, Operator::Gt);
        assert_eq!(Operator::GtEq, Operator::GtEq);
    }

    #[test]
    fn test_operator_logical() {
        assert_eq!(Operator::And, Operator::And);
        assert_eq!(Operator::Or, Operator::Or);
    }

    #[test]
    fn test_operator_arithmetic() {
        assert_eq!(Operator::Plus, Operator::Plus);
        assert_eq!(Operator::Minus, Operator::Minus);
        assert_eq!(Operator::Multiply, Operator::Multiply);
        assert_eq!(Operator::Divide, Operator::Divide);
    }

    #[test]
    fn test_operator_all_variants() {
        let operators = [
            Operator::Eq,
            Operator::NotEq,
            Operator::Lt,
            Operator::LtEq,
            Operator::Gt,
            Operator::GtEq,
            Operator::And,
            Operator::Or,
            Operator::Plus,
            Operator::Minus,
            Operator::Multiply,
            Operator::Divide,
        ];
        assert_eq!(operators.len(), 12);
    }
}

#[cfg(test)]
mod logical_plan_variant_tests {
    use super::*;
    use sqlrustgo_planner::LogicalPlan;

    #[test]
    fn test_logical_plan_subquery() {
        let inner = LogicalPlan::TableScan {
            table_name: "inner".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Subquery {
            subquery: Box::new(inner),
            alias: "sq".to_string(),
        };
        assert!(matches!(plan, LogicalPlan::Subquery { .. } ));
    }

    #[test]
    fn test_logical_plan_union() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Union {
            left: Box::new(left),
            right: Box::new(right),
        };
        assert!(matches!(plan, LogicalPlan::Union { .. }));
    }

    #[test]
    fn test_logical_plan_update() {
        let plan = LogicalPlan::Update {
            table_name: "users".to_string(),
            updates: vec![
                ("age".to_string(), Expr::Literal(Value::Integer(30))),
            ],
            predicate: Some(Expr::Literal(Value::Boolean(true))),
        };
        assert!(matches!(plan, LogicalPlan::Update { .. } ));
    }

    #[test]
    fn test_logical_plan_delete() {
        let plan = LogicalPlan::Delete {
            table_name: "users".to_string(),
            predicate: Some(Expr::Literal(Value::Boolean(true))),
        };
        assert!(matches!(plan, LogicalPlan::Delete { .. } ));
    }

    #[test]
    fn test_logical_plan_create_table() {
        let plan = LogicalPlan::CreateTable {
            table_name: "users".to_string(),
            schema: Schema::empty(),
            if_not_exists: true,
        };
        assert!(matches!(plan, LogicalPlan::CreateTable { .. } ));
    }

    #[test]
    fn test_logical_plan_drop_table() {
        let plan = LogicalPlan::DropTable {
            table_name: "users".to_string(),
            if_exists: true,
        };
        assert!(matches!(plan, LogicalPlan::DropTable { .. } ));
    }

    #[test]
    fn test_logical_plan_values() {
        let plan = LogicalPlan::Values {
            values: vec![
                vec![Value::Integer(1), Value::Text("a".to_string())],
                vec![Value::Integer(2), Value::Text("b".to_string())],
            ],
            schema: Schema::empty(),
        };
        assert!(matches!(plan, LogicalPlan::Values { .. } ));
        if let LogicalPlan::Values { values, .. } = plan {
            assert_eq!(values.len(), 2);
        }
    }

    #[test]
    fn test_logical_plan_empty_relation() {
        let plan = LogicalPlan::EmptyRelation;
        assert!(matches!(plan, LogicalPlan::EmptyRelation));
    }

    #[test]
    fn test_logical_plan_join() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Inner,
            condition: None,
        };
        assert!(matches!(plan, LogicalPlan::Join { join_type: JoinType::Inner, .. }));
    }

    #[test]
    fn test_logical_plan_join_left() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Left,
            condition: None,
        };
        assert!(matches!(plan, LogicalPlan::Join { join_type: JoinType::Left, .. }));
    }

    #[test]
    fn test_logical_plan_join_right() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            join_type: JoinType::Right,
            condition: None,
        };
        assert!(matches!(plan, LogicalPlan::Join { join_type: JoinType::Right, .. }));
    }
}

#[cfg(test)]
mod expr_tests {
    use super::*;

    #[test]
    fn test_expr_column() {
        let expr = Expr::Column(sqlrustgo_planner::Column::new("name".to_string()));
        assert!(matches!(expr, Expr::Column(_)));
    }

    #[test]
    fn test_expr_literal() {
        let expr = Expr::Literal(Value::Integer(42));
        assert!(matches!(expr, Expr::Literal(Value::Integer(42))));
    }

    #[test]
    fn test_expr_binary() {
        let left = Expr::Column(sqlrustgo_planner::Column::new("a".to_string()));
        let right = Expr::Literal(Value::Integer(1));
        let expr = Expr::binary_expr(left, Operator::Plus, right);
        assert!(matches!(
            expr,
            Expr::BinaryExpr {
                op: Operator::Plus,
                ..
            }
        ));
    }

    #[test]
    fn test_expr_unary() {
        let expr = Expr::UnaryExpr {
            op: Operator::Minus,
            expr: Box::new(Expr::Literal(Value::Integer(42))),
        };
        assert!(matches!(
            expr,
            Expr::UnaryExpr {
                op: Operator::Minus,
                ..
            }
        ));
    }
}
