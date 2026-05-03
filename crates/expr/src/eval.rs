//! Expression evaluation engine
//!
//! Single source of truth for all expression evaluation in SQLRustGo.
//! Row format: `HashMap<String, Value>` with keys `"table.column"`.

use sqlrustgo_types::Value;
use std::collections::HashMap;

use crate::expr::Expr;
use crate::op::BinaryOp;

/// Evaluation context — holds row data and external state
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Row data: key = "table.column" or just "column"
    pub values: HashMap<String, Value>,
}

impl EvalContext {
    pub fn new(values: HashMap<String, Value>) -> Self {
        Self { values }
    }

    /// Build from a flat row vec + schema (position-based).
    /// Caller provides (table, column, position) mappings.
    pub fn from_row_vec(row: &[Value], columns: &[(String, String)]) -> Self {
        let mut values = HashMap::new();
        for (i, v) in row.iter().enumerate() {
            if i < columns.len() {
                let (t, c) = &columns[i];
                let key = if t.is_empty() {
                    c.clone()
                } else {
                    format!("{}.{}", t, c)
                };
                values.insert(key, v.clone());
            }
        }
        Self { values }
    }
}

impl Expr {
    /// Evaluate this expression against a row.
    /// Returns Value::Boolean(true) if condition passes, Value::Boolean(false) if not.
    pub fn eval(&self, ctx: &EvalContext) -> Value {
        match self {
            Expr::Literal(v) => v.clone(),

            Expr::Column { table, name } => {
                let key = if let Some(t) = table {
                    format!("{}.{}", t, name)
                } else {
                    format!("{}", name)
                };
                // Try fully-qualified first, then unqualified
                if let Some(v) = ctx.values.get(&key) {
                    v.clone()
                } else {
                    // Fallback: scan for unqualified column
                    for (k, v) in &ctx.values {
                        if k.ends_with(&format!(".{}", name)) {
                            return v.clone();
                        }
                    }
                    ctx.values.get(name.as_str()).cloned().unwrap_or(Value::Null)
                }
            }

            Expr::Binary { left, op, right } => {
                let l = left.eval(ctx);
                let r = right.eval(ctx);
                eval_binary(l, op, r)
            }

            Expr::Unary { op, expr } => {
                let v = expr.eval(ctx);
                eval_unary(op, v)
            }

            Expr::Wildcard => Value::Null,
            Expr::QualifiedWildcard { .. } => Value::Null,

            Expr::Alias { expr, .. } => expr.eval(ctx),

            Expr::Function { name, args } => eval_function(name, args, ctx),

            Expr::Aggregate { .. } => {
                // Aggregate evaluation is done by AggregateExecutor, not here.
                // This path is for embedded aggregate expressions in HAVING etc.
                // For now, return Null (requires context).
                Value::Null
            }

            Expr::Subquery(_) => Value::Null,
        }
    }

    /// Evaluate as a boolean predicate (for WHERE/HAVING).
    /// Returns true if expression evaluates to Boolean(true).
    pub fn eval_predicate(&self, ctx: &EvalContext) -> bool {
        match self.eval(ctx) {
            Value::Boolean(true) => true,
            Value::Null => false, // NULL in predicate = false (standard SQL)
            _ => false,
        }
    }
}

/// Evaluate a binary operation
fn eval_binary(l: Value, op: &BinaryOp, r: Value) -> Value {
    match op {
        BinaryOp::Eq => Value::Boolean(l == r),
        BinaryOp::NotEq => Value::Boolean(l != r),
        BinaryOp::Gt => cmp(l, r, |o| o.is_gt()),
        BinaryOp::Lt => cmp(l, r, |o| o.is_lt()),
        BinaryOp::GtEq => cmp(l, r, |o| o.is_ge()),
        BinaryOp::LtEq => cmp(l, r, |o| o.is_le()),
        BinaryOp::And => bool_op(l, r, |a, b| a && b),
        BinaryOp::Or => bool_op(l, r, |a, b| a || b),
        BinaryOp::Plus => int_op(l, r, |a, b| a + b),
        BinaryOp::Minus => int_op(l, r, |a, b| a - b),
        BinaryOp::Multiply => int_op(l, r, |a, b| a * b),
        BinaryOp::Divide => int_div(l, r),
        BinaryOp::Like => like(l, r),
    }
}

/// Evaluate a unary operation
fn eval_unary(op: &crate::expr::UnaryOp, v: Value) -> Value {
    match op {
        crate::expr::UnaryOp::Not => {
            if let Value::Boolean(b) = v {
                Value::Boolean(!b)
            } else {
                Value::Null
            }
        }
        crate::expr::UnaryOp::Minus => {
            if let Value::Integer(i) = v {
                Value::Integer(-i)
            } else {
                Value::Null
            }
        }
    }
}

/// Compare two values using a closure
fn cmp<F>(l: Value, r: Value, f: F) -> Value
where
    F: Fn(std::cmp::Ordering) -> bool, {
    Value::Boolean(f(l.cmp(&r)))
}

/// Boolean operation
fn bool_op<F>(l: Value, r: Value, f: F) -> Value
where
    F: FnOnce(bool, bool) -> bool,
{
    match (l, r) {
        (Value::Boolean(a), Value::Boolean(b)) => Value::Boolean(f(a, b)),
        _ => Value::Null,
    }
}

/// Integer arithmetic operation
fn int_op<F>(l: Value, r: Value, f: F) -> Value
where
    F: FnOnce(i64, i64) -> i64,
{
    match (l, r) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(f(a, b)),
        _ => Value::Null,
    }
}

/// Integer division
fn int_div(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Integer(_a), Value::Integer(0)) => Value::Null, // SQL: division by zero = NULL
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a / b),
        _ => Value::Null,
    }
}

/// LIKE pattern matching
fn like(l: Value, r: Value) -> Value {
    let (Value::Text(s), Value::Text(pattern)) = (l, r) else {
        return Value::Null;
    };
    // Simple LIKE: support % and _ wildcards
    let regex_pattern = pattern.replace('%', ".*").replace('_', ".");
    let regex_pattern = format!("^{}$", regex_pattern);
    match regex::Regex::new(&regex_pattern) {
        Ok(re) => Value::Boolean(re.is_match(&s)),
        Err(_) => Value::Boolean(false),
    }
}

/// Evaluate a scalar function
fn eval_function(name: &str, args: &[Expr], ctx: &EvalContext) -> Value {
    let name_lower = name.to_lowercase();
    match name_lower.as_str() {
        "lower" => {
            if let [Expr::Literal(Value::Text(s))] = args {
                Value::Text(s.to_lowercase())
            } else {
                Value::Null
            }
        }
        "upper" => {
            if let [Expr::Literal(Value::Text(s))] = args {
                Value::Text(s.to_uppercase())
            } else {
                Value::Null
            }
        }
        "length" | "len" => {
            if let [Expr::Literal(Value::Text(s))] = args {
                Value::Integer(s.len() as i64)
            } else {
                Value::Null
            }
        }
        "concat" => {
            let mut result = String::new();
            for arg in args {
                if let Value::Text(s) = arg.eval(ctx) {
                    result.push_str(&s);
                } else {
                    return Value::Null;
                }
            }
            Value::Text(result)
        }
        "coalesce" => {
            for arg in args {
                let v = arg.eval(ctx);
                if !matches!(v, Value::Null) {
                    return v;
                }
            }
            Value::Null
        }
        "nullif" => {
            if args.len() == 2 {
                let a = args[0].eval(ctx);
                let b = args[1].eval(ctx);
                if a == b {
                    return Value::Null;
                }
                return a;
            }
            Value::Null
        }
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(values: &[(&str, i64)]) -> EvalContext {
        let mut m = HashMap::new();
        for (k, v) in values {
            m.insert((*k).to_string(), Value::Integer(*v));
        }
        EvalContext::new(m)
    }

    #[test]
    fn test_literal() {
        let e = Expr::literal(Value::Integer(42));
        assert_eq!(e.eval(&ctx(&[])), Value::Integer(42));
    }

    #[test]
    fn test_column() {
        let e = Expr::column("x");
        assert_eq!(e.eval(&ctx(&[("x", 10)])), Value::Integer(10));
    }

    #[test]
    fn test_qualified_column() {
        let e = Expr::qualified_column("t", "x");
        assert_eq!(e.eval(&ctx(&[("t.x", 20)])), Value::Integer(20));
    }

    #[test]
    fn test_eq() {
        let e = Expr::binary(
            Expr::column("x"),
            BinaryOp::Eq,
            Expr::literal(Value::Integer(10)),
        );
        assert_eq!(e.eval(&ctx(&[("x", 10)])), Value::Boolean(true));
        assert_eq!(e.eval(&ctx(&[("x", 20)])), Value::Boolean(false));
    }

    #[test]
    fn test_gt() {
        let e = Expr::binary(
            Expr::column("x"),
            BinaryOp::Gt,
            Expr::literal(Value::Integer(10)),
        );
        assert_eq!(e.eval(&ctx(&[("x", 20)])), Value::Boolean(true));
        assert_eq!(e.eval(&ctx(&[("x", 5)])), Value::Boolean(false));
    }

    #[test]
    fn test_and() {
        let e = Expr::binary(
            Expr::binary(
                Expr::column("x"),
                BinaryOp::Gt,
                Expr::literal(Value::Integer(0)),
            ),
            BinaryOp::And,
            Expr::binary(
                Expr::column("x"),
                BinaryOp::Lt,
                Expr::literal(Value::Integer(100)),
            ),
        );
        assert_eq!(e.eval(&ctx(&[("x", 50)])), Value::Boolean(true));
        assert_eq!(e.eval(&ctx(&[("x", -1)])), Value::Boolean(false));
    }

    #[test]
    fn test_or() {
        let e = Expr::binary(Expr::column("x"), BinaryOp::Or, Expr::column("y"));
        assert_eq!(
            e.eval(&EvalContext::new(HashMap::from([
                ("x".to_string(), Value::Boolean(true)),
                ("y".to_string(), Value::Boolean(false)),
            ]))),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_plus() {
        let e = Expr::binary(Expr::column("a"), BinaryOp::Plus, Expr::column("b"));
        assert_eq!(
            e.eval(&EvalContext::new(HashMap::from([
                ("a".to_string(), Value::Integer(3)),
                ("b".to_string(), Value::Integer(4)),
            ]))),
            Value::Integer(7)
        );
    }

    #[test]
    fn test_divide_by_zero() {
        let e = Expr::binary(
            Expr::column("a"),
            BinaryOp::Divide,
            Expr::literal(Value::Integer(0)),
        );
        assert_eq!(e.eval(&ctx(&[("a", 10)])), Value::Null);
    }

    #[test]
    fn test_like() {
        let e = Expr::binary(
            Expr::literal(Value::Text("hello".to_string())),
            BinaryOp::Like,
            Expr::literal(Value::Text("hel%".to_string())),
        );
        assert_eq!(e.eval(&ctx(&[])), Value::Boolean(true));
    }

    #[test]
    fn test_not() {
        let e = Expr::Unary {
            op: crate::expr::UnaryOp::Not,
            expr: Box::new(Expr::literal(Value::Boolean(true))),
        };
        assert_eq!(e.eval(&ctx(&[])), Value::Boolean(false));
    }

    #[test]
    fn test_alias() {
        let e = Expr::Alias {
            expr: Box::new(Expr::column("x")),
            alias: "renamed".to_string(),
        };
        assert_eq!(e.eval(&ctx(&[("x", 99)])), Value::Integer(99));
    }

    #[test]
    fn test_contains_subquery() {
        let e = Expr::binary(
            Expr::column("x"),
            BinaryOp::Eq,
            Expr::Subquery(Box::new(crate::expr::SubqueryExpr {
                query: crate::expr::Subquery::Scalar,
            })),
        );
        assert!(e.contains_subquery());
    }

    #[test]
    fn test_eval_predicate() {
        let e = Expr::binary(
            Expr::column("x"),
            BinaryOp::Gt,
            Expr::literal(Value::Integer(10)),
        );
        assert!(e.eval_predicate(&ctx(&[("x", 20)])));
        assert!(!e.eval_predicate(&ctx(&[("x", 5)])));
        // NULL predicate = false
        let null_e = Expr::literal(Value::Null);
        assert!(!null_e.eval_predicate(&ctx(&[])));
    }
}
