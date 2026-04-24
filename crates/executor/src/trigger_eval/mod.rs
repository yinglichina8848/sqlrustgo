pub mod context;
pub mod resolver;
pub mod expression;

pub use context::{EvalContext, TriggerContext};
pub use resolver::resolve_column;
pub use expression::{expression_to_value, expression_to_bool};
