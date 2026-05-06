use sqlrustgo_optimizer::{
    CostModel, Optimizer, Rule, RuleSet,
};

mod optimizer_trait_tests {
    use super::*;

    #[test]
    fn test_optimizer_trait_object() {
        let mut optimizer: Box<dyn Optimizer> = Box::new(sqlrustgo_optimizer::NoOpOptimizer);
        let mut plan: Box<dyn std::any::Any> = Box::new(42i32);
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_optimizer_trait_with_string() {
        let mut optimizer: Box<dyn Optimizer> = Box::new(sqlrustgo_optimizer::NoOpOptimizer);
        let mut plan: Box<dyn std::any::Any> = Box::new("test_string".to_string());
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_optimizer_trait_with_vec() {
        let mut optimizer: Box<dyn Optimizer> = Box::new(sqlrustgo_optimizer::NoOpOptimizer);
        let mut plan: Box<dyn std::any::Any> = Box::new(vec![1i32, 2, 3]);
        assert!(optimizer.optimize(&mut plan).is_ok());
    }
}

mod noop_optimizer_tests {
    use super::*;

    #[test]
    fn test_noop_optimizer_i32() {
        let mut optimizer = sqlrustgo_optimizer::NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new(42i32);
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_noop_optimizer_f64() {
        let mut optimizer = sqlrustgo_optimizer::NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new(3.14f64);
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_noop_optimizer_string() {
        let mut optimizer = sqlrustgo_optimizer::NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new("hello".to_string());
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_noop_optimizer_option() {
        let mut optimizer = sqlrustgo_optimizer::NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new(Some(42i32));
        assert!(optimizer.optimize(&mut plan).is_ok());
    }

    #[test]
    fn test_noop_optimizer_result() {
        let mut optimizer = sqlrustgo_optimizer::NoOpOptimizer;
        let mut plan: Box<dyn std::any::Any> = Box::new(Ok::<i32, i32>(42));
        assert!(optimizer.optimize(&mut plan).is_ok());
    }
}

mod rule_trait_tests {
    use super::*;

    struct IncrementRule;
    impl Rule<i32> for IncrementRule {
        fn name(&self) -> &str {
            "IncrementRule"
        }
        fn apply(&self, plan: &mut i32) -> bool {
            *plan += 1;
            true
        }
    }

    struct NoChangeRule;
    impl Rule<i32> for NoChangeRule {
        fn name(&self) -> &str {
            "NoChangeRule"
        }
        fn apply(&self, _plan: &mut i32) -> bool {
            false
        }
    }

    #[test]
    fn test_rule_name() {
        let rule = IncrementRule;
        assert_eq!(rule.name(), "IncrementRule");
    }

    #[test]
    fn test_rule_apply_changes() {
        let rule = IncrementRule;
        let mut plan = 10i32;
        assert!(rule.apply(&mut plan));
        assert_eq!(plan, 11);
    }

    #[test]
    fn test_rule_apply_no_change() {
        let rule = NoChangeRule;
        let mut plan = 10i32;
        assert!(!rule.apply(&mut plan));
        assert_eq!(plan, 10);
    }

    #[test]
    fn test_rule_with_different_types() {
        struct StringAppendRule;
        impl Rule<String> for StringAppendRule {
            fn name(&self) -> &str {
                "StringAppendRule"
            }
            fn apply(&self, plan: &mut String) -> bool {
                plan.push_str("_changed");
                true
            }
        }

        let rule = StringAppendRule;
        let mut plan = "original".to_string();
        assert!(rule.apply(&mut plan));
        assert_eq!(plan, "original_changed");
    }
}

mod cost_model_trait_tests {
    use super::*;

    struct TestCostModel {
        cost: f64,
    }

    impl CostModel for TestCostModel {
        fn estimate_cost(&self, _plan: &dyn std::any::Any) -> f64 {
            self.cost
        }
    }

    #[test]
    fn test_cost_model_estimate() {
        let model = TestCostModel { cost: 100.0 };
        let cost = model.estimate_cost(&42i32);
        assert_eq!(cost, 100.0);
    }

    #[test]
    fn test_cost_model_zero_cost() {
        let model = TestCostModel { cost: 0.0 };
        let cost = model.estimate_cost(&"test".to_string());
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_cost_model_high_cost() {
        let model = TestCostModel { cost: f64::MAX };
        let cost = model.estimate_cost(&vec![1, 2, 3]);
        assert_eq!(cost, f64::MAX);
    }

    #[test]
    fn test_cost_model_with_option() {
        let model = TestCostModel { cost: 42.5 };
        let cost = model.estimate_cost(&Some(123i32));
        assert_eq!(cost, 42.5);
    }
}

mod ruleset_tests {
    use super::*;

    #[test]
    fn test_ruleset_new() {
        let mut ruleset = RuleSet::new();
        assert!(!ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_default() {
        let mut ruleset = RuleSet::default();
        assert!(!ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_add_single_rule() {
        let mut ruleset = RuleSet::new();
        ruleset.add_rule(|_| true);
        assert!(ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_add_rule_no_change() {
        let mut ruleset = RuleSet::new();
        ruleset.add_rule(|_| false);
        assert!(!ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_multiple_rules_all_change() {
        let mut ruleset = RuleSet::new();
        ruleset.add_rule(|_| true);
        ruleset.add_rule(|_| true);
        ruleset.add_rule(|_| true);
        assert!(ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_multiple_rules_mixed() {
        let mut ruleset = RuleSet::new();
        ruleset.add_rule(|_| false);
        ruleset.add_rule(|_| true);
        ruleset.add_rule(|_| false);
        assert!(ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_multiple_rules_none_change() {
        let mut ruleset = RuleSet::new();
        ruleset.add_rule(|_| false);
        ruleset.add_rule(|_| false);
        ruleset.add_rule(|_| false);
        assert!(!ruleset.apply(&mut Box::new(0i32) as &mut dyn std::any::Any));
    }

    #[test]
    fn test_ruleset_empty_returns_false() {
        let mut ruleset = RuleSet::new();
        let mut plan: Box<dyn std::any::Any> = Box::new(42i32);
        assert!(!ruleset.apply(&mut plan));
    }
}

mod public_exports_tests {
    use sqlrustgo_optimizer::{
        ConstantFolding, PredicatePushdown, ProjectionPruning, Rule,
    };
    use sqlrustgo_optimizer::unified_plan::UnifiedPlan;

    #[test]
    fn test_constant_folding_is_rule() {
        let rule = ConstantFolding::new();
        assert_eq!(<ConstantFolding as Rule<UnifiedPlan>>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_predicate_pushdown_is_rule() {
        let rule = PredicatePushdown::new();
        assert_eq!(<PredicatePushdown as Rule<UnifiedPlan>>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_projection_pruning_is_rule() {
        let rule = ProjectionPruning::new();
        assert_eq!(<ProjectionPruning as Rule<UnifiedPlan>>::name(&rule), "ProjectionPruning");
    }
}
