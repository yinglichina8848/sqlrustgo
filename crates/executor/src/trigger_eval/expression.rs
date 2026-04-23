use sqlrustgo_parser::Expression;
use sqlrustgo_types::Value;
use sqlrustgo_storage::Record;
use super::context::EvalContext;

pub fn expression_to_value(
    _expr: &Expression,
    _ctx: &EvalContext,
    _column_names: Option<&[String]>,
) -> Value {
    Value::Null
}

pub fn expression_to_bool(
    _expr: &Expression,
    _ctx: &EvalContext,
    _column_names: Option<&[String]>,
) -> bool {
    false
}
