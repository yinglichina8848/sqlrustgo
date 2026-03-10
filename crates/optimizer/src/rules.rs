//! Optimizer Rules Module

use crate::Rule;
use std::any::Any;
use std::fmt::Debug;

// =============================================================================
// Simple Plan Enum for demonstration
// =============================================================================

/// Simple plan node types for the optimizer framework
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// Table scan operation
    TableScan {
        table_name: String,
        projection: Option<Vec<usize>>,
    },
    /// Filter operation (WHERE clause)
    Filter {
        predicate: Expr,
        input: Box<Plan>,
    },
    /// Projection operation (SELECT columns)
    Projection {
        expr: Vec<Expr>,
        input: Box<Plan>,
    },
    /// Join operation
    Join {
        left: Box<Plan>,
        right: Box<Plan>,
        join_type: JoinType,
        condition: Option<Expr>,
    },
    /// Aggregate operation (GROUP BY)
    Aggregate {
        group_by: Vec<Expr>,
        aggregates: Vec<Expr>,
        input: Box<Plan>,
    },
    /// Sort operation (ORDER BY)
    Sort {
        expr: Vec<Expr>,
        input: Box<Plan>,
    },
    /// Limit operation
    Limit {
        limit: usize,
        input: Box<Plan>,
    },
    /// Empty relation
    EmptyRelation,
}

/// Simple expression types for plan nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Column reference
    Column(String),
    /// Literal value
    Literal(Value),
    /// Binary expression
    BinaryExpr {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    /// Unary expression
    UnaryExpr {
        op: Operator,
        expr: Box<Expr>,
    },
}

/// Join types
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// Simple value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
}

/// Operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    And,
    Or,
    Not,
    Like,
}

impl Plan {
    /// Get the type name of this plan node
    pub fn type_name(&self) -> &'static str {
        match self {
            Plan::TableScan { .. } => "TableScan",
            Plan::Filter { .. } => "Filter",
            Plan::Projection { .. } => "Projection",
            Plan::Join { .. } => "Join",
            Plan::Aggregate { .. } => "Aggregate",
            Plan::Sort { .. } => "Sort",
            Plan::Limit { .. } => "Limit",
            Plan::EmptyRelation => "EmptyRelation",
        }
    }

    /// Get mutable reference to child plan if exists
    pub fn get_child_mut(&mut self) -> Option<&mut Box<Plan>> {
        match self {
            Plan::Filter { input, .. } => Some(input),
            Plan::Projection { input, .. } => Some(input),
            Plan::Join { left, right, .. } => Some(left),
            Plan::Aggregate { input, .. } => Some(input),
            Plan::Sort { input, .. } => Some(input),
            Plan::Limit { input, .. } => Some(input),
            _ => None,
        }
    }

    /// Get references to all children
    pub fn get_children(&self) -> Vec<&Plan> {
        match self {
            Plan::Filter { input, .. } => vec![input],
            Plan::Projection { input, .. } => vec![input],
            Plan::Join { left, right, .. } => vec![left, right],
            Plan::Aggregate { input, .. } => vec![input],
            Plan::Sort { input, .. } => vec![input],
            Plan::Limit { input, .. } => vec![input],
            _ => vec![],
        }
    }
}

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

/// PredicatePushdown implementation for Plan type
impl Rule<Plan> for PredicatePushdown {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.pushdown(plan)
    }
}

impl PredicatePushdown {
    /// Push predicates down to table scans
    fn pushdown(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Filter { predicate, input } => {
                match &mut **input {
                    Plan::TableScan { .. } => {
                        // Filter directly on table scan - already optimal
                        false
                    }
                    Plan::Projection { input: proj_input, .. } => {
                        // Push filter through projection to its input
                        let new_filter = Plan::Filter {
                            predicate: predicate.clone(),
                            input: proj_input.clone(),
                        };
                        **input = new_filter;
                        return true;
                    }
                    Plan::Join { left, right, join_type, condition } => {
                        let mut changed = false;

                        // Push into left side
                        if let Some(pred) = condition.as_ref() {
                            if can_push_to_left(pred, join_type) {
                                changed |= self.pushdown(left);
                            }
                        }

                        // Push into right side
                        if let Some(pred) = condition.as_ref() {
                            if can_push_to_right(pred, join_type) {
                                changed |= self.pushdown(right);
                            }
                        }

                        return changed;
                    }
                    _ => false,
                }
            }
            Plan::Projection { input, .. } => self.pushdown(input),
            Plan::Aggregate { .. } => false,
            Plan::Sort { input, .. } => self.pushdown(input),
            Plan::Limit { input, .. } => self.pushdown(input),
            _ => false,
        }
    }
}

fn can_push_to_left(_predicate: &Expr, join_type: &JoinType) -> bool {
    matches!(join_type, JoinType::Inner | JoinType::Left)
}

fn can_push_to_right(_predicate: &Expr, join_type: &JoinType) -> bool {
    matches!(join_type, JoinType::Inner | JoinType::Right)
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

/// ProjectionPruning implementation for Plan type
impl Rule<Plan> for ProjectionPruning {
    fn name(&self) -> &str {
        "ProjectionPruning"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.prune(plan)
    }
}

impl ProjectionPruning {
    /// Remove unnecessary columns from projections
    fn prune(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Projection { input, expr, .. } => {
                // Collect columns used in this projection
                let used_cols = self.collect_columns(expr);

                // Check if input is a table scan that can benefit from projection
                if let Plan::TableScan { projection, .. } = &mut **input {
                    if projection.is_none() && !used_cols.is_all {
                        // Push down projection to table scan
                        let new_projection: Option<Vec<usize>> = Some(used_cols.indices);
                        *projection = new_projection;
                        return true;
                    }
                }

                // Recurse into input
                self.prune(input)
            }
            Plan::Filter { input, .. } => self.prune(input),
            Plan::Aggregate { input, .. } => self.prune(input),
            Plan::Join { left, right, .. } => {
                let changed_left = self.prune(left);
                let changed_right = self.prune(right);
                changed_left || changed_right
            }
            Plan::Sort { input, .. } => self.prune(input),
            Plan::Limit { input, .. } => self.prune(input),
            _ => false,
        }
    }

    /// Collect columns used in expressions
    fn collect_columns(&self, exprs: &[Expr]) -> SimpleColumnSet {
        let mut cols = SimpleColumnSet::new();
        for expr in exprs {
            self.collect_from_expr(expr, &mut cols);
        }
        cols
    }

    fn collect_from_expr(&self, expr: &Expr, cols: &mut SimpleColumnSet) {
        match expr {
            Expr::Column(col_name) => {
                cols.add(col_name);
            }
            Expr::BinaryExpr { left, right, .. } => {
                self.collect_from_expr(left, cols);
                self.collect_from_expr(right, cols);
            }
            Expr::UnaryExpr { expr, .. } => {
                self.collect_from_expr(expr, cols);
            }
            _ => {}
        }
    }
}

/// Simple column set for tracking used columns
#[derive(Debug, Clone, Default)]
pub struct SimpleColumnSet {
    pub indices: Vec<usize>,
    pub is_all: bool,
}

impl SimpleColumnSet {
    pub fn new() -> Self {
        Self {
            indices: vec![],
            is_all: true,
        }
    }

    pub fn add(&mut self, _name: &str) {
        self.is_all = false;
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

/// ConstantFolding implementation for Plan type
impl Rule<Plan> for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.fold(plan)
    }
}

impl ConstantFolding {
    /// Evaluate constant expressions
    fn fold(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Filter { predicate, input } => {
                // Try to simplify the predicate
                let simplified = self.simplify_expr(predicate);
                if simplified != *predicate {
                    *predicate = simplified;
                    return true;
                }
                self.fold(input)
            }
            Plan::Projection { input, expr, .. } => {
                // Try to simplify projection expressions
                let mut changed = false;
                let mut new_exprs = Vec::new();
                for e in expr.iter() {
                    let simplified = self.simplify_expr(e);
                    if simplified != *e {
                        changed = true;
                    }
                    new_exprs.push(simplified);
                }
                if changed {
                    *expr = new_exprs;
                }
                changed || self.fold(input)
            }
            Plan::Aggregate { input, .. } => self.fold(input),
            Plan::Join { left, right, .. } => {
                let changed_left = self.fold(left);
                let changed_right = self.fold(right);
                changed_left || changed_right
            }
            Plan::Sort { input, .. } => self.fold(input),
            Plan::Limit { input, .. } => self.fold(input),
            _ => false,
        }
    }

    /// Simplify an expression by evaluating constants
    fn simplify_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let left_simplified = self.simplify_expr(left);
                let right_simplified = self.simplify_expr(right);

                // If both are literals, try to evaluate
                if let Expr::Literal(lv) = &left_simplified {
                    if let Expr::Literal(rv) = &right_simplified {
                        if let Some(result) = self.eval_binary_op(op, lv, rv) {
                            return Expr::Literal(result);
                        }
                    }
                }

                Expr::BinaryExpr {
                    left: Box::new(left_simplified),
                    op: op.clone(),
                    right: Box::new(right_simplified),
                }
            }
            Expr::UnaryExpr { op, expr } => {
                let simplified = self.simplify_expr(expr);
                if let Expr::Literal(v) = &simplified {
                    if let Some(result) = self.eval_unary_op(op, v) {
                        return Expr::Literal(result);
                    }
                }
                Expr::UnaryExpr {
                    op: op.clone(),
                    expr: Box::new(simplified),
                }
            }
            _ => expr.clone(),
        }
    }

    fn eval_binary_op(&self, op: &Operator, left: &Value, right: &Value) -> Option<Value> {
        match (op, left, right) {
            (Operator::Plus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l + r)),
            (Operator::Minus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l - r)),
            (Operator::Multiply, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l * r)),
            (Operator::Eq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l == r)),
            (Operator::NotEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l != r)),
            (Operator::Gt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l > r)),
            (Operator::Lt, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l < r)),
            (Operator::GtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l >= r)),
            (Operator::LtEq, Value::Integer(l), Value::Integer(r)) => Some(Value::Boolean(l <= r)),
            _ => None,
        }
    }

    fn eval_unary_op(&self, op: &Operator, value: &Value) -> Option<Value> {
        match (op, value) {
            (Operator::Minus, Value::Integer(n)) => Some(Value::Integer(-n)),
            (Operator::Not, Value::Boolean(b)) => Some(Value::Boolean(!b)),
            _ => None,
        }
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
