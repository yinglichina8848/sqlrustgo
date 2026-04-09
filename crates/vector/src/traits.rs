//! Vector index trait definitions

use crate::metrics::DistanceMetric;
use crate::VectorResult;

/// Vector index entry with ID and similarity score
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub id: u64,
    pub score: f32,
}

impl IndexEntry {
    pub fn new(id: u64, score: f32) -> Self {
        Self { id, score }
    }
}

/// A vector with its ID for iteration/serialization
#[derive(Debug, Clone)]
pub struct VectorRecord {
    pub id: u64,
    pub vector: Vec<f32>,
}

/// Core trait for vector indexes
pub trait VectorIndex: Send + Sync {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()>;

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>>;

    fn build_index(&mut self) -> VectorResult<()>;

    fn delete(&mut self, id: u64) -> VectorResult<()>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn dimension(&self) -> usize;

    fn metric(&self) -> DistanceMetric;

    /// Get all vectors as records for iteration/serialization
    fn get_all(&self) -> Vec<VectorRecord>;

    /// Iterate over all vectors without cloning (borrowed references)
    /// Returns iterator of (id, vector_slice) pairs
    fn iter_vectors(&self) -> Box<dyn Iterator<Item = (u64, &[f32])> + '_>;
}

pub trait VectorIndexBuilder: Default {
    fn with_metric(self, metric: DistanceMetric) -> Self;
    fn with_dimension(self, dim: usize) -> Self;
    fn build(self) -> Box<dyn VectorIndex>;
}
