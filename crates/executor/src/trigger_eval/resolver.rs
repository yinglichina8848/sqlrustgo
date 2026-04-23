use sqlrustgo_types::Value;
use super::context::EvalContext;

pub fn resolve_column(
    name: &str,
    ctx: &EvalContext,
    column_names: Option<&[String]>,
) -> Option<Value> {
    None
}
