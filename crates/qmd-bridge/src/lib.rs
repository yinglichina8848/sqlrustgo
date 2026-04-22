//! SQLRustGo QMD Bridge
//!
//! Unified retrieval layer connecting SQLRustGo with QMD (Query Memory Database).

pub mod config;
pub mod error;
pub mod types;
pub mod hybrid;
pub mod bridge;
pub mod sync;

pub use config::QmdConfig;
pub use error::{QmdBridgeError, QmdResult};
pub use bridge::{QmdBridge, QmdBridgeImpl};
pub use types::{Filter, FilterOperator, QmdData, QmdDataType, QmdQuery, QueryType, SearchResult};
pub use hybrid::{HybridQuery, HybridResult, HybridSearchConfig, HybridSearcher, HybridSearchResultItem};
pub use sync::SyncManager;
pub use types::SyncStatus;
