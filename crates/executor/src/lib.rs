// SQLRustGo executor module

pub mod executor;
// stored_proc and trigger modules are temporarily stubbed - full implementation requires:
// 1. Catalog type in sqlrustgo-catalog (tracked in issue #1497)
// 2. Parser Expression/AlterTableOperation exports (tracked in issue #1497)
// 3. Full DDL → LogicalPlan → Executor chain
// pub mod stored_proc;
// pub mod trigger;

pub use executor::{Executor, ExecutorResult};
