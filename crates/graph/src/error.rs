//! Graph error types

use crate::model::{EdgeId, LabelId, NodeId};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0:?}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0:?}")]
    EdgeNotFound(EdgeId),

    #[error("Label not found: {0:?}")]
    LabelNotFound(LabelId),

    #[error("Label already exists: {0}")]
    LabelAlreadyExists(String),

    #[error("Invalid edge: from={from:?} to={to:?}")]
    InvalidEdge { from: NodeId, to: NodeId },

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}
