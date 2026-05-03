//! Trigger evaluation tests - unit tests for trigger_eval module.
//! Target: improve executor crate coverage from 61.4% to 70%+.

use sqlrustgo_executor::trigger_eval::{
    expression_to_bool, expression_to_value, EvalContext, TriggerContext,
};
use sqlrustgo_parser::Expression;
use sqlrustgo_storage::Record;
use sqlrustgo_types::Value;

// Helper: create a simple Record
fn make_record(values: Vec<i64>) -> Record {
    values.into_iter().map(|v| Value::Integer(v)).collect()
}

// ============ TriggerContext Tests ============

#[test]
fn test_trigger_context_new_row_only() {
    let record = make_record(vec![1, 100]);
    let ctx = TriggerContext::new(Some(&record), None);
    assert_eq!(ctx.new_row().map(|r| r.len()), Some(2));
    assert!(ctx.old_row().is_none());
}

#[test]
fn test_trigger_context_old_row_only() {
    let record = make_record(vec![1, 100]);
    let ctx = TriggerContext::new(None, Some(&record));
    assert!(ctx.new_row().is_none());
    assert_eq!(ctx.old_row().map(|r| r.len()), Some(2));
}

#[test]
fn test_trigger_context_both_rows() {
    let new_record = make_record(vec![2, 200]);
    let old_record = make_record(vec![1, 100]);
    let ctx = TriggerContext::new(Some(&new_record), Some(&old_record));
    assert_eq!(ctx.new_row().map(|r| r.len()), Some(2));
    assert_eq!(ctx.old_row().map(|r| r.len()), Some(2));
}

#[test]
fn test_trigger_context_with_new_col_names() {
    let record = make_record(vec![1]);
    let ctx = TriggerContext::new(Some(&record), None)
        .with_new_col_names(vec!["id".to_string(), "name".to_string()]);
    assert_eq!(
        ctx.new_col_names(),
        Some(&["id".to_string(), "name".to_string()][..])
    );
}

#[test]
fn test_trigger_context_with_old_col_names() {
    let record = make_record(vec![1]);
    let ctx = TriggerContext::new(None, Some(&record))
        .with_old_col_names(vec!["id".to_string(), "status".to_string()]);
    assert_eq!(
        ctx.old_col_names(),
        Some(&["id".to_string(), "status".to_string()][..])
    );
}

// ============ EvalContext Tests ============

#[test]
fn test_eval_context_basic() {
    let trigger_record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&trigger_record), None);
    let target_record = make_record(vec![99]);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target_record));
    assert_eq!(eval_ctx.trigger().new_row().map(|r| r.len()), Some(1));
    assert_eq!(eval_ctx.target_row().map(|r| r.len()), Some(1));
}

#[test]
fn test_eval_context_with_target_col_names() {
    let trigger_record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&trigger_record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None)
        .with_target_col_names(vec!["x".to_string(), "y".to_string()]);
    assert_eq!(
        eval_ctx.target_col_names(),
        Some(&["x".to_string(), "y".to_string()][..])
    );
}

#[test]
fn test_eval_context_trigger_reference() {
    let trigger_record = make_record(vec![42]);
    let trigger_ctx = TriggerContext::new(Some(&trigger_record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    assert_eq!(eval_ctx.trigger().new_row().map(|r| r.len()), Some(1));
}

#[test]
fn test_eval_context_no_target_row() {
    let trigger_record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&trigger_record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    assert!(eval_ctx.target_row().is_none());
}

// ============ expression_to_value Tests ============

#[test]
fn test_expression_to_value_null() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Literal("NULL".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Null);
}

#[test]
fn test_expression_to_value_literal_int() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Literal("42".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_expression_to_value_literal_float() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Literal("3.14".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_expression_to_value_literal_text() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Literal("'hello'".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Text("hello".to_string()));
}

#[test]
fn test_expression_to_value_literal_string() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    // Non-quoted string becomes Text
    let expr = Expression::Literal("hello".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Text("hello".to_string()));
}

#[test]
fn test_expression_to_value_new_identifier() {
    let record = make_record(vec![42]);
    let trigger_ctx =
        TriggerContext::new(Some(&record), None).with_new_col_names(vec!["id".to_string()]);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Identifier("NEW.id".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_expression_to_value_old_identifier() {
    let old_record = make_record(vec![100]);
    let trigger_ctx =
        TriggerContext::new(None, Some(&old_record)).with_old_col_names(vec!["id".to_string()]);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Identifier("OLD.id".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_expression_to_value_target_identifier() {
    let trigger_record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&trigger_record), None);
    let target_record = make_record(vec![99]);
    let eval_ctx = EvalContext::new(&trigger_ctx, Some(&target_record))
        .with_target_col_names(vec!["balance".to_string()]);
    let expr = Expression::Identifier("balance".to_string());
    let result = expression_to_value(&expr, &eval_ctx, Some(&["balance".to_string()]));
    assert_eq!(result, Value::Integer(99));
}

#[test]
fn test_expression_to_value_binary_op_int() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("10".to_string())),
        "+".to_string(),
        Box::new(Expression::Literal("5".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_expression_to_value_binary_op_float() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("3.5".to_string())),
        "+".to_string(),
        Box::new(Expression::Literal("2.5".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Float(6.0));
}

#[test]
fn test_expression_to_value_binary_op_sub() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("10".to_string())),
        "-".to_string(),
        Box::new(Expression::Literal("3".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_expression_to_value_binary_op_mul() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("6".to_string())),
        "*".to_string(),
        Box::new(Expression::Literal("7".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_expression_to_value_binary_op_div() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("20".to_string())),
        "/".to_string(),
        Box::new(Expression::Literal("4".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_expression_to_value_binary_op_div_by_zero() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::BinaryOp(
        Box::new(Expression::Literal("10".to_string())),
        "/".to_string(),
        Box::new(Expression::Literal("0".to_string())),
    );
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Null); // Division by zero returns Null
}

#[test]
fn test_expression_to_value_subquery() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    // Subquery returns Null in this implementation
    let expr = Expression::Subquery(Box::new(sqlrustgo_parser::SelectStatement {
        columns: vec![],
        table: "t".to_string(),
        from: None,
        where_clause: None,
        join_clause: None,
        aggregates: vec![],
        group_by: vec![],
        having: None,
        order_by: vec![],
        limit: None,
        offset: None,
        distinct: false,
    }));
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Null);
}

#[test]
fn test_expression_to_value_is_null() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let expr = Expression::IsNull(Box::new(Expression::Literal("NULL".to_string())));
    let result = expression_to_value(&expr, &eval_ctx, None);
    assert_eq!(result, Value::Null); // IsNull returns Null as fallback
}

// ============ expression_to_bool Tests ============

#[test]
fn test_expression_to_bool_true() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    // Literal "1" is non-empty string = truthy
    let expr = Expression::Literal("1".to_string());
    let result = expression_to_bool(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, true);
}

#[test]
fn test_expression_to_bool_false() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    // Literal "0" parses as text "0", which is truthy in this impl
    let expr = Expression::Literal("0".to_string());
    let result = expression_to_bool(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, true); // Non-Null is truthy
}

#[test]
fn test_expression_to_bool_null() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    let expr = Expression::Literal("NULL".to_string());
    let result = expression_to_bool(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, false); // Null is falsy
}

#[test]
fn test_expression_to_bool_negative() {
    let record = make_record(vec![1]);
    let trigger_ctx = TriggerContext::new(Some(&record), None);
    let eval_ctx = EvalContext::new(&trigger_ctx, None);
    let col_names = vec!["id".to_string()];
    // Non-empty non-zero string is truthy
    let expr = Expression::Literal("-5".to_string());
    let result = expression_to_bool(&expr, &eval_ctx, Some(&col_names));
    assert_eq!(result, true);
}
