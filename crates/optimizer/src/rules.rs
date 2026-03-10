//! Optimizer Rules Module

use crate::Rule;
use std::any::Any;
use std::fmt::Debug;

/// PredicatePushdown rule - pushes filter conditions down to the source
pub struct PredicatePushdown;

impl PredicatePushdown {
    /// Create a new PredicatePushdown rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for PredicatePushdown {
    fn default() -> Self {
        Self::new()
    }
}

impl<Plan> Rule<Plan> for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, _plan: &mut Plan) -> bool {
        // TODO: Implement predicate pushdown logic
        // For now, return false (no change)
        false
    }
}

/// ProjectionPruning rule - removes unnecessary columns
pub struct ProjectionPruning;

impl ProjectionPruning {
    /// Create a new ProjectionPruning rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProjectionPruning {
    fn default() -> Self {
        Self::new()
    }
}

impl<Plan> Rule<Plan> for ProjectionPruning {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }

    fn apply(&self, _plan: &mut Plan) -> bool {
        // TODO: Implement projection pruning logic
        false
    }
}

/// ConstantFolding rule - evaluates constant expressions at compile time
pub struct ConstantFolding;

impl ConstantFolding {
    /// Create a new ConstantFolding rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl<Plan> Rule<Plan> for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, _plan: &mut Plan) -> bool {
        // TODO: Implement constant folding logic
        false
    }
}

// =============================================================================
// Rule Matching Framework - for Cascades optimizer
// =============================================================================

/// Rule pattern for matching plan nodes
#[derive(Debug, Clone)]
pub enum PlanPattern {
    /// Match any node type
    Any,
    /// Match specific node by type name
    Type(String),
    /// Match by condition function
    Condition(fn(&dyn Any) -> bool),
}

/// Rule matching result
#[derive(Debug)]
pub struct MatchResult {
    pub matched: bool,
    pub score: f64,
    pub captures: Vec<Box<dyn Any>>,
}

impl MatchResult {
    pub fn no_match() -> Self {
        Self {
            matched: false,
            score: 0.0,
            captures: vec![],
        }
    }

    pub fn match_with_score(score: f64) -> Self {
        Self {
            matched: true,
            score,
            captures: vec![],
        }
    }

    pub fn match_with_captures(score: f64, captures: Vec<Box<dyn Any>>) -> Self {
        Self {
            matched: true,
            score,
            captures,
        }
    }
}

/// Rule metadata for matching and ordering
#[derive(Debug, Clone)]
pub struct RuleMeta {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule priority (higher = earlier execution)
    pub priority: i32,
    /// Whether rule can be applied multiple times
    pub repeatable: bool,
    /// Expected patterns this rule matches
    pub patterns: Vec<PlanPattern>,
}

impl RuleMeta {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            priority: 0,
            repeatable: false,
            patterns: vec![PlanPattern::Any],
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_pattern(mut self, pattern: PlanPattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

/// Rule context for pattern matching
#[derive(Debug, Default)]
pub struct RuleContext {
    /// Current optimization depth
    pub depth: usize,
    /// Total rules applied
    pub rules_applied: usize,
    /// Whether to continue optimization
    pub continue_optimization: bool,
}

impl RuleContext {
    pub fn new() -> Self {
        Self {
            depth: 0,
            rules_applied: 0,
            continue_optimization: true,
        }
    }

    pub fn increment_depth(&mut self) {
        self.depth += 1;
    }

    pub fn decrement_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn record_rule_applied(&mut self) {
        self.rules_applied += 1;
    }
}

/// RuleSet with matching support - collection of optimization rules
#[derive(Debug, Default)]
pub struct OptimizerRuleSet {
    rules: Vec<RuleItem>,
}

#[derive(Debug)]
struct RuleItem {
    meta: RuleMeta,
    apply_fn: fn(&mut dyn Any) -> bool,
}

impl OptimizerRuleSet {
    /// Create a new empty rule set
    pub fn new() -> Self {
        Self { rules: vec![] }
    }

    /// Add a rule with metadata
    pub fn add_rule(
        mut self,
        meta: RuleMeta,
        apply_fn: fn(&mut dyn Any) -> bool,
    ) -> Self {
        self.rules.push(RuleItem { meta, apply_fn });
        self.rules.sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in PredicatePushdown rule
    pub fn with_predicate_pushdown(mut self) -> Self {
        let meta = RuleMeta::new("PredicatePushdown", "Push filter conditions down to data source")
            .with_priority(100)
            .repeatable();
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules.sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in ProjectionPruning rule
    pub fn with_projection_pruning(mut self) -> Self {
        let meta = RuleMeta::new("ProjectionPruning", "Remove unnecessary columns from projection")
            .with_priority(90)
            .repeatable();
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules.sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in ConstantFolding rule
    pub fn with_constant_folding(mut self) -> Self {
        let meta = RuleMeta::new("ConstantFolding", "Evaluate constant expressions at compile time")
            .with_priority(200);
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules.sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add all default rules
    pub fn with_default_rules(mut self) -> Self {
        self = self.with_constant_folding();
        self = self.with_predicate_pushdown();
        self = self.with_projection_pruning();
        self
    }

    /// Get all rules sorted by priority
    pub fn get_rules(&self) -> Vec<&RuleMeta> {
        self.rules.iter().map(|r| &r.meta).collect()
    }

    /// Get rule count
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Find matching rules for a plan
    pub fn find_matching_rules(&self, plan: &dyn Any) -> Vec<&RuleMeta> {
        let mut matching = vec![];
        for rule in &self.rules {
            let mut matched = false;
            for pattern in &rule.meta.patterns {
                matched = match pattern {
                    PlanPattern::Any => true,
                    PlanPattern::Type(_) => {
                        // TODO: Implement type matching
                        true
                    }
                    PlanPattern::Condition(cond) => cond(plan),
                };
                if matched {
                    break;
                }
            }
            if matched {
                matching.push(&rule.meta);
            }
        }
        matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple plan struct for testing
    #[derive(Debug, Default)]
    struct TestPlan {
        modified: bool,
    }

    #[test]
    fn test_predicate_pushdown_name() {
        let rule = PredicatePushdown::new();
        assert_eq!(Rule::<TestPlan>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_predicate_pushdown_apply() {
        let rule = PredicatePushdown::new();
        let mut plan = TestPlan::default();
        let result = Rule::<TestPlan>::apply(&rule, &mut plan);
        assert!(!result); // Returns false (no change) for stub
    }

    #[test]
    fn test_predicate_pushdown_default() {
        let rule = PredicatePushdown::default();
        assert_eq!(Rule::<TestPlan>::name(&rule), "PredicatePushdown");
    }

    #[test]
    fn test_projection_pruning_name() {
        let rule = ProjectionPruning::new();
        assert_eq!(Rule::<TestPlan>::name(&rule), "ProjectionPruning");
    }

    #[test]
    fn test_projection_pruning_apply() {
        let rule = ProjectionPruning::new();
        let mut plan = TestPlan::default();
        let result = Rule::<TestPlan>::apply(&rule, &mut plan);
        assert!(!result);
    }

    #[test]
    fn test_projection_pruning_default() {
        let rule = ProjectionPruning::default();
        assert_eq!(Rule::<TestPlan>::name(&rule), "ProjectionPruning");
    }

    #[test]
    fn test_constant_folding_name() {
        let rule = ConstantFolding::new();
        assert_eq!(Rule::<TestPlan>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_apply() {
        let rule = ConstantFolding::new();
        let mut plan = TestPlan::default();
        let result = Rule::<TestPlan>::apply(&rule, &mut plan);
        assert!(!result);
    }

    #[test]
    fn test_constant_folding_default() {
        let rule = ConstantFolding::default();
        assert_eq!(Rule::<TestPlan>::name(&rule), "ConstantFolding");
    }

    #[test]
    fn test_projection_pruning_apply_with_string() {
        let rule = ProjectionPruning;
        let mut plan = String::from("test");
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_constant_folding_apply_with_string() {
        let rule = ConstantFolding;
        let mut plan = String::from("test");
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_all_rules_apply_return_false() {
        let mut plan1 = String::new();
        let mut plan2 = String::new();
        let mut plan3 = String::new();

        assert!(!PredicatePushdown.apply(&mut plan1));
        assert!(!ProjectionPruning.apply(&mut plan2));
        assert!(!ConstantFolding.apply(&mut plan3));
    }

    // =============================================================================
    // Rule Matching Framework Tests
    // =============================================================================

    #[test]
    fn test_match_result_no_match() {
        let result = MatchResult::no_match();
        assert!(!result.matched);
        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_match_result_with_score() {
        let result = MatchResult::match_with_score(1.5);
        assert!(result.matched);
        assert_eq!(result.score, 1.5);
    }

    #[test]
    fn test_rule_meta_new() {
        let meta = RuleMeta::new("TestRule", "A test rule");
        assert_eq!(meta.name, "TestRule");
        assert_eq!(meta.description, "A test rule");
        assert_eq!(meta.priority, 0);
        assert!(!meta.repeatable);
    }

    #[test]
    fn test_rule_meta_with_priority() {
        let meta = RuleMeta::new("TestRule", "A test rule").with_priority(100);
        assert_eq!(meta.priority, 100);
    }

    #[test]
    fn test_rule_meta_repeatable() {
        let meta = RuleMeta::new("TestRule", "A test rule").repeatable();
        assert!(meta.repeatable);
    }

    #[test]
    fn test_rule_context_new() {
        let ctx = RuleContext::new();
        assert_eq!(ctx.depth, 0);
        assert_eq!(ctx.rules_applied, 0);
        assert!(ctx.continue_optimization);
    }

    #[test]
    fn test_rule_context_increment() {
        let mut ctx = RuleContext::new();
        ctx.increment_depth();
        ctx.record_rule_applied();
        assert_eq!(ctx.depth, 1);
        assert_eq!(ctx.rules_applied, 1);
    }

    #[test]
    fn test_optimizer_rule_set_new() {
        let rule_set = OptimizerRuleSet::new();
        assert!(rule_set.is_empty());
        assert_eq!(rule_set.len(), 0);
    }

    #[test]
    fn test_optimizer_rule_set_with_default_rules() {
        let rule_set = OptimizerRuleSet::new().with_default_rules();
        assert!(!rule_set.is_empty());
        assert_eq!(rule_set.len(), 3);
    }

    #[test]
    fn test_optimizer_rule_set_get_rules() {
        let rule_set = OptimizerRuleSet::new().with_default_rules();
        let rules = rule_set.get_rules();
        assert_eq!(rules.len(), 3);
        // Rules should be sorted by priority (descending)
        assert!(rules[0].priority >= rules[1].priority);
        assert!(rules[1].priority >= rules[2].priority);
    }

    #[test]
    fn test_optimizer_rule_set_find_matching_rules() {
        let rule_set = OptimizerRuleSet::new().with_default_rules();
        let test_plan = String::from("test");
        let matching = rule_set.find_matching_rules(&test_plan);
        // All rules should match PlanPattern::Any
        assert_eq!(matching.len(), 3);
    }

    #[test]
    fn test_plan_pattern_any() {
        let pattern = PlanPattern::Any;
        let test_val = String::from("test");
        match pattern {
            PlanPattern::Any => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_plan_pattern_condition() {
        let pattern = PlanPattern::Condition(|any: &dyn std::any::Any| {
            any.downcast_ref::<i32>().map(|v| *v > 0).unwrap_or(false)
        });
        let value = 10;
        match pattern {
            PlanPattern::Condition(cond) => assert!(cond(&value)),
            _ => assert!(false),
        }
    }
}
