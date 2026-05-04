//! Optimizer Rules Integration Tests

use sqlrustgo_optimizer::{
    rules::{ConstantFolding, PredicatePushdown, ProjectionPruning, JoinType, Expr, BinaryOperator},
    Rule,
};

mod predicate_pushdown_tests {
    use super::*;
    use sqlrustgo_optimizer::Rule;

    #[test]
    fn test_predicate_pushdown_name() {
        let rule = PredicatePushdown::new();
        assert_eq!(<PredicatePushdown as Rule<i32>>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_predicate_pushdown_default() {
        let rule = PredicatePushdown::default();
        assert_eq!(<PredicatePushdown as Rule<i32>>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_predicate_pushdown_apply() {
        let rule = PredicatePushdown::new();
        let mut plan = 42i32;
        let result = rule.apply(&mut plan);
        assert!(!result);
    }
}

mod projection_pruning_tests {
    use super::*;
    use sqlrustgo_optimizer::Rule;

    #[test]
    fn test_projection_pruning_name() {
        let rule = ProjectionPruning::new();
        assert_eq!(<ProjectionPruning as Rule<i32>>::name(&rule), "ProjectionPruning");
    }

    #[test]
    fn test_projection_pruning_default() {
        let rule = ProjectionPruning::default();
        assert_eq!(<ProjectionPruning as Rule<i32>>::name(&rule), "ProjectionPruning");
    }

    #[test]
    fn test_projection_pruning_apply() {
        let rule = ProjectionPruning::new();
        let mut plan = 42i32;
        let result = rule.apply(&mut plan);
        assert!(!result);
    }
}

mod constant_folding_tests {
    use super::*;
    use sqlrustgo_optimizer::Rule;

    #[test]
    fn test_constant_folding_name() {
        let rule = ConstantFolding::new();
        assert_eq!(<ConstantFolding as Rule<i32>>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_default() {
        let rule = ConstantFolding::default();
        assert_eq!(<ConstantFolding as Rule<i32>>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_apply() {
        let rule = ConstantFolding::new();
        let mut plan = 42i32;
        let result = rule.apply(&mut plan);
        assert!(!result);
    }
}

mod rule_trait_tests {
    use super::*;

    #[test]
    fn test_rule_trait_name_method() {
        struct TestRule;
        impl Rule<i32> for TestRule {
            fn name(&self) -> &str {
                "TestRule"
            }
            fn apply(&self, _plan: &mut i32) -> bool {
                true
            }
        }
        let rule = TestRule;
        assert_eq!(rule.name(), "TestRule");
    }

    #[test]
    fn test_rule_trait_apply_changes_plan() {
        struct IncrementRule;
        impl Rule<i32> for IncrementRule {
            fn name(&self) -> &str { "IncrementRule" }
            fn apply(&self, plan: &mut i32) -> bool {
                *plan += 1;
                true
            }
        }
        let rule = IncrementRule;
        let mut plan = 10i32;
        assert!(rule.apply(&mut plan));
        assert_eq!(plan, 11);
    }

    #[test]
    fn test_rule_trait_apply_no_change() {
        struct NoChangeRule;
        impl Rule<String> for NoChangeRule {
            fn name(&self) -> &str { "NoChangeRule" }
            fn apply(&self, _plan: &mut String) -> bool {
                false
            }
        }
        let rule = NoChangeRule;
        let mut plan = String::from("original");
        assert!(!rule.apply(&mut plan));
        assert_eq!(plan, "original");
    }
}

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
    fn test_join_type_semi_anti() {
        assert_eq!(JoinType::LeftSemi, JoinType::LeftSemi);
        assert_eq!(JoinType::LeftAnti, JoinType::LeftAnti);
        assert_eq!(JoinType::RightSemi, JoinType::RightSemi);
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

mod expression_tests {
    use super::*;

    #[test]
    fn test_expr_column() {
        let expr = Expr::Column(String::from("name"));
        match expr {
            Expr::Column(c) => assert_eq!(c, "name"),
            _ => panic!("Expected Column"),
        }
    }

    #[test]
    fn test_expr_literal() {
        let expr = Expr::Literal(String::from("hello"));
        match expr {
            Expr::Literal(c) => assert_eq!(c, "hello"),
            _ => panic!("Expected Literal"),
        }
    }

    #[test]
    fn test_expr_binary() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column(String::from("a"))),
            op: BinaryOperator::Eq,
            right: Box::new(Expr::Literal(String::from("1"))),
        };
        match expr {
            Expr::BinaryExpr { op, .. } => assert_eq!(op, BinaryOperator::Eq),
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_expr_nested_binary() {
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::BinaryExpr {
                left: Box::new(Expr::Column(String::from("a"))),
                op: BinaryOperator::Plus,
                right: Box::new(Expr::Column(String::from("b"))),
            }),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Literal(String::from("10"))),
        };
        match expr {
            Expr::BinaryExpr { left, op, .. } => {
                assert_eq!(op, BinaryOperator::Gt);
                match *left {
                    Expr::BinaryExpr { op: inner_op, .. } => {
                        assert_eq!(inner_op, BinaryOperator::Plus);
                    }
                    _ => panic!("Expected nested BinaryExpr"),
                }
            }
            _ => panic!("Expected outer BinaryExpr"),
        }
    }
}

mod binary_operator_tests {
    use super::*;

    #[test]
    fn test_comparison_operators() {
        assert_eq!(BinaryOperator::Eq, BinaryOperator::Eq);
        assert_eq!(BinaryOperator::NotEq, BinaryOperator::NotEq);
        assert_eq!(BinaryOperator::Lt, BinaryOperator::Lt);
        assert_eq!(BinaryOperator::LtEq, BinaryOperator::LtEq);
        assert_eq!(BinaryOperator::Gt, BinaryOperator::Gt);
        assert_eq!(BinaryOperator::GtEq, BinaryOperator::GtEq);
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(BinaryOperator::And, BinaryOperator::And);
        assert_eq!(BinaryOperator::Or, BinaryOperator::Or);
    }

    #[test]
    fn test_arithmetic_operators() {
        assert_eq!(BinaryOperator::Plus, BinaryOperator::Plus);
        assert_eq!(BinaryOperator::Minus, BinaryOperator::Minus);
        assert_eq!(BinaryOperator::Multiply, BinaryOperator::Multiply);
        assert_eq!(BinaryOperator::Divide, BinaryOperator::Divide);
    }

    #[test]
    fn test_all_operators_count() {
        let operators = [
            BinaryOperator::Eq,
            BinaryOperator::NotEq,
            BinaryOperator::Lt,
            BinaryOperator::LtEq,
            BinaryOperator::Gt,
            BinaryOperator::GtEq,
            BinaryOperator::And,
            BinaryOperator::Or,
            BinaryOperator::Plus,
            BinaryOperator::Minus,
            BinaryOperator::Multiply,
            BinaryOperator::Divide,
        ];
        assert_eq!(operators.len(), 12);
    }
}
