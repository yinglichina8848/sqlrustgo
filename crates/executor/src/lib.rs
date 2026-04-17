// SQLRustGo executor module

pub mod executor;
// stored_proc and trigger modules are temporarily disabled until their API dependencies are implemented
// pub mod stored_proc; // TODO(#1497): re-enable after catalog Catalog type and parser types exist
// pub mod trigger;     // TODO(#1480): re-enable after storage engine adds TriggerInfo/TriggerEvent/TriggerTiming

pub use executor::{Executor, ExecutorResult};
