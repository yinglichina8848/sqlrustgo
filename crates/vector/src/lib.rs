//! SQLRustGo Vector Index Library
//!
//! High-performance vector index supporting Flat, IVF, and HNSW algorithms.
//!
//! # Architecture
//!
//! - `metrics` - Distance metric implementations (Cosine, Euclidean, DotProduct, Manhattan)
//! - `flat` - Flat index (brute-force O(n) search)
//! - `ivf` - IVF index (Inverted File with k-means clustering)
//! - `hnsw` - HNSW index (Hierarchical Navigable Small World)
//!
//! # Usage
//!
//! ```
//! use sqlrustgo_vector::{FlatIndex, DistanceMetric, VectorIndex};
//!
//! let mut index = FlatIndex::new(DistanceMetric::Cosine);
//! index.insert(1, &[0.1, 0.2, 0.3]).unwrap();
//! index.insert(2, &[0.4, 0.5, 0.6]).unwrap();
//! index.build_index().unwrap();
//!
//! let results = index.search(&[0.1, 0.2, 0.3], 1).unwrap();
//! ```

pub mod flat;
pub mod hnsw;
pub mod ivf;
pub mod metrics;

pub mod error;
pub mod traits;

pub use error::{VectorError, VectorResult};
pub use flat::FlatIndex;
pub use hnsw::HnswIndex;
pub use ivf::IvfIndex;
pub use metrics::DistanceMetric;
pub use traits::{VectorIndex, VectorIndexBuilder};
