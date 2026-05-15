pub mod error;
pub mod fallback_manager;
pub mod memory_tracker;
pub mod operators;
pub mod partition_manager;
pub mod r#trait;

pub use error::{SpillError, SpillResult};
pub use fallback_manager::FallbackManager;
pub use memory_tracker::AdaptiveMemoryTracker;
pub use partition_manager::PartitionManager;
pub use r#trait::SpillingIterator;
