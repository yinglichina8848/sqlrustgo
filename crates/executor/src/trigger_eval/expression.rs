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

    if name_upper.starts_with("NEW.") {
        let col_name = &name_upper[4..];
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

    if name_upper.starts_with("OLD.") {
        let col_name = &name_upper[4..];
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
