pub mod cgtid;
pub mod vector_clock;
pub mod client_registry;
pub mod ot_engine;
pub mod endpoint;
pub mod error;
pub mod sync_types;
pub mod sync_result;

pub use cgtid::ClientGtid;
pub use vector_clock::VectorClock;
pub use client_registry::ClientRegistry;
pub use ot_engine::OTEngine;
pub use endpoint::SyncEndpoint;
pub use error::{SyncError, SyncResult};

pub use sync_types::{Operation, OperationType, BusinessOperation, SqlOperation, SyncRequest, ResponseResult, CommitResult, ConflictResult, ErrorResult};
pub use sync_result::SyncResponse;

pub const SYNC_SERVICE_NAME: &str = "sqlrustgo.sync.SyncService";
