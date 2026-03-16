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
    /// Index scan operation
    IndexScan {
        table_name: String,
        index_name: String,
        predicate: Option<Expr>,
    },
    /// Filter operation (WHERE clause)
    Filter { predicate: Expr, input: Box<Plan> },
    /// Projection operation (SELECT columns)
    Projection { expr: Vec<Expr>, input: Box<Plan> },
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
    Sort { expr: Vec<Expr>, input: Box<Plan> },
    /// Limit operation
    Limit { limit: usize, input: Box<Plan> },
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
    UnaryExpr { op: Operator, expr: Box<Expr> },
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
            Plan::IndexScan { .. } => "IndexScan",
            Plan::EmptyRelation => "EmptyRelation",
        }
    }

    /// Get mutable reference to child plan if exists
    pub fn get_child_mut(&mut self) -> Option<&mut Box<Plan>> {
        match self {
            Plan::Filter { input, .. } => Some(input),
            Plan::Projection { input, .. } => Some(input),
            Plan::Join { left, right: _, .. } => Some(left),
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
                    Plan::Projection {
                        input: proj_input, ..
                    } => {
                        // Push filter through projection to its input
                        let new_filter = Plan::Filter {
                            predicate: predicate.clone(),
                            input: proj_input.clone(),
                        };
                        **input = new_filter;
                        true
                    }
                    Plan::Join {
                        left,
                        right,
                        join_type,
                        condition,
                    } => {
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

                        changed
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
            (Operator::Multiply, Value::Integer(l), Value::Integer(r)) => {
                Some(Value::Integer(l * r))
            }
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

/// ExpressionSimplification rule - simplifies boolean and arithmetic expressions
pub struct ExpressionSimplification;

impl ExpressionSimplification {
    /// Create a new ExpressionSimplification rule
    pub fn new() -> Self {
        Self
    }

    /// Rule name
    pub fn name(&self) -> &'static str {
        "ExpressionSimplification"
    }

    /// Simplify expressions in a plan
    fn simplify(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Filter { predicate, input } => {
                let simplified = self.simplify_expr(predicate);
                if simplified != *predicate {
                    *predicate = simplified;
                    return true;
                }
                self.simplify(input)
            }
            Plan::Projection { input, expr, .. } => {
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
                changed || self.simplify(input)
            }
            Plan::Aggregate { input, .. } => self.simplify(input),
            Plan::Join { left, right, .. } => {
                let changed_left = self.simplify(left);
                let changed_right = self.simplify(right);
                changed_left || changed_right
            }
            Plan::Sort { input, .. } => self.simplify(input),
            Plan::Limit { input, .. } => self.simplify(input),
            _ => false,
        }
    }

    /// Simplify an expression
    fn simplify_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::BinaryExpr { left, op, right } => {
                let left_simplified = self.simplify_expr(left);
                let right_simplified = self.simplify_expr(right);

                // Try to simplify boolean expressions
                if let Some(simplified) =
                    self.simplify_binary(&left_simplified, op, &right_simplified)
                {
                    return simplified;
                }

                // Try constant folding
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

                // Try to simplify NOT expressions
                if let Some(simplified) = self.simplify_unary(op, &simplified) {
                    return simplified;
                }

                // Try constant folding
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

    /// Simplify binary expressions (boolean logic)
    fn simplify_binary(&self, left: &Expr, op: &Operator, right: &Expr) -> Option<Expr> {
        // Boolean simplification
        match op {
            Operator::And => {
                // true AND x = x
                if let Expr::Literal(Value::Boolean(true)) = left {
                    return Some(right.clone());
                }
                // x AND true = x
                if let Expr::Literal(Value::Boolean(true)) = right {
                    return Some(left.clone());
                }
                // false AND x = false
                if let Expr::Literal(Value::Boolean(false)) = left {
                    return Some(Expr::Literal(Value::Boolean(false)));
                }
                // x AND false = false
                if let Expr::Literal(Value::Boolean(false)) = right {
                    return Some(Expr::Literal(Value::Boolean(false)));
                }
            }
            Operator::Or => {
                // false OR x = x
                if let Expr::Literal(Value::Boolean(false)) = left {
                    return Some(right.clone());
                }
                // x OR false = x
                if let Expr::Literal(Value::Boolean(false)) = right {
                    return Some(left.clone());
                }
                // true OR x = true
                if let Expr::Literal(Value::Boolean(true)) = left {
                    return Some(Expr::Literal(Value::Boolean(true)));
                }
                // x OR true = true
                if let Expr::Literal(Value::Boolean(true)) = right {
                    return Some(Expr::Literal(Value::Boolean(true)));
                }
            }
            Operator::Eq => {
                // x = x = true
                if left == right {
                    return Some(Expr::Literal(Value::Boolean(true)));
                }
            }
            Operator::NotEq => {
                // x <> x = false
                if left == right {
                    return Some(Expr::Literal(Value::Boolean(false)));
                }
            }
            _ => {}
        }

        // Arithmetic simplification
        match op {
            Operator::Plus => {
                // x + 0 = x
                if let Expr::Literal(Value::Integer(0)) = right {
                    return Some(left.clone());
                }
                // 0 + x = x
                if let Expr::Literal(Value::Integer(0)) = left {
                    return Some(right.clone());
                }
            }
            Operator::Minus => {
                // x - 0 = x
                if let Expr::Literal(Value::Integer(0)) = right {
                    return Some(left.clone());
                }
            }
            Operator::Multiply => {
                // x * 0 = 0
                if let Expr::Literal(Value::Integer(0)) = right {
                    return Some(Expr::Literal(Value::Integer(0)));
                }
                if let Expr::Literal(Value::Integer(0)) = left {
                    return Some(Expr::Literal(Value::Integer(0)));
                }
                // x * 1 = x
                if let Expr::Literal(Value::Integer(1)) = right {
                    return Some(left.clone());
                }
                // 1 * x = x
                if let Expr::Literal(Value::Integer(1)) = left {
                    return Some(right.clone());
                }
            }
            Operator::Divide => {
                // x / 1 = x
                if let Expr::Literal(Value::Integer(1)) = right {
                    return Some(left.clone());
                }
            }
            _ => {}
        }

        None
    }

    /// Simplify unary expressions
    fn simplify_unary(&self, op: &Operator, expr: &Expr) -> Option<Expr> {
        if *op == Operator::Not {
            // NOT NOT x = x
            if let Expr::UnaryExpr {
                op: Operator::Not,
                expr: inner,
            } = expr
            {
                return Some((**inner).clone());
            }
        }
        None
    }

    fn eval_binary_op(&self, op: &Operator, left: &Value, right: &Value) -> Option<Value> {
        match (op, left, right) {
            (Operator::Plus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l + r)),
            (Operator::Minus, Value::Integer(l), Value::Integer(r)) => Some(Value::Integer(l - r)),
            (Operator::Multiply, Value::Integer(l), Value::Integer(r)) => {
                Some(Value::Integer(l * r))
            }
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

impl Default for ExpressionSimplification {
    fn default() -> Self {
        Self::new()
    }
}

/// JoinReordering rule - reorders join operations for optimal performance
pub struct JoinReordering;

impl JoinReordering {
    /// Create a new JoinReordering rule
    pub fn new() -> Self {
        Self
    }

    /// Reorder join operations
    fn reorder(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Join {
                left,
                right,
                join_type,
                condition,
            } => {
                let mut changed = false;

                // First, try to reorder children
                changed |= self.reorder(left);
                changed |= self.reorder(right);

                // Then try to commute/associate this join
                if let Some(new_plan) = self.try_reorder(left, right, join_type, condition) {
                    *plan = new_plan;
                    return true;
                }

                changed
            }
            Plan::Filter { input, .. } => self.reorder(input),
            Plan::Projection { input, .. } => self.reorder(input),
            Plan::Aggregate { input, .. } => self.reorder(input),
            Plan::Sort { input, .. } => self.reorder(input),
            Plan::Limit { input, .. } => self.reorder(input),
            _ => false,
        }
    }

    /// Try to reorder join children based on cost estimation
    fn try_reorder(
        &self,
        left: &mut Box<Plan>,
        right: &mut Box<Plan>,
        join_type: &JoinType,
        condition: &Option<Expr>,
    ) -> Option<Plan> {
        // Get estimated sizes for left and right
        let left_size = self.estimate_size(left);
        let right_size = self.estimate_size(right);

        // For inner joins, prefer smaller table on the right (broadcast join)
        // This is a simple heuristic: put smaller table on the right side
        if matches!(join_type, JoinType::Inner) && left_size > right_size {
            // Try commutativity: swap left and right
            if self.can_commute(condition) {
                return Some(Plan::Join {
                    left: right.clone(),
                    right: left.clone(),
                    join_type: join_type.clone(),
                    condition: condition.clone(),
                });
            }
        }

        None
    }

    /// Estimate the size of a plan (number of rows)
    fn estimate_size(&self, plan: &Plan) -> usize {
        match plan {
            Plan::TableScan { .. } => 1000, // Default estimate
            Plan::IndexScan { .. } => 100,  // Index scan typically returns fewer rows
            Plan::Filter { input, .. } => {
                // Filter typically reduces size by 50%
                self.estimate_size(input) / 2
            }
            Plan::Projection { input, .. } => self.estimate_size(input),
            Plan::Join {
                left,
                right,
                join_type,
                condition: _,
            } => {
                match join_type {
                    JoinType::Inner => {
                        let left_size = self.estimate_size(left);
                        let right_size = self.estimate_size(right);
                        // Estimate: inner join produces ~10% of cross product
                        (left_size * right_size) / 10
                    }
                    JoinType::Left => self.estimate_size(left),
                    JoinType::Right => self.estimate_size(right),
                    JoinType::Full => {
                        let left_size = self.estimate_size(left);
                        let right_size = self.estimate_size(right);
                        left_size + right_size
                    }
                }
            }
            Plan::Aggregate { input, .. } => {
                // Aggregation reduces rows significantly
                self.estimate_size(input) / 10
            }
            Plan::Sort { input, .. } => self.estimate_size(input),
            Plan::Limit { input: _, .. } => {
                // Assume limit reduces to ~100 rows
                100
            }
            Plan::EmptyRelation => 0,
        }
    }

    /// Check if we can commute the join (swap operands)
    fn can_commute(&self, condition: &Option<Expr>) -> bool {
        // For inner join, commutativity is always safe
        // For outer joins, it's more complex
        condition.is_none()
    }
}

/// IndexSelect rule - chooses between index scan and table scan based on cost
#[allow(dead_code)]
pub struct IndexSelect {
    cost_model: crate::SimpleCostModel,
    available_indexes: Vec<(String, String)>,
}

impl IndexSelect {
    pub fn new() -> Self {
        Self {
            cost_model: crate::SimpleCostModel::default_model(),
            available_indexes: Vec::new(),
        }
    }

    pub fn with_index(mut self, table: impl Into<String>, index: impl Into<String>) -> Self {
        self.available_indexes.push((table.into(), index.into()));
        self
    }

    fn should_use_index(&self, table: &str, predicate: &Expr) -> bool {
        for (t, _) in &self.available_indexes {
            if t == table {
                return self.is_indexable_predicate(predicate);
            }
        }
        false
    }

    fn is_indexable_predicate(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::BinaryExpr {
                op: Operator::Eq,
                ..
            }
        )
    }

    fn convert_to_index_scan(&self, table: &str, predicate: &Expr) -> Plan {
        let index_name = self
            .available_indexes
            .iter()
            .find(|(t, _)| t == table)
            .map(|(_, idx)| idx.clone())
            .unwrap_or_else(|| format!("{}_pkey", table));

        Plan::IndexScan {
            table_name: table.to_string(),
            index_name,
            predicate: Some(predicate.clone()),
        }
    }
}

impl Default for IndexSelect {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule<Plan> for IndexSelect {
    fn name(&self) -> &str {
        "IndexSelect"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        match plan {
            Plan::Filter { input, predicate } => {
                if let Plan::TableScan { table_name, .. } = input.as_ref() {
                    if self.should_use_index(table_name, predicate) {
                        let index_scan = self.convert_to_index_scan(table_name, predicate);
                        **input = index_scan;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl Default for JoinReordering {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule implementation for JoinReordering
impl Rule<Plan> for JoinReordering {
    fn name(&self) -> &str {
        "JoinReordering"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.reorder(plan)
    }
}

/// Rule implementation for ExpressionSimplification
impl Rule<Plan> for ExpressionSimplification {
    fn name(&self) -> &str {
        "ExpressionSimplification"
    }

    fn apply(&self, plan: &mut Plan) -> bool {
        self.simplify(plan)
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
#[allow(dead_code)]
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
    pub fn add_rule(mut self, meta: RuleMeta, apply_fn: fn(&mut dyn Any) -> bool) -> Self {
        self.rules.push(RuleItem { meta, apply_fn });
        self.rules
            .sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in PredicatePushdown rule
    pub fn with_predicate_pushdown(mut self) -> Self {
        let meta = RuleMeta::new(
            "PredicatePushdown",
            "Push filter conditions down to data source",
        )
        .with_priority(100)
        .repeatable();
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules
            .sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in ProjectionPruning rule
    pub fn with_projection_pruning(mut self) -> Self {
        let meta = RuleMeta::new(
            "ProjectionPruning",
            "Remove unnecessary columns from projection",
        )
        .with_priority(90)
        .repeatable();
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules
            .sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
        self
    }

    /// Add built-in ConstantFolding rule
    pub fn with_constant_folding(mut self) -> Self {
        let meta = RuleMeta::new(
            "ConstantFolding",
            "Evaluate constant expressions at compile time",
        )
        .with_priority(200);
        self.rules.push(RuleItem {
            meta,
            apply_fn: |_plan: &mut dyn Any| false,
        });
        self.rules
            .sort_by(|a, b| b.meta.priority.cmp(&a.meta.priority));
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

    #[test]
    fn test_predicate_pushdown_name() {
        let rule = PredicatePushdown::new();
        assert_eq!(rule.name(), "PredicatePushdown");
    }

    #[test]
    fn test_predicate_pushdown_apply() {
        let rule = PredicatePushdown::new();
        let mut plan = Plan::EmptyRelation;
        let result = rule.apply(&mut plan);
        assert!(!result); // Returns false (no change) for EmptyRelation
    }

    #[test]
    fn test_predicate_pushdown_default() {
        let rule = PredicatePushdown::default();
        assert_eq!(rule.name(), "PredicatePushdown");
    }

    #[test]
    fn test_projection_pruning_name() {
        let rule = ProjectionPruning::new();
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_projection_pruning_apply() {
        let rule = ProjectionPruning::new();
        let mut plan = Plan::EmptyRelation;
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_projection_pruning_default() {
        let rule = ProjectionPruning::default();
        assert_eq!(rule.name(), "ProjectionPruning");
    }

    #[test]
    fn test_constant_folding_name() {
        let rule = ConstantFolding::new();
        assert_eq!(rule.name(), "ConstantFolding");
    }

    #[test]
    fn test_constant_folding_apply() {
        let rule = ConstantFolding::new();
        let mut plan = Plan::EmptyRelation;
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_constant_folding_default() {
        let rule = ConstantFolding::default();
        assert_eq!(rule.name(), "ConstantFolding");
    }

    #[test]
    fn test_expression_simplification_name() {
        let rule = ExpressionSimplification::new();
        assert_eq!(rule.name(), "ExpressionSimplification");
    }

    #[test]
    fn test_expression_simplification_and_true() {
        let rule = ExpressionSimplification::new();
        // true AND x = x
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Boolean(true))),
            op: Operator::And,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_expression_simplification_and_false() {
        let rule = ExpressionSimplification::new();
        // false AND x = false
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Boolean(false))),
            op: Operator::And,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Literal(Value::Boolean(false)));
    }

    #[test]
    fn test_expression_simplification_or_true() {
        let rule = ExpressionSimplification::new();
        // true OR x = true
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Boolean(true))),
            op: Operator::Or,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Literal(Value::Boolean(true)));
    }

    #[test]
    fn test_expression_simplification_or_false() {
        let rule = ExpressionSimplification::new();
        // false OR x = x
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Literal(Value::Boolean(false))),
            op: Operator::Or,
            right: Box::new(Expr::Column("x".to_string())),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_expression_simplification_not_not() {
        let rule = ExpressionSimplification::new();
        // NOT NOT x = x
        let expr = Expr::UnaryExpr {
            op: Operator::Not,
            expr: Box::new(Expr::UnaryExpr {
                op: Operator::Not,
                expr: Box::new(Expr::Column("x".to_string())),
            }),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_expression_simplification_eq_same() {
        let rule = ExpressionSimplification::new();
        // x = x = true
        let x = Expr::Column("x".to_string());
        let expr = Expr::BinaryExpr {
            left: Box::new(x.clone()),
            op: Operator::Eq,
            right: Box::new(x),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Literal(Value::Boolean(true)));
    }

    #[test]
    fn test_expression_simplification_arithmetic() {
        let rule = ExpressionSimplification::new();
        // x + 0 = x
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: Operator::Plus,
            right: Box::new(Expr::Literal(Value::Integer(0))),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_expression_simplification_multiply_zero() {
        let rule = ExpressionSimplification::new();
        // x * 0 = 0
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: Operator::Multiply,
            right: Box::new(Expr::Literal(Value::Integer(0))),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Literal(Value::Integer(0)));
    }

    #[test]
    fn test_expression_simplification_multiply_one() {
        let rule = ExpressionSimplification::new();
        // x * 1 = x
        let expr = Expr::BinaryExpr {
            left: Box::new(Expr::Column("x".to_string())),
            op: Operator::Multiply,
            right: Box::new(Expr::Literal(Value::Integer(1))),
        };
        let simplified = rule.simplify_expr(&expr);
        assert_eq!(simplified, Expr::Column("x".to_string()));
    }

    #[test]
    fn test_expression_simplification_apply() {
        let rule = ExpressionSimplification::new();
        let mut plan = Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let result = rule.apply(&mut plan);
        assert!(!result); // No change for table scan
    }

    // =============================================================================
    // JoinReordering Tests
    // =============================================================================

    #[test]
    fn test_join_reordering_name() {
        let rule = JoinReordering::new();
        assert_eq!(rule.name(), "JoinReordering");
    }

    #[test]
    fn test_join_reordering_inner_join_swap() {
        let rule = JoinReordering::new();

        // Create join with larger left side (TableScan = 1000) and smaller right side (Limit = 100)
        let left = Box::new(Plan::TableScan {
            table_name: "large_table".to_string(),
            projection: None,
        });
        let right = Box::new(Plan::Limit {
            limit: 10,
            input: Box::new(Plan::TableScan {
                table_name: "small_table".to_string(),
                projection: None,
            }),
        });

        let mut plan = Plan::Join {
            left,
            right,
            join_type: JoinType::Inner,
            condition: None,
        };

        let result = rule.apply(&mut plan);
        assert!(result);
    }

    #[test]
    fn test_index_select_apply_no_index() {
        let rule = IndexSelect::new();
        let mut plan = Plan::Filter {
            predicate: Expr::BinaryExpr {
                op: Operator::Gt,
                left: Box::new(Expr::Column("id".to_string())),
                right: Box::new(Expr::Literal(Value::Integer(100))),
            },
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };

        let result = rule.apply(&mut plan);
        // No index available, so should return false
        assert!(!result);
    }

    #[test]
    fn test_index_select_apply_with_index() {
        let rule = IndexSelect::new().with_index("users", "users_pkey");

        let mut plan = Plan::Filter {
            predicate: Expr::BinaryExpr {
                op: Operator::Eq,
                left: Box::new(Expr::Column("id".to_string())),
                right: Box::new(Expr::Literal(Value::Integer(100))),
            },
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };

        let result = rule.apply(&mut plan);
        assert!(result);
        // Plan should be converted to IndexScan
        if let Plan::Filter { input, .. } = &plan {
            assert!(matches!(**input, Plan::IndexScan { .. }));
        } else {
            panic!("Expected Filter on top of IndexScan");
        }
    }

    #[test]
    fn test_index_select_non_eq_predicate() {
        let rule = IndexSelect::new().with_index("users", "users_pkey");

        let mut plan = Plan::Filter {
            predicate: Expr::BinaryExpr {
                op: Operator::Gt,
                left: Box::new(Expr::Column("id".to_string())),
                right: Box::new(Expr::Literal(Value::Integer(100))),
            },
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };

        let result = rule.apply(&mut plan);
        // Greater than is not indexable, should return false
        assert!(!result);
    }

    #[test]
    fn test_index_select_default() {
        let rule = IndexSelect::default();
        assert_eq!(rule.name(), "IndexSelect");
    }

    #[test]
    fn test_plan_type_name() {
        assert_eq!(Plan::EmptyRelation.type_name(), "EmptyRelation");
        assert_eq!(
            Plan::TableScan {
                table_name: "t".to_string(),
                projection: None
            }
            .type_name(),
            "TableScan"
        );
        assert_eq!(
            Plan::IndexScan {
                table_name: "t".to_string(),
                index_name: "i".to_string(),
                predicate: None
            }
            .type_name(),
            "IndexScan"
        );
    }

    #[test]
    fn test_plan_get_child_mut() {
        let mut plan = Plan::Filter {
            predicate: Expr::Literal(Value::Boolean(true)),
            input: Box::new(Plan::EmptyRelation),
        };
        let child = plan.get_child_mut();
        assert!(child.is_some());
    }

    #[test]
    fn test_plan_get_children() {
        let plan = Plan::Filter {
            predicate: Expr::Literal(Value::Boolean(true)),
            input: Box::new(Plan::EmptyRelation),
        };
        let children = plan.get_children();
        assert_eq!(children.len(), 1);

        let join_plan = Plan::Join {
            join_type: crate::JoinType::Inner,
            left: Box::new(Plan::EmptyRelation),
            right: Box::new(Plan::EmptyRelation),
            condition: Some(Expr::Literal(Value::Boolean(true))),
        };
        let children = join_plan.get_children();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_plan_get_child_mut_no_child() {
        let mut plan = Plan::EmptyRelation;
        let child = plan.get_child_mut();
        assert!(child.is_none());

        let mut plan = Plan::TableScan {
            table_name: "t".to_string(),
            projection: None,
        };
        let child = plan.get_child_mut();
        assert!(child.is_none());
    }

    #[test]
    fn test_join_reordering_new() {
        let rule = JoinReordering::new();
        assert_eq!(rule.name(), "JoinReordering");
    }

    #[test]
    fn test_join_reordering_default() {
        let rule = JoinReordering::default();
        assert_eq!(rule.name(), "JoinReordering");
    }

    #[test]
    fn test_join_reordering_estimate_size_tablescan() {
        let rule = JoinReordering::new();
        let plan = Plan::TableScan {
            table_name: "users".to_string(),
            projection: None,
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 1000);
    }

    #[test]
    fn test_join_reordering_estimate_size_indexscan() {
        let rule = JoinReordering::new();
        let plan = Plan::IndexScan {
            table_name: "users".to_string(),
            index_name: "idx".to_string(),
            predicate: None,
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 100);
    }

    #[test]
    fn test_join_reordering_estimate_size_filter() {
        let rule = JoinReordering::new();
        let plan = Plan::Filter {
            predicate: Expr::Literal(Value::Boolean(true)),
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 500); // 1000 / 2
    }

    #[test]
    fn test_join_reordering_estimate_size_aggregate() {
        let rule = JoinReordering::new();
        let plan = Plan::Aggregate {
            group_by: vec![],
            aggregates: vec![],
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 100); // 1000 / 10
    }

    #[test]
    fn test_join_reordering_estimate_size_limit() {
        let rule = JoinReordering::new();
        let plan = Plan::Limit {
            limit: 50,
            input: Box::new(Plan::TableScan {
                table_name: "users".to_string(),
                projection: None,
            }),
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 100);
    }

    #[test]
    fn test_join_reordering_estimate_size_empty() {
        let rule = JoinReordering::new();
        let plan = Plan::EmptyRelation;
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_join_reordering_estimate_size_inner_join() {
        let rule = JoinReordering::new();
        let plan = Plan::Join {
            join_type: JoinType::Inner,
            left: Box::new(Plan::TableScan {
                table_name: "a".to_string(),
                projection: None,
            }),
            right: Box::new(Plan::TableScan {
                table_name: "b".to_string(),
                projection: None,
            }),
            condition: None,
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 100000); // 1000 * 1000 / 10
    }

    #[test]
    fn test_join_reordering_estimate_size_left_join() {
        let rule = JoinReordering::new();
        let plan = Plan::Join {
            join_type: JoinType::Left,
            left: Box::new(Plan::TableScan {
                table_name: "a".to_string(),
                projection: None,
            }),
            right: Box::new(Plan::TableScan {
                table_name: "b".to_string(),
                projection: None,
            }),
            condition: None,
        };
        let size = rule.estimate_size(&plan);
        assert_eq!(size, 1000); // left size
    }

    #[test]
    fn test_join_reordering_can_commute() {
        let rule = JoinReordering::new();
        assert!(rule.can_commute(&None));
        assert!(!rule.can_commute(&Some(Expr::Literal(Value::Boolean(true)))));
    }

    #[test]
    fn test_predicate_pushdown_apply_with_projection() {
        let rule = PredicatePushdown::new();
        let mut plan = Plan::Projection {
            expr: vec![],
            input: Box::new(Plan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_predicate_pushdown_apply_with_join() {
        let rule = PredicatePushdown::new();
        let mut plan = Plan::Join {
            join_type: JoinType::Inner,
            left: Box::new(Plan::TableScan {
                table_name: "a".to_string(),
                projection: None,
            }),
            right: Box::new(Plan::TableScan {
                table_name: "b".to_string(),
                projection: None,
            }),
            condition: Some(Expr::Column("id".to_string())),
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_projection_pruning_apply_with_projection() {
        let rule = ProjectionPruning::new();
        let mut plan = Plan::Projection {
            expr: vec![],
            input: Box::new(Plan::TableScan {
                table_name: "t".to_string(),
                projection: None,
            }),
        };
        let result = rule.apply(&mut plan);
        assert!(!result);
    }

    #[test]
    fn test_constant_folding_with_literal() {
        let mut plan = Plan::Projection {
            expr: vec![Expr::BinaryExpr {
                left: Box::new(Expr::Literal(Value::Integer(1))),
                op: Operator::Plus,
                right: Box::new(Expr::Literal(Value::Integer(2))),
            }],
            input: Box::new(Plan::EmptyRelation),
        };
        let rule = ConstantFolding::new();
        let result = rule.apply(&mut plan);
        assert!(result);
    }
}
