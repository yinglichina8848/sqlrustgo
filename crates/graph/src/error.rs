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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EdgeId, LabelId, NodeId};

    #[test]
    fn test_graph_error_node_not_found() {
        let err = GraphError::NodeNotFound(NodeId(42));
        assert!(err.to_string().contains("Node not found"));
    }

    #[test]
    fn test_graph_error_edge_not_found() {
        let err = GraphError::EdgeNotFound(EdgeId(100));
        assert!(err.to_string().contains("Edge not found"));
    }

    #[test]
    fn test_graph_error_label_not_found() {
        let err = GraphError::LabelNotFound(LabelId(5));
        assert!(err.to_string().contains("Label not found"));
    }

    #[test]
    fn test_graph_error_label_already_exists() {
        let err = GraphError::LabelAlreadyExists("User".to_string());
        assert!(err.to_string().contains("Label already exists"));
        assert!(err.to_string().contains("User"));
    }

    #[test]
    fn test_graph_error_invalid_edge() {
        let err = GraphError::InvalidEdge {
            from: NodeId(1),
            to: NodeId(2),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid edge"));
    }

    #[test]
    fn test_graph_error_storage() {
        let err = GraphError::StorageError("disk full".to_string());
        assert!(err.to_string().contains("Storage error"));
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_graph_error_parse() {
        let err = GraphError::ParseError("invalid syntax".to_string());
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("invalid syntax"));
    }

    #[test]
    fn test_graph_error_debug() {
        let err = GraphError::NodeNotFound(NodeId(42));
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NodeNotFound"));
    }
}
