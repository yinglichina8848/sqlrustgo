use super::context::EvalContext;
use sqlrustgo_types::Value;

#[allow(dead_code)]
pub fn resolve_column(
    _name: &str,
    _ctx: &EvalContext,
) -> Value {
    Value::Null
}