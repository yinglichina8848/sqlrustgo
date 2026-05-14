pub mod cgtid;
pub mod client_registry;
pub mod endpoint;
pub mod error;
pub mod ot_engine;
pub mod sync_result;
pub mod sync_types;
pub mod vector_clock;

pub use cgtid::ClientGtid;
pub use client_registry::ClientRegistry;
pub use endpoint::SyncEndpoint;
pub use error::{SyncError, SyncResult};
pub use ot_engine::OTEngine;
pub use vector_clock::VectorClock;

pub use sync_result::SyncResponse;
pub use sync_types::{
    BusinessOperation, CommitResult, ConflictResult, ErrorResult, Operation, OperationType,
    ResponseResult, SqlOperation, SyncRequest,
};

pub const SYNC_SERVICE_NAME: &str = "sqlrustgo.sync.SyncService";
