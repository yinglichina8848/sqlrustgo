//! B+ Tree Index Module
//!
//! This module provides a disk-based B+ Tree index implementation for efficient
//! key-value lookups and range queries. Used by the storage layer for table indexing.
//!
//! ## Architecture
//!
//! ```mermaid
//! graph TB
//!     BPlusTree["BPlusTree"] --> Node["Node"]
//!     Node --> Leaf["LeafNode"]
//!     Node --> Internal["InternalNode"]
//!     Leaf --> KV1["Key-Value Pairs"]
//!     Leaf --> Next["Next Pointer"]
//!     Internal --> Keys["Separating Keys"]
//!     Internal --> Children["Child Pointers"]
//! ```
//!
//! ## Operations
//!
//! - `insert(key, value)`: Insert a key-value pair, splitting nodes as needed
//! - `search(key)`: O(log n) lookup returning the value
//! - `range_query(start, end)`: Efficient range scan using leaf node linked list
//! - `keys()`: Return all keys in sorted order
//!
//! ## Constants
//!
//! - `MAX_KEYS = 4`: Maximum keys per node (fanout-1)
//!
//! ## Usage Example
//!
//! ```rust
//! let mut tree = BPlusTree::new();
//! tree.insert(1, 100);
//! tree.insert(2, 200);
//! assert_eq!(tree.search(1), Some(100));
//! let results = tree.range_query(1, 3);
//! ```

mod tree;
pub use tree::BPlusTree;
