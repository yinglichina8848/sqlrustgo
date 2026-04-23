use super::context::EvalContext;
use sqlrustgo_types::Value;

pub fn resolve_column(
    _name: &str,
    _ctx: &EvalContext,
    _column_names: Option<&[String]>,
) -> Option<Value> {
    None
}
