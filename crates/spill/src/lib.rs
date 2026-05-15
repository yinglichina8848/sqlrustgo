pub mod error;
pub mod r#trait;
pub mod memory_tracker;
pub mod partition_manager;
pub mod fallback_manager;
pub mod operators;

pub use error::{SpillError, SpillResult};
pub use r#trait::SpillingIterator;
pub use memory_tracker::AdaptiveMemoryTracker;
pub use partition_manager::PartitionManager;
pub use fallback_manager::FallbackManager;
