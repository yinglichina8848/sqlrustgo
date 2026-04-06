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

pub mod batch_writer;
pub mod flat;
pub mod gpu_accel;
pub mod hnsw;
pub mod ivf;
pub mod metrics;
pub mod parallel_knn;
pub mod simd_explicit;
pub mod sql_vector_hybrid;

pub mod error;
pub mod traits;

pub use batch_writer::{BatchVectorWriter, BatchWriteConfig};
pub use error::{VectorError, VectorResult};
pub use flat::FlatIndex;
pub use gpu_accel::{
    CpuSimdAccelerator, GpuAccelerator, GpuConfig, GpuDevice, GpuStatus, PerformanceComparison,
};
pub use hnsw::HnswIndex;
pub use ivf::IvfIndex;
pub use metrics::DistanceMetric;
pub use parallel_knn::{ParallelKnn, ParallelKnnConfig, ParallelKnnIndex, ParallelSearchResult};
pub use simd_explicit::{dot_product_simd, euclidean_distance_simd, cosine_similarity_simd, manhattan_distance_simd, compute_similarity_simd, detect_simd_lanes, batch_compute_distances};
pub use sql_vector_hybrid::{HybridSearchConfig, HybridSearcher, HybridSearchResult};
pub use traits::{VectorIndex, VectorIndexBuilder};
