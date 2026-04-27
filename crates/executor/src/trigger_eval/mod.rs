pub mod context;
pub mod expression;
pub mod resolver;

pub use context::{EvalContext, TriggerContext};
pub use expression::{expression_to_bool, expression_to_value};
pub use resolver::resolve_column;