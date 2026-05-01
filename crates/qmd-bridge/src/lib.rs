//! SQLRustGo QMD Bridge
//!
//! Unified retrieval layer connecting SQLRustGo with QMD (Query Memory Database).

pub mod bridge;
pub mod config;
pub mod error;
pub mod hybrid;
pub mod sync;
pub mod types;

pub use bridge::{QmdBridge, QmdBridgeImpl};
pub use config::QmdConfig;
pub use error::{QmdBridgeError, QmdResult};
pub use hybrid::{
    HybridQuery, HybridResult, HybridSearchConfig, HybridSearchResultItem, HybridSearcher,
};
pub use sync::SyncManager;
pub use types::SyncStatus;
pub use types::{Filter, FilterOperator, QmdData, QmdDataType, QmdQuery, QueryType, SearchResult};
