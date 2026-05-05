//! Join Planner - logical join graph builder
//!
//! Transforms a flat FROM clause + WHERE predicate list into a chain of
//! binary hash joins. This is the "logical plan" that feeds the physical
//! execution layer.
//!
//! # What it does
//!
//! 1. **Split predicates** - WHERE is split into:
//!    - JOIN predicates: cross-table `=` conditions (go into HashJoin ON clause)
//!    - FILTER predicates: everything else (applied after joins)
//!
//! 2. **Build join chain** - greedily connect tables using join predicates
//!    until all tables are in the plan
//!
//! 3. **Execute** - the resulting JoinPlan is handed to the physical executor
//!    which performs nested hash joins.

use std::collections::HashMap;
use std::fmt;

/// A cross-table equality predicate suitable for a hash join
#[derive(Debug, Clone, PartialEq)]
pub struct JoinPredicate {
    /// Table on the left side of the equality
    pub left_table: String,
    /// Column on the left side
    pub left_col: String,
    /// Table on the right side of the equality
    pub right_table: String,
    /// Column on the right side
    pub right_col: String,
}

impl JoinPredicate {
    /// Returns the fully-qualified left key: "table.column"
    pub fn left_key(&self) -> String {
        format!("{}.{}", self.left_table, self.left_col)
    }

    /// Returns the fully-qualified right key: "table.column"
    pub fn right_key(&self) -> String {
        format!("{}.{}", self.right_table, self.right_col)
    }
}

/// A predicate that cannot be used for a hash join (non-equality, same-table, etc.)
/// Gets applied as a filter after all joins are complete.
#[derive(Debug, Clone)]
pub struct FilterPredicate {
    /// The original expression AST node (opaque to this module)
    pub expr: String,
}

impl fmt::Display for FilterPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filter({})", self.expr)
    }
}

/// One step in the join chain: hash-join with the given table on the given predicate
#[derive(Debug, Clone)]
pub struct JoinStep {
    /// The table to join on the right side of this hash join
    pub right_table: String,
    /// The join predicate (left side is always the result of previous steps)
    pub on: JoinPredicate,
}

/// The complete logical join plan
#[derive(Debug, Clone)]
pub struct JoinPlan {
    /// The first (leftmost) table in the FROM clause
    pub base_table: String,
    /// Ordered list of join steps to apply
    pub joins: Vec<JoinStep>,
    /// Remaining predicates that are not join conditions
    pub filters: Vec<FilterPredicate>,
}

impl JoinPlan {
    /// Returns the total number of tables involved in this join
    pub fn table_count(&self) -> usize {
        1 + self.joins.len()
    }

    /// Returns all table names in join order
    pub fn tables(&self) -> Vec<&str> {
        let mut t = vec![self.base_table.as_str()];
        t.extend(self.joins.iter().map(|j| j.right_table.as_str()));
        t
    }
}

// ---------------------------------------------------------------------------
// Expression type used internally for predicate analysis
// ---------------------------------------------------------------------------

/// Internal expression representation for predicate splitting.
/// We only care about a subset of expression shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateExpr {
    /// table.column reference
    Column { table: String, column: String },
    /// Literal value
    Literal(String),
    /// AND of two expressions
    And(Box<PredicateExpr>, Box<PredicateExpr>),
    /// OR of two expressions
    Or(Box<PredicateExpr>, Box<PredicateExpr>),
    /// Binary comparison
    Compare { op: String, left: Box<PredicateExpr>, right: Box<PredicateExpr> },
    /// IS NULL
    IsNull(Box<PredicateExpr>),
    /// IS NOT NULL
    IsNotNull(Box<PredicateExpr>),
    /// Unknown / unsupported expression (stored as raw string)
    Unknown(String),
}

impl PredicateExpr {
    /// Returns the table name if this is a column reference
    pub fn table(&self) -> Option<&str> {
        match self {
            PredicateExpr::Column { table, .. } => Some(table),
            _ => None,
        }
    }

    /// Returns the column name if this is a column reference
    pub fn column(&self) -> Option<&str> {
        match self {
            PredicateExpr::Column { column, .. } => Some(column),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate extraction utilities
// ---------------------------------------------------------------------------

/// Extract table and column from a possibly-qualified identifier string.
/// E.g. "t1.id" -> Some("t1", "id"), "id" -> Some("", "id")
fn parse_qualified_name(name: &str) -> (String, String) {
    if let Some(pos) = name.find('.') {
        (name[..pos].to_string(), name[pos + 1..].to_string())
    } else {
        (String::new(), name.to_string())
    }
}

/// Returns true if two column references refer to different tables
fn is_cross_table(l_table: &str, r_table: &str) -> bool {
    !l_table.is_empty() && !r_table.is_empty() && l_table != r_table
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Split a list of raw WHERE conditions into JOIN predicates and FILTER predicates.
///
/// JOIN predicates are cross-table equality conditions that can drive hash joins.
/// Everything else becomes a filter applied after the join chain is complete.
pub fn split_predicates(
    conditions: &[String],
) -> (Vec<JoinPredicate>, Vec<FilterPredicate>) {
    let mut joins = Vec::new();
    let mut filters = Vec::new();

    for cond in conditions {
        if let Some(pred) = analyze_condition(cond) {
            split_recursive(&pred, &mut joins, &mut filters);
        } else {
            // Couldn't parse - treat as filter
            filters.push(FilterPredicate { expr: cond.clone() });
        }
    }

    (joins, filters)
}

/// Recursively split a predicate tree into join vs filter predicates.
fn split_recursive(expr: &PredicateExpr, joins: &mut Vec<JoinPredicate>, filters: &mut Vec<FilterPredicate>) {
    match expr {
        // AND: recurse into both sides
        PredicateExpr::And(l, r) => {
            split_recursive(l, joins, filters);
            split_recursive(r, joins, filters);
        }
        // OR: whole expression is a filter (we don't yet try to optimize OR)
        PredicateExpr::Or(_, _) => {
            filters.push(FilterPredicate { expr: expr.to_string() });
        }
        // Compare: might be a join predicate
        PredicateExpr::Compare { op, left, right }
            if op.to_uppercase() == "=" || op == "==" =>
        {
            if let (Some(lc), Some(rc)) = (left.as_ref().column(), right.as_ref().column()) {
                let (lt, lc2) = parse_qualified_name(lc);
                let (rt, rc2) = parse_qualified_name(rc);
                if is_cross_table(&lt, &rt) {
                    joins.push(JoinPredicate {
                        left_table: lt,
                        left_col: lc2,
                        right_table: rt,
                        right_col: rc2,
                    });
                    return;
                }
            }
            filters.push(FilterPredicate { expr: expr.to_string() });
        }
        // Everything else → filter
        _ => {
            filters.push(FilterPredicate { expr: expr.to_string() });
        }
    }
}

/// Build a join plan from a list of tables and their join predicates.
///
/// Uses a greedy algorithm: start with the first table, then repeatedly find
/// a join predicate that connects an already-visited table to an unvisited table
/// until all tables are in the plan.
///
/// # Panics
///
/// Panics if the join predicates do not connect all tables (i.e., a cartesian
/// product would be required).
pub fn build_join_plan(
    tables: &[String],
    join_preds: Vec<JoinPredicate>,
    filters: Vec<FilterPredicate>,
) -> JoinPlan {
    let mut used: HashMap<String, ()> = HashMap::new();
    let mut joins = Vec::new();
    let mut remaining: Vec<JoinPredicate> = join_preds;

    if tables.is_empty() {
        return JoinPlan {
            base_table: String::new(),
            joins: vec![],
            filters,
        };
    }

    let base = tables[0].clone();
    used.insert(base.clone(), ());

    while used.len() < tables.len() {
        let mut progress = false;

        for i in 0..remaining.len() {
            let p = &remaining[i];

            // Try left → right
            if used.contains_key(&p.left_table) && !used.contains_key(&p.right_table) {
                joins.push(JoinStep {
                    right_table: p.right_table.clone(),
                    on: p.clone(),
                });
                used.insert(p.right_table.clone(), ());
                remaining.remove(i);
                progress = true;
                break;
            }

            // Try right → left (flip the predicate)
            if used.contains_key(&p.right_table) && !used.contains_key(&p.left_table) {
                joins.push(JoinStep {
                    right_table: p.left_table.clone(),
                    on: JoinPredicate {
                        left_table: p.right_table.clone(),
                        left_col: p.right_col.clone(),
                        right_table: p.left_table.clone(),
                        right_col: p.left_col.clone(),
                    },
                });
                used.insert(p.left_table.clone(), ());
                remaining.remove(i);
                progress = true;
                break;
            }
        }

        if !progress {
            // No predicate found - remaining tables are disconnected
            // They will be cross-joined (cartesian product)
            for table in tables {
                if !used.contains_key(table) {
                    // Add a dummy cross-join step
                    joins.push(JoinStep {
                        right_table: table.clone(),
                        on: JoinPredicate {
                            left_table: String::new(),
                            left_col: String::new(),
                            right_table: String::new(),
                            right_col: String::new(),
                        },
                    });
                    used.insert(table.clone(), ());
                }
            }
        }
    }

    JoinPlan {
        base_table: base,
        joins,
        filters,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum PredicateClass {
    Join(JoinPredicate),
    Filter(FilterPredicate),
    Unsupported(String),
}

impl std::fmt::Display for PredicateExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PredicateExpr::Column { table, column } => {
                if table.is_empty() {
                    write!(f, "{}", column)
                } else {
                    write!(f, "{}.{}", table, column)
                }
            }
            PredicateExpr::Literal(s) => write!(f, "{}", s),
            PredicateExpr::And(l, r) => write!(f, "({} AND {})", l, r),
            PredicateExpr::Or(l, r) => write!(f, "({} OR {})", l, r),
            PredicateExpr::Compare { op, left, right } => {
                write!(f, "{} {} {}", left, op, right)
            }
            PredicateExpr::IsNull(e) => write!(f, "{} IS NULL", e),
            PredicateExpr::IsNotNull(e) => write!(f, "{} IS NOT NULL", e),
            PredicateExpr::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Attempt to parse a raw condition string into a PredicateExpr.
/// Returns None if parsing fails (expression is too complex).
fn analyze_condition(cond: &str) -> Option<PredicateExpr> {
    // Fast path: look for simple "a = b" or "a.x = b.y" patterns
    let cond = cond.trim();

    // Handle IS NULL / IS NOT NULL
    if cond.to_uppercase().ends_with(" IS NULL") {
        let inner = &cond[..cond.len() - 8].trim();
        if let Some(e) = parse_simple_expr(inner) {
            return Some(PredicateExpr::IsNull(Box::new(e)));
        }
    }
    if cond.to_uppercase().ends_with(" IS NOT NULL") {
        let inner = &cond[..cond.len() - 12].trim();
        if let Some(e) = parse_simple_expr(inner) {
            return Some(PredicateExpr::IsNotNull(Box::new(e)));
        }
    }

    // Handle AND (binary, non-nested)
    if let Some(pos) = find_top_level_and(cond) {
        let left = analyze_condition(&cond[..pos])?;
        let right = analyze_condition(&cond[pos + 4..])?;
        return Some(PredicateExpr::And(Box::new(left), Box::new(right)));
    }

    // Handle OR (binary, non-nested)
    if let Some(pos) = find_top_level_or(cond) {
        let left = analyze_condition(&cond[..pos])?;
        let right = analyze_condition(&cond[pos + 3..])?;
        return Some(PredicateExpr::Or(Box::new(left), Box::new(right)));
    }

    // Handle comparison operators
    for (op, display) in [
        ("!=", "!="),
        ("<>", "<>"),
        ("=", "="),
        ("==", "=="),
        (">=", ">="),
        ("<=", "<="),
        (">", ">"),
        ("<", "<"),
    ] {
        if let Some(pos) = cond.find(op) {
            let left = cond[..pos].trim();
            let right = cond[pos + op.len()..].trim();
            let left_expr = parse_simple_expr(left)?;
            let right_expr = parse_simple_expr(right)?;
            return Some(PredicateExpr::Compare {
                op: display.to_string(),
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            });
        }
    }

    parse_simple_expr(cond)
}

/// Parse a simple expression: either a column or a literal.
fn parse_simple_expr(s: &str) -> Option<PredicateExpr> {
    let s = s.trim();

    // Remove parentheses
    if s.starts_with('(') && s.ends_with(')') {
        return analyze_condition(&s[1..s.len() - 1]);
    }

    // Try column
    if !s.is_empty()
        && !s.starts_with('\'')
        && !s.starts_with('"')
        && !s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
    {
        let (table, col) = parse_qualified_name(s);
        return Some(PredicateExpr::Column {
            table,
            column: col,
        });
    }

    // Literal
    Some(PredicateExpr::Literal(s.to_string()))
}

/// Find the position of a top-level AND (not inside parentheses)
fn find_top_level_and(s: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    let bytes = s.as_bytes();
    let mut in_string = false;
    let mut escape_next = false;

    for (i, &b) in bytes.iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if b == b'\\' {
            escape_next = true;
            continue;
        }

        if b == b'\'' || b == b'"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'A' if depth == 0 => {
                if i + 3 <= bytes.len() {
                    let chunk = &s[i..i + 3];
                    if chunk.eq_ignore_ascii_case("AND") {
                        return Some(i);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Find the position of a top-level OR (not inside parentheses)
fn find_top_level_or(s: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    let bytes = s.as_bytes();
    let mut in_string = false;
    let mut escape_next = false;

    for (i, &b) in bytes.iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if b == b'\\' {
            escape_next = true;
            continue;
        }

        if b == b'\'' || b == b'"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b'O' if depth == 0 => {
                if i + 2 <= bytes.len() {
                    let chunk = &s[i..i + 2];
                    if chunk.eq_ignore_ascii_case("OR") {
                        return Some(i);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_simple_equality() {
        let (joins, filters) = split_predicates(&["t1.id = t2.id".to_string()]);
        assert_eq!(joins.len(), 1);
        assert_eq!(joins[0].left_table, "t1");
        assert_eq!(joins[0].left_col, "id");
        assert_eq!(joins[0].right_table, "t2");
        assert_eq!(joins[0].right_col, "id");
        assert!(filters.is_empty());
    }

    #[test]
    fn test_split_filter_non_equality() {
        let (joins, filters) = split_predicates(&["t1.x > 10".to_string()]);
        assert!(joins.is_empty());
        assert_eq!(filters.len(), 1);
    }

    #[test]
    fn test_split_same_table() {
        let (joins, filters) = split_predicates(&["t1.id = t1.parent_id".to_string()]);
        assert!(joins.is_empty());
        assert_eq!(filters.len(), 1);
    }

    #[test]
    fn test_build_join_plan_three_tables() {
        let tables = vec!["t1".into(), "t2".into(), "t3".into()];
        let joins = vec![
            JoinPredicate {
                left_table: "t1".into(),
                left_col: "id".into(),
                right_table: "t2".into(),
                right_col: "id".into(),
            },
            JoinPredicate {
                left_table: "t2".into(),
                left_col: "id".into(),
                right_table: "t3".into(),
                right_col: "id".into(),
            },
        ];

        let plan = build_join_plan(&tables, joins, vec![]);

        assert_eq!(plan.base_table, "t1");
        assert_eq!(plan.joins.len(), 2);
        assert_eq!(plan.joins[0].right_table, "t2");
        assert_eq!(plan.joins[1].right_table, "t3");
    }

    #[test]
    fn test_build_join_plan_reverse_order() {
        // Predicate connects t2→t1 first, then t3→t2
        let tables = vec!["t1".into(), "t2".into(), "t3".into()];
        let joins = vec![
            JoinPredicate {
                left_table: "t2".into(),
                left_col: "id".into(),
                right_table: "t1".into(),
                right_col: "id".into(),
            },
            JoinPredicate {
                left_table: "t2".into(),
                left_col: "id".into(),
                right_table: "t3".into(),
                right_col: "id".into(),
            },
        ];

        let plan = build_join_plan(&tables, joins, vec![]);

        assert_eq!(plan.base_table, "t1");
        assert_eq!(plan.joins.len(), 2);
        // t2 should join to t1 first (flipped)
        assert_eq!(plan.joins[0].right_table, "t2");
    }
}
