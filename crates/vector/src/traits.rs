//! Vector index trait definitions
//!
//! Provides core traits for implementing vector indexes (HNSW, IVF, Flat, etc.)

use crate::metrics::DistanceMetric;
use crate::VectorResult;

/// Vector index entry with ID and similarity score
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Unique vector identifier
    pub id: u64,
    /// Similarity score (higher is better for cosine/dot, lower is better for distance)
    pub score: f32,
}

impl IndexEntry {
    /// Create a new index entry
    pub fn new(id: u64, score: f32) -> Self {
        Self { id, score }
    }
}

/// A vector with its ID for iteration/serialization
#[derive(Debug, Clone)]
pub struct VectorRecord {
    /// Unique vector identifier
    pub id: u64,
    /// Vector data
    pub vector: Vec<f32>,
}

/// Core trait for vector indexes
pub trait VectorIndex: Send + Sync {
    /// Insert a vector into the index
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()>;

    /// Search for k nearest neighbors
    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>>;

    /// Build the index (required after bulk inserts)
    fn build_index(&mut self) -> VectorResult<()>;

    /// Delete a vector from the index
    fn delete(&mut self, id: u64) -> VectorResult<()>;

    /// Number of vectors in index
    fn len(&self) -> usize;

    /// Check if index is empty
    fn is_empty(&self) -> bool;

    /// Vector dimension
    fn dimension(&self) -> usize;

    /// Distance metric used
    fn metric(&self) -> DistanceMetric;

    /// Get all vectors as records for iteration/serialization
    fn get_all(&self) -> Vec<VectorRecord>;

    /// Iterate over all vectors without cloning (borrowed references)
    /// Returns iterator of (id, vector_slice) pairs
    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_>;
}

/// Builder trait for vector indexes
pub trait VectorIndexBuilder: Default {
    /// Set distance metric
    fn with_metric(self, metric: DistanceMetric) -> Self;
    /// Set vector dimension
    fn with_dimension(self, dim: usize) -> Self;
    /// Build the index
    fn build(self) -> Box<dyn VectorIndex>;
}
