//! SQLRustGo Unified Storage Layer
//!
//! Provides unified Document table support with SQL + Vector + Graph integration.
//!
//! # Features
//!
//! - **DocumentTable**: Unified table structure supporting both SQL columns and vector columns
//! - **UnifiedIndex**: Unified index management for SQL, vector, and graph indexes
//! - **GraphLink**: Association between document rows and graph nodes
//!
//! # Example
//!
//! ```
//! use sqlrustgo_unified_storage::{DocumentTable, UnifiedIndex, GraphLink};
//!
//! // Create a document table with mixed columns
//! let mut table = DocumentTable::new("products");
//! table.add_sql_column("id", "INTEGER");
//! table.add_sql_column("name", "TEXT");
//! table.add_vector_column("embedding", 384, sqlrustgo_vector::DistanceMetric::Cosine);
//!
//! // Create unified indexes
//! let mut index = UnifiedIndex::new("products");
//! index.add_btree_index(vec!["category".to_string()]);
//! index.add_hnsw_vector_index("embedding", 384, 16, 200);
//!
//! // Link to graph
//! let link = GraphLink::new("products", "id", "product_node");
//! ```

pub mod document_table;
pub mod graph_link;
pub mod unified_index;

// Re-exports for convenience
pub use document_table::{DocumentColumn, DocumentTable, VectorColumn};
pub use graph_link::{GraphLink, NodeRef};
pub use unified_index::{UnifiedIndex, UnifiedIndexMeta, UnifiedIndexType};
