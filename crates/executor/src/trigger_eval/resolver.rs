use super::context::EvalContext;
use sqlrustgo_types::Value;

pub fn resolve_column(
    name: &str,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> Option<Value> {
    None
}
