use super::context::EvalContext;
use sqlrustgo_parser::Expression;
use sqlrustgo_types::Value;

pub fn expression_to_value(
    expr: &Expression,
    ctx: &EvalContext,
    _column_names: Option<&[String]>,
) -> Value {
    match expr {
        Expression::Literal(s) => parse_literal(s),
        Expression::Identifier(name) => resolve_identifier(name, ctx),
        Expression::BinaryOp(left, op, right) => {
            let l = expression_to_value(left, ctx, None);
            let r = expression_to_value(right, ctx, None);
            evaluate_binary_op(&l, op, &r)
        }
        _ => Value::Null,
    }
}

pub fn expression_to_bool(
    expr: &Expression,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> bool {
    match expression_to_value(expr, ctx, column_names) {
        Value::Boolean(b) => b,
        Value::Null => false,
        _ => true,
    }
}

fn parse_literal(s: &str) -> Value {
    let s = s.trim();
    if s.eq_ignore_ascii_case("NULL") {
        return Value::Null;
    }
    if let Ok(n) = s.parse::<i64>() {
        return Value::Integer(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }
    if s.starts_with('\'') && s.ends_with('\'') {
        return Value::Text(s[1..s.len() - 1].to_string());
    }
    Value::Text(s.to_string())
}

fn resolve_identifier(name: &str, ctx: &EvalContext) -> Value {
    let name_upper = name.to_uppercase();

    if let Some(col_name) = name_upper.strip_prefix("NEW.") {
        if let Some(new_row) = ctx.trigger().new_row() {
            if let Some(cols) = ctx.trigger().new_col_names() {
                if let Some(idx) = cols.iter().position(|c| c.eq_ignore_ascii_case(col_name)) {
                    if idx < new_row.len() {
                        return new_row[idx].clone();
                    }
                }
            }
        }
        return Value::Null;
    }

    if let Some(col_name) = name_upper.strip_prefix("OLD.") {
        if let Some(old_row) = ctx.trigger().old_row() {
            if let Some(cols) = ctx.trigger().old_col_names() {
                if let Some(idx) = cols.iter().position(|c| c.eq_ignore_ascii_case(col_name)) {
                    if idx < old_row.len() {
                        return old_row[idx].clone();
                    }
                }
            }
        }
        return Value::Null;
    }

    if let Some(target_row) = ctx.target_row() {
        if let Some(cols) = ctx.target_col_names() {
            if let Some(idx) = cols.iter().position(|c| c.eq_ignore_ascii_case(name)) {
                if idx < target_row.len() {
                    return target_row[idx].clone();
                }
            }
        }
    }

    Value::Null
}

fn evaluate_binary_op(left: &Value, op: &str, right: &Value) -> Value {
    match op {
        "+" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                return Value::Integer(l + r);
            }
            if let (Value::Float(l), Value::Float(r)) = (left, right) {
                return Value::Float(l + r);
            }
            if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                return Value::Float(*l as f64 + r);
            }
            if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                return Value::Float(l + *r as f64);
            }
        }
        "-" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                return Value::Integer(l - r);
            }
            if let (Value::Float(l), Value::Float(r)) = (left, right) {
                return Value::Float(l - r);
            }
            if let (Value::Integer(l), Value::Float(r)) = (left, right) {
                return Value::Float(*l as f64 - r);
            }
            if let (Value::Float(l), Value::Integer(r)) = (left, right) {
                return Value::Float(l - *r as f64);
            }
        }
        "*" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                return Value::Integer(l * r);
            }
            if let (Value::Float(l), Value::Float(r)) = (left, right) {
                return Value::Float(l * r);
            }
        }
        "/" => {
            if let (Value::Integer(l), Value::Integer(r)) = (left, right) {
                if *r != 0 {
                    return Value::Integer(l / r);
                }
            }
            if let (Value::Float(l), Value::Float(r)) = (left, right) {
                if *r != 0.0 {
                    return Value::Float(l / r);
                }
            }
        }
        _ => {}
    }
    Value::Null
}

#[cfg(test)]
mod tests {
    use crate::trigger_eval::context::{EvalContext, TriggerContext};
    use crate::trigger_eval::expression::{evaluate_binary_op, expression_to_bool, expression_to_value, parse_literal, resolve_identifier};
    use sqlrustgo_parser::Expression;
    use sqlrustgo_storage::Record;
    use sqlrustgo_types::Value;

    fn make_record(values: &[i64]) -> Record {
        values.iter().map(|&v| Value::Integer(v)).collect()
    }

    fn lit(s: &str) -> Expression {
        Expression::Literal(s.to_string())
    }

    fn ident(s: &str) -> Expression {
        Expression::Identifier(s.to_string())
    }

    fn binop(left: Expression, op: &str, right: Expression) -> Expression {
        Expression::BinaryOp(Box::new(left), op.to_string(), Box::new(right))
    }

    // --- parse_literal tests ---
    #[test]
    fn test_parse_literal_null() {
        assert_eq!(parse_literal("NULL"), Value::Null);
        assert_eq!(parse_literal("null"), Value::Null);
        assert_eq!(parse_literal("  Null  "), Value::Null);
    }

    #[test]
    fn test_parse_literal_integer() {
        assert_eq!(parse_literal("42"), Value::Integer(42));
        assert_eq!(parse_literal("-10"), Value::Integer(-10));
        assert_eq!(parse_literal("  100  "), Value::Integer(100));
    }

    #[test]
    fn test_parse_literal_float() {
        assert_eq!(parse_literal("3.14"), Value::Float(3.14));
        assert_eq!(parse_literal("-2.5"), Value::Float(-2.5));
        assert_eq!(parse_literal("  1.5  "), Value::Float(1.5));
    }

    #[test]
    fn test_parse_literal_string() {
        assert_eq!(parse_literal("'hello'"), Value::Text("hello".to_string()));
        assert_eq!(parse_literal("'foo bar'"), Value::Text("foo bar".to_string()));
        // unquoted falls through to Text
        assert_eq!(parse_literal("hello"), Value::Text("hello".to_string()));
    }

    // --- evaluate_binary_op tests ---
    #[test]
    fn test_evaluate_binary_op_add() {
        assert_eq!(
            evaluate_binary_op(&Value::Integer(3), "+", &Value::Integer(5)),
            Value::Integer(8)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(1.5), "+", &Value::Float(2.5)),
            Value::Float(4.0)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Integer(3), "+", &Value::Float(2.5)),
            Value::Float(5.5)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(3.0), "+", &Value::Integer(2)),
            Value::Float(5.0)
        );
    }

    #[test]
    fn test_evaluate_binary_op_sub() {
        assert_eq!(
            evaluate_binary_op(&Value::Integer(10), "-", &Value::Integer(3)),
            Value::Integer(7)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(5.0), "-", &Value::Float(2.0)),
            Value::Float(3.0)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Integer(10), "-", &Value::Float(3.0)),
            Value::Float(7.0)
        );
    }

    #[test]
    fn test_evaluate_binary_op_mul() {
        assert_eq!(
            evaluate_binary_op(&Value::Integer(3), "*", &Value::Integer(5)),
            Value::Integer(15)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(2.0), "*", &Value::Float(4.0)),
            Value::Float(8.0)
        );
    }

    #[test]
    fn test_evaluate_binary_op_div() {
        assert_eq!(
            evaluate_binary_op(&Value::Integer(10), "/", &Value::Integer(2)),
            Value::Integer(5)
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(10.0), "/", &Value::Float(2.0)),
            Value::Float(5.0)
        );
        // division by zero
        assert_eq!(
            evaluate_binary_op(&Value::Integer(10), "/", &Value::Integer(0)),
            Value::Null
        );
        assert_eq!(
            evaluate_binary_op(&Value::Float(10.0), "/", &Value::Float(0.0)),
            Value::Null
        );
    }

    #[test]
    fn test_evaluate_binary_op_null_coalesce() {
        // mixed type with null yields null
        assert_eq!(
            evaluate_binary_op(&Value::Text("x".into()), "+", &Value::Null),
            Value::Null
        );
        assert_eq!(
            evaluate_binary_op(&Value::Null, "+", &Value::Integer(5)),
            Value::Null
        );
    }

    // --- resolve_identifier tests ---

    #[test]
    fn test_resolve_identifier_new_row() {
        let new_record = make_record(&[0, 100, 5000]);
        let trigger_ctx = TriggerContext::new(Some(&new_record), None)
            .with_new_col_names(vec!["id".into(), "amt".into(), "total".into()]);
        let eval = EvalContext::new(&trigger_ctx, None);

        assert_eq!(resolve_identifier("NEW.id", &eval), Value::Integer(0));
        assert_eq!(resolve_identifier("NEW.amt", &eval), Value::Integer(100));
        assert_eq!(resolve_identifier("NEW.total", &eval), Value::Integer(5000));
        // case insensitive
        assert_eq!(resolve_identifier("new.id", &eval), Value::Integer(0));
    }

    #[test]
    fn test_resolve_identifier_old_row() {
        let old_record = make_record(&[1, 50, 3000]);
        let trigger_ctx = TriggerContext::new(None, Some(&old_record))
            .with_new_col_names(vec![])
            .with_old_col_names(vec!["id".into(), "amt".into(), "total".into()]);
        let eval = EvalContext::new(&trigger_ctx, None);

        assert_eq!(resolve_identifier("OLD.id", &eval), Value::Integer(1));
        assert_eq!(resolve_identifier("OLD.amt", &eval), Value::Integer(50));
        assert_eq!(resolve_identifier("OLD.total", &eval), Value::Integer(3000));
    }

    #[test]
    fn test_resolve_identifier_target_row() {
        let trigger_ctx = TriggerContext::new(None, None);
        let target = make_record(&[7, 8, 9]);
        let eval = EvalContext::new(&trigger_ctx, Some(&target))
            .with_target_col_names(vec!["x".into(), "y".into(), "z".into()]);

        assert_eq!(resolve_identifier("x", &eval), Value::Integer(7));
        assert_eq!(resolve_identifier("y", &eval), Value::Integer(8));
        assert_eq!(resolve_identifier("z", &eval), Value::Integer(9));
    }

    #[test]
    fn test_resolve_identifier_null_when_no_row() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        // no NEW/OLD row available
        assert_eq!(resolve_identifier("NEW.id", &eval), Value::Null);
        assert_eq!(resolve_identifier("OLD.id", &eval), Value::Null);
        // target column when no target
        assert_eq!(resolve_identifier("col", &eval), Value::Null);
    }

    // --- expression_to_value tests ---
    #[test]
    fn test_expression_to_value_literal() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        assert_eq!(expression_to_value(&lit("42"), &eval, None), Value::Integer(42));
        assert_eq!(expression_to_value(&lit("NULL"), &eval, None), Value::Null);
    }

    #[test]
    fn test_expression_to_value_binary_op() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        let expr = binop(lit("3"), "+", lit("5"));
        assert_eq!(expression_to_value(&expr, &eval, None), Value::Integer(8));
    }

    #[test]
    fn test_expression_to_value_unmatched_variant() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        // Subquery/In/Exists/etc. all fall through to Value::Null
        assert_eq!(
            expression_to_value(&Expression::IsNull(Box::new(lit("x"))), &eval, None),
            Value::Null
        );
        assert_eq!(
            expression_to_value(&Expression::IsNotNull(Box::new(lit("x"))), &eval, None),
            Value::Null
        );
    }

    // --- expression_to_bool tests ---
    #[test]
    fn test_expression_to_bool_true_value() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        // non-null/non-boolean -> true
        assert!(expression_to_bool(&lit("hello"), &eval, None));
        assert!(expression_to_bool(&lit("42"), &eval, None));
    }

    #[test]
    fn test_expression_to_bool_false_null() {
        let trigger_ctx = TriggerContext::new(None, None);
        let eval = EvalContext::new(&trigger_ctx, None);
        // Null -> false
        assert!(!expression_to_bool(&lit("NULL"), &eval, None));
        // Boolean false -> false
        let new_record2 = make_record(&[0]);
        let trigger_ctx2 = TriggerContext::new(Some(&new_record2), None)
            .with_new_col_names(vec!["b".into()]);
        let eval2 = EvalContext::new(&trigger_ctx2, None);
        // A literal boolean expression is not directly supported,
        // but a NEW.b = false expression would evaluate to Value::Boolean(false)
        let eq_expr = binop(ident("NEW.b"), "=", lit("0"));
        assert!(!expression_to_bool(&eq_expr, &eval2, None));
    }
}
