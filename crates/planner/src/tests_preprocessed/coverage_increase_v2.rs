#[cfg(test)]
mod optimizer_tests {
    use crate::optimizer::{DefaultOptimizer, NoOpOptimizer, Optimizer, OptimizerRule};
    use crate::LogicalPlan;
    use crate::Expr;
    use sqlrustgo_types::Value;

    #[test]
    fn test_noop_optimizer_runs_empty_rules() {
        let plan = LogicalPlan::TableScan {
            table_name: "test".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_optimizer_initialization() {
        let optimizer = DefaultOptimizer::new();
        assert!(optimizer.rules().is_empty());
    }

    #[test]
    fn test_optimize_empty_plan() {
        let plan = LogicalPlan::EmptyRelation;
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_projection() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Projection {
            input: Box::new(inner),
            expr: vec![Expr::Literal(Value::Integer(1))],
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_filter() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Filter {
            predicate: Expr::Literal(Value::Boolean(true)),
            input: Box::new(inner),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_aggregate() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Aggregate {
            input: Box::new(inner),
            group_by: vec![],
            aggregates: vec![],
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_sort() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Sort {
            input: Box::new(inner),
            expr: vec![],
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_limit() {
        let inner = LogicalPlan::TableScan {
            table_name: "t".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Limit {
            input: Box::new(inner),
            limit: Some(10),
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_join() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Join {
            left: Box::new(left),
            right: Box::new(right),
            on: vec![],
            join_type: crate::JoinType::Inner,
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_union() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Union {
            left: Box::new(left),
            right: Box::new(right),
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_except() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Except {
            left: Box::new(left),
            right: Box::new(right),
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimize_intersect() {
        let left = LogicalPlan::TableScan {
            table_name: "t1".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let right = LogicalPlan::TableScan {
            table_name: "t2".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
        };
        let plan = LogicalPlan::Intersect {
            left: Box::new(left),
            right: Box::new(right),
            schema: crate::Schema::empty(),
        };
        let optimizer = NoOpOptimizer {};
        let result = optimizer.optimize(plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rule_trait_name() {
        struct TestRule;
        impl OptimizerRule for TestRule {
            fn name(&self) -> &str {
                "test_rule"
            }
            fn optimize(&self, plan: LogicalPlan) -> crate::error::Result<LogicalPlan> {
                Ok(plan)
            }
        }
        let rule = TestRule;
        assert_eq!(rule.name(), "test_rule");
    }

    #[test]
    fn test_default_optimizer_with_rules() {
        let optimizer = DefaultOptimizer::with_rules(vec![]);
        assert!(optimizer.rules().is_empty());
    }
}

#[cfg(test)]
mod physical_plan_tests {
    use crate::physical_plan::*;
    use crate::LogicalPlan;
    use crate::Expr;
    use sqlrustgo_types::Value;

    #[test]
    fn test_seq_scan_exec_table_name() {
        let exec = SeqScanExec::new("users".to_string(), None, None);
        assert_eq!(exec.table_name(), "users");
    }

    #[test]
    fn test_seq_scan_exec_with_projection() {
        let exec = SeqScanExec::new("users".to_string(), Some(vec![0, 1]), None);
        assert_eq!(exec.table_name(), "users");
    }

    #[test]
    fn test_filter_exec_predicate() {
        let predicate = Expr::Literal(Value::Boolean(true));
        let exec = FilterExec::new(Box::new(SeqScanExec::new("t".to_string(), None, None)), predicate.clone());
        assert!(exec.predicate() == &predicate);
    }

    #[test]
    fn test_projection_exec_expressions() {
        let exprs = vec![Expr::Literal(Value::Integer(1)), Expr::Literal(Value::Text("a".to_string()))];
        let exec = ProjectionExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            exprs.clone(),
        );
        assert_eq!(exec.expr().len(), 2);
    }

    #[test]
    fn test_sort_exec_empty() {
        let exec = SortExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            vec![],
        );
        assert!(exec.expr().is_empty());
    }

    #[test]
    fn test_sort_exec_with_expressions() {
        let sort_exprs = vec![crate::SortExpr {
            expr: Expr::Column("name".to_string()),
            asc: true,
            nulls_first: false,
        }];
        let exec = SortExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            sort_exprs,
        );
        assert!(!exec.expr().is_empty());
    }

    #[test]
    fn test_limit_exec_limit() {
        let exec = LimitExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            Some(100),
        );
        assert_eq!(exec.limit(), Some(100));
    }

    #[test]
    fn test_limit_exec_no_limit() {
        let exec = LimitExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            None,
        );
        assert_eq!(exec.limit(), None);
    }

    #[test]
    fn test_aggregate_exec_group_by() {
        let aggr_exprs = vec![crate::AggregateFunctionExpr {
            func: crate::AggregateFunction::Count,
            args: vec![],
        }];
        let exec = AggregateExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            vec![],
            aggr_exprs,
        );
        assert!(exec.group_expr().is_empty());
    }

    #[test]
    fn test_hash_join_exec_basic() {
        let left = Box::new(SeqScanExec::new("t1".to_string(), None, None));
        let right = Box::new(SeqScanExec::new("t2".to_string(), None, None));
        let on = vec![];
        let exec = HashJoinExec::new(left, right, on, crate::JoinType::Inner);
        assert_eq!(exec.join_type(), crate::JoinType::Inner);
    }

    #[test]
    fn test_hash_join_exec_left() {
        let left = Box::new(SeqScanExec::new("t1".to_string(), None, None));
        let right = Box::new(SeqScanExec::new("t2".to_string(), None, None));
        let on = vec![];
        let exec = HashJoinExec::new(left, right, on, crate::JoinType::Left);
        assert_eq!(exec.join_type(), crate::JoinType::Left);
    }

    #[test]
    fn test_hash_join_exec_right() {
        let left = Box::new(SeqScanExec::new("t1".to_string(), None, None));
        let right = Box::new(SeqScanExec::new("t2".to_string(), None, None));
        let on = vec![];
        let exec = HashJoinExec::new(left, right, on, crate::JoinType::Right);
        assert_eq!(exec.join_type(), crate::JoinType::Right);
    }

    #[test]
    fn test_hash_join_exec_full() {
        let left = Box::new(SeqScanExec::new("t1".to_string(), None, None));
        let right = Box::new(SeqScanExec::new("t2".to_string(), None, None));
        let on = vec![];
        let exec = HashJoinExec::new(left, right, on, crate::JoinType::Full);
        assert_eq!(exec.join_type(), crate::JoinType::Full);
    }

    #[test]
    fn test_index_scan_exec_table() {
        let exec = IndexScanExec::new("users".to_string(), "idx".to_string(), None, None);
        assert_eq!(exec.table_name(), "users");
    }

    #[test]
    fn test_index_scan_exec_with_range() {
        let exec = IndexScanExec::new(
            "users".to_string(),
            "idx".to_string(),
            Some(Expr::Literal(Value::Integer(1))),
            Some(Expr::Literal(Value::Integer(100))),
        );
        assert!(exec.lower_bound().is_some());
        assert!(exec.upper_bound().is_some());
    }

    #[test]
    fn test_delete_exec_table() {
        let exec = DeleteExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            "users".to_string(),
        );
        assert_eq!(exec.table_name(), "users");
    }

    #[test]
    fn test_window_exec_basic() {
        let window_exprs = vec![crate::WindowFunctionExpr {
            func: crate::WindowFunction::RowNumber,
            args: vec![],
        }];
        let exec = WindowExec::new(
            Box::new(SeqScanExec::new("t".to_string(), None, None)),
            window_exprs,
        );
        assert!(!exec.window_expr().is_empty());
    }

    #[test]
    fn test_physical_plan_display() {
        let plan = PhysicalPlan::TableScan {
            table_name: "users".to_string(),
            schema: crate::Schema::empty(),
            projection: None,
            filter: None,
        };
        let display = format!("{}", plan);
        assert!(display.contains("users"));
    }
}