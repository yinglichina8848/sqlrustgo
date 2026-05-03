//! SQLRustGo Expression Engine
//!
//! Unified expression evaluation system — single source of truth for all
//! expression semantics across parser, planner, optimizer, and executor.
//!
//! # Architecture
//!
//! - Parser produces `Expr`
//! - Planner/CBO transform `Expr`
//! - Executor evaluates `Expr` against row data
//!
//! # Core types
//!
//! - [`Expr`] — unified expression AST
//! - [`EvalContext`] — row data for evaluation
//! - [`Value`] — SQL values (from `sqlrustgo_types`)
//!
//! # Key invariants
//!
//! 1. All expression evaluation goes through `Expr::eval()` / `Expr::eval_predicate()`
//! 2. Row format: `HashMap<String, Value>` with keys `"table.column"`
//! 3. `EvalContext` carries all state needed for evaluation

pub mod eval;
pub mod expr;
pub mod op;

pub use eval::EvalContext;
pub use expr::Expr;
