//! SQLRustGo Code Intelligence Graph System (P0)
//!
//! # P0 Capabilities
//!
//! - ✅ Build graph from Rust source files
//! - ✅ Index nodes by ID, file, type
//! - ✅ Query: get_node, get_neighbors, locate_by_file
//! - ✅ JSON persistence
//! - ✅ CLI tool
//!
//! # P1 (next)
//!
//! - Context Router (token reduction)
//! - Call graph edges (function→function)
//! - Test impact graph
//! - Incremental updates

pub mod ast;
pub mod graph;
pub mod runtime;
pub mod store;

pub use graph::{CodeGraph, Edge, EdgeType, Node, NodeType};
pub use runtime::GraphRuntime;
pub use store::{load_graph, save_graph};
