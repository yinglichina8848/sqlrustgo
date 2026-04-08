//! SQLRustGo Graph Engine
//!
//! Property Graph with GMP (Good Manufacturing Practice) traceability support.
//!
//! # Core Concepts
//!
//! - **Node**: Entity with label and properties (e.g., Batch, Device, SOP)
//! - **Edge**: Relationship between nodes with label and properties
//! - **Label**: Type identifier for nodes and edges
//! - **PropertyMap**: Flexible key-value property storage
//!
//! # GMP Use Case
//!
//! Traceability chain: Batch → Device → Calibration → Regulation

pub mod model;
pub mod store;
pub mod traversal;
pub mod error;

pub use model::*;
pub use store::*;
pub use traversal::*;
pub use error::*;